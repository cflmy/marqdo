//! Audit library source with the real parser, and compare executable ASTs across prose refactors.
use anyhow::{Context, Result};
use marqdo::{ast::*, diagnostics::Span};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
fn files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        if p.is_dir() {
            files(&p, out)?
        } else if p.to_string_lossy().ends_with(".mq.md") {
            out.push(p)
        }
    }
    Ok(())
}
fn expr(e: Expr) -> Expr {
    match e {
        Expr::Occluded { payload, .. } => payload
            .map(|p| expr(*p))
            .unwrap_or(Expr::Literal(Literal::None)),
        Expr::Unary { op, expr: e } => Expr::Unary {
            op,
            expr: Box::new(expr(*e)),
        },
        Expr::Binary { op, left, right } => Expr::Binary {
            op,
            left: Box::new(expr(*left)),
            right: Box::new(expr(*right)),
        },
        Expr::Call(c) => {
            let c = call(c);
            pure_literal_call(&c).unwrap_or(Expr::Call(c))
        }
        Expr::List(xs) => Expr::List(xs.into_iter().map(expr).collect()),
        Expr::Map(xs) => Expr::Map(xs.into_iter().map(|(k, v)| (k, expr(v))).collect()),
        Expr::Index { base, label } => Expr::Index {
            base: Box::new(expr(*base)),
            label,
        },
        other => other,
    }
}
fn call(mut c: CallExpr) -> CallExpr {
    for a in &mut c.args {
        match a {
            Arg::Positional(e) | Arg::Named { value: e, .. } => *e = expr(e.clone()),
        }
    }
    c
}
// Only the explicitly supported pure literal-building cases are folded. This is
// a refactor audit, not an evaluator: unknown calls and nonconstant data remain.
fn literal_json_value(v: serde_json::Value) -> Option<Expr> {
    use serde_json::Value as J;
    Some(Expr::Literal(match v {
        J::Null => Literal::None,
        J::Bool(b) => Literal::Bool(b),
        J::String(s) => Literal::Text(s),
        J::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i.unsigned_abs() > 9_007_199_254_740_991 {
                    return None;
                }
                Literal::Int(i)
            } else {
                let n = n.as_f64()?;
                if !n.is_finite() || (n.fract() == 0.0 && n.abs() > 9_007_199_254_740_991.0) {
                    return None;
                }
                Literal::Num(n)
            }
        }
        _ => return None,
    }))
}
// Read JSON literals without accepting duplicate keys or unsafe numeric coercions.
fn flat_json_record(text: &str) -> Option<Expr> {
    struct Constant;
    impl<'de> serde::Deserialize<'de> for ConstantExpr {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            de.deserialize_any(Constant).map(ConstantExpr)
        }
    }
    struct ConstantExpr(Expr);
    impl<'de> serde::de::Visitor<'de> for Constant {
        type Value = Expr;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("constant JSON in the exact audit numeric domain")
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Expr, E> {
            Ok(Expr::Literal(Literal::None))
        }
        fn visit_bool<E: serde::de::Error>(self, b: bool) -> Result<Expr, E> {
            Ok(Expr::Literal(Literal::Bool(b)))
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Expr, E> {
            Ok(Expr::Literal(Literal::Text(s.into())))
        }
        fn visit_i64<E: serde::de::Error>(self, n: i64) -> Result<Expr, E> {
            literal_json_value(n.into()).ok_or_else(|| E::custom("unsafe integer"))
        }
        fn visit_u64<E: serde::de::Error>(self, n: u64) -> Result<Expr, E> {
            literal_json_value(n.into()).ok_or_else(|| E::custom("unsafe integer"))
        }
        fn visit_f64<E: serde::de::Error>(self, n: f64) -> Result<Expr, E> {
            let n = serde_json::Number::from_f64(n).ok_or_else(|| E::custom("nonfinite number"))?;
            literal_json_value(n.into()).ok_or_else(|| E::custom("unsafe number"))
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Expr, A::Error> {
            let mut values = Vec::new();
            while let Some(ConstantExpr(v)) = seq.next_element()? {
                values.push(v);
            }
            Ok(Expr::List(values))
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Expr, A::Error> {
            use serde::de::Error;
            let mut fields = std::collections::BTreeMap::new();
            while let Some((key, ConstantExpr(value))) = map.next_entry::<String, ConstantExpr>()? {
                if fields.insert(key, value).is_some() {
                    return Err(A::Error::custom("duplicate key"));
                }
            }
            Ok(Expr::Map(fields.into_iter().collect()))
        }
    }
    let ConstantExpr(value) = serde_json::from_str::<ConstantExpr>(text).ok()?;
    Some(value)
}
fn pure_literal_call(c: &CallExpr) -> Option<Expr> {
    let name = c
        .path
        .as_ref()
        .map(|p| p.join("."))
        .unwrap_or_else(|| c.callee.clone());
    let get = |key: &str| {
        c.args.iter().find_map(|a| match a {
            Arg::Named { name, value } if name == key => Some(value),
            _ => None,
        })
    };
    match name.as_str() {
        "json.parse" => {
            if let Expr::Literal(Literal::Text(text)) = get("text")? {
                flat_json_record(text)
            } else {
                None
            }
        }
        "table.put" if matches!(get("in")?, Expr::Literal(Literal::None)) => {
            if let Expr::Literal(Literal::Text(key)) = get("at")? {
                let value = get("value")?;
                if matches!(value, Expr::Literal(_)) {
                    Some(Expr::Map(vec![(key.clone(), value.clone())]))
                } else {
                    None
                }
            } else {
                None
            }
        }
        "table.empty_list" => Some(Expr::List(Vec::new())),
        "table.from_pairs" => {
            let Expr::List(rows) = get("pairs")? else {
                return None;
            };
            let mut fields = Vec::new();
            let mut seen = std::collections::BTreeSet::new();
            for row in rows {
                let Expr::Map(pairs) = row else {
                    return None;
                };
                let Some((_, Expr::Literal(Literal::Text(key)))) =
                    pairs.iter().find(|(k, _)| k == "key")
                else {
                    return None;
                };
                let Some((_, value)) = pairs.iter().find(|(k, _)| k == "value") else {
                    return None;
                };
                if !constant(value) || !seen.insert(key.clone()) {
                    return None;
                }
                fields.push((key.clone(), value.clone()));
            }
            Some(Expr::Map(fields))
        }
        "text.str_join" => {
            let Expr::List(rows) = get("xs")? else {
                return None;
            };
            let Expr::Literal(Literal::Text(separator)) = get("sep")? else {
                return None;
            };
            let rows = rows
                .iter()
                .map(|e| {
                    if let Expr::Literal(Literal::Text(s)) = e {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Option<Vec<_>>>()?;
            Some(Expr::Literal(Literal::Text(rows.join(separator))))
        }
        _ => None,
    }
}
fn constant(e: &Expr) -> bool {
    match e {
        Expr::Literal(_) => true,
        Expr::List(xs) => xs.iter().all(constant),
        Expr::Map(xs) => xs.iter().all(|(_, v)| constant(v)),
        _ => false,
    }
}
fn substitute(e: Expr, temps: &std::collections::BTreeMap<String, Expr>) -> Expr {
    let e = match e {
        Expr::Var(name) if temps.contains_key(&name) => temps[&name].clone(),
        Expr::Call(mut c) => {
            for a in &mut c.args {
                match a {
                    Arg::Named { value, .. } | Arg::Positional(value) => {
                        *value = substitute(value.clone(), temps)
                    }
                }
            }
            Expr::Call(c)
        }
        Expr::Map(xs) => Expr::Map(
            xs.into_iter()
                .map(|(k, v)| (k, substitute(v, temps)))
                .collect(),
        ),
        Expr::List(xs) => Expr::List(xs.into_iter().map(|v| substitute(v, temps)).collect()),
        Expr::Occluded {
            payload: Some(v), ..
        } => substitute(*v, temps),
        Expr::Unary { op, expr: v } => Expr::Unary {
            op,
            expr: Box::new(substitute(*v, temps)),
        },
        Expr::Binary { op, left, right } => Expr::Binary {
            op,
            left: Box::new(substitute(*left, temps)),
            right: Box::new(substitute(*right, temps)),
        },
        other => other,
    };
    expr(e)
}
fn canonical_statements(statements: Vec<Stmt>) -> Vec<Stmt> {
    fn block(
        statements: Vec<Stmt>,
        mut temps: std::collections::BTreeMap<String, Expr>,
    ) -> Vec<Stmt> {
        let mut out = Vec::new();
        for s in statements {
            let s = statement(s);
            let s = match s {
                Stmt::Assign {
                    name,
                    value,
                    span,
                    end_line,
                } => {
                    let value = substitute(value, &temps);
                    if name.starts_with("_literal_") && constant(&value) {
                        temps.insert(name, value);
                        continue;
                    }
                    Stmt::Assign {
                        name,
                        value,
                        span,
                        end_line,
                    }
                }
                Stmt::Return { value, span } => Stmt::Return {
                    value: substitute(value, &temps),
                    span,
                },
                Stmt::Expr { value, span } => Stmt::Expr {
                    value: substitute(value, &temps),
                    span,
                },
                Stmt::Branch { arms, span } => Stmt::Branch {
                    arms: arms
                        .into_iter()
                        .map(|a| BranchArm {
                            condition: a.condition.map(|e| substitute(e, &temps)),
                            body: block(a.body, temps.clone()),
                        })
                        .collect(),
                    span,
                },
                Stmt::ForEach {
                    item,
                    collection,
                    label,
                    body,
                    span,
                } => Stmt::ForEach {
                    item,
                    collection,
                    label,
                    body: block(body, temps.clone()),
                    span,
                },
                Stmt::While {
                    condition,
                    body,
                    span,
                } => Stmt::While {
                    condition: substitute(condition, &temps),
                    body: block(body, temps.clone()),
                    span,
                },
                other => other,
            };
            out.push(s);
        }
        out
    }
    block(statements, std::collections::BTreeMap::new())
}

fn statement(s: Stmt) -> Stmt {
    let span = Span::new(0, 0);
    match s {
        Stmt::Assign { name, value, .. } => Stmt::Assign {
            name,
            value: expr(value),
            span,
            end_line: 0,
        },
        Stmt::Return { value, .. } => Stmt::Return {
            value: expr(value),
            span,
        },
        Stmt::Call { call: c, .. } => Stmt::Expr {
            value: Expr::Call(call(c)),
            span,
        },
        Stmt::Expr { value, .. } => Stmt::Expr {
            value: expr(value),
            span,
        },
        Stmt::Branch { arms, .. } => Stmt::Branch {
            arms: arms
                .into_iter()
                .map(|a| BranchArm {
                    condition: a.condition.map(expr),
                    body: a.body.into_iter().map(statement).collect(),
                })
                .collect(),
            span,
        },
        Stmt::While {
            condition, body, ..
        } => Stmt::While {
            condition: expr(condition),
            body: body.into_iter().map(statement).collect(),
            span,
        },
        Stmt::ForEach {
            item,
            collection,
            body,
            ..
        } => Stmt::ForEach {
            item,
            collection,
            body: body.into_iter().map(statement).collect(),
            span,
            label: None,
        },
    }
}
fn quote(s: &str) -> String {
    serde_json::to_string(s).unwrap().replace('`', "\\`")
}
fn render(e: &Expr) -> Result<String> {
    Ok(match e {
        Expr::Literal(l) => match l {
            Literal::None => "None".into(),
            Literal::Bool(b) => if *b { "True" } else { "False" }.into(),
            Literal::Int(n) => n.to_string(),
            Literal::Num(n) => {
                let text = n.to_string();
                if text.contains('.') || text.contains(['e', 'E']) {
                    text
                } else {
                    format!("{text}.0")
                }
            }
            Literal::Text(s) => quote(s),
        },
        Expr::Var(n) => format!("`{n}`"),
        Expr::Interp(parts) => {
            let mut s = String::new();
            for p in parts {
                match p {
                    InterpPart::Lit(t) => s.push_str(&t.replace('`', "\\`")),
                    InterpPart::Var(v) => s.push_str(&format!("`{v}`")),
                    InterpPart::Index { .. } => {
                        anyhow::bail!("legacy interpolated index needs review")
                    }
                }
            }
            serde_json::to_string(&s)?.replace("\\\\`", "\\`")
        }
        Expr::Call(c) => render_call(c)?,
        Expr::Unary { op, expr: e } => format!(
            "({}{})",
            match op {
                UnaryOp::Not => "not ",
                UnaryOp::Neg => "-",
            },
            render(e)?
        ),
        Expr::Binary { op, left, right } => format!(
            "({} {} {})",
            render(left)?,
            match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
            },
            render(right)?
        ),
        Expr::List(xs) => format!(
            "[{}]",
            xs.iter()
                .map(render)
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        ),
        Expr::Map(xs) => format!(
            "{{{}}}",
            xs.iter()
                .map(|(k, v)| Ok(format!("{}: {}", quote(k), render(v)?)))
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        ),
        Expr::Occluded {
            label,
            payload: Some(p),
        } => format!("[{}]({})", label.replace(']', "\\]"), render(p)?),
        Expr::Occluded { payload: None, .. } => "None".into(),
        _ => anyhow::bail!("expression needs manual rendering: {e:?}"),
    })
}
fn render_call(c: &CallExpr) -> Result<String> {
    let target = if let Some(r) = &c.receiver {
        format!("`{r}`.{}", c.callee)
    } else if let Some(p) = &c.path {
        p.join(".")
    } else {
        c.callee.clone()
    };
    let c = c.with_modifiers_expanded().map_err(anyhow::Error::msg)?;
    let args = c
        .args
        .iter()
        .map(|a| {
            Ok(match a {
                Arg::Positional(e) => render(e)?,
                Arg::Named { name, value } => format!("{name}={}", render(value)?),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if c.receiver.is_some() {
        Ok(format!("> {target} {} ", args.join(" ")))
    } else {
        Ok(format!("{target}({})", args.join(", ")))
    }
}
fn receiver_call(c: &CallExpr, f: &Function, imports: &[String]) -> CallExpr {
    let mut c = c.clone();
    if let Some(path) = &c.path {
        if path.len() == 2
            && f.params.iter().any(|p| p.name == path[0])
            && !imports.contains(&path[0])
        {
            c.receiver = Some(path[0].clone());
            c.path = None;
        }
    }
    c
}
fn render_value(v: &Expr, f: &Function, imports: &[String]) -> Result<String> {
    match v {
        Expr::Call(c) => render_call(&receiver_call(c, f, imports)),
        Expr::Occluded {
            label,
            payload: Some(v),
        } => Ok(format!(
            "[{}]({})",
            label.replace(']', "\\]"),
            render_value(v, f, imports)?
        )),
        other => render(other),
    }
}
// A qualified call through a parameter and its explicit receiver form resolve
// the same object when that name is not an imported module binding.
fn normalize_receiver_text(mut body: String, params: &Value, imports: &[String]) -> String {
    for p in params.as_array().unwrap() {
        let name = p["name"].as_str().unwrap();
        if imports.iter().any(|i| i == name) {
            continue;
        }
        let quoted = serde_json::to_string(name).unwrap();
        let pattern = format!(
            r#"path: Some\(\[{}, "[^"\n]+"\]\), receiver: None"#,
            regex::escape(&quoted)
        );
        let re = regex::Regex::new(&pattern).unwrap();
        body = re
            .replace_all(&body, format!("path: None, receiver: Some({quoted})"))
            .into_owned();
    }
    body
}
fn collect(fs: &[Function], prefix: &str, source: &str, imports: &[String], out: &mut Vec<Value>) {
    for f in fs {
        let name = if prefix.is_empty() {
            f.name.clone()
        } else {
            format!("{prefix}.{}", f.name)
        };
        let mut calls = Vec::new();
        marqdo::ast::walk_stmts(f, &mut |s| {
            if let Stmt::Call { call, span } = s {
                calls.push(json!({"line":span.line,"rendered":render_call(&receiver_call(call, f, imports)).ok(),"call":format!("{call:?}")}))
            }
        });
        let mut statements = Vec::new();
        marqdo::ast::walk_stmts(f, &mut |s| {
            let (kind, name, value, line, end) = match s {
                Stmt::Assign {
                    name,
                    value,
                    span,
                    end_line,
                } => (
                    "assign",
                    Some(name.as_str()),
                    Some(value),
                    span.line,
                    *end_line,
                ),
                Stmt::Return { value, span } => ("return", None, Some(value), span.line, span.line),
                Stmt::Expr { value, span } => ("expr", None, Some(value), span.line, span.line),
                Stmt::Call { call, span } => {
                    statements.push(json!({"kind":"call", "line":span.line, "rendered":render_call(&receiver_call(call, f, imports)).ok(), "callee":call.callee, "path":call.path, "receiver":call.receiver}));
                    return;
                }
                _ => return,
            };
            let mut inner = value;
            while let Some(Expr::Occluded {
                payload: Some(p), ..
            }) = inner
            {
                inner = Some(p);
            }
            let target = match inner {
                Some(Expr::Call(c)) => {
                    Some(json!({"callee":c.callee,"path":c.path,"receiver":c.receiver}))
                }
                _ => None,
            };
            statements.push(json!({"kind":kind,"name":name,"line":line,"end":end,"rendered":value.and_then(|v|render_value(v, f, imports).ok()),"target":target,"value":value.map(|v|format!("{v:?}"))}));
        });
        out.push(json!({"name":name,"short_name":f.name,"line":f.span.line,"level":f.level,"base":f.base,"receiver":function_uses_receiver(f),"params":f.params.iter().map(|p|json!({"name":p.name,"line":p.span.line,"default":p.default.as_ref().map(|e|format!("{:?}",expr(e.clone()))),"default_source":p.default.as_ref().and_then(|e|render(e).ok()),"type":f.contract.as_ref().and_then(|c|c.params.iter().find(|x|x.name==p.name)).map(|p|p.ty.name()).unwrap_or("any".into())})).collect::<Vec<_>>(),"returns":f.contract.as_ref().and_then(|c|c.returns.as_ref()).map(|r|r.ty.name()).unwrap_or("any".into()),"contract":f.contract.as_ref().map(|c|format!("{c:?}")),"body":format!("{:?}",canonical_statements(f.body.clone())),"calls":calls,"statements":statements,"narrative":marqdo::narrative_ratio::measure_source_units(source,std::slice::from_ref(f)).first().map(|r|r.ratio)}));
        collect(&f.children, &name, source, imports, out);
    }
}
fn difference(description: String, before: Value, after: Value) -> Value {
    use sha2::Digest;
    let digest = |value: Value| {
        format!(
            "{:x}",
            sha2::Sha256::digest(serde_json::to_vec(&value).unwrap())
        )
    };
    json!({"difference":description,"before":digest(before),"after":digest(after)})
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let baseline = args
        .iter()
        .position(|x| x == "--compare")
        .map(|i| fs::read_to_string(&args[i + 1]))
        .transpose()?
        .map(|s| serde_json::from_str::<Value>(&s))
        .transpose()?;
    let original = args
        .iter()
        .position(|x| x == "--from")
        .map(|i| fs::read_to_string(&args[i + 1]))
        .transpose()?
        .map(|s| serde_json::from_str::<Value>(&s))
        .transpose()?;
    let mut paths = Vec::new();
    files(Path::new("lib"), &mut paths)?;
    files(Path::new("ext"), &mut paths)?;
    paths.sort();
    if let Some(input) = &original {
        paths.retain(|p| input.get(p.to_str().unwrap()).is_some());
    }
    let mut result = serde_json::Map::new();
    let mut differences = Vec::new();
    for path in paths {
        let key = path.to_str().context("UTF-8 library path")?;
        let source = if let Some(s) = original
            .as_ref()
            .and_then(|m| m.get(key))
            .and_then(|m| m["source"].as_str())
        {
            s.to_owned()
        } else {
            fs::read_to_string(&path)?
        };
        let module = marqdo::parse::parse_source(&source).with_context(|| key.to_owned())?;
        let mut functions = Vec::new();
        let imports: Vec<_> = module
            .imports
            .iter()
            .map(|i| i.bind.clone())
            .chain(module.uses.iter().map(|i| i.bind.clone()))
            .collect();
        collect(&module.functions, "", &source, &imports, &mut functions);
        let reports = marqdo::narrative_ratio::measure_source_units(&source, &module.functions);
        for f in &mut functions {
            let report = reports.iter().find(|r| json!(r.line) == f["line"]).unwrap();
            f["narrative_chars"] = json!(report.narrative);
            f["code_chars"] = json!(report.total - report.narrative);
            f["narrative"] = json!(
                reports
                    .iter()
                    .find(|r| json!(r.line) == f["line"])
                    .unwrap()
                    .ratio
            );
        }
        if let Some(old) = baseline.as_ref().and_then(|b| b.get(key)) {
            for o in old["functions"].as_array().unwrap() {
                match functions.iter().find(|f| f["name"] == o["name"]) {
                    None => differences.push(difference(
                        format!("removed {key} {}", o["name"]),
                        json!({"level":o["level"],"base":o["base"],"receiver":o["receiver"],"params":o["params"].as_array().unwrap().iter().map(|p|json!([p["name"],p["default"]])).collect::<Vec<_>>(),"body":o["body"]}),
                        Value::Null,
                    )),
                    Some(n) => {
                        for field in ["level", "base", "receiver", "body"] {
                            let same = if field == "body" {
                                normalize_receiver_text(
                                    n[field].as_str().unwrap().to_owned(),
                                    &o["params"],
                                    &imports,
                                ) == normalize_receiver_text(
                                    o[field].as_str().unwrap().to_owned(),
                                    &o["params"],
                                    &imports,
                                )
                            } else {
                                n[field] == o[field]
                            };
                            if !same {
                                differences.push(difference(format!("{key} {} changed {field}", o["name"]), o[field].clone(), n[field].clone()));
                            }
                        }
                        let sig = |f: &Value| {
                            f["params"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|p| json!([p["name"], p["default"]]))
                                .collect::<Vec<_>>()
                        };
                        if sig(n) != sig(o) {
                            differences.push(difference(format!("{key} {} changed parameters", o["name"]),json!(sig(o)),json!(sig(n))));
                        }
                    }
                }
            }
        }
        result.insert(
            key.to_owned(),
            json!({"source": source, "functions": functions}),
        );
    }
    if let Some(i) = args.iter().position(|x| x == "--snapshot") {
        fs::write(&args[i + 1], serde_json::to_vec_pretty(&result)?)?;
    }
    println!(
        "{} library files, {} functions",
        result.len(),
        result
            .values()
            .map(|f| f["functions"].as_array().unwrap().len())
            .sum::<usize>()
    );
    if let Some(i) = args.iter().position(|x| x == "--differences") {
        fs::write(&args[i + 1], serde_json::to_vec_pretty(&differences)?)?;
    }
    if baseline.is_some() {
        let reviewed = args
            .iter()
            .position(|x| x == "--reviewed-fixes")
            .map(|i| fs::read_to_string(&args[i + 1]))
            .transpose()?
            .map(|text| serde_json::from_str::<Vec<Value>>(&text))
            .transpose()?
            .unwrap_or_default();
        let matches = |d: &Value, r: &Value| {
            ["difference", "before", "after"]
                .iter()
                .all(|k| d[*k] == r[*k])
                && r["reason"].as_str().is_some_and(|s| !s.trim().is_empty())
        };
        let unreviewed: Vec<_> = differences
            .iter()
            .filter(|d| !reviewed.iter().any(|r| matches(d, r)))
            .collect();
        let stale: Vec<_> = reviewed
            .iter()
            .filter(|r| !differences.iter().any(|d| matches(d, r)))
            .collect();
        println!("{} executable differences, {} exact reviewed fixes, {} unreviewed, {} stale review records",
            differences.len(),differences.len()-unreviewed.len(),unreviewed.len(),stale.len());
        for d in &unreviewed {
            println!("{}", d["difference"]);
        }
        if !unreviewed.is_empty() || !stale.is_empty() {
            anyhow::bail!(
                "review executable differences; only exact before/after digests are accepted"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_literal_audit_rejects_duplicates_unsafe_numbers_and_trailing_data() {
        for text in [
            r#"{"x":1,"x":2}"#,
            r#"{"nested":{"x":1,"x":2}}"#,
            r#"{"x":9007199254740992}"#,
            r#"{} false"#,
            r#"{"x":NaN}"#,
        ] {
            assert!(flat_json_record(text).is_none(), "{text}");
        }
        let value = flat_json_record(r#"{"z":[null,true,"a\nb"],"a":1}"#).unwrap();
        assert!(constant(&value));
        let Expr::Map(fields) = value else {
            panic!("map expected");
        };
        assert_eq!(fields[0].0, "a");
        assert_eq!(fields[1].0, "z");
    }
    #[test]
    fn only_supported_constant_builders_are_folded() {
        for code in [
            "unknown(text=\"{}\")",
            "json.parse(text=`input`)",
            "table.put(in=`input`, at=\"key\", value=1)",
            "text.str_join(xs=`rows`, sep=\"\\n\")",
        ] {
            let Expr::Call(call) = marqdo::parse::expr::parse_expr(code).unwrap() else {
                panic!("call expected");
            };
            assert!(pure_literal_call(&call).is_none(), "{code}");
        }
        let Expr::Call(call) =
            marqdo::parse::expr::parse_expr("json.parse(text=\"{\\\"key\\\":null}\")").unwrap()
        else {
            panic!("call expected");
        };
        assert!(pure_literal_call(&call).is_some());
    }
}

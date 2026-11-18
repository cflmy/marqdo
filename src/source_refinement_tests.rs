//! Export the actual loaded AST and independently observe both real backends.
//! Unsupported AST constructs are errors, never omitted or normalized to no-op.
use crate::ast::{Arg, BinaryOp, CallExpr, Expr, Function, Literal, Module, Stmt};
use crate::host::{foreign_session, json::value_to_json, HostContext};
use crate::value::Value;
use serde_json::{json, Value as Json};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub(crate) fn boundary_evidence(
    ctx: &mut HostContext,
) -> (Vec<Value>, Vec<Value>, Vec<foreign_session::ProtocolStep>) {
    let observations = ctx.oracle_observations.clone();
    let protocol = ctx.foreign_protocol_steps.clone();
    let mut ids = ctx.foreign_sessions.keys().cloned().collect::<Vec<_>>();
    ids.sort();
    // These verification reads bypass the document observation API. They occur
    // only after its journal was captured, and never alter that journal.
    let values = ids
        .iter()
        .map(|id| {
            foreign_session::get(ctx, &Value::Handle(id.clone()), &Value::Text("n".into())).unwrap()
        })
        .collect();
    assert_eq!(observations, ctx.oracle_observations);
    (observations, values, protocol)
}

fn expr(e: &Expr) -> Result<Json, String> {
    Ok(match e {
        Expr::Literal(Literal::None) => json!(["literal", null]),
        Expr::Literal(Literal::Bool(b)) => json!(["literal", b]),
        Expr::Literal(Literal::Int(n)) => json!(["literal", n]),
        Expr::Literal(Literal::Text(s)) => json!(["literal", s]),
        Expr::Var(name) => json!(["var", name]),
        Expr::Binary { op, left, right } => {
            let tag = match op {
                BinaryOp::Add => "add",
                BinaryOp::Sub => "sub",
                BinaryOp::Lt => "lt",
                _ => return Err(format!("unsupported source binary op {op:?}")),
            };
            json!([tag, expr(left)?, expr(right)?])
        }
        _ => return Err(format!("unsupported source expression {e:?}")),
    })
}

fn call(c: &CallExpr, dest: Option<&str>, defs: &[&Function]) -> Result<Json, String> {
    if c.receiver.is_some() || !c.pre_modifiers.is_empty() {
        return Err("unsupported receiver/modifiers".into());
    }
    if let Some(path) = &c.path {
        if path.len() != 2 || path[0] != "foreign" {
            return Err(format!("unsupported library path {path:?}"));
        }
        let mut args = std::collections::BTreeMap::new();
        for a in &c.args {
            let Arg::Named { name, value } = a else {
                return Err("foreign primitive needs named arguments".into());
            };
            if args.insert(name.as_str(), value).is_some() {
                return Err("duplicate argument".into());
            }
        }
        let allowed: &[&str] = match path[1].as_str() {
            "session_open" => &["lang"],
            "session_set" => &["id", "name", "value"],
            "session_get" => &["id", "name"],
            "session_eval" => &["id", "source", "input", "stdin"],
            "session_close" => &["id"],
            _ => return Err(format!("unsupported primitive {path:?}")),
        };
        if args.keys().any(|k| !allowed.contains(k)) {
            return Err("unsupported primitive option".into());
        }
        let arg = |name: &str| {
            args.get(name)
                .copied()
                .ok_or_else(|| format!("missing {name}"))
        };
        let name = || match arg("name")? {
            Expr::Literal(Literal::Text(s)) => Ok(s.clone()),
            _ => Err("dynamic binding name unsupported".to_string()),
        };
        match path[1].as_str() {
            "session_open" => Ok(json!([
                "open",
                dest.ok_or("open destination required")?,
                expr(arg("lang")?)?
            ])),
            "session_get" => Ok(json!([
                "get",
                dest.ok_or("get destination required")?,
                expr(arg("id")?)?,
                name()?
            ])),
            _ if dest.is_some() => {
                Err("assigned set/eval/close record requires a richer adapter".into())
            }
            "session_set" => Ok(json!([
                "set",
                expr(arg("id")?)?,
                name()?,
                expr(arg("value")?)?
            ])),
            "session_eval" => Ok(json!([
                "eval",
                expr(arg("id")?)?,
                expr(arg("source")?)?,
                args.get("input")
                    .map(|v| expr(v))
                    .transpose()?
                    .unwrap_or(json!(["literal", null])),
                args.get("stdin").map(|v| expr(v)).transpose()?
            ])),
            "session_close" => Ok(json!(["close", expr(arg("id")?)?])),
            _ => unreachable!(),
        }
    } else {
        let f = defs
            .iter()
            .find(|f| f.name == c.callee)
            .ok_or_else(|| format!("unknown function {}", c.callee))?;
        let mut values = std::collections::BTreeMap::new();
        let mut positional = 0;
        for a in &c.args {
            let (name, value) = match a {
                Arg::Named { name, value } => (name.as_str(), value),
                Arg::Positional(value) => {
                    let p = f.params.get(positional).ok_or("too many arguments")?;
                    positional += 1;
                    (p.name.as_str(), value)
                }
            };
            if !f.params.iter().any(|p| p.name == name) || values.insert(name, value).is_some() {
                return Err("invalid call argument".into());
            }
        }
        let ordered = f
            .params
            .iter()
            .map(|p| expr(values.get(p.name.as_str()).ok_or("missing parameter")?))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(json!(["call", dest, c.callee, ordered]))
    }
}

fn block(body: &[Stmt], defs: &[&Function]) -> Result<Vec<Json>, String> {
    body.iter()
        .map(|s| {
            Ok(match s {
                Stmt::Assign {
                    name,
                    value: Expr::Call(c),
                    ..
                } => call(c, Some(name), defs)?,
                Stmt::Assign { name, value, .. } => json!(["assign", name, expr(value)?]),
                Stmt::Return { value, .. } => json!(["return", expr(value)?]),
                Stmt::Call { call: c, .. } => call(c, None, defs)?,
                Stmt::While {
                    condition, body, ..
                } => json!(["while", expr(condition)?, block(body, defs)?]),
                Stmt::Branch { arms, .. } => {
                    let mut rest = Vec::new();
                    for (i, arm) in arms.iter().enumerate().rev() {
                        let body = block(&arm.body, defs)?;
                        if let Some(condition) = &arm.condition {
                            rest = vec![json!(["if", expr(condition)?, body, rest])];
                        } else {
                            if i != arms.len() - 1 {
                                return Err("nonterminal else".into());
                            }
                            rest = body;
                        }
                    }
                    if rest.len() != 1 || rest[0][0] != "if" {
                        return Err("branch needs condition".into());
                    }
                    rest.pop().unwrap()
                }
                _ => return Err(format!("unsupported source statement {s:?}")),
            })
        })
        .collect()
}

fn source_ir(module: &Module) -> Result<Json, String> {
    if module.functions.len() != 1 || !module.uses.is_empty() {
        return Err("module needs the declared single-entry adapter".into());
    }
    let main = module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .ok_or("missing main")?;
    let defs = main.children.iter().collect::<Vec<_>>();
    if !main.params.is_empty() {
        return Err("entry parameters need an explicit environment adapter".into());
    }
    let mut seen = std::collections::HashSet::new();
    for f in &defs {
        if !seen.insert(&f.name)
            || !f.children.is_empty()
            || f.params.iter().any(|p| p.default.is_some())
        {
            return Err("unsupported function scope/default".into());
        }
    }
    let functions = defs.iter().map(|f| Ok(json!({"name":f.name,"params":f.params.iter().map(|p|p.name.clone()).collect::<Vec<_>>(),"body":block(&f.body,&defs)?}))).collect::<Result<Vec<_>,String>>()?;
    let initial = module
        .metadata
        .iter()
        .map(|(name, value)| {
            if !matches!(
                value,
                Value::None | Value::Bool(_) | Value::Int(_) | Value::Text(_)
            ) {
                return Err("non-scalar metadata needs an explicit initial-value adapter".into());
            }
            Ok(json!([name, value_to_json(value)?]))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"initial":initial,"entry":block(&main.body,&defs)?,"functions":functions}))
}

fn source(lang: &str, seed: i64, rounds: usize, scenario: &str) -> String {
    let increment = "n += 1; n";
    let bad = "n = 99; 9007199254740992";
    let stdin = if scenario == "explicit_empty_stdin" {
        " stdin=\"\""
    } else {
        ""
    };
    let condition = if scenario == "else_branch" {
        seed - 1
    } else {
        seed + 1
    };
    let function = match scenario {
        "recursive" => "advance_rec",
        "early_return" => "early",
        _ => "adjust",
    };
    let failure = match scenario {
        "closed_get" => "**> foreign.session_close id=`alias`**\n**lost = > foreign.session_get id=`alias` name=\"n\"**".into(),
        "copy_handle" => "**> foreign.session_set id=`a` name=\"n\" value=`b`**".into(),
        "closed_copy" => "**> foreign.session_close id=`alias`**\n**> foreign.session_set id=`alias` name=\"n\" value=`b`**".into(),
        "wide_input" => "**> foreign.session_set id=`a` name=\"n\" value=9007199254740992**".into(),
        "wide_output" => format!("**> foreign.session_eval id=`a` source=\"{bad}\"**"),
        _ => String::new(),
    };
    format!(
        r#"---
markup: occlusion
import:
  foreign: lib/foreign.mq.md
---
# main
**a = > foreign.session_open lang="{lang}"**
**b = > foreign.session_open lang="{lang}"**
**> foreign.session_set id=`a` name="n" value={seed}**
**> foreign.session_set id=`b` name="n" value=100**
**alias = `a`**
**counter = 777**
**before = > foreign.session_get id=`a` name="n"**
{failure}

1. `before` < {condition}
  **answer = > {function} s=`alias` rounds={rounds}**
2. *
  **> foreign.session_set id=`a` name="n" value={else_value}**

**after = > foreign.session_get id=`a` name="n"**
**peer = > foreign.session_get id=`b` name="n"**
**> foreign.session_close id=`alias`**
**c = > foreign.session_open lang="{lang}"**
**> foreign.session_set id=`c` name="n" value=7**
**fresh = > foreign.session_get id=`c` name="n"**
**> foreign.session_close id=`b`**
**> foreign.session_close id=`c`**
**total = after + peer + fresh + counter**
*total*

## adjust
    + `s`
    + `rounds`
**counter = 0**
- `counter` < `rounds`
  > bump s=`s`
  **counter = counter + 1**

**answer = > foreign.session_get id=`s` name="n"**
*answer*

## bump
    + `s`
> foreign.session_eval id=`s` source="{increment}"{stdin}

## advance_rec
    + `s`
    + `rounds`
1. 0 < `rounds`
  > bump s=`s`
  **remaining = rounds - 1**
  **answer = > advance_rec s=`s` rounds=`remaining`**
  *answer*
2. *
  **answer = > foreign.session_get id=`s` name="n"**
  *answer*

## early
    + `s`
    + `rounds`
**counter = 0**
- `counter` < `rounds`
  **answer = > foreign.session_get id=`s` name="n"**
  *answer*

**answer = > foreign.session_get id=`s` name="n"**
*answer*
"#,
        else_value = seed + 20
    )
}

#[test]
fn parsed_source_control_matches_independent_oracles_on_both_backends() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::var_os("MARQDO_SOURCE_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/source-refinement"));
    std::fs::create_dir_all(&out).unwrap();
    let seeds = std::env::var("MARQDO_SOURCE_SEEDS")
        .unwrap_or("3".into())
        .split(',')
        .map(|x| x.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    let lengths = std::env::var("MARQDO_SOURCE_ROUNDS")
        .unwrap_or("0,3".into())
        .split(',')
        .map(|x| x.parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    let scenarios = [
        "normal",
        "recursive",
        "early_return",
        "else_branch",
        "closed_get",
        "copy_handle",
        "closed_copy",
        "wide_input",
        "wide_output",
        "explicit_empty_stdin",
    ];
    let mut count = 0;
    for lang in ["python", "node"] {
        for &seed in &seeds {
            for &rounds in &lengths {
                for scenario in scenarios {
                    let id = format!("{lang}-{seed}-{rounds}-{scenario}");
                    let text = source(lang, seed, rounds, scenario);
                    // Absolute import is only a fixture location; the actual module is loaded.
                    let text = text.replace(
                        "foreign: lib/foreign.mq.md",
                        &format!("foreign: {}", root.join("lib/foreign.mq.md").display()),
                    );
                    let path = out.join(format!("{id}.mq.md"));
                    std::fs::write(&path, &text).unwrap();
                    let module = crate::load::load_module(&path).unwrap();
                    let ir = source_ir(&module).unwrap_or_else(|e| panic!("{id}: AST export: {e}"));
                    let mut executions = Vec::new();
                    for backend in ["tree", "bytecode"] {
                        let (result, journal, values, protocol) = if backend == "tree" {
                            let mut interpreter =
                                crate::interp::Interpreter::with_capture(Some(&path), false);
                            let result = interpreter.run_module(&module);
                            let (journal, values, protocol) = interpreter.foreign_source_evidence();
                            assert!(interpreter.captured_stdout.is_empty(), "unexpected stdout");
                            (result, journal, values, protocol)
                        } else {
                            let code =
                                crate::bytecode::compile_module(Some(&path), &module).unwrap();
                            let mut vm = crate::bytecode::Vm::with_capture(Some(&path));
                            let result = vm.run(&code);
                            let (journal, values, protocol) = vm.foreign_source_evidence();
                            assert!(vm.captured_stdout.is_empty(), "unexpected stdout");
                            (result, journal, values, protocol)
                        };
                        let failure = matches!(
                            scenario,
                            "closed_get"
                                | "copy_handle"
                                | "closed_copy"
                                | "wide_input"
                                | "wide_output"
                        );
                        assert_eq!(result.is_err(), failure, "{id} {backend}: {result:?}");
                        if !failure {
                            let expected = seed
                                + if scenario == "early_return" {
                                    0
                                } else if scenario == "else_branch" {
                                    20
                                } else {
                                    rounds as i64
                                };
                            assert_eq!(
                                result.as_ref().unwrap(),
                                &Value::Int(expected + 884),
                                "{id} {backend}"
                            );
                        }
                        let error = result.as_ref().err().map(|e| e.to_string());
                        let diagnostic = result.as_ref().err().and_then(crate::diagnostics::Diagnostic::find).map(crate::diagnostics::Diagnostic::to_json);
                        let fault = if let Some(e) = &error {
                            if e.contains("not found") {
                                "unknown"
                            } else if e.contains("outside V_safe")
                                || e.contains("cannot be serialized")
                            {
                                if protocol.last().is_some_and(|p| {
                                    p.response.as_ref().is_ok_and(|reply| {
                                        reply.get("domain_error").and_then(Json::as_bool)
                                            == Some(true)
                                    })
                                }) {
                                    "outputDomain"
                                } else {
                                    "inputDomain"
                                }
                            } else {
                                panic!("{id} {backend}: unclassified error {e}")
                            }
                        } else {
                            "none"
                        };
                        let protocol = protocol.iter().map(|p| json!({"session":p.session,"request":p.request,"response":p.response})).collect::<Vec<_>>();
                        executions.push(json!({"backend":backend,"ok":result.is_ok(),"result":result.ok().map(|v|value_to_json(&v).unwrap()),"error":error,"fault":fault,"protocol":protocol,"diagnostic":diagnostic,
                "journal":value_to_json(&Value::List(journal)).unwrap(),"live_values":value_to_json(&Value::List(values)).unwrap()}));
                    }
                    std::fs::write(out.join(format!("{id}.json")),serde_json::to_vec_pretty(&json!({"version":1,"id":id,"language":lang,"seed":seed,"rounds":rounds,"scenario":scenario,"source_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"ast":ir,"executions":executions})).unwrap()).unwrap();
                    count += 1;
                }
            }
        }
    }
    println!(
        "SOURCE_METRIC:{}",
        json!({"cases":count,"executions":count*2,"scenarios":scenarios,"correct":true})
    );
}

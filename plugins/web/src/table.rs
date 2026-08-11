//! GFM table helpers + site path parsing (cells stay string literals).

use serde_json::{json, Map, Value};

const FIELD_KEYS: &[&str] = &["字段", "name", "列", "column"];
const TYPE_KEYS: &[&str] = &["类型", "type"];
const NULL_KEYS: &[&str] = &["可空", "null", "nullable"];
const FRONT_KEYS: &[&str] = &["属性", "前端变量", "front", "field"];
const BACK_KEYS: &[&str] = &["值", "后端数据库", "back", "db"];
const CSS_KEYS: &[&str] = &["样式", "绑定css样式", "css", "class", "style"];
const SRC_KEYS: &[&str] = &["组件", "导入的页面", "src", "page"];
const STYLE_KEYS: &[&str] = &["样式", "style", "class"];
const PROP_KEYS: &[&str] = &["属性", "property", "prop", "名", "name"];
const VAL_KEYS: &[&str] = &["值", "value"];

pub fn normalize_ref(s: &str) -> String {
    let s = s.trim();
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('`') {
        out.push_str(&rest[..start]);
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('`') {
            out.push_str(&rest[..end]);
            rest = &rest[end + 1..];
        } else {
            out.push('`');
            out.push_str(rest);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn pick<'a>(m: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| m.get(*k))
}

fn cell_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn as_str_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a.iter().map(cell_str).collect(),
        Value::Null => Vec::new(),
        other => vec![cell_str(other)],
    }
}

fn boolish(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => matches!(s.trim(), "1" | "true" | "True" | "yes" | "是" | "可"),
        Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    }
}

/// Schema table → `[{name,type,nullable}, …]`
pub fn as_fields(table: &Value) -> Value {
    match table {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let name = pick(m, FIELD_KEYS).map(cell_str).unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    let ty = pick(m, TYPE_KEYS)
                        .map(cell_str)
                        .unwrap_or_else(|| "text".into());
                    let nullable = pick(m, NULL_KEYS).map(boolish).unwrap_or(true);
                    out.push(json!({ "name": name, "type": ty, "nullable": nullable }));
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let names = pick(m, FIELD_KEYS).map(as_str_list).unwrap_or_default();
            let types = pick(m, TYPE_KEYS).map(as_str_list).unwrap_or_default();
            let nulls = pick(m, NULL_KEYS).map(as_str_list).unwrap_or_default();
            let mut out = Vec::new();
            for (i, name) in names.iter().enumerate() {
                if name.is_empty() {
                    continue;
                }
                let ty = types.get(i).cloned().unwrap_or_else(|| "text".into());
                let nullable = nulls
                    .get(i)
                    .map(|s| boolish(&Value::String(s.clone())))
                    .unwrap_or(true);
                out.push(json!({ "name": name, "type": ty, "nullable": nullable }));
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

/// Seed / row table → list of maps.
pub fn as_rows(table: &Value) -> Value {
    match table {
        Value::Array(a) => Value::Array(a.clone()),
        Value::Object(m) => {
            // @ / 行 / row column → list of maps
            let keys: Vec<String> = m.keys().cloned().collect();
            let row_key = ["@", "行", "row"]
                .iter()
                .find(|k| m.contains_key(**k))
                .map(|s| s.to_string());
            if let Some(rk) = row_key {
                let n = as_str_list(m.get(&rk).unwrap()).len();
                let mut out = Vec::new();
                for i in 0..n {
                    let mut row = Map::new();
                    for k in &keys {
                        if k == &rk {
                            continue;
                        }
                        let list = as_str_list(m.get(k).unwrap_or(&Value::Null));
                        row.insert(k.clone(), json!(list.get(i).cloned().unwrap_or_default()));
                    }
                    if let Some(id) = as_str_list(m.get(&rk).unwrap()).get(i) {
                        if !id.is_empty() {
                            row.insert("id".into(), json!(id));
                        }
                    }
                    out.push(Value::Object(row));
                }
                return Value::Array(out);
            }
            // Columnar GFM table: each value is a same-length array → zip into rows.
            let mut col_lens: Vec<usize> = Vec::new();
            let mut all_arrays = !m.is_empty();
            for v in m.values() {
                match v {
                    Value::Array(a) => col_lens.push(a.len()),
                    _ => {
                        all_arrays = false;
                        break;
                    }
                }
            }
            if all_arrays {
                let n = col_lens.iter().copied().max().unwrap_or(0);
                if n > 0 && col_lens.iter().all(|&l| l == n || l == 0) {
                    let mut out = Vec::new();
                    for i in 0..n {
                        let mut row = Map::new();
                        for (k, v) in m {
                            if let Value::Array(a) = v {
                                row.insert(
                                    k.clone(),
                                    a.get(i).cloned().unwrap_or(Value::Null),
                                );
                            }
                        }
                        out.push(Value::Object(row));
                    }
                    return Value::Array(out);
                }
            }
            // single map
            Value::Array(vec![Value::Object(m.clone())])
        }
        _ => json!([]),
    }
}

/// Bind table → `[{front,back,css}, …]`
pub fn as_bind(table: &Value) -> Value {
    match table {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let front = normalize_ref(&pick(m, FRONT_KEYS).map(cell_str).unwrap_or_default());
                    let back = normalize_ref(&pick(m, BACK_KEYS).map(cell_str).unwrap_or_default());
                    let css = normalize_ref(&pick(m, CSS_KEYS).map(cell_str).unwrap_or_default());
                    if !front.is_empty() || !back.is_empty() {
                        out.push(json!({ "front": front, "back": back, "css": css }));
                    }
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let fronts = pick(m, FRONT_KEYS).map(as_str_list).unwrap_or_default();
            let backs = pick(m, BACK_KEYS).map(as_str_list).unwrap_or_default();
            let csses = pick(m, CSS_KEYS).map(as_str_list).unwrap_or_default();
            let n = fronts.len().max(backs.len());
            let mut out = Vec::new();
            for i in 0..n {
                let front = normalize_ref(&fronts.get(i).cloned().unwrap_or_default());
                let back = normalize_ref(&backs.get(i).cloned().unwrap_or_default());
                let css = normalize_ref(&csses.get(i).cloned().unwrap_or_default());
                if !front.is_empty() || !back.is_empty() {
                    out.push(json!({ "front": front, "back": back, "css": css }));
                }
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

/// Page table → `[{src,style,slot}, …]`
pub fn as_compose(table: &Value) -> Value {
    match table {
        Value::Object(m) => {
            let srcs = pick(m, SRC_KEYS).map(as_str_list).unwrap_or_default();
            let styles = pick(m, STYLE_KEYS).map(as_str_list).unwrap_or_default();
            let mut out = Vec::new();
            for (i, src_raw) in srcs.iter().enumerate() {
                let src = normalize_ref(src_raw);
                if src.is_empty() {
                    continue;
                }
                let style = normalize_ref(&styles.get(i).cloned().unwrap_or_default());
                let slot = infer_slot(&src);
                out.push(json!({ "src": src, "style": style, "slot": slot }));
            }
            Value::Array(out)
        }
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let src = normalize_ref(&pick(m, SRC_KEYS).map(cell_str).unwrap_or_default());
                    if src.is_empty() {
                        continue;
                    }
                    let style = normalize_ref(&pick(m, STYLE_KEYS).map(cell_str).unwrap_or_default());
                    let slot = infer_slot(&src);
                    out.push(json!({ "src": src, "style": style, "slot": slot }));
                }
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

fn infer_slot(src: &str) -> String {
    let base = src.rsplit('.').next().unwrap_or(src);
    match base {
        "nav" | "links" => "nav".into(),
        "side" | "sidebar" => "sidebar".into(),
        "foot" | "footer" => "footer".into(),
        other => other.to_string(),
    }
}

pub fn normalize_slot(name: &str) -> String {
    match name {
        "nav" | "links" => "nav".into(),
        "side" | "sidebar" => "sidebar".into(),
        "foot" | "footer" => "footer".into(),
        "main" | "index" | "" => "main".into(),
        other => other.to_string(),
    }
}

/// Style table → CSS text for `.name { … }`
pub fn as_css_named(name: &str, table: &Value) -> String {
    let name = normalize_ref(name);
    if name.is_empty() {
        return String::new();
    }
    let mut rules = Vec::new();
    match table {
        Value::Object(m) => {
            let props = pick(m, PROP_KEYS).map(as_str_list).unwrap_or_default();
            let vals = pick(m, VAL_KEYS).map(as_str_list).unwrap_or_default();
            for (i, p) in props.iter().enumerate() {
                if p.is_empty() {
                    continue;
                }
                let v = vals.get(i).cloned().unwrap_or_default();
                rules.push(format!("  {p}: {v};"));
            }
        }
        Value::Array(rows) => {
            for row in rows {
                if let Some(m) = row.as_object() {
                    let p = pick(m, PROP_KEYS).map(cell_str).unwrap_or_default();
                    let v = pick(m, VAL_KEYS).map(cell_str).unwrap_or_default();
                    if !p.is_empty() {
                        rules.push(format!("  {p}: {v};"));
                    }
                }
            }
        }
        _ => {}
    }
    if rules.is_empty() {
        return String::new();
    }
    format!(".{} {{\n{}\n}}\n", name, rules.join("\n"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SitePath {
    LibMember { lib: String, member: String },
    DbField { table: String, field: String },
    Plain(String),
}

pub fn parse_site_path(raw: &str) -> SitePath {
    let s = normalize_ref(raw);
    if s.is_empty() {
        return SitePath::Plain(String::new());
    }
    let parts: Vec<&str> = s.split('.').filter(|p| !p.is_empty()).collect();
    match parts.as_slice() {
        [lib, member] => SitePath::LibMember {
            lib: (*lib).into(),
            member: (*member).into(),
        },
        [_, table, field] => SitePath::DbField {
            table: (*table).into(),
            field: (*field).into(),
        },
        _ => SitePath::Plain(s),
    }
}

pub fn bind_table_name(binds: &[Value]) -> Option<String> {
    let mut found: Option<String> = None;
    for b in binds {
        let back = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
        if let SitePath::DbField { table, .. } = parse_site_path(back) {
            match &found {
                None => found = Some(table),
                Some(prev) if prev != &table => return None,
                _ => {}
            }
        } else if back.contains('.') {
            let mut p = back.splitn(2, '.');
            if let (Some(t), Some(_)) = (p.next(), p.next()) {
                match &found {
                    None => found = Some(t.into()),
                    Some(prev) if prev != t => return None,
                    _ => {}
                }
            }
        }
    }
    found
}

pub fn project_rows(binds: &[Value], rows: &[Value]) -> Value {
    let mut out = Vec::new();
    for row in rows {
        let obj = row.as_object().cloned().unwrap_or_default();
        let mut m = Map::new();
        let mut css = Map::new();
        for b in binds {
            let front = b.get("front").and_then(|v| v.as_str()).unwrap_or("");
            let back = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
            let class = b.get("css").and_then(|v| v.as_str()).unwrap_or("");
            let col = match parse_site_path(back) {
                SitePath::DbField { field, .. } => field,
                SitePath::Plain(s) if s.contains('.') => {
                    s.split('.').next_back().unwrap_or(&s).to_string()
                }
                SitePath::Plain(s) => s,
                SitePath::LibMember { .. } => back.to_string(),
            };
            if let Some(v) = obj.get(&col).or_else(|| obj.get(front)) {
                m.insert(front.to_string(), v.clone());
            }
            if !class.is_empty() {
                css.insert(front.to_string(), json!(class));
            }
        }
        if let Some(id) = obj.get("id") {
            m.insert("id".into(), id.clone());
        }
        if !css.is_empty() {
            m.insert("_css".into(), Value::Object(css));
        }
        out.push(Value::Object(m));
    }
    Value::Array(out)
}

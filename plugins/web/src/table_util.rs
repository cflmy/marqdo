//! Convert Marqdo GFM table values (list-of-maps or column-oriented map) into
//! nav links / schema fields — so authors write readable tables, not JSON.

use serde_json::{json, Map, Value};

const LABEL_KEYS: &[&str] = &[
    "页面导航",
    "导航",
    "label",
    "标签",
    "title",
    "标题",
    "名",
    "name",
];
const HREF_KEYS: &[&str] = &[
    "对应路由",
    "路由",
    "href",
    "链接",
    "路径",
    "path",
    "url",
];
const FIELD_KEYS: &[&str] = &["字段", "name", "列", "名", "column"];
const TYPE_KEYS: &[&str] = &["类型", "type"];
const NULL_KEYS: &[&str] = &["可空", "null", "nullable"];
const DEFAULT_KEYS: &[&str] = &["默认", "default"];
const UNIQUE_KEYS: &[&str] = &["唯一", "unique"];

/// Unified UI bind: frontend var ↔ optional DB path / href ↔ optional CSS.
const FRONT_KEYS: &[&str] = &["前端变量", "front", "变量", "名", "field"];
const BACK_KEYS: &[&str] = &[
    "后端数据库",
    "back",
    "数据库",
    "字段绑定",
    "db",
    "绑定",
];
const CSS_KEYS: &[&str] = &["绑定css样式", "css", "样式", "class", "style", "classes"];

fn pick<'a>(m: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for k in keys {
        if let Some(v) = m.get(*k) {
            return Some(v);
        }
    }
    None
}

fn as_str_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|x| match x {
                Value::String(s) => s.clone(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::Null => String::new(),
                other => other.to_string(),
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        Value::Null => Vec::new(),
        other => vec![other.to_string()],
    }
}

fn cell_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn boolish(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => matches!(
            s.trim(),
            "1" | "true" | "True" | "yes" | "是" | "可" | "nullable"
        ),
        Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    }
}

fn ident_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `table.column` DB reference (not a URL path).
pub fn is_db_ref(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.starts_with('/') || s.contains("://") {
        return false;
    }
    let mut parts = s.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(t), Some(c), None) => ident_ok(t) && ident_ok(c),
        _ => false,
    }
}

pub fn parse_db_ref(s: &str) -> Option<(String, String)> {
    if !is_db_ref(s) {
        return None;
    }
    let mut parts = s.trim().splitn(2, '.');
    Some((parts.next()?.to_string(), parts.next()?.to_string()))
}

fn bind_row(front: String, back: String, css: String) -> Value {
    json!({ "front": front, "back": back, "css": css })
}

fn looks_like_bind_map(m: &Map<String, Value>) -> bool {
    pick(m, FRONT_KEYS).is_some()
}

/// Turn the unified 3-col bind table into `[{front, back, css}, ...]`.
/// Also accepts legacy nav tables (`页面导航` / `对应路由`) as static link binds.
pub fn as_bind(table: &Value) -> Value {
    match table {
        Value::Null => json!([]),
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    if looks_like_bind_map(m) {
                        let front = pick(m, FRONT_KEYS).map(cell_str).unwrap_or_default();
                        let back = pick(m, BACK_KEYS).map(cell_str).unwrap_or_default();
                        let css = pick(m, CSS_KEYS).map(cell_str).unwrap_or_default();
                        if !front.is_empty() || !back.is_empty() {
                            out.push(bind_row(front, back, css));
                        }
                    } else if pick(m, LABEL_KEYS).is_some() || pick(m, HREF_KEYS).is_some() {
                        let front = pick(m, LABEL_KEYS).map(cell_str).unwrap_or_default();
                        let back = pick(m, HREF_KEYS)
                            .map(cell_str)
                            .unwrap_or_else(|| "#".into());
                        out.push(bind_row(front, back, String::new()));
                    }
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            if m.get("_type").and_then(|v| v.as_str()) == Some("live") {
                return json!([]);
            }
            if looks_like_bind_map(m) {
                let fronts = pick(m, FRONT_KEYS)
                    .map(as_str_list)
                    .unwrap_or_default();
                let backs = pick(m, BACK_KEYS).map(as_str_list).unwrap_or_default();
                let csses = pick(m, CSS_KEYS).map(as_str_list).unwrap_or_default();
                let n = fronts.len().max(backs.len()).max(csses.len());
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let front = fronts.get(i).cloned().unwrap_or_default();
                    let back = backs.get(i).cloned().unwrap_or_default();
                    let css = csses.get(i).cloned().unwrap_or_default();
                    if front.is_empty() && back.is_empty() {
                        continue;
                    }
                    out.push(bind_row(front, back, css));
                }
                return Value::Array(out);
            }
            // Legacy column nav → bind rows (require label column; bare `url` is not nav)
            let labels = pick(m, LABEL_KEYS)
                .map(as_str_list)
                .unwrap_or_default();
            let hrefs = pick(m, HREF_KEYS)
                .map(as_str_list)
                .unwrap_or_default();
            if !labels.is_empty() {
                let n = labels.len().max(hrefs.len());
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let front = labels.get(i).cloned().unwrap_or_default();
                    let back = hrefs.get(i).cloned().unwrap_or_else(|| "#".into());
                    out.push(bind_row(front, back, String::new()));
                }
                return Value::Array(out);
            }
            json!([])
        }
        _ => json!([]),
    }
}

/// True when any bind row points at `table.column`.
pub fn bind_has_db(binds: &[Value]) -> bool {
    binds.iter().any(|b| {
        b.get("back")
            .and_then(|v| v.as_str())
            .map(is_db_ref)
            .unwrap_or(false)
    })
}

/// Infer the single DB table name from bind rows (`posts.title` → `posts`).
pub fn bind_table_name(binds: &[Value]) -> Option<String> {
    let mut found: Option<String> = None;
    for b in binds {
        let back = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
        if let Some((t, _)) = parse_db_ref(back) {
            match &found {
                None => found = Some(t),
                Some(prev) if prev != &t => return None, // mixed tables
                _ => {}
            }
        }
    }
    found
}

/// Static link region: each bind row is one link (`front`=label, `back`=href).
pub fn binds_as_static_links(binds: &[Value]) -> Value {
    let mut out = Vec::new();
    for b in binds {
        let label = b
            .get("front")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let href = b
            .get("back")
            .and_then(|v| v.as_str())
            .map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    "#"
                } else {
                    s
                }
            })
            .unwrap_or("#")
            .to_string();
        let css = b
            .get("css")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if label.is_empty() && href == "#" {
            continue;
        }
        out.push(json!({ "label": label, "href": href, "css": css }));
    }
    Value::Array(out)
}

/// Loop DB rows through field binds → `[{front: value, ..., _css: {front: class}}]`.
pub fn project_rows(binds: &[Value], rows: &[Value]) -> Value {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let Some(obj) = row.as_object() else {
            continue;
        };
        let mut item = Map::new();
        let mut css_map = Map::new();
        for b in binds {
            let front = b
                .get("front")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if front.is_empty() {
                continue;
            }
            let back = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
            let css = b.get("css").and_then(|v| v.as_str()).unwrap_or("");
            let val = if let Some((_, col)) = parse_db_ref(back) {
                obj.get(&col).cloned().unwrap_or(Value::Null)
            } else if back.is_empty() {
                Value::String(front.clone())
            } else if ident_ok(back) {
                obj.get(back).cloned().unwrap_or(Value::Null)
            } else {
                Value::String(back.to_string())
            };
            item.insert(front.clone(), val);
            if !css.is_empty() {
                css_map.insert(front, json!(css));
            }
        }
        // Keep id for meta when present on source row
        if let Some(id) = obj.get("id") {
            item.entry("id".to_string()).or_insert_with(|| id.clone());
        }
        if !css_map.is_empty() {
            item.insert("_css".into(), Value::Object(css_map));
        }
        out.push(Value::Object(item));
    }
    Value::Array(out)
}

fn looks_like_href(s: &str) -> bool {
    let s = s.trim();
    s.starts_with('/')
        || s.starts_with('#')
        || s.starts_with("http://")
        || s.starts_with("https://")
}

/// When `page.table=` is set, bare column backs become `table.column`.
/// There is **no** default table name — callers must pass one.
pub fn qualify_binds(binds: &[Value], table: Option<&str>) -> Vec<Value> {
    let Some(t) = table.map(str::trim).filter(|s| !s.is_empty()) else {
        return binds.to_vec();
    };
    if !ident_ok(t) {
        return binds.to_vec();
    }
    binds
        .iter()
        .map(|b| {
            let front = b.get("front").cloned().unwrap_or(json!(""));
            let css = b.get("css").cloned().unwrap_or(json!(""));
            let back = b
                .get("back")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let back = if back.is_empty() || is_db_ref(&back) || looks_like_href(&back) {
                back
            } else if ident_ok(&back) {
                format!("{t}.{back}")
            } else {
                back
            };
            json!({ "front": front, "back": back, "css": css })
        })
        .collect()
}

const ROW_INDEX_KEYS: &[&str] = &["@", "行", "row"];

/// GFM data table → `[{col: val}, …]` for inserts.
/// Accepts: list-of-maps (`@`/`行` geometry), column-oriented map (dict of lists), or one map row.
pub fn as_rows(table: &Value) -> Value {
    match table {
        Value::Null => json!([]),
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let mut clean = Map::new();
                    for (k, v) in m {
                        if ROW_INDEX_KEYS.contains(&k.as_str()) {
                            continue;
                        }
                        clean.insert(k.clone(), v.clone());
                    }
                    if !clean.is_empty() {
                        out.push(Value::Object(clean));
                    }
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let keys: Vec<String> = m
                .keys()
                .filter(|k| !ROW_INDEX_KEYS.contains(&k.as_str()))
                .cloned()
                .collect();
            if keys.is_empty() {
                return json!([]);
            }
            let any_list = keys.iter().any(|k| m.get(k).is_some_and(|v| v.is_array()));
            if any_list {
                let cols: Vec<(String, Vec<String>)> = keys
                    .iter()
                    .map(|k| (k.clone(), as_str_list(m.get(k).unwrap_or(&Value::Null))))
                    .collect();
                let n = cols.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let mut row = Map::new();
                    for (k, vals) in &cols {
                        row.insert(
                            k.clone(),
                            json!(vals.get(i).cloned().unwrap_or_default()),
                        );
                    }
                    out.push(Value::Object(row));
                }
                Value::Array(out)
            } else {
                let mut row = Map::new();
                for k in keys {
                    if let Some(v) = m.get(&k) {
                        row.insert(k, v.clone());
                    }
                }
                json!([Value::Object(row)])
            }
        }
        _ => json!([]),
    }
}

/// Turn a nav/side/footer table into `[{label, href}, ...]`.
pub fn as_links(table: &Value) -> Value {
    let binds = as_bind(table);
    let arr = binds.as_array().cloned().unwrap_or_default();
    if bind_has_db(&arr) {
        // Without DB handle here, return empty — callers should use project_rows.
        return json!([]);
    }
    binds_as_static_links(&arr)
}

/// Turn a schema table into `[{name, type, null, ...}, ...]`.
pub fn as_fields(table: &Value) -> Value {
    match table {
        Value::Null => json!([]),
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    if let Some(f) = field_from_map(m) {
                        out.push(f);
                    }
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let names = pick(m, FIELD_KEYS)
                .map(as_str_list)
                .unwrap_or_default();
            let types = pick(m, TYPE_KEYS)
                .map(as_str_list)
                .unwrap_or_default();
            let nulls = pick(m, NULL_KEYS).map(as_str_list).unwrap_or_default();
            let defaults = pick(m, DEFAULT_KEYS)
                .map(as_str_list)
                .unwrap_or_default();
            let uniques = pick(m, UNIQUE_KEYS)
                .map(as_str_list)
                .unwrap_or_default();
            if names.is_empty() {
                return json!([]);
            }
            let n = names.len();
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                let name = &names[i];
                if name.is_empty() || name == "@" || name == "行" || name == "row" {
                    continue;
                }
                let ty = types.get(i).cloned().unwrap_or_else(|| "text".into());
                let nullable = nulls
                    .get(i)
                    .map(|s| {
                        matches!(
                            s.trim(),
                            "1" | "true" | "True" | "yes" | "是" | "可" | "nullable"
                        )
                    })
                    .unwrap_or(true);
                let mut field = json!({
                    "name": name,
                    "type": ty,
                    "null": nullable,
                });
                if let Some(d) = defaults.get(i) {
                    if !d.is_empty() {
                        field
                            .as_object_mut()
                            .unwrap()
                            .insert("default".into(), json!(d));
                    }
                }
                if let Some(u) = uniques.get(i) {
                    let uniq = matches!(u.trim(), "1" | "true" | "True" | "yes" | "是" | "唯一");
                    field
                        .as_object_mut()
                        .unwrap()
                        .insert("unique".into(), json!(uniq));
                }
                out.push(field);
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

fn field_from_map(m: &Map<String, Value>) -> Option<Value> {
    let name = pick(m, FIELD_KEYS).map(cell_str)?;
    if name.is_empty() || name == "@" || name == "行" || name == "row" {
        return None;
    }
    let ty = pick(m, TYPE_KEYS)
        .map(cell_str)
        .unwrap_or_else(|| "text".into());
    let nullable = pick(m, NULL_KEYS).map(boolish).unwrap_or(true);
    let mut field = json!({
        "name": name,
        "type": ty,
        "null": nullable,
    });
    if let Some(d) = pick(m, DEFAULT_KEYS) {
        field
            .as_object_mut()
            .unwrap()
            .insert("default".into(), d.clone());
    }
    if let Some(u) = pick(m, UNIQUE_KEYS) {
        field
            .as_object_mut()
            .unwrap()
            .insert("unique".into(), json!(boolish(u)));
    }
    Some(field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_nav_table() {
        let t = json!({
            "页面导航": ["首页", "管理"],
            "对应路由": ["/", "/admin"],
        });
        let links = as_links(&t);
        assert_eq!(links.as_array().unwrap().len(), 2);
        assert_eq!(links[0]["label"], "首页");
        assert_eq!(links[0]["href"], "/");
        assert_eq!(links[1]["label"], "管理");
    }

    #[test]
    fn column_schema_table() {
        let t = json!({
            "字段": ["id", "title"],
            "类型": ["integer", "text"],
            "可空": ["false", "false"],
        });
        let fields = as_fields(&t);
        assert_eq!(fields[0]["name"], "id");
        assert_eq!(fields[0]["type"], "integer");
        assert_eq!(fields[0]["null"], false);
    }

    #[test]
    fn column_bind_table() {
        let t = json!({
            "前端变量": ["title", "body"],
            "后端数据库": ["posts.title", "posts.body"],
            "绑定css样式": ["card-title", "card-body"],
        });
        let binds = as_bind(&t);
        assert_eq!(binds.as_array().unwrap().len(), 2);
        assert_eq!(binds[0]["front"], "title");
        assert_eq!(binds[0]["back"], "posts.title");
        assert_eq!(binds[0]["css"], "card-title");
        assert_eq!(bind_table_name(binds.as_array().unwrap()).as_deref(), Some("posts"));
    }

    #[test]
    fn static_nav_via_bind() {
        let t = json!({
            "前端变量": ["首页", "管理"],
            "后端数据库": ["/", "/admin"],
            "绑定css样式": ["", ""],
        });
        let links = as_links(&t);
        assert_eq!(links[0]["label"], "首页");
        assert_eq!(links[0]["href"], "/");
    }

    #[test]
    fn project_rows_applies_css() {
        let binds = as_bind(&json!({
            "前端变量": ["title", "body"],
            "后端数据库": ["posts.title", "posts.body"],
            "绑定css样式": ["t", "b"],
        }));
        let rows = json!([{"id": 1, "title": "Hello", "body": "World"}]);
        let out = project_rows(
            binds.as_array().unwrap(),
            rows.as_array().unwrap(),
        );
        assert_eq!(out[0]["title"], "Hello");
        assert_eq!(out[0]["_css"]["title"], "t");
        assert_eq!(out[0]["_css"]["body"], "b");
    }

    #[test]
    fn qualify_bare_columns() {
        let binds = as_bind(&json!({
            "前端变量": ["title"],
            "后端数据库": ["title"],
            "绑定css样式": ["t"],
        }));
        let q = qualify_binds(binds.as_array().unwrap(), Some("articles"));
        assert_eq!(q[0]["back"], "articles.title");
    }

    #[test]
    fn as_rows_column_dict() {
        let t = json!({
            "title": ["hello", "second"],
            "body": ["a", "b"],
        });
        let rows = as_rows(&t);
        assert_eq!(rows.as_array().unwrap().len(), 2);
        assert_eq!(rows[0]["title"], "hello");
        assert_eq!(rows[1]["body"], "b");
    }
}

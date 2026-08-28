//! Form fields / validate / render / submit (design §5.5).

use serde_json::{json, Map, Value};

use crate::db;
use crate::table::normalize_ref;

const FIELD_KEYS: &[&str] = &["字段", "name", "field", "列"];
const LABEL_KEYS: &[&str] = &["标签", "label"];
const TYPE_KEYS: &[&str] = &["类型", "type"];
const REQ_KEYS: &[&str] = &["必填", "required"];
const DEF_KEYS: &[&str] = &["默认", "default"];
const RULE_FIELD_KEYS: &[&str] = &["字段", "field", "name"];
const RULE_KEYS: &[&str] = &["规则", "rule"];
const MSG_KEYS: &[&str] = &["消息", "message", "msg"];

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

fn pick<'a>(m: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| m.get(*k))
}

fn boolish(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => matches!(
            s.trim(),
            "1" | "true" | "True" | "yes" | "是" | "必填" | "required"
        ),
        Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Normalize author field table → `[{name,label,type,required,default}, …]`
pub fn as_form_fields(table: &Value) -> Value {
    match table {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let name = normalize_ref(&pick(m, FIELD_KEYS).map(cell_str).unwrap_or_default());
                    if name.is_empty() {
                        continue;
                    }
                    let label = pick(m, LABEL_KEYS).map(cell_str).unwrap_or_else(|| name.clone());
                    let ty = pick(m, TYPE_KEYS)
                        .map(cell_str)
                        .unwrap_or_else(|| "text".into());
                    let required = pick(m, REQ_KEYS).map(boolish).unwrap_or(false);
                    let default = pick(m, DEF_KEYS).map(cell_str).unwrap_or_default();
                    out.push(json!({
                        "name": name,
                        "label": label,
                        "type": ty,
                        "required": required,
                        "default": default,
                    }));
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let names = pick(m, FIELD_KEYS).map(as_str_list).unwrap_or_default();
            let labels = pick(m, LABEL_KEYS).map(as_str_list).unwrap_or_default();
            let types = pick(m, TYPE_KEYS).map(as_str_list).unwrap_or_default();
            let reqs = pick(m, REQ_KEYS).map(as_str_list).unwrap_or_default();
            let defs = pick(m, DEF_KEYS).map(as_str_list).unwrap_or_default();
            let mut out = Vec::new();
            for (i, name_raw) in names.iter().enumerate() {
                let name = normalize_ref(name_raw);
                if name.is_empty() {
                    continue;
                }
                let label = labels.get(i).cloned().unwrap_or_else(|| name.clone());
                let ty = types.get(i).cloned().unwrap_or_else(|| "text".into());
                let required = reqs
                    .get(i)
                    .map(|s| boolish(&Value::String(s.clone())))
                    .unwrap_or(false);
                let default = defs.get(i).cloned().unwrap_or_default();
                out.push(json!({
                    "name": name,
                    "label": label,
                    "type": ty,
                    "required": required,
                    "default": default,
                }));
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

/// Normalize rules table → `[{field,rule,message}, …]`
pub fn as_form_rules(table: &Value) -> Value {
    match table {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let field =
                        normalize_ref(&pick(m, RULE_FIELD_KEYS).map(cell_str).unwrap_or_default());
                    let rule = pick(m, RULE_KEYS).map(cell_str).unwrap_or_default();
                    if field.is_empty() || rule.is_empty() {
                        continue;
                    }
                    let message = pick(m, MSG_KEYS).map(cell_str).unwrap_or_default();
                    out.push(json!({ "field": field, "rule": rule, "message": message }));
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let fields = pick(m, RULE_FIELD_KEYS).map(as_str_list).unwrap_or_default();
            let rules = pick(m, RULE_KEYS).map(as_str_list).unwrap_or_default();
            let msgs = pick(m, MSG_KEYS).map(as_str_list).unwrap_or_default();
            let n = fields.len().max(rules.len());
            let mut out = Vec::new();
            for i in 0..n {
                let field = normalize_ref(&fields.get(i).cloned().unwrap_or_default());
                let rule = rules.get(i).cloned().unwrap_or_default();
                if field.is_empty() || rule.is_empty() {
                    continue;
                }
                let message = msgs.get(i).cloned().unwrap_or_default();
                out.push(json!({ "field": field, "rule": rule, "message": message }));
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

pub fn form_new(table: Option<&str>, action: &str, id: Option<&str>) -> Value {
    let action = match action {
        "update" | "更新" => "update",
        _ => "insert",
    };
    let mut m = Map::new();
    if let Some(t) = table.filter(|s| !s.is_empty()) {
        m.insert("table".into(), json!(t));
    }
    m.insert("action".into(), json!(action));
    if let Some(i) = id.filter(|s| !s.is_empty()) {
        m.insert("id".into(), json!(i));
    }
    m.insert("fields".into(), json!([]));
    m.insert("rules".into(), json!([]));
    m.insert("redirect".into(), json!("/"));
    Value::Object(m)
}

fn sql_type_to_input(sql_type: &str, name: &str) -> &'static str {
    let t = sql_type.to_ascii_uppercase();
    if t.contains("INT") || t.contains("REAL") || t.contains("FLOAT") || t.contains("DOUBLE") {
        return "number";
    }
    if name == "body" || name.ends_with("_body") || name.ends_with("_text") {
        return "textarea";
    }
    if name.contains("email") {
        return "email";
    }
    if name.contains("url") || name == "href" {
        return "url";
    }
    "text"
}

/// Build a form handle from live SQLite schema (admin new/edit).
pub fn from_schema(
    db_url: &str,
    table: &str,
    action: &str,
    id: Option<&str>,
) -> Result<Value, String> {
    let action = match action {
        "update" | "更新" => "update",
        _ => "insert",
    };
    let cols = db::table_info(db_url, table)?;
    if cols.is_empty() {
        return Err(format!("unknown table `{table}`"));
    }
    let mut fields = Vec::new();
    for c in &cols {
        if action == "insert" && c.pk && c.name == "id" {
            continue; // SERIAL / AUTOINCREMENT
        }
        let input = sql_type_to_input(&c.sql_type, &c.name);
        let required = c.notnull || (action == "update" && c.pk);
        fields.push(json!({
            "name": c.name,
            "label": c.name,
            "type": input,
            "required": required,
            "default": "",
            "pk": c.pk,
        }));
    }
    let mut form = form_new(Some(table), action, id);
    if let Some(obj) = form.as_object_mut() {
        obj.insert("fields".into(), Value::Array(fields));
        obj.insert("redirect".into(), json!(format!("/admin/{table}")));
        if action == "insert" {
            obj.insert("action_url".into(), json!(format!("/admin/{table}/new")));
            obj.insert("title".into(), json!(format!("New {table}")));
        } else if let Some(i) = id {
            obj.insert(
                "action_url".into(),
                json!(format!("/admin/{table}/{i}/edit")),
            );
            obj.insert("title".into(), json!(format!("Edit {table} #{i}")));
            obj.insert("id".into(), json!(i));
        }
        obj.insert("cancel_href".into(), json!(format!("/admin/{table}")));
    }
    Ok(form)
}

pub fn set_fields(form: &Value, fields: &Value) -> Value {
    let mut obj = form.as_object().cloned().unwrap_or_default();
    obj.insert("fields".into(), as_form_fields(fields));
    Value::Object(obj)
}

pub fn set_rules(form: &Value, rules: &Value) -> Value {
    let mut obj = form.as_object().cloned().unwrap_or_default();
    obj.insert("rules".into(), as_form_rules(rules));
    Value::Object(obj)
}

fn data_map(data: &Value) -> Map<String, Value> {
    let rows = crate::table::as_rows(data);
    if let Some(Value::Object(m)) = rows.as_array().and_then(|a| a.first()) {
        return m.clone();
    }
    match data {
        Value::Object(m) => m.clone(),
        _ => Map::new(),
    }
}

fn field_text(data: &Map<String, Value>, name: &str) -> String {
    data.get(name).map(cell_str).unwrap_or_default()
}

fn default_message(rule: &str, field: &str) -> String {
    if let Some(rest) = rule.strip_prefix("min:") {
        return format!("{field} must be at least {rest}");
    }
    if let Some(rest) = rule.strip_prefix("max:") {
        return format!("{field} must be at most {rest}");
    }
    if let Some(other) = rule.strip_prefix("match:") {
        return format!("{field} must match {other}");
    }
    if rule.starts_with("in:") {
        return format!("{field} is not an allowed value");
    }
    match rule {
        "required" => format!("{field} is required"),
        "email" => format!("{field} must be an email"),
        "url" => format!("{field} must be a URL"),
        _ => format!("{field} failed `{rule}`"),
    }
}

fn check_rule(rule: &str, field: &str, data: &Map<String, Value>) -> Option<String> {
    let val = field_text(data, field);
    let trimmed = val.trim();
    if rule == "required" {
        return if trimmed.is_empty() {
            Some(default_message(rule, field))
        } else {
            None
        };
    }
    if trimmed.is_empty() {
        return None; // other rules skip empty (required covers emptiness)
    }
    if let Some(n) = rule.strip_prefix("min:") {
        if let Ok(min) = n.parse::<i64>() {
            if let Ok(num) = trimmed.parse::<f64>() {
                if num < min as f64 {
                    return Some(default_message(rule, field));
                }
            } else if (trimmed.chars().count() as i64) < min {
                return Some(default_message(rule, field));
            }
        }
        return None;
    }
    if let Some(n) = rule.strip_prefix("max:") {
        if let Ok(max) = n.parse::<i64>() {
            if let Ok(num) = trimmed.parse::<f64>() {
                if num > max as f64 {
                    return Some(default_message(rule, field));
                }
            } else if (trimmed.chars().count() as i64) > max {
                return Some(default_message(rule, field));
            }
        }
        return None;
    }
    if let Some(other) = rule.strip_prefix("match:") {
        let other_val = field_text(data, other);
        if val != other_val {
            return Some(default_message(rule, field));
        }
        return None;
    }
    if let Some(list) = rule.strip_prefix("in:") {
        let allowed: Vec<&str> = list.split(',').map(|s| s.trim()).collect();
        if !allowed.contains(&trimmed) {
            return Some(default_message(rule, field));
        }
        return None;
    }
    match rule {
        "email" => {
            if !(trimmed.contains('@') && trimmed.contains('.')) {
                Some(default_message(rule, field))
            } else {
                None
            }
        }
        "url" => {
            if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
                Some(default_message(rule, field))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Merge field-required + author rules; return `{ok, errors:[{field,message}]}`.
pub fn validate(form: &Value, extra_rules: Option<&Value>, data: &Value) -> Value {
    let data = data_map(data);
    let fields = form
        .get("fields")
        .cloned()
        .unwrap_or(json!([]));
    let mut rules = form
        .get("rules")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if let Some(extra) = extra_rules {
        if let Some(arr) = as_form_rules(extra).as_array() {
            rules.extend(arr.clone());
        }
    }
    // Auto required from field table (message overridable by later explicit rules)
    if let Some(arr) = fields.as_array() {
        for f in arr {
            let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let required = f.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
            if required && !name.is_empty() {
                let already = rules.iter().any(|r| {
                    r.get("field").and_then(|v| v.as_str()) == Some(name)
                        && r.get("rule").and_then(|v| v.as_str()) == Some("required")
                });
                if !already {
                    rules.insert(
                        0,
                        json!({
                            "field": name,
                            "rule": "required",
                            "message": format!("{name} is required"),
                        }),
                    );
                }
            }
        }
    }

    let mut errors = Vec::new();
    for r in &rules {
        let field = r.get("field").and_then(|v| v.as_str()).unwrap_or("");
        let rule = r.get("rule").and_then(|v| v.as_str()).unwrap_or("");
        let msg = r.get("message").and_then(|v| v.as_str()).unwrap_or("");
        if field.is_empty() || rule.is_empty() {
            continue;
        }
        if let Some(default_msg) = check_rule(rule, field, &data) {
            errors.push(json!({
                "field": field,
                "message": if msg.is_empty() { default_msg } else { msg.to_string() },
            }));
        }
    }
    json!({
        "ok": errors.is_empty(),
        "errors": errors,
    })
}

/// Form markup only (no document shell) — for page main-slot embed and `/_form`.
pub fn render_body(
    form: &Value,
    form_id: &str,
    data: Option<&Value>,
    errors: Option<&Value>,
    csrf: Option<&str>,
) -> String {
    let fields = form
        .get("fields")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let data = data.map(data_map).unwrap_or_default();
    let mut err_map = Map::new();
    if let Some(Value::Array(errs)) = errors {
        for e in errs {
            if let (Some(f), Some(m)) = (
                e.get("field").and_then(|v| v.as_str()),
                e.get("message").and_then(|v| v.as_str()),
            ) {
                err_map.insert(f.to_string(), json!(m));
            }
        }
    } else if let Some(Value::Object(m)) = errors {
        err_map = m.clone();
    }

    let action = form
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("insert");
    let table = form
        .get("table")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let heading = form
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(form_id);
    let post_to = form
        .get("action_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("/_form/{form_id}"));
    let cancel = form
        .get("cancel_href")
        .and_then(|v| v.as_str())
        .unwrap_or("/");
    let row_id = form.get("id").map(cell_str).unwrap_or_default();
    let show_meta = form
        .get("show_meta")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut body = String::from("<div class=\"site-form\">");
    if show_meta {
        body.push_str(&format!(
            "<p class=\"meta\">table=<code>{}</code> · action=<code>{}</code> · id=<code>{}</code></p>",
            esc(table),
            esc(action),
            esc(heading)
        ));
    }
    body.push_str(&format!(
        "<form method=\"post\" action=\"{}\"{}>",
        esc(&post_to),
        if fields.iter().any(|f| {
            f.get("type")
                .and_then(|v| v.as_str())
                .is_some_and(|t| t.eq_ignore_ascii_case("file"))
        }) {
            " enctype=\"multipart/form-data\""
        } else {
            ""
        }
    ));
    if let Some(token) = csrf.filter(|s| !s.is_empty()) {
        body.push_str(&format!(
            "<input type=\"hidden\" name=\"_csrf\" value=\"{}\"/>",
            esc(token)
        ));
    }
    if action == "update" && !row_id.is_empty() {
        body.push_str(&format!(
            "<input type=\"hidden\" name=\"id\" value=\"{}\"/>",
            esc(&row_id)
        ));
    }
    for f in &fields {
        let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let label = f.get("label").and_then(|v| v.as_str()).unwrap_or(name);
        let ty = f.get("type").and_then(|v| v.as_str()).unwrap_or("text");
        let required = f.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
        let default = f.get("default").and_then(|v| v.as_str()).unwrap_or("");
        let pk = f.get("pk").and_then(|v| v.as_bool()).unwrap_or(false);
        let value = if data.contains_key(name) {
            field_text(&data, name)
        } else if name == "id" && !row_id.is_empty() {
            row_id.clone()
        } else {
            default.to_string()
        };
        let readonly = action == "update" && pk && name == "id";
        if readonly {
            body.push_str(&format!(
                "<label>{}<input type=\"text\" value=\"{}\" readonly/></label>",
                esc(label),
                esc(&value)
            ));
            continue;
        }
        let req_attr = if required { " required" } else { "" };
        body.push_str("<label>");
        body.push_str(&esc(label));
        if ty == "textarea" {
            body.push_str(&format!(
                "<textarea name=\"{}\" rows=\"5\"{}>{}</textarea>",
                esc(name),
                req_attr,
                esc(&value)
            ));
        } else {
            let input_type = match ty {
                "number" | "email" | "url" | "checkbox" | "file" => ty,
                _ => "text",
            };
            if ty == "checkbox" {
                let checked = matches!(value.as_str(), "1" | "true" | "on" | "yes");
                body.push_str(&format!(
                    "<input type=\"checkbox\" name=\"{}\" value=\"1\"{}{}/>",
                    esc(name),
                    if checked { " checked" } else { "" },
                    req_attr
                ));
            } else if ty == "file" {
                body.push_str(&format!(
                    "<input type=\"file\" name=\"{}\"{}/>",
                    esc(name),
                    req_attr
                ));
            } else {
                body.push_str(&format!(
                    "<input type=\"{}\" name=\"{}\" value=\"{}\"{}/>",
                    esc(input_type),
                    esc(name),
                    esc(&value),
                    req_attr
                ));
            }
        }
        if let Some(msg) = err_map.get(name).and_then(|v| v.as_str()) {
            body.push_str(&format!("<span class=\"err\">{}</span>", esc(msg)));
        }
        body.push_str("</label>");
    }
    body.push_str(&format!(
        "<div class=\"actions\"><button type=\"submit\">Submit</button><a href=\"{}\">cancel</a></div>",
        esc(cancel)
    ));
    body.push_str("</form></div>");
    body
}

/// Standalone form document (`GET|POST /_form/{id}`).
pub fn render(
    form: &Value,
    form_id: &str,
    data: Option<&Value>,
    errors: Option<&Value>,
    csrf: Option<&str>,
) -> String {
    let heading = form
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(form_id);
    let mut body = String::from(
        r#"<!DOCTYPE html><html lang="zh-CN"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>"#,
    );
    body.push_str(&format!("<title>{}</title>", esc(heading)));
    body.push_str(
        r#"<style>
body{font-family:"IBM Plex Sans","Noto Sans SC",sans-serif;margin:1.5rem;background:#fafaf9;color:#1c1917}
.site-form form{max-width:28rem;display:grid;gap:.85rem}
.site-form label{display:grid;gap:.25rem;font-size:.9rem}
.site-form input,.site-form textarea{padding:.5rem .6rem;border:1px solid #e7e5e4;border-radius:4px;font:inherit}
.site-form input[readonly]{background:#f5f5f4;color:#57534e}
.site-form .err{color:#b91c1c;font-size:.85rem}
.site-form .actions{display:flex;gap:.75rem;align-items:center;flex-wrap:wrap}
.site-form button{background:#0f766e;color:#fff;border:0;padding:.55rem 1rem;border-radius:4px;cursor:pointer}
.site-form a{color:#0f766e}
.site-form .meta{color:#57534e;font-size:.9rem}
</style></head><body>"#,
    );
    body.push_str(&format!("<h1>{}</h1>", esc(heading)));
    body.push_str(&render_body(form, form_id, data, errors, csrf));
    body.push_str("</body></html>");
    body
}

pub fn submit(form: &Value, data: &Value, db_url: &str) -> Result<Value, String> {
    let action = form
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("insert");
    let mut row_map = data_map(data);
    if action == "update" {
        if let Some(fid) = form.get("id").map(cell_str).filter(|s| !s.is_empty()) {
            if !row_map.contains_key("id") {
                row_map.insert("id".to_string(), json!(fid));
            }
        }
    }
    let data = Value::Object(row_map.clone());
    let v = validate(form, None, &data);
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        return Ok(json!({
            "ok": false,
            "errors": v.get("errors").cloned().unwrap_or(json!([])),
        }));
    }
    let table = form
        .get("table")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "form missing `table`".to_string())?;
    let row = Value::Object(row_map);
    let result = if action == "update" {
        let id = form
            .get("id")
            .map(cell_str)
            .filter(|s| !s.is_empty())
            .or_else(|| data_map(&data).get("id").map(cell_str))
            .ok_or_else(|| "update requires `id`".to_string())?;
        db::update(db_url, table, &id, &row, None)?
    } else {
        db::insert(db_url, table, &json!([row]), None)?
    };
    let redirect = form
        .get("redirect")
        .and_then(|v| v.as_str())
        .unwrap_or("/");
    Ok(json!({
        "ok": true,
        "result": result,
        "redirect": redirect,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn file_field_adds_multipart_enctype() {
        let form = json!({
            "table": "x",
            "fields": [
                {"name": "title", "label": "Title", "type": "text"},
                {"name": "file", "label": "File", "type": "file"},
            ]
        });
        let html = render(&form, "up", None, None, None);
        assert!(html.contains("enctype=\"multipart/form-data\""), "{html}");
        assert!(html.contains("type=\"file\""), "{html}");
        assert!(html.contains("name=\"file\""), "{html}");
    }
}


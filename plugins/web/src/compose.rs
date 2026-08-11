//! Page assemble: compose_components / compose_main.

use serde_json::{json, Map, Value};

use crate::table::{
    as_bind, as_compose, as_css_named, bind_table_name, normalize_ref, normalize_slot,
    parse_site_path, SitePath,
};

pub fn compose_components(
    page: &Value,
    layout: &Value,
    mut call_lib: impl FnMut(&str) -> Result<Value, String>,
) -> Result<Value, String> {
    let mut obj = page.as_object().cloned().unwrap_or_default();
    let rows = as_compose(layout);
    let mut css = obj
        .get("styles_css")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut slot_class = obj
        .get("slot_class")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut parts = Map::new();
    let mut compose_out = Vec::new();

    for row in rows.as_array().cloned().unwrap_or_default() {
        let src_raw = row.get("src").and_then(|v| v.as_str()).unwrap_or("");
        let style_raw = row.get("style").and_then(|v| v.as_str()).unwrap_or("");
        if src_raw.is_empty() {
            continue;
        }
        let (name, bind_table) = match parse_site_path(src_raw) {
            SitePath::LibMember { lib, member } => {
                let t = call_lib(&format!("{lib}.{member}"))?;
                (member, t)
            }
            SitePath::Plain(s) => {
                let t = call_lib(&format!("{s}.{s}"))?;
                (s, t)
            }
            SitePath::DbField { .. } => {
                return Err(format!("component cell cannot be a db field: {src_raw}"));
            }
        };
        let slot = normalize_slot(&name);
        let style_name = if style_raw.is_empty() {
            String::new()
        } else {
            match parse_site_path(style_raw) {
                SitePath::LibMember { lib, member } => {
                    let st = call_lib(&format!("{lib}.{member}"))?;
                    css.push_str(&as_css_named(&member, &st));
                    member
                }
                SitePath::Plain(s) => s,
                SitePath::DbField { .. } => normalize_ref(style_raw),
            }
        };
        let binds = as_bind(&bind_table);
        match slot.as_str() {
            "nav" => obj.insert("nav".into(), binds.clone()),
            "sidebar" => obj.insert("sidebar".into(), binds.clone()),
            "footer" => obj.insert("footer".into(), binds.clone()),
            _ => obj.insert("main".into(), binds.clone()),
        };
        if !style_name.is_empty() {
            slot_class.insert(slot.clone(), json!(style_name));
        }
        compose_out.push(json!({ "src": name, "style": style_name, "slot": slot }));

        let mut part = Map::new();
        part.insert("_type".into(), json!("page"));
        part.insert("slot".into(), json!(slot));
        part.insert("fragment".into(), json!(slot));
        match slot.as_str() {
            "nav" => {
                part.insert("nav".into(), binds);
            }
            "sidebar" => {
                part.insert("sidebar".into(), binds);
            }
            "footer" => {
                part.insert("footer".into(), binds);
            }
            _ => {
                part.insert("main".into(), binds);
            }
        }
        parts.insert(name, Value::Object(part));
    }

    obj.insert("compose".into(), Value::Array(compose_out));
    if !css.is_empty() {
        obj.insert("styles_css".into(), json!(css));
    }
    if !slot_class.is_empty() {
        obj.insert("slot_class".into(), Value::Object(slot_class));
    }
    if !parts.is_empty() {
        obj.insert("parts".into(), Value::Object(parts));
    }
    Ok(Value::Object(obj))
}

pub fn compose_main(
    page: &Value,
    main_table: &Value,
    mut call_lib: impl FnMut(&str) -> Result<Value, String>,
) -> Result<Value, String> {
    let mut obj = page.as_object().cloned().unwrap_or_default();
    let raw = as_bind(main_table);
    let arr = raw.as_array().cloned().unwrap_or_default();
    let mut binds = Vec::new();
    let mut css = obj
        .get("styles_css")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    for b in arr {
        let front = b
            .get("front")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let back_raw = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
        let css_raw = b.get("css").and_then(|v| v.as_str()).unwrap_or("");

        let back = match parse_site_path(back_raw) {
            SitePath::DbField { table, field } => format!("{table}.{field}"),
            SitePath::Plain(s) => s,
            SitePath::LibMember { lib, member } => {
                let v = call_lib(&format!("{lib}.{member}"))?;
                match v {
                    Value::String(s) => s,
                    other => other.to_string(),
                }
            }
        };
        let css_name = match parse_site_path(css_raw) {
            SitePath::LibMember { lib, member } => {
                let st = call_lib(&format!("{lib}.{member}"))?;
                css.push_str(&as_css_named(&member, &st));
                member
            }
            SitePath::Plain(s) => s,
            SitePath::DbField { .. } => normalize_ref(css_raw),
        };
        binds.push(json!({ "front": front, "back": back, "css": css_name }));
    }

    if let Some(t) = bind_table_name(&binds) {
        obj.insert("table".into(), json!(t));
    }
    obj.insert("main".into(), Value::Array(binds.clone()));
    if !css.is_empty() {
        obj.insert("styles_css".into(), json!(css));
    }

    let mut parts = obj
        .get("parts")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut part = Map::new();
    part.insert("_type".into(), json!("page"));
    part.insert("slot".into(), json!("main"));
    part.insert("fragment".into(), json!("main"));
    part.insert("main".into(), Value::Array(binds));
    if let Some(intro) = obj.get("intro") {
        part.insert("intro".into(), intro.clone());
    }
    parts.insert("index".into(), Value::Object(part));
    obj.insert("parts".into(), Value::Object(parts));
    Ok(Value::Object(obj))
}

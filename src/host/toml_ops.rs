//! Minimal TOML → Marqdo Value for Mid2 M8 (`lib/toml`).
//! Subset: comments, keys, strings, ints, floats, bools, arrays of scalars,
//! `[table]` / dotted tables. No datetime, inline tables, or array-of-tables.

use crate::host::json::json_to_value;
use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

pub fn parse(text: &Value) -> Result<Value, String> {
    let raw = as_text(text, "text")?;
    let json = toml_to_json(raw)?;
    json_to_value(&json)
}

fn toml_to_json(src: &str) -> Result<serde_json::Value, String> {
    let mut root = serde_json::Map::new();
    let mut current_path: Vec<String> = Vec::new();
    for (lineno, line) in src.lines().enumerate() {
        let line_no = lineno + 1;
        let stripped = strip_comment(line);
        let t = stripped.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('[') {
            if !t.ends_with(']') {
                return Err(format!("toml.parse:{line_no}: bad table header"));
            }
            if t.starts_with("[[") {
                return Err(format!(
                    "toml.parse:{line_no}: array-of-tables [[...]] not supported"
                ));
            }
            let inner = t[1..t.len() - 1].trim();
            current_path = split_dotted_keys(inner, line_no)?;
            ensure_table_path(&mut root, &current_path, line_no)?;
            continue;
        }
        let (key, val) = t
            .split_once('=')
            .ok_or_else(|| format!("toml.parse:{line_no}: expected key = value"))?;
        let key = key.trim();
        if key.is_empty() {
            return Err(format!("toml.parse:{line_no}: empty key"));
        }
        let key_parts = split_dotted_keys(key, line_no)?;
        let value = parse_value(val.trim(), line_no)?;
        let parent = table_at_path(&mut root, &current_path, line_no)?;
        insert_dotted(parent, &key_parts, value, line_no)?;
    }
    Ok(serde_json::Value::Object(root))
}

fn strip_comment(line: &str) -> String {
    // Strip unquoted `#` comments.
    let mut out = String::new();
    let mut in_str = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if !in_str && c == '#' {
            break;
        }
        if c == '"' {
            out.push(c);
            in_str = !in_str;
            continue;
        }
        if in_str && c == '\\' {
            out.push(c);
            if let Some(n) = chars.next() {
                out.push(n);
            }
            continue;
        }
        out.push(c);
    }
    out
}

fn split_dotted_keys(s: &str, line_no: usize) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            in_quote = !in_quote;
            continue;
        }
        if c == '.' && !in_quote {
            if cur.is_empty() {
                return Err(format!("toml.parse:{line_no}: empty key segment"));
            }
            parts.push(std::mem::take(&mut cur));
            continue;
        }
        cur.push(c);
    }
    if in_quote {
        return Err(format!("toml.parse:{line_no}: unclosed quoted key"));
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    if parts.is_empty() {
        return Err(format!("toml.parse:{line_no}: empty path"));
    }
    Ok(parts)
}

fn ensure_table_path(
    root: &mut serde_json::Map<String, serde_json::Value>,
    path: &[String],
    line_no: usize,
) -> Result<(), String> {
    let mut cur = root;
    for seg in path {
        if !cur.contains_key(seg) {
            cur.insert(seg.clone(), serde_json::Value::Object(serde_json::Map::new()));
        }
        cur = match cur.get_mut(seg) {
            Some(serde_json::Value::Object(m)) => m,
            Some(_) => {
                return Err(format!("toml.parse:{line_no}: {seg} is not a table"));
            }
            None => unreachable!(),
        };
    }
    Ok(())
}

fn table_at_path<'a>(
    root: &'a mut serde_json::Map<String, serde_json::Value>,
    path: &[String],
    line_no: usize,
) -> Result<&'a mut serde_json::Map<String, serde_json::Value>, String> {
    let mut cur = root;
    for seg in path {
        cur = match cur.get_mut(seg) {
            Some(serde_json::Value::Object(m)) => m,
            Some(_) => {
                return Err(format!("toml.parse:{line_no}: {seg} is not a table"));
            }
            None => {
                return Err(format!("toml.parse:{line_no}: missing table {seg}"));
            }
        };
    }
    Ok(cur)
}

fn insert_dotted(
    parent: &mut serde_json::Map<String, serde_json::Value>,
    parts: &[String],
    value: serde_json::Value,
    line_no: usize,
) -> Result<(), String> {
    if parts.len() == 1 {
        if parent.contains_key(&parts[0]) {
            return Err(format!(
                "toml.parse:{line_no}: duplicate key {}",
                parts[0]
            ));
        }
        parent.insert(parts[0].clone(), value);
        return Ok(());
    }
    let head = &parts[0];
    let entry = parent
        .entry(head.clone())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let obj = match entry {
        serde_json::Value::Object(m) => m,
        _ => {
            return Err(format!("toml.parse:{line_no}: {head} is not a table"));
        }
    };
    insert_dotted(obj, &parts[1..], value, line_no)
}

fn parse_value(s: &str, line_no: usize) -> Result<serde_json::Value, String> {
    if s.is_empty() {
        return Err(format!("toml.parse:{line_no}: empty value"));
    }
    if s == "true" {
        return Ok(serde_json::Value::Bool(true));
    }
    if s == "false" {
        return Ok(serde_json::Value::Bool(false));
    }
    if s.starts_with('"') {
        return Ok(serde_json::Value::String(parse_basic_string(s, line_no)?));
    }
    if s.starts_with('\'') {
        return Ok(serde_json::Value::String(parse_literal_string(s, line_no)?));
    }
    if s.starts_with('[') {
        return parse_array(s, line_no);
    }
    if let Ok(i) = s.replace('_', "").parse::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }
    if let Ok(f) = s.replace('_', "").parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
    }
    Err(format!("toml.parse:{line_no}: unsupported value {s:?}"))
}

fn parse_basic_string(s: &str, line_no: usize) -> Result<String, String> {
    if !s.ends_with('"') || s.len() < 2 {
        return Err(format!("toml.parse:{line_no}: bad string"));
    }
    let inner = &s[1..s.len() - 1];
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    return Err(format!(
                        "toml.parse:{line_no}: bad escape \\{other}"
                    ));
                }
                None => return Err(format!("toml.parse:{line_no}: trailing escape")),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn parse_literal_string(s: &str, line_no: usize) -> Result<String, String> {
    if !s.ends_with('\'') || s.len() < 2 {
        return Err(format!("toml.parse:{line_no}: bad literal string"));
    }
    Ok(s[1..s.len() - 1].to_string())
}

fn parse_array(s: &str, line_no: usize) -> Result<serde_json::Value, String> {
    if !s.ends_with(']') {
        return Err(format!("toml.parse:{line_no}: bad array"));
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Ok(serde_json::Value::Array(vec![]));
    }
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut str_ch = '"';
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if in_str {
            cur.push(c);
            if c == '\\' {
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
                continue;
            }
            if c == str_ch {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_str = true;
                str_ch = c;
                cur.push(c);
            }
            '[' => {
                depth += 1;
                cur.push(c);
            }
            ']' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                items.push(parse_value(cur.trim(), line_no)?);
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        items.push(parse_value(cur.trim(), line_no)?);
    }
    Ok(serde_json::Value::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let t = r#"
# comment
name = "marqdo"
count = 3
ok = true
[pkg]
version = "0.1"
"#;
        let v = parse(&Value::Text(t.into())).unwrap();
        let Value::Map(entries) = v else { panic!() };
        let get = |k: &str| {
            entries
                .iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(get("name"), Value::Text("marqdo".into()));
        assert_eq!(get("count"), Value::Int(3));
        assert_eq!(get("ok"), Value::Bool(true));
        let Value::Map(pkg) = get("pkg") else { panic!() };
        assert_eq!(
            pkg.iter().find(|(k, _)| k == "version").unwrap().1,
            Value::Text("0.1".into())
        );
    }
}

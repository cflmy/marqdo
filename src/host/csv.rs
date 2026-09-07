//! Minimal RFC4180 CSV for Mid M4 (`lib/csv`).

use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

/// Parse CSV text into a list of maps (first row = headers).
pub fn parse(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    let rows = parse_rows(s)?;
    if rows.is_empty() {
        return Ok(Value::List(vec![]));
    }
    let headers = rows[0].clone();
    if headers.is_empty() {
        return Err("csv.parse: header row is empty".into());
    }
    let mut out = Vec::new();
    for row in rows.into_iter().skip(1) {
        let mut map = Vec::new();
        for (i, key) in headers.iter().enumerate() {
            let val = row.get(i).cloned().unwrap_or_default();
            map.push((key.clone(), Value::Text(val)));
        }
        out.push(Value::Map(map));
    }
    Ok(Value::List(out))
}

/// Serialize list of maps to CSV (header order = first row's keys).
pub fn stringify(rows: &Value) -> Result<Value, String> {
    let list = match rows {
        Value::List(xs) => xs,
        _ => return Err("csv.stringify: rows needs list".into()),
    };
    if list.is_empty() {
        return Ok(Value::Text(String::new()));
    }
    let first = match &list[0] {
        Value::Map(m) => m,
        _ => return Err("csv.stringify: each row needs map".into()),
    };
    let headers: Vec<String> = first.iter().map(|(k, _)| k.clone()).collect();
    let mut lines = Vec::new();
    lines.push(headers.iter().map(|h| escape_field(h)).collect::<Vec<_>>().join(","));
    for row in list {
        let map = match row {
            Value::Map(m) => m,
            _ => return Err("csv.stringify: each row needs map".into()),
        };
        let mut fields = Vec::new();
        for h in &headers {
            let cell = map
                .iter()
                .find(|(k, _)| k == h)
                .map(|(_, v)| value_to_cell(v))
                .unwrap_or_default();
            fields.push(escape_field(&cell));
        }
        lines.push(fields.join(","));
    }
    Ok(Value::Text(lines.join("\n")))
}

fn value_to_cell(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        Value::None => String::new(),
        other => other.as_display(),
    }
}

fn escape_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn parse_rows(input: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes = false;
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => {
                    row.push(std::mem::take(&mut field));
                }
                '\r' => {
                    // swallow optional \n
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                _ => field.push(c),
            }
        }
    }
    if in_quotes {
        return Err("csv.parse: unterminated quote".into());
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    } else if !input.is_empty() && input.ends_with('\n') {
        // trailing newline already flushed last row
    }
    // Drop a single trailing empty row produced by final newline only when last row empty?
    while rows.last().is_some_and(|r| r.len() == 1 && r[0].is_empty()) {
        // only drop if it's a spurious empty line at end with no fields
        if rows.len() > 1 {
            rows.pop();
        } else {
            break;
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        let text = Value::Text("a,b\n1,2\n3,\"x,y\"\n".into());
        let rows = parse(&text).unwrap();
        let Value::List(ref xs) = rows else {
            panic!("list");
        };
        assert_eq!(xs.len(), 2);
        let out = stringify(&rows).unwrap();
        let again = parse(&out).unwrap();
        assert_eq!(again, rows);
    }
}

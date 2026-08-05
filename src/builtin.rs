//! Built-in helpers shared by tree-walk and bytecode.

use crate::value::Value;

/// `len`: text → Unicode scalar count; list → element count.
pub fn builtin_len(v: &Value) -> Result<i64, String> {
    match v {
        Value::Text(s) => Ok(s.chars().count() as i64),
        Value::List(xs) => Ok(xs.len() as i64),
        _ => Err("len needs text or list".into()),
    }
}

/// `str`: display conversion (same surface as `print`).
pub fn builtin_str(v: &Value) -> Value {
    Value::Text(v.as_display())
}

/// `int`: Int passthrough; Bool → 0/1; Text → parse trimmed decimal.
pub fn builtin_int(v: &Value) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        Value::Bool(true) => Ok(1),
        Value::Bool(false) => Ok(0),
        Value::Text(s) => {
            let t = s.trim();
            t.parse::<i64>()
                .map_err(|_| format!("cannot convert to int: {s:?}"))
        }
        _ => Err("int needs int, bool, or text".into()),
    }
}

/// `type`: type tag as text (`none` / `bool` / `int` / `text` / `list` / `map`).
pub fn builtin_type(v: &Value) -> Value {
    Value::Text(
        match v {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Text(_) => "text",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
        .into(),
    )
}

/// `trim`: strip leading/trailing Unicode whitespace from text.
pub fn builtin_trim(v: &Value) -> Result<Value, String> {
    match v {
        Value::Text(s) => Ok(Value::Text(s.trim().to_string())),
        _ => Err("trim needs text".into()),
    }
}

/// `split`: split text by separator → list of text parts.
pub fn builtin_split(value: &Value, sep: &Value) -> Result<Value, String> {
    let text = match value {
        Value::Text(s) => s.as_str(),
        _ => return Err("split needs text value".into()),
    };
    let sep = match sep {
        Value::Text(s) => s.as_str(),
        _ => return Err("split needs text sep".into()),
    };
    if sep.is_empty() {
        return Err("split sep must not be empty".into());
    }
    let parts: Vec<Value> = text
        .split(sep)
        .map(|p| Value::Text(p.to_string()))
        .collect();
    Ok(Value::List(parts))
}

/// `join`: join list elements with separator → text.
pub fn builtin_join(value: &Value, sep: &Value) -> Result<Value, String> {
    let items = match value {
        Value::List(xs) => xs,
        _ => return Err("join needs list value".into()),
    };
    let sep = match sep {
        Value::Text(s) => s.as_str(),
        _ => return Err("join needs text sep".into()),
    };
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(&item.as_display());
    }
    Ok(Value::Text(out))
}

/// `at`: list element at index (0-based); out of range → None.
pub fn builtin_at(value: &Value, index: &Value) -> Result<Value, String> {
    let xs = match value {
        Value::List(xs) => xs,
        _ => return Err("at needs list value".into()),
    };
    let i = match index {
        Value::Int(n) => *n,
        _ => return Err("at needs int index".into()),
    };
    if i < 0 || (i as usize) >= xs.len() {
        return Ok(Value::None);
    }
    Ok(xs[i as usize].clone())
}

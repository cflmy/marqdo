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

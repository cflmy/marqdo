//! Regex host primitives for Mid M1 (`lib/re`).

use crate::value::Value;
use regex::Regex;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

fn compile(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern).map_err(|e| format!("invalid regex: {e}"))
}

pub fn is_match(text: &Value, pattern: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let p = as_text(pattern, "pattern")?;
    let re = compile(p)?;
    Ok(Value::Bool(re.is_match(t)))
}

pub fn find(text: &Value, pattern: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let p = as_text(pattern, "pattern")?;
    let re = compile(p)?;
    Ok(match re.find(t) {
        Some(m) => Value::Text(m.as_str().to_string()),
        None => Value::None,
    })
}

pub fn find_all(text: &Value, pattern: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let p = as_text(pattern, "pattern")?;
    let re = compile(p)?;
    let xs: Vec<Value> = re
        .find_iter(t)
        .map(|m| Value::Text(m.as_str().to_string()))
        .collect();
    Ok(Value::List(xs))
}

pub fn replace(
    text: &Value,
    pattern: &Value,
    with: &Value,
    count: Option<&Value>,
) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let p = as_text(pattern, "pattern")?;
    let w = as_text(with, "with")?;
    let re = compile(p)?;
    let limit = match count {
        None | Some(Value::None) => None,
        Some(Value::Int(n)) => {
            if *n < 0 {
                return Err("count must be non-negative".into());
            }
            Some(*n as usize)
        }
        Some(_) => return Err("count needs int".into()),
    };
    let out = match limit {
        None => re.replace_all(t, w).into_owned(),
        Some(0) => t.to_string(),
        Some(n) => re.replacen(t, n, w).into_owned(),
    };
    Ok(Value::Text(out))
}

pub fn split(text: &Value, pattern: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let p = as_text(pattern, "pattern")?;
    let re = compile(p)?;
    let xs: Vec<Value> = re
        .split(t)
        .map(|s| Value::Text(s.to_string()))
        .collect();
    Ok(Value::List(xs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_and_find() {
        assert_eq!(
            is_match(
                &Value::Text("abc123".into()),
                &Value::Text(r"\d+".into())
            )
            .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            find(
                &Value::Text("ab12cd34".into()),
                &Value::Text(r"\d+".into())
            )
            .unwrap(),
            Value::Text("12".into())
        );
    }
}

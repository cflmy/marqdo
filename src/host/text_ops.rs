//! Text helpers for Mid M4 (`lib/text` thicken).

use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

fn as_usize(v: &Value, label: &str) -> Result<usize, String> {
    match v {
        Value::Int(n) if *n >= 0 => Ok(*n as usize),
        Value::Int(_) => Err(format!("{label} must be non-negative")),
        _ => Err(format!("{label} needs int")),
    }
}

pub fn contains(text: &Value, sub: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let s = as_text(sub, "sub")?;
    Ok(Value::Bool(t.contains(s)))
}

pub fn starts_with(text: &Value, prefix: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let p = as_text(prefix, "prefix")?;
    Ok(Value::Bool(t.starts_with(p)))
}

pub fn ends_with(text: &Value, suffix: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let s = as_text(suffix, "suffix")?;
    Ok(Value::Bool(t.ends_with(s)))
}

pub fn replace(
    text: &Value,
    old: &Value,
    new: &Value,
    count: Option<&Value>,
) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let old = as_text(old, "old")?;
    let new = as_text(new, "new")?;
    if old.is_empty() {
        return Err("replace: old must be non-empty".into());
    }
    let limit = match count {
        None | Some(Value::None) => None,
        Some(v) => Some(as_usize(v, "count")?),
    };
    let out = match limit {
        None => t.replace(old, new),
        Some(0) => t.to_string(),
        Some(n) => {
            let mut out = String::new();
            let mut rest = t;
            let mut done = 0usize;
            while done < n {
                if let Some(i) = rest.find(old) {
                    out.push_str(&rest[..i]);
                    out.push_str(new);
                    rest = &rest[i + old.len()..];
                    done += 1;
                } else {
                    break;
                }
            }
            out.push_str(rest);
            out
        }
    };
    Ok(Value::Text(out))
}

pub fn to_upper(text: &Value) -> Result<Value, String> {
    Ok(Value::Text(as_text(text, "text")?.to_uppercase()))
}

pub fn to_lower(text: &Value) -> Result<Value, String> {
    Ok(Value::Text(as_text(text, "text")?.to_lowercase()))
}

pub fn repeat(text: &Value, n: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let n = as_usize(n, "n")?;
    if n > 10_000 {
        return Err("repeat: n too large".into());
    }
    Ok(Value::Text(t.repeat(n)))
}

pub fn pad(
    text: &Value,
    width: &Value,
    fill: Option<&Value>,
    align: Option<&Value>,
) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let width = as_usize(width, "width")?;
    let fill = match fill {
        None | Some(Value::None) => " ".to_string(),
        Some(v) => as_text(v, "fill")?.to_string(),
    };
    if fill.is_empty() {
        return Err("pad: fill must be non-empty".into());
    }
    let align = match align {
        None | Some(Value::None) => "left",
        Some(v) => as_text(v, "align")?,
    };
    let chars: Vec<char> = t.chars().collect();
    if chars.len() >= width {
        return Ok(Value::Text(t.to_string()));
    }
    let pad_n = width - chars.len();
    let fill_chars: Vec<char> = fill.chars().collect();
    let make_pad = |n: usize| {
        let mut s = String::new();
        for i in 0..n {
            s.push(fill_chars[i % fill_chars.len()]);
        }
        s
    };
    let out = match align {
        "right" => format!("{}{}", make_pad(pad_n), t),
        "center" => {
            let left = pad_n / 2;
            format!("{}{}{}", make_pad(left), t, make_pad(pad_n - left))
        }
        _ => format!("{}{}", t, make_pad(pad_n)),
    };
    Ok(Value::Text(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_and_contains() {
        assert_eq!(
            contains(&Value::Text("Hello".into()), &Value::Text("ell".into())).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            to_upper(&Value::Text("ß".into())).unwrap(),
            Value::Text("SS".into())
        );
    }
}

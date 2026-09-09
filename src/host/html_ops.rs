//! HTML escape / unescape for Mid2 M8 (`lib/html`).

use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

pub fn escape(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    Ok(Value::Text(out))
}

pub fn unescape(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if let Some(end) = rest.find(';') {
            let ent = &rest[..=end];
            let ch = match ent {
                "&amp;" => Some('&'),
                "&lt;" => Some('<'),
                "&gt;" => Some('>'),
                "&quot;" => Some('"'),
                "&#x27;" | "&#39;" | "&apos;" => Some('\''),
                _ => None,
            };
            if let Some(c) = ch {
                out.push(c);
                rest = &rest[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &rest[1..];
    }
    out.push_str(rest);
    Ok(Value::Text(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let e = escape(&Value::Text(r#"a<b>&"c'"#.into())).unwrap();
        assert_eq!(
            e,
            Value::Text("a&lt;b&gt;&amp;&quot;c&#x27;".into())
        );
        assert_eq!(unescape(&e).unwrap(), Value::Text(r#"a<b>&"c'"#.into()));
    }
}

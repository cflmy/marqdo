//! URL parse / query helpers for Mid2 M7 (`lib/url`).
//! Distinct from `net::url_encode` (single-segment form encoding).
//! Hand-rolled subset (no `url` crate): scheme://[userinfo@]host[:port]/path[?query][#frag].

use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

fn encode_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn decode_component(s: &str) -> Result<String, String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let h = |c: u8| -> Result<u8, String> {
                    match c {
                        b'0'..=b'9' => Ok(c - b'0'),
                        b'a'..=b'f' => Ok(c - b'a' + 10),
                        b'A'..=b'F' => Ok(c - b'A' + 10),
                        _ => Err(format!("bad percent escape in {s:?}")),
                    }
                };
                let v = (h(bytes[i + 1])? << 4) | h(bytes[i + 2])?;
                out.push(v);
                i += 3;
            }
            b'%' => return Err(format!("truncated percent escape in {s:?}")),
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|e| format!("url decode utf8: {e}"))
}

/// Parse `scheme://[userinfo@]host[:port]/path[?query][#fragment]`.
pub fn parse(text: &Value) -> Result<Value, String> {
    let raw = as_text(text, "text")?;
    let (scheme, rest) = raw
        .split_once("://")
        .ok_or_else(|| format!("url.parse: missing :// in {raw:?}"))?;
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return Err(format!("url.parse: bad scheme in {raw:?}"));
    }

    let (before_frag, fragment) = match rest.split_once('#') {
        Some((a, b)) => (a, b),
        None => (rest, ""),
    };
    let (before_query, query) = match before_frag.split_once('?') {
        Some((a, b)) => (a, b),
        None => (before_frag, ""),
    };

    let (authority, path) = if let Some(i) = before_query.find('/') {
        (&before_query[..i], &before_query[i..])
    } else {
        (before_query, "")
    };

    let (userinfo, hostport) = if let Some((u, h)) = authority.rsplit_once('@') {
        (u, h)
    } else {
        ("", authority)
    };

    let (host, port) = if hostport.starts_with('[') {
        // IPv6 literal [addr]:port
        let end = hostport
            .find(']')
            .ok_or_else(|| format!("url.parse: bad IPv6 host in {raw:?}"))?;
        let host = &hostport[1..end];
        let rest = &hostport[end + 1..];
        if rest.is_empty() {
            (host, Value::None)
        } else if let Some(p) = rest.strip_prefix(':') {
            let n: i64 = p
                .parse()
                .map_err(|_| format!("url.parse: bad port in {raw:?}"))?;
            (host, Value::Int(n))
        } else {
            return Err(format!("url.parse: bad hostport in {raw:?}"));
        }
    } else if let Some((h, p)) = hostport.rsplit_once(':') {
        if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() {
            let n: i64 = p
                .parse()
                .map_err(|_| format!("url.parse: bad port in {raw:?}"))?;
            (h, Value::Int(n))
        } else {
            (hostport, Value::None)
        }
    } else {
        (hostport, Value::None)
    };

    let path = if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    };

    Ok(Value::Map(vec![
        ("scheme".into(), Value::Text(scheme.to_string())),
        ("userinfo".into(), Value::Text(userinfo.to_string())),
        ("host".into(), Value::Text(host.to_string())),
        ("port".into(), port),
        ("path".into(), Value::Text(path)),
        ("query".into(), Value::Text(query.to_string())),
        ("fragment".into(), Value::Text(fragment.to_string())),
    ]))
}

pub fn query_parse(text: &Value) -> Result<Value, String> {
    let raw = as_text(text, "text")?;
    let raw = raw.strip_prefix('?').unwrap_or(raw);
    let mut map: Vec<(String, Value)> = Vec::new();
    if raw.is_empty() {
        return Ok(Value::Map(map));
    }
    for pair in raw.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        let key = decode_component(k)?;
        let val = Value::Text(decode_component(v)?);
        if let Some((_, slot)) = map.iter_mut().find(|(kk, _)| *kk == key) {
            match slot {
                Value::List(xs) => xs.push(val),
                other => {
                    let prev = std::mem::replace(other, Value::None);
                    *other = Value::List(vec![prev, val]);
                }
            }
        } else {
            map.push((key, val));
        }
    }
    Ok(Value::Map(map))
}

pub fn query_stringify(map: &Value) -> Result<Value, String> {
    let entries = match map {
        Value::Map(m) => m,
        Value::None => return Ok(Value::Text(String::new())),
        _ => return Err("url.query_stringify: map required".into()),
    };
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in entries {
        let key = encode_component(k);
        match v {
            Value::List(xs) => {
                for x in xs {
                    match x {
                        Value::Text(s) => {
                            parts.push(format!("{key}={}", encode_component(s)));
                        }
                        Value::Int(n) => {
                            parts.push(format!("{key}={}", encode_component(&n.to_string())));
                        }
                        Value::Bool(b) => {
                            parts.push(format!("{key}={}", encode_component(&b.to_string())));
                        }
                        Value::None => parts.push(format!("{key}=")),
                        _ => {
                            return Err("url.query_stringify: list values must be scalar".into())
                        }
                    }
                }
            }
            Value::Text(s) => parts.push(format!("{key}={}", encode_component(s))),
            Value::Int(n) => parts.push(format!("{key}={}", encode_component(&n.to_string()))),
            Value::Bool(b) => parts.push(format!("{key}={}", encode_component(&b.to_string()))),
            Value::None => parts.push(format!("{key}=")),
            _ => return Err("url.query_stringify: values must be scalar or list".into()),
        }
    }
    Ok(Value::Text(parts.join("&")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_query() {
        let m = parse(&Value::Text(
            "https://user:pass@example.com:8443/a/b?x=1&y=2#frag".into(),
        ))
        .unwrap();
        let Value::Map(entries) = m else { panic!() };
        let get = |k: &str| {
            entries
                .iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(get("scheme"), Value::Text("https".into()));
        assert_eq!(get("host"), Value::Text("example.com".into()));
        assert_eq!(get("port"), Value::Int(8443));
        assert_eq!(get("path"), Value::Text("/a/b".into()));
        assert_eq!(get("query"), Value::Text("x=1&y=2".into()));
        assert_eq!(get("fragment"), Value::Text("frag".into()));
        assert_eq!(get("userinfo"), Value::Text("user:pass".into()));
    }

    #[test]
    fn query_roundtrip() {
        let q = query_parse(&Value::Text("a=1&b=hello%20w&a=2".into())).unwrap();
        let s = query_stringify(&q).unwrap();
        assert_eq!(s, Value::Text("a=1&a=2&b=hello%20w".into()));
    }

    #[test]
    fn empty_path_becomes_slash() {
        let m = parse(&Value::Text("https://example.com".into())).unwrap();
        let Value::Map(entries) = m else { panic!() };
        let path = entries.iter().find(|(k, _)| k == "path").unwrap().1.clone();
        assert_eq!(path, Value::Text("/".into()));
    }
}

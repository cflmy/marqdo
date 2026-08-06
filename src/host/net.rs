//! HTTP host primitives (HTTP and HTTPS via ureq).

use std::time::Duration;

use crate::host::HostContext;
use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

fn headers_from_value(headers: Option<&Value>) -> Result<Vec<(String, String)>, String> {
    let Some(h) = headers else {
        return Ok(Vec::new());
    };
    match h {
        Value::None => Ok(Vec::new()),
        Value::Map(entries) => Ok(entries
            .iter()
            .map(|(k, v)| (k.clone(), v.as_display()))
            .collect()),
        _ => Err("headers must be a map (e.g. from json parse)".into()),
    }
}

fn http_exchange(
    method: &str,
    url: &str,
    body: Option<&str>,
    content_type: Option<&str>,
    extra_headers: &[(String, String)],
) -> Result<(u16, String), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("url must start with http:// or https://".into());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .build();

    let mut req = match method {
        "GET" => agent.get(url),
        "POST" => agent.post(url),
        "PUT" => agent.put(url),
        "DELETE" => agent.request("DELETE", url),
        other => return Err(format!("unsupported http method: {other}")),
    };
    req = req.set("User-Agent", "marqdo");
    if let Some(ct) = content_type {
        if !ct.is_empty() {
            req = req.set("Content-Type", ct);
        }
    }
    for (k, v) in extra_headers {
        req = req.set(k, v);
    }

    let resp = if let Some(b) = body {
        req.send_string(b)
    } else {
        req.call()
    };

    match resp {
        Ok(r) => {
            let status = r.status();
            let text = r
                .into_string()
                .map_err(|e| format!("http read body: {e}"))?;
            if text.len() > 8 * 1024 * 1024 {
                return Err("http response exceeds 8MiB limit".into());
            }
            Ok((status, text))
        }
        Err(ureq::Error::Status(code, r)) => {
            let text = r.into_string().unwrap_or_default();
            Ok((code, text))
        }
        Err(e) => Err(format!("http {method} {url}: {e}")),
    }
}

fn response_map(status: u16, body: String) -> Value {
    Value::Map(vec![
        ("status".into(), Value::Int(status as i64)),
        ("body".into(), Value::Text(body)),
    ])
}

pub fn http_get(
    ctx: &HostContext,
    url: &Value,
    headers: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("http_get denied by host policy".into());
    }
    let url = as_text(url, "url")?;
    let hdrs = headers_from_value(headers)?;
    let (status, body) = http_exchange("GET", url, None, None, &hdrs)?;
    Ok(response_map(status, body))
}

pub fn http_post(
    ctx: &HostContext,
    url: &Value,
    body: &Value,
    content_type: Option<&Value>,
    headers: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("http_post denied by host policy".into());
    }
    let url = as_text(url, "url")?;
    let body = as_text(body, "body")?;
    let ct = match content_type {
        Some(v) => as_text(v, "content_type")?,
        None => "application/json; charset=utf-8",
    };
    let hdrs = headers_from_value(headers)?;
    let (status, resp) = http_exchange("POST", url, Some(body), Some(ct), &hdrs)?;
    Ok(response_map(status, resp))
}

/// Generic request: method + url + optional body / content_type / headers.
pub fn http_request(
    ctx: &HostContext,
    method: &Value,
    url: &Value,
    body: Option<&Value>,
    content_type: Option<&Value>,
    headers: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("http_request denied by host policy".into());
    }
    let method = as_text(method, "method")?.to_uppercase();
    let url = as_text(url, "url")?;
    let body_s = match body {
        None | Some(Value::None) => None,
        Some(v) => Some(as_text(v, "body")?),
    };
    let ct = match content_type {
        None | Some(Value::None) => {
            if body_s.is_some() && method == "POST" {
                Some("application/json; charset=utf-8")
            } else {
                None
            }
        }
        Some(v) => Some(as_text(v, "content_type")?),
    };
    let hdrs = headers_from_value(headers)?;
    let (status, resp) = http_exchange(&method, url, body_s, ct, &hdrs)?;
    Ok(response_map(status, resp))
}

pub fn url_encode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    Ok(Value::Text(
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                b' ' => "+".into(),
                _ => format!("%{b:02X}"),
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_from_map() {
        let m = Value::Map(vec![
            ("Authorization".into(), Value::Text("Bearer x".into())),
            ("X-Test".into(), Value::Int(1)),
        ]);
        let h = headers_from_value(Some(&m)).unwrap();
        assert_eq!(h.len(), 2);
        assert!(h
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer x"));
    }
}

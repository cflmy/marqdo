//! Minimal HTTP host primitives (HTTP cleartext in v1; HTTPS deferred).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::host::HostContext;
use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

struct UrlParts {
    host: String,
    port: u16,
    path: String,
}

fn parse_http_url(url: &str) -> Result<UrlParts, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| {
            if url.starts_with("https://") {
                "https is not supported yet (use http:// or wait for TLS)".to_string()
            } else {
                "url must start with http://".into()
            }
        })?;
    let (hostport, path) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{p}")),
        None => (rest, "/".into()),
    };
    let (host, port) = if let Some((h, p)) = hostport.split_once(':') {
        let port: u16 = p
            .parse()
            .map_err(|_| format!("invalid port in url: {p}"))?;
        (h.to_string(), port)
    } else {
        (hostport.to_string(), 80)
    };
    if host.is_empty() {
        return Err("url missing host".into());
    }
    Ok(UrlParts { host, port, path })
}

fn http_exchange(
    method: &str,
    url: &str,
    body: Option<&str>,
    content_type: &str,
) -> Result<(u16, String), String> {
    let parts = parse_http_url(url)?;
    let addr = format!("{}:{}", parts.host, parts.port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("http connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .ok();

    let body = body.unwrap_or("");
    let mut req = format!(
        "{method} {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\nUser-Agent: marqdo\r\n",
        path = parts.path,
        host = parts.host,
    );
    if method == "POST" {
        req.push_str(&format!(
            "Content-Type: {content_type}\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    req.push_str("\r\n");
    if method == "POST" {
        req.push_str(body);
    }
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("http write: {e}"))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("http read: {e}"))?;
    if buf.len() > 2 * 1024 * 1024 {
        return Err("http response exceeds 2MiB limit".into());
    }
    let text = String::from_utf8_lossy(&buf);
    let (header, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_ref(), ""));
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    Ok((status, body.to_string()))
}

pub fn http_get(ctx: &HostContext, url: &Value) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("http_get denied by host policy".into());
    }
    let url = as_text(url, "url")?;
    let (status, body) = http_exchange("GET", url, None, "")?;
    Ok(Value::Map(vec![
        ("status".into(), Value::Int(status as i64)),
        ("body".into(), Value::Text(body)),
    ]))
}

pub fn http_post(
    ctx: &HostContext,
    url: &Value,
    body: &Value,
    content_type: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("http_post denied by host policy".into());
    }
    let url = as_text(url, "url")?;
    let body = as_text(body, "body")?;
    let ct = match content_type {
        Some(v) => as_text(v, "content_type")?,
        None => "text/plain; charset=utf-8",
    };
    let (status, resp) = http_exchange("POST", url, Some(body), ct)?;
    Ok(Value::Map(vec![
        ("status".into(), Value::Int(status as i64)),
        ("body".into(), Value::Text(resp)),
    ]))
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

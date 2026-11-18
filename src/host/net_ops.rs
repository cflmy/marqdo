//! Configurable bounded HTTP with one recoverable Result envelope.
#[cfg(feature = "net-host")]
use super::as_text;
use super::result;
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use std::collections::HashMap;
#[cfg(feature = "net-host")]
use std::{
    io::{Read, Write},
    time::Duration,
};
type Args = HashMap<String, Value>;
#[cfg(feature = "net-host")]
fn arg<'a>(b: &'a Args, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
pub(super) fn call(ctx: &mut HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    #[cfg(feature = "net-host")]
    let answer = request(ctx, id, b);
    #[cfg(not(feature = "net-host"))]
    let answer = {
        let _ = (ctx, id, b);
        Err("HTTP unavailable in this build".into())
    };
    Ok(result::capture(
        if id == HostFn::HttpDownload {
            "net.download"
        } else {
            "net.request"
        },
        answer,
    ))
}
#[cfg(feature = "net-host")]
fn integer(b: &Args, k: &str, max: u64) -> Result<u64, String> {
    match arg(b, k)? {
        Value::Int(n) if *n > 0 && (*n as u64) <= max => Ok(*n as u64),
        _ => Err(format!("{k} must be 1..{max}")),
    }
}
#[cfg(feature = "net-host")]
fn request(ctx: &HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("network denied by host policy".into());
    }
    let url = as_text(arg(b, "url")?, "url")?;
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("url must use http/https".into());
    }
    let timeout = Duration::from_millis(integer(b, "timeout_ms", 600000)?);
    let max = integer(b, "max_bytes", 1024 * 1024 * 1024)?;
    let method = if matches!(id, HostFn::HttpDownload | HostFn::HttpGetResult) {
        "GET"
    } else {
        as_text(arg(b, "method")?, "method")?
    };
    if !matches!(
        method,
        "GET" | "HEAD" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS"
    ) {
        return Err("unsupported method".into());
    }
    // Reject invalid output settings and destinations before any HTTP exchange.
    let destination = if id == HostFn::HttpDownload {
        if !ctx.allow_fs_write() {
            return Err("filesystem write denied by host policy".into());
        }
        if ctx.sandbox_session.is_some() {
            return Err("unsupported: stream download in CoW sandbox".into());
        }
        let path = ctx.resolve_path(as_text(arg(b, "path")?, "path")?)?;
        if path.is_dir() {
            return Err("invalid_argument: download destination is a directory".into());
        }
        Some(path)
    } else {
        if max > 64 * 1024 * 1024 {
            return Err(
                "resource_limit: in-memory response limit exceeds 64 MiB; use download".into(),
            );
        }
        if !matches!(as_text(arg(b, "response")?, "response")?, "text" | "bytes") {
            return Err("invalid_argument: response must be text or bytes".into());
        }
        None
    };
    let agent = ureq::AgentBuilder::new()
        .timeout(timeout)
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let mut request = agent.request(method, url).set("User-Agent", "marqdo");
    if let Some(Value::Map(headers)) = b.get("headers") {
        for (k, v) in headers {
            request = request.set(k, as_text(v, "header value")?)
        }
    } else if !matches!(b.get("headers"), None | Some(Value::None)) {
        return Err("headers must be map or None".into());
    }
    let exchange = match b.get("body") {
        None | Some(Value::None) => request.call(),
        Some(Value::Text(text)) => request.send_bytes(text.as_bytes()),
        Some(v) => request.send_bytes(&super::fs_ops::bytes(v)?),
    };
    let response = match exchange {
        Ok(r) => r,
        Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(format!("transport_error: {e}")),
    };
    let status = response.status();
    let mut headers = Vec::new();
    for name in response.headers_names() {
        let values = response
            .all(&name)
            .into_iter()
            .map(|v| Value::Text(v.into()))
            .collect();
        headers.push(Value::Map(vec![
            ("name".into(), Value::Text(name.to_ascii_lowercase())),
            ("values".into(), Value::List(values)),
        ]));
    }
    let mut reader = response.into_reader();
    if id == HostFn::HttpDownload {
        if !(200..300).contains(&status) {
            return Err(format!("http_status: {status}; destination unchanged"));
        }
        let path = destination.ok_or("missing validated download destination")?;
        let parent = path.parent().ok_or("destination needs parent")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut entropy = [0u8; 16];
        getrandom::getrandom(&mut entropy).map_err(|e| e.to_string())?;
        let token = entropy
            .iter()
            .map(|n| format!("{n:02x}"))
            .collect::<String>();
        let temp = parent.join(format!(".mq-download-{token}"));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        let downloaded = (|| {
            let mut total = 0u64;
            let mut buffer = [0u8; 65536];
            loop {
                let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                total += n as u64;
                if total > max {
                    return Err("resource_limit: HTTP response exceeds max_bytes".into());
                }
                file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            }
            file.sync_all().map_err(|e| e.to_string())?;
            drop(file);
            std::fs::rename(&temp, &path).map_err(|e| e.to_string())?;
            Ok(total)
        })();
        let total = match downloaded {
            Ok(n) => n,
            Err(e) => {
                let _ = std::fs::remove_file(temp);
                return Err(e);
            }
        };
        return Ok(Value::Map(vec![
            ("status".into(), Value::Int(status as i64)),
            ("headers".into(), Value::List(headers)),
            ("size".into(), Value::Int(total as i64)),
            ("path".into(), arg(b, "path")?.clone()),
        ]));
    }
    let mut bytes = Vec::new();
    reader
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err("resource_limit: HTTP response exceeds max_bytes".into());
    }
    let mode = as_text(arg(b, "response")?, "response")?;
    let body = match mode {
        "text" => Value::Text(String::from_utf8(bytes).map_err(|e| {
            format!("invalid_encoding: response is not UTF-8; request bytes instead: {e}")
        })?),
        "bytes" => super::fs_ops::bytes_value(&bytes),
        _ => return Err("response must be text or bytes".into()),
    };
    Ok(Value::Map(vec![
        ("status".into(), Value::Int(status as i64)),
        ("headers".into(), Value::List(headers)),
        ("body".into(), body),
    ]))
}

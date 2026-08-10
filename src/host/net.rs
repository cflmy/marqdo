//! HTTP host primitives (HTTP and HTTPS via ureq).

use std::io::{BufRead, BufReader, Write};
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
        None | Some(Value::None) => "application/json; charset=utf-8",
        Some(v) => as_text(v, "content_type")?,
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

fn truthy_flag(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::None) => false,
        Some(v) => v.truthy(),
    }
}

/// Extract `data:` payloads from an SSE body (ignores comments / event: / id:).
pub fn sse_data_payloads(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n"));
                cur.clear();
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("data:") {
            let payload = rest.strip_prefix(' ').unwrap_or(rest);
            cur.push(payload.to_string());
        }
        // Other SSE fields ignored for chat-completions wire format.
    }
    if !cur.is_empty() {
        out.push(cur.join("\n"));
    }
    out
}

fn event_map(pairs: Vec<(&str, Value)>) -> Value {
    Value::Map(
        pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

/// Push stream events from one OpenAI-compatible SSE JSON object.
/// DeepSeek (and similar) may stream chain-of-thought in `delta.reasoning_content`
/// while the final answer arrives later in `delta.content`.
/// Returns `Err` only on JSON issues handled by caller; `Ok(true)` means stop (error frame).
fn push_openai_delta_events(
    v: &serde_json::Value,
    events: &mut Vec<Value>,
    result: &mut String,
    echoed: &mut bool,
    saw_reasoning: &mut bool,
    echo: bool,
) -> bool {
    if let Some(msg) = v.get("error").and_then(|e| {
        e.get("message")
            .and_then(|m| m.as_str())
            .or_else(|| e.as_str())
    }) {
        let ev = event_map(vec![
            ("type", Value::Text("error".into())),
            ("message", Value::Text(msg.into())),
        ]);
        crate::host::event_bus::publish(&ev);
        events.push(ev);
        return true;
    }
    let reasoning = v
        .pointer("/choices/0/delta/reasoning_content")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    if !reasoning.is_empty() {
        *saw_reasoning = true;
        if echo {
            print!("{reasoning}");
            let _ = std::io::stdout().flush();
            *echoed = true;
        }
        let ev = event_map(vec![
            ("type", Value::Text("reasoning".into())),
            ("text", Value::Text(reasoning.into())),
        ]);
        crate::host::event_bus::publish(&ev);
        events.push(ev);
    }
    let content = v
        .pointer("/choices/0/delta/content")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    if !content.is_empty() {
        if echo && *saw_reasoning && result.is_empty() {
            println!();
            *echoed = true;
        }
        result.push_str(content);
        if echo {
            print!("{content}");
            let _ = std::io::stdout().flush();
            *echoed = true;
        }
        let ev = event_map(vec![
            ("type", Value::Text("delta".into())),
            ("text", Value::Text(content.into())),
        ]);
        crate::host::event_bus::publish(&ev);
        events.push(ev);
    }
    false
}

/// Map OpenAI-compatible chat SSE payloads → Marqdo stream events
/// (`reasoning` / `delta` / `done` / `error`). Optional `echo` prints text to stdout as it arrives.
pub fn openai_chat_sse_events(text: &str, echo: bool) -> Result<Vec<Value>, String> {
    let mut events = Vec::new();
    let mut result = String::new();
    let mut saw_done = false;
    let mut echoed = false;
    let mut saw_reasoning = false;
    for data in sse_data_payloads(text) {
        let data = data.trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" {
            saw_done = true;
            break;
        }
        let v: serde_json::Value = serde_json::from_str(data)
            .map_err(|e| format!("openai sse json: {e}"))?;
        if push_openai_delta_events(
            &v,
            &mut events,
            &mut result,
            &mut echoed,
            &mut saw_reasoning,
            echo,
        ) {
            return Ok(events);
        }
    }
    if echo && echoed {
        println!();
    }
    if !saw_done && events.is_empty() && result.is_empty() {
        // Non-SSE / empty: treat as error for clearer gold diagnostics.
        events.push(event_map(vec![
            ("type", Value::Text("error".into())),
            (
                "message",
                Value::Text("openai sse: no data frames".into()),
            ),
        ]));
        return Ok(events);
    }
    let done = event_map(vec![
        ("type", Value::Text("done".into())),
        ("result", Value::Text(result)),
    ]);
    crate::host::event_bus::publish(&done);
    events.push(done);
    Ok(events)
}

/// Offline: parse OpenAI chat SSE text → event list (no network).
pub fn openai_sse_parse(text: &Value, echo: Option<&Value>) -> Result<Value, String> {
    let text = as_text(text, "text")?;
    let echo = truthy_flag(echo);
    Ok(Value::List(openai_chat_sse_events(text, echo)?))
}

fn read_sse_body_to_events<R: BufRead>(
    mut reader: R,
    echo: bool,
) -> Result<Vec<Value>, String> {
    let mut buf = String::new();
    let mut raw = String::new();
    let mut events = Vec::new();
    let mut result = String::new();
    let mut data_buf: Vec<String> = Vec::new();
    let mut echoed = false;
    let mut saw_reasoning = false;

    let flush_data = |data_buf: &mut Vec<String>,
                      events: &mut Vec<Value>,
                      result: &mut String,
                      echoed: &mut bool,
                      saw_reasoning: &mut bool,
                      echo: bool|
     -> Result<bool, String> {
        if data_buf.is_empty() {
            return Ok(false);
        }
        let data = data_buf.join("\n");
        data_buf.clear();
        let data = data.trim();
        if data.is_empty() {
            return Ok(false);
        }
        if data == "[DONE]" {
            return Ok(true);
        }
        let v: serde_json::Value =
            serde_json::from_str(data).map_err(|e| format!("openai sse json: {e}"))?;
        Ok(push_openai_delta_events(
            &v,
            events,
            result,
            echoed,
            saw_reasoning,
            echo,
        ))
    };

    loop {
        buf.clear();
        let n = reader
            .read_line(&mut buf)
            .map_err(|e| format!("http sse read: {e}"))?;
        if n == 0 {
            break;
        }
        raw.push_str(&buf);
        if raw.len() > 8 * 1024 * 1024 {
            return Err("http sse response exceeds 8MiB limit".into());
        }
        let line = buf.strip_suffix('\n').unwrap_or(&buf);
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            if flush_data(
                &mut data_buf,
                &mut events,
                &mut result,
                &mut echoed,
                &mut saw_reasoning,
                echo,
            )? {
                break;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("data:") {
            let payload = rest.strip_prefix(' ').unwrap_or(rest);
            data_buf.push(payload.to_string());
        }
    }
    let _ = flush_data(
        &mut data_buf,
        &mut events,
        &mut result,
        &mut echoed,
        &mut saw_reasoning,
        echo,
    )?;
    if echo && echoed {
        println!();
    }
    if events.iter().any(|e| matches!(e, Value::Map(m) if m.iter().any(|(k,v)| k == "type" && matches!(v, Value::Text(t) if t == "error")))) {
        return Ok(events);
    }
    let done = event_map(vec![
        ("type", Value::Text("done".into())),
        ("result", Value::Text(result)),
    ]);
    crate::host::event_bus::publish(&done);
    events.push(done);
    Ok(events)
}

/// POST and consume OpenAI-compatible SSE; return `{status, events}` list of stream maps.
pub fn http_post_sse(
    ctx: &HostContext,
    url: &Value,
    body: &Value,
    content_type: Option<&Value>,
    headers: Option<&Value>,
    echo: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.allow_net() {
        return Err("http_post_sse denied by host policy".into());
    }
    let url = as_text(url, "url")?;
    let body = as_text(body, "body")?;
    let ct = match content_type {
        None | Some(Value::None) => "application/json; charset=utf-8",
        Some(v) => as_text(v, "content_type")?,
    };
    let hdrs = headers_from_value(headers)?;
    let echo = truthy_flag(echo);
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("url must start with http:// or https://".into());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .build();
    let mut req = agent.post(url).set("User-Agent", "marqdo").set("Content-Type", ct);
    req = req.set("Accept", "text/event-stream");
    for (k, v) in &hdrs {
        req = req.set(k, v);
    }
    match req.send_string(body) {
        Ok(r) => {
            let status = r.status();
            let reader = BufReader::new(r.into_reader());
            let events = read_sse_body_to_events(reader, echo)?;
            Ok(Value::Map(vec![
                ("status".into(), Value::Int(status as i64)),
                ("events".into(), Value::List(events)),
            ]))
        }
        Err(ureq::Error::Status(code, r)) => {
            let text = r.into_string().unwrap_or_default();
            Ok(Value::Map(vec![
                ("status".into(), Value::Int(code as i64)),
                (
                    "events".into(),
                    Value::List(vec![event_map(vec![
                        ("type", Value::Text("error".into())),
                        ("message", Value::Text(text)),
                    ])]),
                ),
            ]))
        }
        Err(e) => Err(format!("http POST sse {url}: {e}")),
    }
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

    #[test]
    fn openai_sse_fixture_to_events() {
        let fixture = "\
data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\
\n\
data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\
\n\
data: [DONE]\n\
";
        let ev = openai_chat_sse_events(fixture, false).unwrap();
        assert_eq!(ev.len(), 3);
        match &ev[0] {
            Value::Map(m) => {
                assert!(m.iter().any(|(k, v)| k == "type" && matches!(v, Value::Text(t) if t == "delta")));
                assert!(m.iter().any(|(k, v)| k == "text" && matches!(v, Value::Text(t) if t == "Hel")));
            }
            _ => panic!("expected map"),
        }
        match &ev[2] {
            Value::Map(m) => {
                assert!(m.iter().any(|(k, v)| k == "type" && matches!(v, Value::Text(t) if t == "done")));
                assert!(m.iter().any(|(k, v)| k == "result" && matches!(v, Value::Text(t) if t == "Hello")));
            }
            _ => panic!("expected done map"),
        }
    }

    #[test]
    fn openai_sse_reasoning_then_content() {
        let fixture = "\
data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"think\"}}]}\n\
\n\
data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\
\n\
data: [DONE]\n\
";
        let ev = openai_chat_sse_events(fixture, false).unwrap();
        assert_eq!(ev.len(), 3);
        match &ev[0] {
            Value::Map(m) => {
                assert!(m.iter().any(|(k, v)| k == "type" && matches!(v, Value::Text(t) if t == "reasoning")));
                assert!(m.iter().any(|(k, v)| k == "text" && matches!(v, Value::Text(t) if t == "think")));
            }
            _ => panic!("expected reasoning map"),
        }
        match &ev[2] {
            Value::Map(m) => {
                assert!(m.iter().any(|(k, v)| k == "type" && matches!(v, Value::Text(t) if t == "done")));
                assert!(m.iter().any(|(k, v)| k == "result" && matches!(v, Value::Text(t) if t == "Hi")));
            }
            _ => panic!("expected done map"),
        }
    }
}

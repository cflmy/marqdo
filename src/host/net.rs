//! HTTP host primitives (HTTP and HTTPS via ureq).

use std::io::Write;
#[cfg(feature = "net-host")]
use std::io::{BufRead, BufReader};
#[cfg(feature = "net-host")]
use std::time::Duration;

use crate::value::Value;
#[cfg(feature = "net-host")]
use crate::host::HostContext;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

#[cfg(feature = "net-host")]
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

#[cfg(feature = "net-host")]
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

#[cfg(feature = "net-host")]
fn response_map(status: u16, body: String) -> Value {
    Value::Map(vec![
        ("status".into(), Value::Int(status as i64)),
        ("body".into(), Value::Text(body)),
    ])
}

#[cfg(feature = "net-host")]
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

#[cfg(feature = "net-host")]
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
#[cfg(feature = "net-host")]
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

/// Split a Cookie / Set-Cookie header value into individual cookie attribute tokens.
/// `Set-Cookie` may contain multiple `name=value` cookies plus attributes per cookie;
/// a single request `Cookie` header is a flat `name=value; name=value` list.
fn split_cookie_tokens(header: &str, is_set_cookie: bool) -> Vec<Vec<(String, String)>> {
    // Set-Cookie: each cookie block is separated by ", " only when a next cookie begins.
    // We split on ',' that is NOT preceded by an Expires/Max-Age value (contains '-' or ':')
    // — heuristic good enough for common cases; value may contain ';' (e.g. quoted).
    if !is_set_cookie {
        let mut cookies = Vec::new();
        for seg in header.split(';') {
            let seg = seg.trim();
            if seg.is_empty() {
                continue;
            }
            let mut cur = Vec::new();
            if let Some((k, v)) = seg.split_once('=') {
                cur.push((k.trim().to_string(), v.trim().trim_matches('"').to_string()));
            } else {
                cur.push((seg.to_string(), String::new()));
            }
            cookies.push(cur);
        }
        return cookies;
    }

    let mut cookies: Vec<Vec<(String, String)>> = Vec::new();
    let mut cur: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = header.chars().collect();
    while i < chars.len() {
        let c = chars[i];
        if c == ',' {
            // A comma is part of an `Expires` date value (e.g. "Wed, 21 Oct 2026")
            // only when the previous attribute is `Expires`; otherwise it separates
            // one Set-Cookie block from the next.
            let in_expires_date = cur
                .last()
                .map(|(k, _)| k.eq_ignore_ascii_case("expires"))
                .unwrap_or(false);
            if in_expires_date {
                if let Some((_, v)) = cur.last_mut() {
                    v.push(',');
                }
                i += 1;
                continue;
            }
            // A real cookie separator: flush current block and start a new one.
            cookies.push(std::mem::take(&mut cur));
            i += 1;
            continue;
        }
        if c == ';' {
            // Cookie request header: `;` separates independent cookies.
            // Set-Cookie: `;` separates attributes *within* the same cookie block.
            if !is_set_cookie || cur.is_empty() {
                if !cur.is_empty() {
                    cookies.push(std::mem::take(&mut cur));
                }
            }
            i += 1;
            continue;
        }
        // Accumulate a token: name=value until ; or ,
        let mut j = i;
        while j < chars.len() && chars[j] != ';' && chars[j] != ',' {
            j += 1;
        }
        let token: String = chars[i..j].iter().collect();
        let token = token.trim();
        if !token.is_empty() {
            if let Some((k, v)) = token.split_once('=') {
                cur.push((
                    k.trim().to_string(),
                    v.trim().trim_matches('"').to_string(),
                ));
            } else {
                cur.push((token.to_string(), String::new()));
            }
        }
        i = j;
    }
    if !cur.is_empty() {
        cookies.push(cur);
    }
    cookies
}

fn normalized_key(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '_' && *c != '-')
        .collect::<String>()
        .to_ascii_lowercase()
}

fn cookie_attr_value<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    let want = normalized_key(key);
    pairs
        .iter()
        .find(|(k, _)| normalized_key(k) == want)
        .map(|(_, v)| v.as_str())
}

fn bool_attr(pairs: &[(String, String)], key: &str) -> Option<bool> {
    let want = normalized_key(key);
    pairs
        .iter()
        .find(|(k, _)| normalized_key(k) == want)
        .map(|(_, v)| {
            if v.is_empty() {
                // Present-without-value boolean flag (e.g. `Secure` / `HttpOnly`).
                true
            } else {
                !(v.eq_ignore_ascii_case("false")
                    || v.eq_ignore_ascii_case("0")
                    || v.eq_ignore_ascii_case("no"))
            }
        })
}

/// Parse a Cookie request header or Set-Cookie response header (RFC 6265 subset)
/// into a list of `{name, value, path, domain, expires, max_age, secure, http_only, same_site}`.
/// `is_response=true` treats the input as one or more `Set-Cookie` blocks.
pub fn cookie_parse(text: &Value, is_response: Option<&Value>) -> Result<Value, String> {
    let header = as_text(text, "text")?;
    let is_response = truthy_flag(is_response);
    let blocks = split_cookie_tokens(header, is_response);
    let mut out = Vec::new();
    for block in blocks {
        let name = block
            .first()
            .map(|(k, _)| k.clone())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let value = block.first().map(|(_, v)| v.clone()).unwrap_or_default();
        let mut entry: Vec<(String, Value)> = vec![
            ("name".into(), Value::Text(name)),
            ("value".into(), Value::Text(value)),
        ];
        // Attribute with optional value -> Text or None.
        let text_attrs = [
            ("path", "path"),
            ("domain", "domain"),
            ("expires", "expires"),
            ("max_age", "max_age"),
            ("sameSite", "same_site"),
        ];
        for (attr, key) in text_attrs {
            let v = cookie_attr_value(&block, attr);
            entry.push((
                key.to_string(),
                match v {
                    Some(s) => Value::Text(s.to_string()),
                    None => Value::None,
                },
            ));
        }
        // Boolean flags (Secure / HttpOnly are present-without-value).
        for attr in ["secure", "http_only"] {
            let present = bool_attr(&block, attr);
            entry.push((
                attr.to_string(),
                match present {
                    Some(b) => Value::Bool(b),
                    None => Value::None,
                },
            ));
        }
        out.push(Value::Map(entry));
    }
    Ok(Value::List(out))
}

/// Parse a `multipart/form-data` body (given its boundary) into
/// `[{name, filename?, content_type?, value}]`. Field values are decoded as text.
pub fn multipart_parse(body: &Value, boundary: &Value) -> Result<Value, String> {
    let body = as_text(body, "body")?;
    let boundary = as_text(boundary, "boundary")?;
    let boundary = boundary.trim();
    if boundary.is_empty() {
        return Err("multipart boundary is empty".into());
    }
    let delim = format!("--{boundary}");
    let mut parts = Vec::new();
    let mut rest = body;
    loop {
        let Some(pos) = rest.find(&delim) else {
            break;
        };
        rest = &rest[pos + delim.len()..];
        // After boundary: either `--` (end) or CRLF + headers
        if rest.strip_prefix("--").is_some() {
            break;
        }
        let rest_trim = rest.strip_prefix("\r\n").or_else(|| rest.strip_prefix('\n'));
        let Some(after) = rest_trim else {
            break;
        };
        rest = after;
        // Headers until blank line
        let Some(header_end) = rest.find("\r\n\r\n").or_else(|| rest.find("\n\n")) else {
            break;
        };
        let header_block = &rest[..header_end];
        let header_len = if rest[header_end..].starts_with("\r\n\r\n") {
            4
        } else {
            2
        };
        let body_start = header_end + header_len;
        // Part body until next boundary
        let body_end = rest[body_start..]
            .find(&delim)
            .map(|p| body_start + p)
            .unwrap_or(rest.len());
        let part_body = &rest[body_start..body_end];
        // Trim trailing CRLF before boundary
        let part_body = part_body
            .strip_suffix("\r\n")
            .or_else(|| part_body.strip_suffix('\n'))
            .unwrap_or(part_body);

        let mut name = String::new();
        let mut filename = None;
        let mut content_type = None;
        for line in header_block.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let lk = line.to_ascii_lowercase();
            if let Some(v) = lk.strip_prefix("content-disposition:") {
                // form-data; name="x"; filename="y.txt"
                for seg in v.split(';') {
                    let seg = seg.trim();
                    if let Some(val) = seg.strip_prefix("name=") {
                        name = val.trim_matches('"').to_string();
                    } else if let Some(val) = seg.strip_prefix("filename=") {
                        filename = Some(val.trim_matches('"').to_string());
                    }
                }
            } else if let Some(v) = lk.strip_prefix("content-type:") {
                content_type = Some(v.trim().to_string());
            }
        }
        let mut entry = vec![("name".to_string(), Value::Text(name))];
        match filename {
            Some(f) if !f.is_empty() => {
                entry.push(("filename".to_string(), Value::Text(f)));
                entry.push((
                    "content_type".to_string(),
                    Value::Text(content_type.unwrap_or_default()),
                ));
                entry.push(("value".to_string(), Value::Text(part_body.to_string())));
            }
            _ => {
                entry.push(("value".to_string(), Value::Text(part_body.to_string())));
            }
        }
        parts.push(Value::Map(entry));
        rest = &rest[body_end..];
        if rest.starts_with(&delim) {
            continue;
        }
        break;
    }
    Ok(Value::List(parts))
}

/// Parse Markdown / GFM into HTML (pure transform, no network I/O).
pub fn markdown_parse(text: &Value) -> Result<Value, String> {
    let md = as_text(text, "text")?;
    Ok(Value::Text(markdown_to_html(md)))
}

fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut opts = Options::empty();
    opts.insert(
        Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS,
    );
    let parser = Parser::new_ext(md, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
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

#[cfg(feature = "net-host")]
fn read_sse_body_to_events<R: BufRead>(
    mut reader: R,
    echo: bool,
) -> Result<Vec<Value>, String> {
    let mut buf = String::new();
    let mut events = Vec::new();
    let mut result = String::new();
    let mut data_buf: Vec<String> = Vec::new();
    let mut echoed = false;
    let mut saw_reasoning = false;
    // Soft cap on *stored* event payload (bus still gets every frame). DeepSeek
    // thinking can exceed tens of MiB; we must not retain the raw wire body.
    let mut stored_bytes: usize = 0;
    const STORE_CAP: usize = 2 * 1024 * 1024;

    let flush_data = |data_buf: &mut Vec<String>,
                      events: &mut Vec<Value>,
                      result: &mut String,
                      echoed: &mut bool,
                      saw_reasoning: &mut bool,
                      stored_bytes: &mut usize,
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
        // Always publish to the view bus; optionally skip retaining huge histories.
        let before = events.len();
        let stop = push_openai_delta_events(
            &v,
            events,
            result,
            echoed,
            saw_reasoning,
            echo,
        );
        // Cap retained history only; EventBus already has every frame for the view.
        let mut i = before;
        while i < events.len() {
            let add = match &events[i] {
                Value::Map(m) => m
                    .iter()
                    .filter(|(k, _)| *k == "text" || *k == "result" || *k == "message")
                    .filter_map(|(_, val)| match val {
                        Value::Text(t) => Some(t.len()),
                        _ => None,
                    })
                    .sum::<usize>(),
                _ => 0,
            };
            if stored_bytes.saturating_add(add) > STORE_CAP {
                events.truncate(i);
                break;
            }
            *stored_bytes = stored_bytes.saturating_add(add);
            i += 1;
        }
        Ok(stop)
    };

    loop {
        buf.clear();
        let n = reader
            .read_line(&mut buf)
            .map_err(|e| format!("http sse read: {e}"))?;
        if n == 0 {
            break;
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
                &mut stored_bytes,
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
        &mut stored_bytes,
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
#[cfg(feature = "net-host")]
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

    #[test]
    fn sse_body_over_8mib_does_not_error() {
        // Formerly failed with "http sse response exceeds 8MiB limit" because raw
        // wire bytes were accumulated. Streaming must keep going; store is soft-capped.
        let frame =
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"0123456789\"}}]}\n\n";
        let mut body = String::with_capacity(9 * 1024 * 1024);
        while body.len() < 8 * 1024 * 1024 + 1024 {
            body.push_str(frame);
        }
        body.push_str("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n");
        body.push_str("data: [DONE]\n\n");
        let ev = read_sse_body_to_events(std::io::Cursor::new(body), false).unwrap();
        assert!(
            ev.iter().any(|e| matches!(
                e,
                Value::Map(m) if m.iter().any(|(k, v)| k == "type" && matches!(v, Value::Text(t) if t == "done"))
                    && m.iter().any(|(k, v)| k == "result" && matches!(v, Value::Text(t) if t == "ok"))
            )),
            "expected done with result ok"
        );
    }

    #[test]
    fn cookie_request_header_parse() {
        let v = cookie_parse(
            &Value::Text("session=abc123; theme=dark; lang=zh-CN".into()),
            Some(&Value::Bool(false)),
        )
        .unwrap();
        let Value::List(items) = v else {
            panic!("expected list");
        };
        assert_eq!(items.len(), 3);
        let Value::Map(first) = &items[0] else {
            panic!("expected map");
        };
        assert!(first
            .iter()
            .any(|(k, v)| k == "name" && matches!(v, Value::Text(t) if t == "session")));
        assert!(first
            .iter()
            .any(|(k, v)| k == "value" && matches!(v, Value::Text(t) if t == "abc123")));
    }

    #[test]
    fn set_cookie_response_parse() {
        let header = "id=42; Path=/; HttpOnly; Secure; SameSite=Lax, theme=light; Expires=Wed, 21 Oct 2026 07:28:00 GMT; Max-Age=3600";
        let blocks = split_cookie_tokens(header, true);
        eprintln!("blocks = {blocks:?}");
        let v = cookie_parse(
            &Value::Text(header.into()),
            Some(&Value::Bool(true)),
        )
        .unwrap();
        let Value::List(items) = v else {
            panic!("expected list");
        };
        assert_eq!(items.len(), 2, "two Set-Cookie blocks");
        let Value::Map(first) = &items[0] else {
            panic!("expected map");
        };
        assert!(first
            .iter()
            .any(|(k, v)| k == "path" && matches!(v, Value::Text(t) if t == "/")));
        assert!(first
            .iter()
            .any(|(k, v)| k == "http_only" && matches!(v, Value::Bool(true))));
        assert!(first
            .iter()
            .any(|(k, v)| k == "secure" && matches!(v, Value::Bool(true))));
        assert!(first
            .iter()
            .any(|(k, v)| k == "same_site" && matches!(v, Value::Text(t) if t == "Lax")));
        let Value::Map(second) = &items[1] else {
            panic!("expected map");
        };
        assert!(second
            .iter()
            .any(|(k, v)| k == "name" && matches!(v, Value::Text(t) if t == "theme")));
        assert!(second.iter().any(|(k, v)| k == "max_age"
            && matches!(v, Value::Text(t) if t == "3600")));
    }

    #[test]
    fn multipart_form_parse() {
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nHello\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\r\nContent-Type: text/plain\r\n\r\nfile body\r\n--{boundary}--\r\n"
        );
        let v = multipart_parse(&Value::Text(body.into()), &Value::Text(boundary.into())).unwrap();
        let Value::List(items) = v else {
            panic!("expected list");
        };
        assert_eq!(items.len(), 2);
        let Value::Map(f0) = &items[0] else {
            panic!("expected map");
        };
        assert!(f0
            .iter()
            .any(|(k, v)| k == "name" && matches!(v, Value::Text(t) if t == "title")));
        assert!(f0
            .iter()
            .any(|(k, v)| k == "value" && matches!(v, Value::Text(t) if t == "Hello")));
        let Value::Map(f1) = &items[1] else {
            panic!("expected map");
        };
        assert!(f1
            .iter()
            .any(|(k, v)| k == "name" && matches!(v, Value::Text(t) if t == "file")));
        assert!(f1
            .iter()
            .any(|(k, v)| k == "filename" && matches!(v, Value::Text(t) if t == "a.txt")));
        assert!(f1
            .iter()
            .any(|(k, v)| k == "value" && matches!(v, Value::Text(t) if t == "file body")));
    }

    #[test]
    fn markdown_parse_basic() {
        let md = Value::Text("# Hello\n\nA **bold** word.".into());
        let out = markdown_parse(&md).unwrap();
        let Value::Text(html) = out else {
            panic!("expected text html");
        };
        assert!(html.contains("<h1"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}

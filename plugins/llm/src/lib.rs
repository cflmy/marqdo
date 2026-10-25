//! Marqdo LLM transport plugin (C ABI v2): OpenAI-compatible chat completions.

use std::io::{BufRead, BufReader, Read};
use std::os::raw::{c_char, c_int};
use std::time::Duration;

use serde_json::{json, Map, Value};

use marqdo_plugin_sdk::*;

const MAX_BODY: usize = 32 * 1024 * 1024;

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    opt_text(args, key).ok_or_else(|| format!("missing text `{key}`"))
}

fn opt_bool(args: &Value, key: &str, default: bool) -> bool {
    match args.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => matches!(s.as_str(), "true" | "True" | "1" | "yes"),
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        _ => default,
    }
}

fn read_limited(mut r: impl Read) -> Result<String, String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = r.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if buf.len() + n > MAX_BODY {
            return Err("response body too large".into());
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    String::from_utf8(buf).map_err(|e| e.to_string())
}

/// Prefer null over zero when the provider omitted a field (ModelCallUsageV1 / F04).
fn token_or_null(v: Option<i64>) -> Value {
    match v {
        Some(n) if n >= 0 => json!(n),
        _ => Value::Null,
    }
}

fn usage_from(data: &Value) -> Value {
    let u = data.get("usage").cloned().unwrap_or(Value::Null);
    if u.is_null() || !u.is_object() {
        return json!({
            "prompt_tokens": Value::Null,
            "completion_tokens": Value::Null,
            "total_tokens": Value::Null,
            "usage_state": "unknown",
            "measurement_source": "none",
        });
    }
    let pt = u.get("prompt_tokens").and_then(|v| v.as_i64());
    let ct = u.get("completion_tokens").and_then(|v| v.as_i64());
    let tt = u
        .get("total_tokens")
        .and_then(|v| v.as_i64())
        .or_else(|| match (pt, ct) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        });
    json!({
        "prompt_tokens": token_or_null(pt),
        "completion_tokens": token_or_null(ct),
        "total_tokens": token_or_null(tt),
        "usage_state": "final",
        "measurement_source": "provider",
    })
}

fn tool_calls_from(data: &Value) -> Value {
    data.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c0| c0.get("message"))
        .and_then(|m| m.get("tool_calls"))
        .cloned()
        .unwrap_or_else(|| json!([]))
}

fn cancel_requested() -> bool {
    if std::env::var("MARQDO_CANCEL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return true;
    }
    if let Ok(tok) = std::env::var("MARQDO_CANCEL_TOKEN") {
        let path = std::path::Path::new(&tok);
        if path.is_file() {
            if let Ok(body) = std::fs::read_to_string(path) {
                let t = body.trim();
                return t.is_empty() || t == "1" || t.eq_ignore_ascii_case("cancelled");
            }
            return true;
        }
    }
    false
}

fn parse_sse_events(reader: impl BufRead, echo: bool) -> Result<Vec<Value>, String> {
    let mut events = Vec::new();
    let mut data_lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        if cancel_requested() {
            events.push(json!({"type": "cancelled", "message": "MARQDO_CANCEL"}));
            return Ok(events);
        }
        let line = line.map_err(|e| e.to_string())?;
        if echo {
            eprint!("{line}\n");
        }
        if line.is_empty() {
            if !data_lines.is_empty() {
                let data = data_lines.join("\n");
                data_lines.clear();
                if data.trim() == "[DONE]" {
                    events.push(json!({"type": "done", "data": "[DONE]"}));
                    break;
                }
                events.push(json!({"type": "data", "data": data}));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }
    if !data_lines.is_empty() {
        let data = data_lines.join("\n");
        if data.trim() == "[DONE]" {
            events.push(json!({"type": "done", "data": "[DONE]"}));
        } else {
            events.push(json!({"type": "data", "data": data}));
        }
    }
    Ok(events)
}

fn build_messages(args: &Value) -> Result<Value, String> {
    if let Some(arr) = args.get("messages").and_then(|v| v.as_array()) {
        if arr.is_empty() {
            return Err("messages array must not be empty".into());
        }
        // Validate minimal role/content shape; reject unsupported part types loudly.
        for (i, m) in arr.iter().enumerate() {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
            if role.is_empty() {
                return Err(format!("llm.unsupported_input: messages[{i}].role missing"));
            }
            if let Some(parts) = m.get("content").and_then(|c| c.as_array()) {
                for (j, p) in parts.iter().enumerate() {
                    let ty = p.get("type").and_then(|t| t.as_str()).unwrap_or("text");
                    if ty != "text" {
                        return Err(format!(
                            "llm.unsupported_input: messages[{i}].content[{j}] type={ty}"
                        ));
                    }
                }
            }
        }
        return Ok(Value::Array(arr.clone()));
    }
    let prompt = arg_text(args, "prompt")?;
    Ok(json!([{"role": "user", "content": prompt}]))
}

fn chat_completions(args: &Value) -> Result<Value, String> {
    if cancel_requested() {
        return Ok(json!({
            "text": "",
            "usage": {
                "prompt_tokens": Value::Null,
                "completion_tokens": Value::Null,
                "total_tokens": Value::Null,
                "usage_state": "unknown",
                "measurement_source": "none",
            },
            "finish": "cancelled",
            "error": "cancelled",
            "tool_calls": [],
            "transport": "plugin",
        }));
    }
    // Strict selection mode: refuse env fallback when selection.strict is set.
    let strict = opt_bool(args, "strict_selection", false)
        || args
            .get("selection")
            .and_then(|s| s.get("strict"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    let selection = args.get("selection").cloned();
    if strict {
        if selection.is_none() {
            return Err("llm.selection_missing:selection".into());
        }
        let sel = selection.as_ref().unwrap();
        for key in ["selection_id", "provider_id", "model_id", "upstream_model_id"] {
            if sel
                .get(key)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .is_none()
            {
                return Err(format!("llm.selection_missing:{key}"));
            }
        }
    }

    let base_url = match arg_text(args, "base_url") {
        Ok(u) => u,
        Err(_) if strict => return Err("llm.selection_missing:base_url".into()),
        Err(e) => return Err(e),
    };
    let auth_mode = opt_text(args, "auth_mode")
        .or_else(|| {
            selection
                .as_ref()
                .and_then(|s| s.get("auth_mode"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("api_key");
    let api_key = if auth_mode == "none" {
        ""
    } else {
        arg_text(args, "api_key").unwrap_or("")
    };
    if strict && auth_mode != "none" && api_key.is_empty() {
        return Err("llm.selection_missing:api_key".into());
    }
    let model = selection
        .as_ref()
        .and_then(|s| s.get("upstream_model_id"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| arg_text(args, "model").ok().map(|s| s.to_string()))
        .ok_or_else(|| "missing text `model`".to_string())?;
    let messages = build_messages(args)?;
    let stream = opt_bool(args, "stream", false);
    let echo = opt_bool(args, "echo", false);
    let suffix = opt_text(args, "suffix").unwrap_or("/chat/completions");
    let bearer = opt_text(args, "bearer").unwrap_or("Bearer ");

    let url = format!("{}{}", base_url.trim_end_matches('/'), suffix);
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("url must start with http:// or https://".into());
    }

    let mut req_body = Map::new();
    req_body.insert("model".into(), Value::String(model.clone()));
    req_body.insert("messages".into(), messages);
    if let Some(params) = selection
        .as_ref()
        .and_then(|s| s.get("parameters"))
        .and_then(|p| p.as_object())
    {
        if let Some(max) = params.get("max_output_tokens").and_then(|v| v.as_u64()) {
            req_body.insert("max_tokens".into(), json!(max));
        }
        if let Some(temp) = params.get("temperature") {
            req_body.insert("temperature".into(), temp.clone());
        }
    }
    if stream {
        req_body.insert("stream".into(), Value::Bool(true));
    }
    let body = Value::Object(req_body).to_string();
    let auth = format!("{bearer}{api_key}");

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .build();
    let mut req = agent
        .post(&url)
        .set("User-Agent", "marqdo-plugin-llm")
        .set("Content-Type", "application/json; charset=utf-8");
    if stream {
        req = req.set("Accept", "text/event-stream");
    }
    if !api_key.is_empty() {
        req = req.set("Authorization", &auth);
    }

    if stream {
        match req.send_string(&body) {
            Ok(r) => {
                let status = r.status();
                if status != 200 {
                    let text = read_limited(r.into_reader())?;
                    return Ok(json!({
                        "events": [{"type": "error", "message": format!("HTTP {status}: {text}")}],
                        "finish": "error",
                        "error": format!("HTTP {status}: {text}"),
                        "transport": "plugin",
                    }));
                }
                let events = parse_sse_events(BufReader::new(r.into_reader()), echo)?;
                Ok(json!({
                    "events": events,
                    "finish": "stop",
                    "transport": "plugin",
                }))
            }
            Err(ureq::Error::Status(code, r)) => {
                let text = read_limited(r.into_reader()).unwrap_or_default();
                Ok(json!({
                    "events": [{"type": "error", "message": format!("HTTP {code}: {text}")}],
                    "finish": "error",
                    "error": format!("HTTP {code}: {text}"),
                    "transport": "plugin",
                }))
            }
            Err(e) => Ok(json!({
                "events": [{"type": "error", "message": format!("http POST sse {url}: {e}")}],
                "finish": "error",
                "error": format!("http POST sse {url}: {e}"),
                "transport": "plugin",
            })),
        }
    } else {
        match req.send_string(&body) {
            Ok(r) => {
                let status = r.status();
                let text = read_limited(r.into_reader())?;
                if status != 200 {
                    return Ok(json!({
                        "text": "",
                        "usage": {
                            "prompt_tokens": Value::Null,
                            "completion_tokens": Value::Null,
                            "total_tokens": Value::Null,
                            "usage_state": "unknown",
                            "measurement_source": "none",
                        },
                        "finish": "error",
                        "error": format!("HTTP {status}: {text}"),
                        "tool_calls": [],
                        "transport": "plugin",
                        "requested_model": model,
                        "resolved_model": model,
                    }));
                }
                let data: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                let msg = data
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c0| c0.get("message"))
                    .cloned()
                    .unwrap_or(json!({}));
                // Some gateways (reasoning models) return null content + reasoning_content.
                let content = msg
                    .get("content")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| msg.get("reasoning_content").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();
                let fr = data
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c0| c0.get("finish_reason"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("stop")
                    .to_string();
                let resolved = data
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or(model.as_str())
                    .to_string();
                Ok(json!({
                    "text": content,
                    "usage": usage_from(&data),
                    "finish": fr,
                    "tool_calls": tool_calls_from(&data),
                    "transport": "plugin",
                    "requested_model": model,
                    "resolved_model": resolved,
                    "selection": selection.as_ref().map(|s| {
                        let mut snap = s.clone();
                        if let Some(obj) = snap.as_object_mut() {
                            obj.remove("api_key");
                        }
                        snap
                    }),
                }))
            }
            Err(ureq::Error::Status(code, r)) => {
                let text = read_limited(r.into_reader()).unwrap_or_default();
                Ok(json!({
                    "text": "",
                    "usage": {
                        "prompt_tokens": Value::Null,
                        "completion_tokens": Value::Null,
                        "total_tokens": Value::Null,
                        "usage_state": "unknown",
                        "measurement_source": "none",
                    },
                    "finish": "error",
                    "error": format!("HTTP {code}: {text}"),
                    "tool_calls": [],
                    "transport": "plugin",
                    "requested_model": model,
                    "resolved_model": model,
                }))
            }
            Err(e) => Ok(json!({
                "text": "",
                "usage": {
                    "prompt_tokens": Value::Null,
                    "completion_tokens": Value::Null,
                    "total_tokens": Value::Null,
                    "usage_state": "unknown",
                    "measurement_source": "none",
                },
                "finish": "error",
                "error": format!("http POST {url}: {e}"),
                "tool_calls": [],
                "transport": "plugin",
                "requested_model": model,
                "resolved_model": model,
            })),
        }
    }
}

unsafe extern "C" fn llm_chat_completions(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let r = (|| {
        let args = parse_args(args_json)?;
        chat_completions(&args)
    })();
    reply(out_json, err_msg, r)
}

unsafe extern "C" fn llm_ping(
    _args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    reply(out_json, err_msg, Ok(json!({"ok": true, "plugin": "llm"})))
}

marqdo_plugin!(marqdo_plugin_sdk::ABI_VERSION, [
    ("llm_ping", "", llm_ping),
    ("llm_chat_completions", "base_url,api_key,model,prompt,messages,selection,strict_selection,auth_mode,stream,echo,suffix,bearer", llm_chat_completions),
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f03_messages_preserve_roles_and_reject_image() {
        let msgs = build_messages(&json!({
            "messages": [
                {"role": "system", "content": "rules"},
                {"role": "user", "content": "你好"},
                {"role": "assistant", "content": "先前"},
                {"role": "user", "content": "最新用户"}
            ]
        }))
        .unwrap();
        let arr = msgs.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[3]["role"], "user");
        assert_eq!(arr[3]["content"], "最新用户");

        let err = build_messages(&json!({
            "messages": [{"role": "user", "content": [{"type": "image", "url": "x"}]}]
        }))
        .unwrap_err();
        assert!(err.contains("unsupported_input"), "{err}");
    }

    #[test]
    fn f01_strict_selection_does_not_invent_env_model() {
        std::env::remove_var("MARQDO_CANCEL");
        std::env::remove_var("MARQDO_CANCEL_TOKEN");
        let err = chat_completions(&json!({
            "strict_selection": true,
            "base_url": "http://127.0.0.1:9",
            "api_key": "k",
            "prompt": "hi"
            // missing selection → must fail before any HTTP
        }))
        .unwrap_err();
        assert!(err.contains("selection_missing"), "{err}");
    }

    #[test]
    fn f06_cancel_short_circuits_before_http() {
        // Prefer token file over process-global MARQDO_CANCEL to avoid parallel test races.
        let tok = std::env::temp_dir().join(format!("llm-cancel-{}", std::process::id()));
        std::fs::write(&tok, "cancelled").unwrap();
        std::env::set_var("MARQDO_CANCEL_TOKEN", &tok);
        let out = chat_completions(&json!({
            "base_url": "http://127.0.0.1:9",
            "api_key": "k",
            "model": "m",
            "prompt": "hi"
        }));
        std::env::remove_var("MARQDO_CANCEL_TOKEN");
        let _ = std::fs::remove_file(&tok);
        let out = out.unwrap();
        assert_eq!(out["finish"], "cancelled");
    }

    /// F01: two local endpoints receive distinct models/auth — no cross-talk.
    #[test]
    fn f01_dual_endpoint_isolation() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::{Arc, Mutex};
        use std::thread;

        fn spawn_mock(tag: &'static str, seen: Arc<Mutex<Vec<String>>>) -> String {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut raw = Vec::new();
                    let mut buf = [0u8; 4096];
                    loop {
                        let n = stream.read(&mut buf).unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        raw.extend_from_slice(&buf[..n]);
                        if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                            let header = String::from_utf8_lossy(&raw[..pos]);
                            let cl = header
                                .lines()
                                .find_map(|l| l.strip_prefix("Content-Length:"))
                                .and_then(|v| v.trim().parse::<usize>().ok())
                                .unwrap_or(0);
                            let body_start = pos + 4;
                            while raw.len() < body_start + cl {
                                let n = stream.read(&mut buf).unwrap_or(0);
                                if n == 0 {
                                    break;
                                }
                                raw.extend_from_slice(&buf[..n]);
                            }
                            break;
                        }
                    }
                    let req = String::from_utf8_lossy(&raw).to_string();
                    seen.lock().unwrap().push(format!("{tag}:{req}"));
                    let body = r#"{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
            });
            format!("http://{addr}")
        }

        let seen_a = Arc::new(Mutex::new(Vec::new()));
        let seen_b = Arc::new(Mutex::new(Vec::new()));
        let url_a = spawn_mock("A", seen_a.clone());
        let url_b = spawn_mock("B", seen_b.clone());

        let out_a = chat_completions(&json!({
            "base_url": url_a,
            "api_key": "key-a",
            "model": "model-a",
            "auth_mode": "api_key",
            "messages": [{"role":"user","content":"from-a"}],
            "selection": {
                "selection_id": "sel-a",
                "provider_id": "p-a",
                "model_id": "m-a",
                "upstream_model_id": "model-a",
                "strict": true
            },
            "strict_selection": true
        }))
        .unwrap();
        let out_b = chat_completions(&json!({
            "base_url": url_b,
            "api_key": "key-b",
            "model": "model-b",
            "auth_mode": "api_key",
            "messages": [{"role":"user","content":"from-b"}],
            "selection": {
                "selection_id": "sel-b",
                "provider_id": "p-b",
                "model_id": "m-b",
                "upstream_model_id": "model-b",
                "strict": true
            },
            "strict_selection": true
        }))
        .unwrap();

        assert_eq!(out_a["finish"], "stop", "{out_a}");
        assert_eq!(out_b["finish"], "stop", "{out_b}");
        let a = seen_a.lock().unwrap().join("\n");
        let b = seen_b.lock().unwrap().join("\n");
        assert!(a.contains("model-a"), "A saw wrong model: {a}");
        assert!(a.contains("key-a") || a.contains("Authorization"), "{a}");
        assert!(!a.contains("model-b"), "A must not see B's model: {a}");
        assert!(b.contains("model-b"), "B saw wrong model: {b}");
        assert!(!b.contains("model-a"), "B must not see A's model: {b}");
    }
}

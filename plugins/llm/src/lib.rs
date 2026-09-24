//! Marqdo LLM transport plugin (C ABI v2): OpenAI-compatible chat completions.

use std::ffi::{CStr, CString};
use std::io::{BufRead, BufReader, Read};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::time::Duration;

use serde_json::{json, Map, Value};

const ABI_VERSION: u32 = 2;
const MAX_BODY: usize = 32 * 1024 * 1024;

type PluginFn = unsafe extern "C" fn(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

type HostQueryFn = unsafe extern "C" fn(
    userdata: *mut c_void,
    name: *const c_char,
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

#[repr(C)]
pub struct MarqdoHostApi {
    pub userdata: *mut c_void,
    pub register_fn: Option<
        unsafe extern "C" fn(
            userdata: *mut c_void,
            name: *const c_char,
            params: *const c_char,
            fn_ptr: PluginFn,
        ) -> c_int,
    >,
    pub alloc: Option<unsafe extern "C" fn(n: usize) -> *mut c_void>,
    pub free: Option<unsafe extern "C" fn(p: *mut c_void)>,
    pub host_query: Option<HostQueryFn>,
}

static mut HOST_FREE: Option<unsafe extern "C" fn(*mut c_void)> = None;
static mut HOST_ALLOC: Option<unsafe extern "C" fn(usize) -> *mut c_void> = None;

unsafe fn host_strdup(s: &str) -> *mut c_char {
    let alloc = HOST_ALLOC.expect("host alloc");
    let bytes = s.as_bytes();
    let p = alloc(bytes.len() + 1) as *mut u8;
    if p.is_null() {
        return ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    *p.add(bytes.len()) = 0;
    p as *mut c_char
}

fn set_out(out: *mut *mut c_char, s: &str) {
    if out.is_null() {
        return;
    }
    unsafe {
        *out = host_strdup(s);
    }
}

fn set_err(err: *mut *mut c_char, s: &str) {
    if err.is_null() {
        return;
    }
    unsafe {
        *err = host_strdup(s);
    }
}

fn parse_args(args_json: *const c_char) -> Result<Value, String> {
    if args_json.is_null() {
        return Ok(json!({}));
    }
    let s = unsafe { CStr::from_ptr(args_json) }
        .to_str()
        .map_err(|_| "args not utf-8".to_string())?;
    if s.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn reply(out_json: *mut *mut c_char, err_msg: *mut *mut c_char, r: Result<Value, String>) -> c_int {
    match r {
        Ok(v) => {
            set_out(out_json, &v.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

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

fn usage_from(data: &Value) -> Value {
    let u = data.get("usage").cloned().unwrap_or(Value::Null);
    let pt = u.get("prompt_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
    let ct = u
        .get("completion_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let tt = u
        .get("total_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(pt + ct);
    json!({
        "prompt_tokens": pt,
        "completion_tokens": ct,
        "total_tokens": tt,
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

fn parse_sse_events(reader: impl BufRead, echo: bool) -> Result<Vec<Value>, String> {
    let mut events = Vec::new();
    let mut data_lines: Vec<String> = Vec::new();
    for line in reader.lines() {
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

fn chat_completions(args: &Value) -> Result<Value, String> {
    let base_url = arg_text(args, "base_url")?;
    let api_key = arg_text(args, "api_key").unwrap_or("");
    let model = arg_text(args, "model")?;
    let prompt = arg_text(args, "prompt")?;
    let stream = opt_bool(args, "stream", false);
    let echo = opt_bool(args, "echo", false);
    let suffix = opt_text(args, "suffix").unwrap_or("/chat/completions");
    let bearer = opt_text(args, "bearer").unwrap_or("Bearer ");

    let url = format!("{}{}", base_url.trim_end_matches('/'), suffix);
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("url must start with http:// or https://".into());
    }

    let mut req_body = Map::new();
    req_body.insert("model".into(), Value::String(model.to_string()));
    req_body.insert(
        "messages".into(),
        json!([{"role": "user", "content": prompt}]),
    );
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
                    return Err(format!("HTTP {status}: {text}"));
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
                Err(format!("HTTP {code}: {text}"))
            }
            Err(e) => Err(format!("http POST sse {url}: {e}")),
        }
    } else {
        match req.send_string(&body) {
            Ok(r) => {
                let status = r.status();
                let text = read_limited(r.into_reader())?;
                if status != 200 {
                    return Err(format!("HTTP {status}: {text}"));
                }
                let data: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                let msg = data
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c0| c0.get("message"))
                    .cloned()
                    .unwrap_or(json!({}));
                let content = msg
                    .get("content")
                    .and_then(|v| v.as_str())
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
                Ok(json!({
                    "text": content,
                    "usage": usage_from(&data),
                    "finish": fr,
                    "tool_calls": tool_calls_from(&data),
                    "transport": "plugin",
                }))
            }
            Err(ureq::Error::Status(code, r)) => {
                let text = read_limited(r.into_reader()).unwrap_or_default();
                Err(format!("HTTP {code}: {text}"))
            }
            Err(e) => Err(format!("http POST {url}: {e}")),
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

fn register(host: &MarqdoHostApi, name: &str, params: &str, fn_ptr: PluginFn) -> c_int {
    let register = match host.register_fn {
        Some(f) => f,
        None => return 1,
    };
    let n = CString::new(name).unwrap();
    let p = CString::new(params).unwrap();
    unsafe { register(host.userdata, n.as_ptr(), p.as_ptr(), fn_ptr) }
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_shutdown() {}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_init(host: *const MarqdoHostApi) -> c_int {
    if host.is_null() {
        return 1;
    }
    let host = &*host;
    HOST_ALLOC = host.alloc;
    HOST_FREE = host.free;
    let regs = [
        ("llm_ping", "", llm_ping as PluginFn),
        (
            "llm_chat_completions",
            "base_url,api_key,model,prompt,stream,echo,suffix,bearer",
            llm_chat_completions as PluginFn,
        ),
    ];
    for (name, params, f) in regs {
        if register(host, name, params, f) != 0 {
            return 1;
        }
    }
    0
}

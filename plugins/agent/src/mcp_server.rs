//! Minimal MCP Server — stdio JSON-RPC + Streamable HTTP (spec 2025-03-26).

use serde_json::{json, Map, Value};
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{host_query_json_args, parse_args, set_err, set_out};

fn read_line_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut line = String::new();
    let n = reader
        .read_line(&mut line)
        .map_err(|e| format!("stdin: {e}"))?;
    if n == 0 {
        return Ok(None);
    }
    let line = line.trim();
    if line.is_empty() {
        return read_line_message(reader);
    }
    let v: Value = serde_json::from_str(line).map_err(|e| format!("json-rpc parse: {e}"))?;
    Ok(Some(v))
}

fn write_message(msg: &Value) -> Result<(), String> {
    let mut out = std::io::stdout().lock();
    let s = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    writeln!(out, "{s}").map_err(|e| format!("stdout: {e}"))?;
    out.flush().map_err(|e| format!("flush: {e}"))?;
    Ok(())
}

fn rpc_result(id: &Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn rpc_error(id: &Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn tools_list(tools: &[Value]) -> Value {
    let mut arr = Vec::new();
    for t in tools {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let desc = t
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if name.is_empty() {
            continue;
        }
        arr.push(json!({
            "name": name,
            "description": desc,
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": true
            }
        }));
    }
    json!({ "tools": arr })
}

fn call_tool(tools: &[Value], name: &str, arguments: &Value) -> Result<Value, String> {
    let tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some(name))
        .ok_or_else(|| format!("unknown tool `{name}`"))?;
    let fn_path = tool
        .get("fn")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("tool `{name}` missing fn"))?
        .to_string();
    let result = host_query_json_args(
        "call_lib_path",
        &json!({
            "path": fn_path,
            "args": arguments,
        }),
    )?;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".into())
        }],
        "isError": false,
        "authority": "workbook"
    }))
}

/// Handle one JSON-RPC message. Returns `None` for notifications (no response body).
fn dispatch_rpc(name: &str, tools: &[Value], msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let method = msg.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(json!({}));

    if matches!(id, Value::Null) && method.starts_with("notifications/") {
        return None;
    }

    Some(match method {
        "initialize" => rpc_result(
            &id,
            json!({
                "protocolVersion": "2025-03-26",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": name, "version": "0.1.0" }
            }),
        ),
        "tools/list" | "list_tools" => rpc_result(&id, tools_list(tools)),
        "tools/call" | "call_tool" => {
            let tname = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(tools, tname, &args) {
                Ok(r) => rpc_result(&id, r),
                Err(e) => rpc_error(&id, -32000, &e),
            }
        }
        "ping" => rpc_result(&id, json!({})),
        "" => rpc_error(&id, -32600, "missing method"),
        other => rpc_error(&id, -32601, &format!("method not found: {other}")),
    })
}

/// Blocking stdio MCP loop. `tools` is a JSON array of `{name, fn, description}`.
pub fn serve_stdio(name: &str, tools: Vec<Value>) -> Result<Value, String> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    loop {
        let Some(msg) = read_line_message(&mut reader)? else {
            break;
        };
        if let Some(reply) = dispatch_rpc(name, &tools, &msg) {
            write_message(&reply)?;
        }
    }
    Ok(json!({ "ok": true, "transport": "stdio" }))
}

fn new_session_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("mcp-{ms}-{n}")
}

fn read_http_request(
    reader: &mut BufReader<TcpStream>,
) -> Result<(String, String, Vec<(String, String)>, Vec<u8>), String> {
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| format!("read request: {e}"))?;
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err("bad request line".into());
    }
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let mut headers = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("read header: {e}"))?;
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().to_string();
            if key.eq_ignore_ascii_case("content-length") {
                content_length = val.parse().unwrap_or(0);
            }
            headers.push((key, val));
        }
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|e| format!("read body: {e}"))?;
    }
    Ok((method, path, headers, body))
}

fn header_get<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn write_http(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
    extra: &[(&str, &str)],
) -> Result<(), String> {
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (k, v) in extra {
        head.push_str(k);
        head.push_str(": ");
        head.push_str(v);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    stream
        .write_all(head.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn path_matches(req_path: &str, endpoint: &str) -> bool {
    let base = req_path.split('?').next().unwrap_or(req_path);
    base == endpoint || base.trim_end_matches('/') == endpoint.trim_end_matches('/')
}

fn handle_http_conn(
    stream: TcpStream,
    name: &str,
    tools: &[Value],
    endpoint: &str,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let (method, path, headers, body) = read_http_request(&mut reader)?;
    let mut stream = reader.into_inner();
    if !path_matches(&path, endpoint) {
        write_http(
            &mut stream,
            404,
            "Not Found",
            "application/json",
            br#"{"error":"not_found"}"#,
            &[],
        )?;
        return Ok(());
    }

    if method.eq_ignore_ascii_case("GET") {
        // Optional SSE stream — empty keepalive then close (clients may 405).
        let accept = header_get(&headers, "Accept").unwrap_or("");
        if accept.contains("text/event-stream") {
            let body = b": mcp streamable-http ready\n\n";
            write_http(
                &mut stream,
                200,
                "OK",
                "text/event-stream",
                body,
                &[("Cache-Control", "no-cache")],
            )?;
        } else {
            write_http(
                &mut stream,
                405,
                "Method Not Allowed",
                "application/json",
                br#"{"error":"use_post_for_jsonrpc"}"#,
                &[("Allow", "POST, GET")],
            )?;
        }
        return Ok(());
    }

    if method.eq_ignore_ascii_case("DELETE") {
        write_http(&mut stream, 200, "OK", "application/json", b"{}", &[])?;
        return Ok(());
    }

    if !method.eq_ignore_ascii_case("POST") {
        write_http(
            &mut stream,
            405,
            "Method Not Allowed",
            "application/json",
            br#"{"error":"method_not_allowed"}"#,
            &[("Allow", "POST, GET, DELETE")],
        )?;
        return Ok(());
    }

    let text = String::from_utf8_lossy(&body);
    let msg: Value = serde_json::from_str(text.trim()).map_err(|e| format!("json-rpc: {e}"))?;
    let is_init = msg.get("method").and_then(|v| v.as_str()) == Some("initialize");
    let existing_sid = header_get(&headers, "Mcp-Session-Id")
        .or_else(|| header_get(&headers, "MCP-Session-Id"))
        .map(|s| s.to_string());

    // Notifications → 202 Accepted, empty body.
    if msg.get("id").is_none() || msg.get("id") == Some(&Value::Null) {
        if let Some(m) = msg.get("method").and_then(|v| v.as_str()) {
            if m.starts_with("notifications/") {
                write_http(
                    &mut stream,
                    202,
                    "Accepted",
                    "application/json",
                    b"",
                    &[],
                )?;
                return Ok(());
            }
        }
    }

    let reply = match dispatch_rpc(name, tools, &msg) {
        Some(r) => r,
        None => {
            write_http(
                &mut stream,
                202,
                "Accepted",
                "application/json",
                b"",
                &[],
            )?;
            return Ok(());
        }
    };

    let body = serde_json::to_vec(&reply).map_err(|e| e.to_string())?;
    let mut extras: Vec<(&str, String)> = Vec::new();
    if is_init {
        let sid = existing_sid.unwrap_or_else(new_session_id);
        extras.push(("Mcp-Session-Id", sid));
    } else if let Some(sid) = existing_sid {
        extras.push(("Mcp-Session-Id", sid));
    }
    let extra_refs: Vec<(&str, &str)> = extras.iter().map(|(k, v)| (*k, v.as_str())).collect();
    write_http(
        &mut stream,
        200,
        "OK",
        "application/json",
        &body,
        &extra_refs,
    )?;
    Ok(())
}

/// Streamable HTTP MCP (POST JSON-RPC + optional GET SSE). Spec 2025-03-26.
pub fn serve_http(
    name: &str,
    tools: Vec<Value>,
    host: &str,
    port: u16,
    path: &str,
) -> Result<Value, String> {
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr).map_err(|e| format!("bind {addr}: {e}"))?;
    let name = Arc::new(name.to_string());
    let tools = Arc::new(tools);
    let endpoint = Arc::new(if path.is_empty() {
        "/mcp".to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    });
    eprintln!(
        "mcp streamable-http listening on http://{addr}{} (POST JSON-RPC)",
        &*endpoint
    );
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let name = Arc::clone(&name);
                let tools = Arc::clone(&tools);
                let endpoint = Arc::clone(&endpoint);
                thread::spawn(move || {
                    if let Err(e) = handle_http_conn(stream, &name, &tools, &endpoint) {
                        eprintln!("mcp http conn: {e}");
                    }
                });
            }
            Err(e) => eprintln!("mcp http accept: {e}"),
        }
    }
    Ok(json!({
        "ok": true,
        "transport": "http",
        "host": host,
        "port": port,
        "path": *endpoint
    }))
}

pub fn tools_from_value(v: &Value) -> Vec<Value> {
    if let Some(arr) = v.as_array() {
        return arr.clone();
    }
    if let Value::Object(map) = v {
        let mut out = Vec::new();
        for (name, spec) in map {
            let mut m = Map::new();
            m.insert("name".into(), json!(name));
            if let Some(obj) = spec.as_object() {
                for (k, val) in obj {
                    m.insert(k.clone(), val.clone());
                }
            } else if let Some(s) = spec.as_str() {
                m.insert("fn".into(), json!(s));
            }
            out.push(Value::Object(m));
        }
        return out;
    }
    Vec::new()
}

pub unsafe extern "C" fn agent_mcp_serve(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let args = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &e);
            return 1;
        }
    };
    let transport = args
        .get("transport")
        .and_then(|v| v.as_str())
        .unwrap_or("stdio");
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("marqdo");
    let tools = args
        .get("tools")
        .map(tools_from_value)
        .unwrap_or_default();

    let result = match transport {
        "stdio" => serve_stdio(name, tools),
        "http" | "streamable-http" | "sse" => {
            let host = args
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("127.0.0.1");
            let port = args
                .get("port")
                .and_then(|v| {
                    v.as_u64()
                        .or_else(|| v.as_i64().map(|n| n as u64))
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .unwrap_or(7434) as u16;
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("/mcp");
            serve_http(name, tools, host, port, path)
        }
        other => {
            set_err(
                err_msg,
                &format!("mcp serve: transport `{other}` not supported (use stdio|http)"),
            );
            return 1;
        }
    };

    match result {
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

#[allow(dead_code)]
fn _unused(_: &CStr) {}

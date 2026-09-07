//! Minimal MCP Server (stdio JSON-RPC) — tools map to `lib.member` via host `call_lib_path`.

use serde_json::{json, Map, Value};
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Write};
use std::os::raw::{c_char, c_int};

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

/// Blocking stdio MCP loop. `tools` is a JSON array of `{name, fn, description}`.
pub fn serve_stdio(name: &str, tools: Vec<Value>) -> Result<Value, String> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    loop {
        let Some(msg) = read_line_message(&mut reader)? else {
            break;
        };
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        if matches!(id, Value::Null) && method.starts_with("notifications/") {
            continue;
        }

        let reply_msg = match method {
            "initialize" => rpc_result(
                &id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": name, "version": "0.1.0" }
                }),
            ),
            "tools/list" | "list_tools" => rpc_result(&id, tools_list(&tools)),
            "tools/call" | "call_tool" => {
                let tname = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match call_tool(&tools, tname, &args) {
                    Ok(r) => rpc_result(&id, r),
                    Err(e) => rpc_error(&id, -32000, &e),
                }
            }
            "ping" => rpc_result(&id, json!({})),
            "" => rpc_error(&id, -32600, "missing method"),
            other => rpc_error(&id, -32601, &format!("method not found: {other}")),
        };
        write_message(&reply_msg)?;
    }
    Ok(json!({ "ok": true, "transport": "stdio" }))
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
    if transport != "stdio" {
        set_err(
            err_msg,
            &format!("mcp serve: transport `{transport}` not implemented yet (use stdio)"),
        );
        return 1;
    }
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("marqdo");
    let tools = args
        .get("tools")
        .map(tools_from_value)
        .unwrap_or_default();
    match serve_stdio(name, tools) {
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

// Silence unused import if CStr unused in this module.
#[allow(dead_code)]
fn _unused(_: &CStr) {}

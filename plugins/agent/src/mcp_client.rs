//! MCP Client (stdio JSON-RPC) — spawn a server process, initialize, list/call tools.

use serde_json::{json, Value};
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Write};
use std::os::raw::{c_char, c_int};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use crate::{parse_args, set_err, set_out};

struct Session {
    child: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: i64,
    server_name: String,
}

static SESSIONS: Mutex<Vec<(String, Session)>> = Mutex::new(Vec::new());

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    opt_text(args, key).ok_or_else(|| format!("missing text `{key}`"))
}

fn write_rpc(stdin: &mut impl Write, msg: &Value) -> Result<(), String> {
    let s = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    writeln!(stdin, "{s}").map_err(|e| format!("mcp client write: {e}"))?;
    stdin.flush().map_err(|e| format!("mcp client flush: {e}"))?;
    Ok(())
}

fn read_rpc(reader: &mut impl BufRead) -> Result<Value, String> {
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .map_err(|e| format!("mcp client read: {e}"))?;
        if n == 0 {
            return Err("mcp client: server closed stdout".into());
        }
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return serde_json::from_str(t).map_err(|e| format!("mcp client json: {e}"));
    }
}

fn rpc_call(session: &mut Session, method: &str, params: Value) -> Result<Value, String> {
    let id = session.next_id;
    session.next_id += 1;
    let req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    write_rpc(&mut session.stdin, &req)?;
    // Read until matching id (skip notifications).
    for _ in 0..64 {
        let resp = read_rpc(&mut session.reader)?;
        if resp.get("id") == Some(&json!(id)) {
            if let Some(err) = resp.get("error") {
                let msg = err
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("rpc error");
                return Err(msg.to_string());
            }
            return Ok(resp.get("result").cloned().unwrap_or(Value::Null));
        }
    }
    Err("mcp client: no matching response".into())
}

fn spawn_session(command: &str, args: &[String], name: &str) -> Result<Session, String> {
    let mut cmd = Command::new(command);
    for a in args {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("mcp client spawn `{command}`: {e}"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "mcp client: no stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "mcp client: no stdout".to_string())?;
    let mut session = Session {
        child,
        stdin,
        reader: BufReader::new(stdout),
        next_id: 1,
        server_name: name.to_string(),
    };
    // initialize
    let _ = rpc_call(
        &mut session,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "marqdo-agent", "version": "0.1.0" }
        }),
    )?;
    // optional initialized notification
    let _ = write_rpc(
        &mut session.stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
    );
    let _ = session.server_name;
    // Give slow servers a beat (ignore).
    let _ = Duration::from_millis(1);
    Ok(session)
}

fn command_args(args: &Value) -> Result<(String, Vec<String>), String> {
    let command = arg_text(args, "command")?;
    let mut argv = Vec::new();
    if let Some(arr) = args.get("args").and_then(|v| v.as_array()) {
        for a in arr {
            if let Some(s) = a.as_str() {
                argv.push(s.to_string());
            } else {
                argv.push(a.to_string());
            }
        }
    } else if let Some(s) = opt_text(args, "args") {
        // space-split simple args
        for part in s.split_whitespace() {
            argv.push(part.to_string());
        }
    }
    Ok((command.to_string(), argv))
}

fn session_id_of(args: &Value) -> String {
    opt_text(args, "session")
        .or_else(|| opt_text(args, "id"))
        .unwrap_or("default")
        .to_string()
}

fn with_session<F>(args: &Value, f: F) -> Result<Value, String>
where
    F: FnOnce(&mut Session) -> Result<Value, String>,
{
    let sid = session_id_of(args);
    let mut guard = SESSIONS
        .lock()
        .map_err(|_| "mcp client: session lock poisoned".to_string())?;
    let idx = guard
        .iter()
        .position(|(id, _)| id == &sid)
        .ok_or_else(|| format!("mcp client: no session `{sid}` (call connect first)"))?;
    f(&mut guard[idx].1)
}

fn action_connect(args: &Value) -> Result<Value, String> {
    let (command, argv) = command_args(args)?;
    let name = opt_text(args, "name").unwrap_or("mcp");
    let sid = session_id_of(args);
    let session = spawn_session(&command, &argv, name)?;
    let mut guard = SESSIONS
        .lock()
        .map_err(|_| "mcp client: session lock poisoned".to_string())?;
    // Replace existing
    guard.retain(|(id, _)| id != &sid);
    guard.push((sid.clone(), session));
    Ok(json!({
        "ok": true,
        "session": sid,
        "transport": "stdio",
        "command": command,
    }))
}

fn action_list(args: &Value) -> Result<Value, String> {
    with_session(args, |s| {
        let result = rpc_call(s, "tools/list", json!({}))?;
        let mut out = result;
        if let Some(obj) = out.as_object_mut() {
            obj.insert("authority".into(), json!("workbook"));
            obj.insert("session".into(), json!(session_id_of(args)));
        }
        Ok(out)
    })
}

fn action_call(args: &Value) -> Result<Value, String> {
    let name = arg_text(args, "name")?;
    let arguments = args.get("arguments").cloned().unwrap_or(json!({}));
    with_session(args, |s| {
        let result = rpc_call(
            s,
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )?;
        let mut out = if result.is_object() {
            result
        } else {
            json!({ "result": result })
        };
        if let Some(obj) = out.as_object_mut() {
            obj.insert("authority".into(), json!("workbook"));
            obj.insert("session".into(), json!(session_id_of(args)));
        }
        Ok(out)
    })
}

fn action_close(args: &Value) -> Result<Value, String> {
    let sid = session_id_of(args);
    let mut guard = SESSIONS
        .lock()
        .map_err(|_| "mcp client: session lock poisoned".to_string())?;
    if let Some(idx) = guard.iter().position(|(id, _)| id == &sid) {
        let (_, mut session) = guard.remove(idx);
        let _ = session.child.kill();
        let _ = session.child.wait();
        Ok(json!({ "ok": true, "session": sid, "closed": true }))
    } else {
        Ok(json!({ "ok": true, "session": sid, "closed": false }))
    }
}

/// One-shot: connect → list or call → close (no persistent session).
fn action_oneshot(args: &Value, kind: &str) -> Result<Value, String> {
    let (command, argv) = command_args(args)?;
    let name = opt_text(args, "name").unwrap_or("mcp");
    let mut session = spawn_session(&command, &argv, name)?;
    let result = match kind {
        "list" => {
            let r = rpc_call(&mut session, "tools/list", json!({}))?;
            Ok(r)
        }
        "call" => {
            let tname = arg_text(args, "tool").or_else(|_| arg_text(args, "name"))?;
            // When oneshot call, `name` may be server name — prefer `tool=`.
            let tname = opt_text(args, "tool").unwrap_or(tname);
            let arguments = args.get("arguments").cloned().unwrap_or(json!({}));
            rpc_call(
                &mut session,
                "tools/call",
                json!({ "name": tname, "arguments": arguments }),
            )
        }
        _ => Err(format!("unknown oneshot kind `{kind}`")),
    };
    let _ = session.child.kill();
    let _ = session.child.wait();
    let mut out = result?;
    if let Some(obj) = out.as_object_mut() {
        obj.insert("authority".into(), json!("workbook"));
        obj.insert("transport".into(), json!("stdio"));
    }
    Ok(out)
}

pub fn mcp_client(args: &Value) -> Result<Value, String> {
    let action = opt_text(args, "action").unwrap_or("list");
    match action {
        "connect" => action_connect(args),
        "list" => {
            if opt_text(args, "command").is_some() && opt_text(args, "session").is_none() {
                action_oneshot(args, "list")
            } else {
                action_list(args)
            }
        }
        "call" => {
            if opt_text(args, "command").is_some() && opt_text(args, "session").is_none() {
                action_oneshot(args, "call")
            } else {
                action_call(args)
            }
        }
        "close" | "disconnect" => action_close(args),
        other => Err(format!(
            "mcp client: unknown action `{other}` (connect|list|call|close)"
        )),
    }
}

pub unsafe extern "C" fn agent_mcp_client(
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
    match mcp_client(&args) {
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

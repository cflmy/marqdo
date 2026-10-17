//! Persistent foreign kernels (Python / Node) and the session state bridge.
//!
//! Protocol: one JSON request per stdin line, one JSON response per original
//! stdout line. User stdout/stderr are captured inside the bootstrap.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::host::foreign::{ensure_config, resolve_session_spec};
use crate::host::json::{json_to_value, value_to_json};
use crate::host::HostContext;
use crate::value::Value;

const SESSION_TIMEOUT_SECS: f64 = 10.0;
const MAX_SESSION_TIMEOUT_SECS: f64 = 3600.0;
const MAX_PROCESS_STDERR: usize = 1_048_576;

const PY_BOOTSTRAP: &str = r#"import sys, json, io, ast, traceback, contextlib
_out = sys.stdout
def _send(o):
    _out.write(json.dumps(o, ensure_ascii=False, default=str) + "\n")
    _out.flush()
_g = {"__name__": "__main__", "marqdo_input": None}
for _line in sys.stdin:
    _line = _line.strip()
    if not _line:
        continue
    try:
        req = json.loads(_line)
    except Exception:
        _send({"id": None, "ok": False, "stdout": "", "stderr": "", "result": None, "error": "bad request"})
        continue
    rid = req.get("id")
    op = req.get("op", "eval")
    if op == "set":
        _g[req.get("name")] = req.get("value")
        _send({"id": rid, "ok": True, "stdout": "", "stderr": "", "result": None, "error": ""})
        continue
    if op == "get":
        try:
            _val = _g[req.get("name")]
        except KeyError:
            _val = None
        _send({"id": rid, "ok": True, "stdout": "", "stderr": "", "result": _val, "error": ""})
        continue
    if op == "names":
        _names = sorted(k for k in _g.keys() if not k.startswith("__"))
        _send({"id": rid, "ok": True, "stdout": "", "stderr": "", "result": _names, "error": ""})
        continue
    _code = req.get("code", "")
    _g["marqdo_input"] = req.get("input")
    _out_buf = io.StringIO()
    _err_buf = io.StringIO()
    try:
        _tree = ast.parse(_code, "<marqdo>")
        _result = None
        with contextlib.redirect_stdout(_out_buf), contextlib.redirect_stderr(_err_buf):
            if _tree.body and isinstance(_tree.body[-1], ast.Expr):
                _last = _tree.body.pop()
                if _tree.body:
                    exec(compile(_tree, "<marqdo>", "exec"), _g)
                _result = eval(compile(ast.Expression(_last.value), "<marqdo>", "eval"), _g)
            else:
                exec(compile(_tree, "<marqdo>", "exec"), _g)
        _send({"id": rid, "ok": True, "stdout": _out_buf.getvalue(), "stderr": _err_buf.getvalue(), "result": _result, "error": ""})
    except BaseException:
        _send({"id": rid, "ok": False, "stdout": _out_buf.getvalue(), "stderr": _err_buf.getvalue(), "result": None, "error": traceback.format_exc(limit=8)})
"#;

const JS_BOOTSTRAP: &str = r#"const readline = require("readline");
const util = require("util");
const vm = require("vm");
const out = process.stdout.write.bind(process.stdout);
globalThis.require = require;
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
function send(o) { out(JSON.stringify(o) + "\n"); }
function toJSON(v) { try { return JSON.parse(JSON.stringify(v)); } catch (e) { return String(v); } }
async function capture(fn) {
  const logs = [], errs = [];
  const ow = process.stdout.write.bind(process.stdout);
  const ew = process.stderr.write.bind(process.stderr);
  const ol = console.log, oi = console.info, owr = console.warn, oe = console.error;
  console.log = console.info = (...a) => { logs.push(util.format(...a) + "\n"); };
  console.warn = console.error = (...a) => { errs.push(util.format(...a) + "\n"); };
  process.stdout.write = (c) => { logs.push(String(c)); return true; };
  process.stderr.write = (c) => { errs.push(String(c)); return true; };
  let result, error = "";
  try { result = await fn(); } catch (e) { error = (e && e.stack) || String(e); }
  process.stdout.write = ow; process.stderr.write = ew;
  console.log = ol; console.info = oi; console.warn = owr; console.error = oe;
  return { logs: logs.join(""), errs: errs.join(""), result: result === undefined ? null : result, error };
}
rl.on("line", async (line) => {
  let req;
  try { req = JSON.parse(line); } catch (e) { send({ id: null, ok: false, stdout: "", stderr: "", result: null, error: "bad request" }); return; }
  const rid = req.id, op = req.op || "eval";
  if (op === "set") { globalThis[req.name] = req.value; send({ id: rid, ok: true, stdout: "", stderr: "", result: null, error: "" }); return; }
  if (op === "get") {
    let result = null;
    try { result = vm.runInThisContext(String(req.name), { filename: "<marqdo-get>" }); }
    catch (e) { result = globalThis[req.name] === undefined ? null : globalThis[req.name]; }
    send({ id: rid, ok: true, stdout: "", stderr: "", result: toJSON(result), error: "" });
    return;
  }
  if (op === "names") {
    const names = Object.keys(globalThis).filter(k => !k.startsWith("__") && typeof globalThis[k] !== "function");
    send({ id: rid, ok: true, stdout: "", stderr: "", result: names, error: "" });
    return;
  }
  globalThis.marqdo_input = req.input === undefined ? null : req.input;
  const cap = await capture(async () => vm.runInThisContext(req.code || "", { filename: "<marqdo>" }));
  send({ id: rid, ok: !cap.error, stdout: cap.logs, stderr: cap.errs, result: toJSON(cap.result), error: cap.error });
});
"#;

pub struct Session {
    pub id: String,
    pub lang: String,
    pub cmd: String,
    pub started_ms: i64,
    pub timeout: Duration,
    pub evals: u64,
    pub alive: bool,
    child: Child,
    stdin: std::process::ChildStdin,
    rx: Receiver<String>,
    _reader: JoinHandle<()>,
    stderr: Arc<Mutex<String>>,
    _stderr_reader: JoinHandle<()>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("lang", &self.lang)
            .field("evals", &self.evals)
            .field("alive", &self.alive)
            .finish()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn as_text<'a>(v: &'a Value, what: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{what} must be text")),
    }
}

fn opt_text(v: Option<&Value>) -> Result<Option<String>, String> {
    match v {
        None | Some(Value::None) => Ok(None),
        Some(Value::Text(s)) => Ok(Some(s.clone())),
        Some(Value::Secret(s)) => Ok(Some(s.clone())),
        Some(_) => Err("value must be text".into()),
    }
}

fn parse_timeout(v: Option<&Value>, default: f64) -> Result<Duration, String> {
    let secs = match v {
        None | Some(Value::None) => default,
        Some(Value::Int(n)) => *n as f64,
        Some(Value::Num(n)) => *n,
        Some(_) => return Err("timeout must be a number of seconds".into()),
    };
    if !secs.is_finite() || secs <= 0.0 {
        return Err("timeout must be > 0 seconds".into());
    }
    Ok(Duration::from_secs_f64(secs.min(MAX_SESSION_TIMEOUT_SECS)))
}

fn parse_env(v: Option<&Value>) -> Result<Vec<(String, String)>, String> {
    match v {
        None | Some(Value::None) => Ok(Vec::new()),
        Some(Value::Map(entries)) => Ok(entries
            .iter()
            .map(|(k, v)| {
                let val = match v {
                    Value::Secret(s) => s.clone(),
                    other => other.as_display(),
                };
                (k.clone(), val)
            })
            .collect()),
        Some(_) => Err("env must be a map".into()),
    }
}

fn record(
    id: Option<&str>,
    lang: &str,
    cmd: &str,
    ok: bool,
    stdout: String,
    stderr: String,
    result: Value,
    error: String,
    ms: u64,
    timed_out: bool,
    alive: bool,
) -> Value {
    let mut fields = vec![
        ("ok".into(), Value::Bool(ok)),
        ("code".into(), Value::Int(if ok { 0 } else { -1 })),
        ("stdout".into(), Value::Text(stdout)),
        ("stderr".into(), Value::Text(stderr)),
        ("ms".into(), Value::Int(ms as i64)),
        ("lang".into(), Value::Text(lang.to_string())),
        ("cmd".into(), Value::Text(cmd.to_string())),
        ("timed_out".into(), Value::Bool(timed_out)),
        ("truncated".into(), Value::Bool(false)),
        ("result".into(), result),
        ("result_error".into(), Value::Text(String::new())),
        ("error".into(), Value::Text(error)),
        ("script".into(), Value::Text(String::new())),
        ("cached".into(), Value::Bool(false)),
        ("alive".into(), Value::Bool(alive)),
    ];
    if let Some(id) = id {
        fields.push(("id".into(), Value::Text(id.to_string())));
    }
    Value::Map(fields)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn open(
    ctx: &mut HostContext,
    lang: &Value,
    cmd: Option<&Value>,
    env: Option<&Value>,
    cwd: Option<&Value>,
    timeout: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    if !ctx.allow_exec() {
        return Err("foreign session denied by host policy".into());
    }
    let requested = as_text(lang, "lang")?.to_ascii_lowercase();
    let canonical = crate::foreign::registry::canonical(&requested);
    if !crate::foreign::registry::session_supported(&canonical) {
        return Err(format!(
            "lang=`{canonical}` has no session bootstrap; use run/invoke"
        ));
    }
    let spec = resolve_session_spec(ctx, &canonical)?;
    let mut argv = match opt_text(cmd)? {
        Some(c) => {
            let argv = crate::foreign::split_cmd_line(&c);
            if argv.is_empty() {
                return Err("session cmd override is empty".into());
            }
            argv
        }
        None => spec.argv.clone(),
    };
    if argv.is_empty() {
        return Err(format!("no interpreter configured for lang=`{canonical}`"));
    }
    match canonical.as_str() {
        "python" => {
            argv.push("-u".into());
            argv.push("-c".into());
            argv.push(PY_BOOTSTRAP.into());
        }
        "node" => {
            argv.push("-e".into());
            argv.push(JS_BOOTSTRAP.into());
        }
        _ => return Err(format!("lang=`{canonical}` has no session bootstrap")),
    }
    let cwd_path = match opt_text(cwd)? {
        Some(c) => ctx.resolve_path(&c)?,
        None => ctx.cwd.clone(),
    };
    if !cwd_path.is_dir() {
        return Err(format!(
            "session cwd is not a directory: {}",
            cwd_path.display()
        ));
    }
    let mut command = Command::new(&argv[0]);
    for a in &argv[1..] {
        command.arg(a);
    }
    command.current_dir(&cwd_path);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.env("MARQDO_FOREIGN", "1");
    command.env("MARQDO_FOREIGN_LANG", &canonical);
    command.env("MARQDO_FOREIGN_CWD", &cwd_path);
    for (k, v) in &spec.env {
        command.env(k, v);
    }
    for (k, v) in parse_env(env)? {
        command.env(k, v);
    }
    command.env("PYTHONUNBUFFERED", "1");
    command.env("PYTHONIOENCODING", "utf-8");
    command.env("PYTHONUTF8", "1");
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().map_err(|e| {
        format!(
            "foreign session spawn failed (lang={canonical}, cmd={}): {e}",
            argv[0]
        )
    })?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "session stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "session stdout unavailable".to_string())?;
    let stderr_pipe = child.stderr.take();
    let (tx, rx) = mpsc::channel::<String>();
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    let _ = tx.send("__EOF__".into());
                    break;
                }
                Ok(_) => {
                    let _ = tx.send(line.trim_end_matches(['\r', '\n']).to_string());
                }
                Err(_) => {
                    let _ = tx.send("__EOF__".into());
                    break;
                }
            }
        }
    });
    let stderr_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf_t = Arc::clone(&stderr_buf);
    let stderr_reader = thread::spawn(move || {
        if let Some(mut r) = stderr_pipe {
            let mut kept = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                match r.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if kept.len() < MAX_PROCESS_STDERR {
                            let take = (MAX_PROCESS_STDERR - kept.len()).min(n);
                            kept.extend_from_slice(&buf[..take]);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
            if let Ok(mut s) = stderr_buf_t.lock() {
                s.push_str(&String::from_utf8_lossy(&kept));
            }
        }
    });
    let seq = ctx.foreign_session_seq + 1;
    ctx.foreign_session_seq = seq;
    let id = format!("s{seq}");
    let started_ms = now_ms();
    let session_timeout = parse_timeout(timeout, SESSION_TIMEOUT_SECS)?;
    let cmd_display = argv[0].clone();
    let session = Session {
        id: id.clone(),
        lang: canonical.clone(),
        cmd: cmd_display.clone(),
        started_ms,
        timeout: session_timeout,
        evals: 0,
        alive: true,
        child,
        stdin,
        rx,
        _reader: reader,
        stderr: stderr_buf,
        _stderr_reader: stderr_reader,
    };
    ctx.foreign_sessions.insert(id.clone(), session);
    let _ = timeout;
    Ok(Value::Text(id))
}

fn session_dead_stderr(ctx: &HostContext, id: &str) -> String {
    ctx.foreign_sessions
        .get(id)
        .and_then(|s| s.stderr.lock().ok().map(|v| v.clone()))
        .unwrap_or_default()
}

fn remove_session(ctx: &mut HostContext, id: &str) {
    ctx.foreign_sessions.remove(id);
}

pub fn eval(
    ctx: &mut HostContext,
    id: &Value,
    code: Option<&Value>,
    source: Option<&Value>,
    input: Option<&Value>,
    timeout: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    let id = as_text(id, "id")?.to_string();
    if !ctx.foreign_sessions.contains_key(&id) {
        return Err(format!("foreign session `{id}` not found"));
    }
    let session_lang = ctx.foreign_sessions[&id].lang.clone();
    let code = match (present(code), present(source)) {
        (Some(_), Some(_)) => {
            return Err("session_eval: pass either code or source, not both".into())
        }
        (Some(c), None) => match c {
            Value::Code(b) => {
                if crate::foreign::registry::canonical(&b.lang) != session_lang {
                    return Err(format!(
                        "session_eval code language `{}` does not match session language `{}`",
                        b.lang, session_lang
                    ));
                }
                b.source.clone()
            }
            _ => return Err("session_eval code must be a code value".into()),
        },
        (None, Some(s)) => as_text(s, "source")?.to_string(),
        (None, None) => return Err("session_eval needs code or source".into()),
    };
    let input_json = match input {
        None | Some(Value::None) => serde_json::Value::Null,
        Some(v) => value_to_json(v)?,
    };
    let default_timeout = ctx.foreign_sessions[&id].timeout.as_secs_f64();
    let dur = parse_timeout(timeout, default_timeout)?;
    let request = serde_json::json!({
        "id": 1,
        "op": "eval",
        "code": code,
        "input": input_json,
    });
    let start = Instant::now();
    let (lang, cmd) = {
        let s = &ctx.foreign_sessions[&id];
        (s.lang.clone(), s.cmd.clone())
    };
    let reply = match send_and_wait(ctx, &id, &request, dur) {
        Ok(r) => r,
        Err(e) => {
            let timed_out = e.contains("timed out");
            let stderr = session_dead_stderr(ctx, &id);
            remove_session(ctx, &id);
            return Ok(record(
                Some(&id),
                &lang,
                &cmd,
                false,
                String::new(),
                stderr,
                Value::None,
                e,
                start.elapsed().as_millis() as u64,
                timed_out,
                false,
            ));
        }
    };
    let ms = start.elapsed().as_millis() as u64;
    let alive = ctx
        .foreign_sessions
        .get_mut(&id)
        .map(|s| {
            s.evals += 1;
            s.alive
        })
        .unwrap_or(false);
    let ok = reply
        .get("ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let stdout = reply
        .get("stdout")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let stderr = reply
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let error = reply
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let result = reply
        .get("result")
        .map(json_to_value)
        .transpose()?
        .unwrap_or(Value::None);
    Ok(record(
        Some(&id),
        &lang,
        &cmd,
        ok,
        stdout,
        stderr,
        result,
        error,
        ms,
        false,
        alive,
    ))
}

fn present(v: Option<&Value>) -> Option<&Value> {
    match v {
        None | Some(Value::None) => None,
        Some(x) => Some(x),
    }
}

fn send_and_wait(
    ctx: &mut HostContext,
    id: &str,
    request: &serde_json::Value,
    timeout: Duration,
) -> Result<serde_json::Value, String> {
    let line = serde_json::to_string(request).map_err(|e| format!("session request: {e}"))?;
    {
        let session = ctx
            .foreign_sessions
            .get_mut(id)
            .ok_or_else(|| format!("foreign session `{id}` not found"))?;
        if let Ok(Some(_)) = session.child.try_wait() {
            return Err("session process exited".into());
        }
        session
            .stdin
            .write_all(line.as_bytes())
            .and_then(|_| session.stdin.write_all(b"\n"))
            .and_then(|_| session.stdin.flush())
            .map_err(|e| format!("session write: {e}"))?;
    }
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            kill_session(ctx, id);
            return Err(format!(
                "session timed out after {:.3}s",
                timeout.as_secs_f64()
            ));
        }
        let session = ctx
            .foreign_sessions
            .get_mut(id)
            .ok_or_else(|| format!("foreign session `{id}` not found"))?;
        match session.rx.recv_timeout(remaining.min(Duration::from_millis(200))) {
            Ok(line) => {
                if line == "__EOF__" {
                    return Err("session process exited".into());
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                    let rid = v.get("id").and_then(|x| x.as_u64()).unwrap_or(0);
                    if rid != 1 {
                        continue;
                    }
                    return Ok(v);
                }
                // Unexpected non-protocol output: keep waiting until the deadline.
                continue;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(Some(_)) = session.child.try_wait() {
                    return Err("session process exited".into());
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("session reader disconnected".into())
            }
        }
    }
}

fn kill_session(ctx: &mut HostContext, id: &str) {
    if let Some(mut s) = ctx.foreign_sessions.remove(id) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(s.child.id() as i32), libc::SIGKILL);
        }
        let _ = s.child.kill();
        let _ = s.child.wait();
        s.alive = false;
    }
}

pub fn set(
    ctx: &mut HostContext,
    id: &Value,
    name: &Value,
    value: &Value,
) -> Result<Value, String> {
    let id = as_text(id, "id")?.to_string();
    let name = as_text(name, "name")?.to_string();
    if !valid_name(&name) {
        return Err("session_set: name must be an identifier".into());
    }
    let json = value_to_json(value)?;
    let request = serde_json::json!({"id": 1, "op": "set", "name": name, "value": json});
    let reply = send_and_wait(ctx, &id, &request, Duration::from_secs(10))?;
    if reply.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        Ok(Value::None)
    } else {
        Err(reply
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("session_set failed")
            .to_string())
    }
}

pub fn get(ctx: &mut HostContext, id: &Value, name: &Value) -> Result<Value, String> {
    let id = as_text(id, "id")?.to_string();
    let name = as_text(name, "name")?.to_string();
    if !valid_name(&name) {
        return Err("session_get: name must be an identifier".into());
    }
    let request = serde_json::json!({"id": 1, "op": "get", "name": name});
    let reply = send_and_wait(ctx, &id, &request, Duration::from_secs(10))?;
    if !reply.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err(reply
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("session_get failed")
            .to_string());
    }
    match reply.get("result") {
        Some(v) => json_to_value(v),
        None => Ok(Value::None),
    }
}

pub fn names(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = as_text(id, "id")?.to_string();
    let request = serde_json::json!({"id": 1, "op": "names"});
    let reply = send_and_wait(ctx, &id, &request, Duration::from_secs(10))?;
    if !reply.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err(reply
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("session_names failed")
            .to_string());
    }
    match reply.get("result") {
        Some(v) => json_to_value(v),
        None => Ok(Value::List(Vec::new())),
    }
}

pub fn close(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = as_text(id, "id")?.to_string();
    let existed = ctx.foreign_sessions.contains_key(&id);
    kill_session(ctx, &id);
    Ok(Value::Bool(existed))
}

pub fn list(ctx: &HostContext) -> Result<Value, String> {
    let mut sessions: Vec<&Session> = ctx.foreign_sessions.values().collect();
    sessions.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = Vec::new();
    for s in sessions {
        out.push(Value::Map(vec![
            ("id".into(), Value::Text(s.id.clone())),
            ("lang".into(), Value::Text(s.lang.clone())),
            ("cmd".into(), Value::Text(s.cmd.clone())),
            ("evals".into(), Value::Int(s.evals as i64)),
            ("alive".into(), Value::Bool(s.alive)),
            ("started_ms".into(), Value::Int(s.started_ms)),
        ]));
    }
    Ok(Value::List(out))
}

pub fn shutdown(ctx: &mut HostContext) {
    let ids: Vec<String> = ctx.foreign_sessions.keys().cloned().collect();
    for id in ids {
        kill_session(ctx, &id);
    }
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers() {
        assert!(valid_name("x"));
        assert!(valid_name("_x1"));
        assert!(!valid_name("1x"));
        assert!(!valid_name("a-b"));
    }
}

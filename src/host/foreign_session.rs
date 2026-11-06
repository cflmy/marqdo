//! Persistent foreign kernels (Python / Node) and the session state bridge.
//!
//! Protocol: one JSON request per stdin line, one JSON response per original
//! stdout line. User stdout/stderr are captured inside the bootstrap.

use crate::host::as_text;
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

mod control;
pub mod validation;
pub use control::InterruptHandle;

const SESSION_TIMEOUT_SECS: f64 = 10.0;
const MAX_SESSION_TIMEOUT_SECS: f64 = 3600.0;
const MAX_PROCESS_STDERR: usize = 1_048_576;

const PY_BOOTSTRAP: &str = r#"import sys, json, io, ast, traceback, contextlib, math, signal
_out = sys.stdout
_MAX_SAFE = 9007199254740991
def _sanitize(o):
    if o is None or isinstance(o, bool):
        return o
    if isinstance(o, int):
        if abs(o) > _MAX_SAFE:
            raise ValueError("json: integer outside V_safe (±(2^53-1)): %s" % o)
        return o
    if isinstance(o, float):
        if not math.isfinite(o):
            raise ValueError("json: non-finite number outside V_safe")
        if o.is_integer() and abs(o) > _MAX_SAFE:
            raise ValueError("json: integer outside V_safe (±(2^53-1)): %s" % o)
        return o
    if isinstance(o, str):
        return o
    if isinstance(o, (list, tuple)):
        return [_sanitize(x) for x in o]
    if isinstance(o, dict):
        if any(not isinstance(k, str) for k in o):
            raise TypeError("not JSON-safe: object keys must be strings")
        return {k: _sanitize(v) for k, v in o.items()}
    raise TypeError("not JSON-safe: %s" % type(o).__name__)
def _send(o):
    try:
        line = json.dumps(_sanitize(o), ensure_ascii=False)
    except Exception as e:
        rid = o.get("id") if isinstance(o, dict) else None
        line = json.dumps({
            "id": rid, "ok": False,
            "stdout": (o.get("stdout") if isinstance(o, dict) else "") or "",
            "stderr": (o.get("stderr") if isinstance(o, dict) else "") or "",
            "result": None, "error": str(e), "domain_error": True,
        }, ensure_ascii=False)
    _out.write(line + "\n")
    _out.flush()
_busy = False
def _interrupt(signum, frame):
    global _busy
    if _busy:
        _busy = False
        raise KeyboardInterrupt
signal.signal(signal.SIGINT, _interrupt)
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
    _old_stdin = sys.stdin
    if req.get("stdin") is not None:
        sys.stdin = io.StringIO(str(req.get("stdin")))
    _out_buf = io.StringIO()
    _err_buf = io.StringIO()
    try:
        _busy = True
        _send({"id": rid, "phase": "ready"})
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
        _busy = False
        _send({"id": rid, "ok": True, "stdout": _out_buf.getvalue(), "stderr": _err_buf.getvalue(), "result": _result, "error": ""})
    except BaseException:
        _busy = False
        _send({"id": rid, "ok": False, "stdout": _out_buf.getvalue(), "stderr": _err_buf.getvalue(), "result": None, "error": traceback.format_exc(limit=8)})
    finally:
        _busy = False
        sys.stdin = _old_stdin
"#;

const JS_BOOTSTRAP: &str = r#"const readline = require("readline");
const util = require("util");
const vm = require("vm");
const out = process.stdout.write.bind(process.stdout);
process.on("SIGINT", () => {});
const MAX_SAFE = Number.MAX_SAFE_INTEGER;
globalThis.require = require;
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
function sanitize(v, active = new Set()) {
  if (v === null || typeof v === "boolean" || typeof v === "string") return v;
  if (typeof v === "number") {
    if (!Number.isFinite(v)) throw new Error("json: non-finite number outside V_safe");
    if (Number.isInteger(v) && Math.abs(v) > MAX_SAFE)
      throw new Error("json: integer outside V_safe (±(2^53-1)): " + v);
    return v;
  }
  if (typeof v === "bigint") {
    if (v > BigInt(MAX_SAFE) || v < -BigInt(MAX_SAFE))
      throw new Error("json: integer outside V_safe (±(2^53-1)): " + v.toString());
    return Number(v);
  }
  if (v && typeof v === "object") {
    if (active.has(v)) throw new Error("not JSON-safe: cyclic value");
    active.add(v);
    try {
      const keys = Reflect.ownKeys(v);
      if (Array.isArray(v)) {
        if (keys.length !== v.length + 1)
          throw new Error("not JSON-safe: sparse array or extra array properties");
        const rows = [];
        for (let i = 0; i < v.length; i++) {
          const d = Object.getOwnPropertyDescriptor(v, String(i));
          if (!d || !d.enumerable || !("value" in d))
            throw new Error("not JSON-safe: array accessor or missing element");
          rows.push(sanitize(d.value, active));
        }
        return rows;
      }
      const prototype = Object.getPrototypeOf(v);
      if (prototype !== Object.prototype && prototype !== null)
        throw new Error("not JSON-safe: expected plain record");
      const o = Object.create(null);
      for (const k of keys) {
        if (typeof k !== "string") throw new Error("not JSON-safe: symbol record key");
        const d = Object.getOwnPropertyDescriptor(v, k);
        if (!d || !d.enumerable || !("value" in d))
          throw new Error("not JSON-safe: record accessor or hidden property");
        o[k] = sanitize(d.value, active);
      }
      return o;
    } finally { active.delete(v); }
  }
  throw new Error("not JSON-safe: " + (typeof v));
}
function send(o) {
  try { out(JSON.stringify(sanitize(o)) + "\n"); }
  catch (e) {
    out(JSON.stringify({
      id: o && o.id !== undefined ? o.id : null, ok: false,
      stdout: (o && o.stdout) || "", stderr: (o && o.stderr) || "",
      result: null, error: String(e && e.message ? e.message : e), domain_error: true
    }) + "\n");
  }
}
function toJSON(v) { return sanitize(v); }
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
    send({ id: rid, ok: true, stdout: "", stderr: "", result: result, error: "" });
    return;
  }
  if (op === "names") {
    const skip = new Set(["global","globalThis","process","Buffer","console","require","module","exports","__dirname","__filename","performance","crypto","navigator","fetch","URL","URLSearchParams","TextEncoder","TextDecoder","setTimeout","clearTimeout","setInterval","clearInterval","queueMicrotask","structuredClone","atob","btoa","marqdo_input"]);
    const names = Object.keys(globalThis).filter(k => !k.startsWith("__") && !skip.has(k) && typeof globalThis[k] !== "function");
    send({ id: rid, ok: true, stdout: "", stderr: "", result: names, error: "" });
    return;
  }
  globalThis.marqdo_input = req.input === undefined ? null : req.input;
  const oldStdin = process.stdin;
  let stdinPatched = false;
  if (req.stdin !== undefined && req.stdin !== null) {
    try {
      const { PassThrough } = require("stream");
      const ps = new PassThrough();
      ps.end(String(req.stdin));
      Object.defineProperty(process, "stdin", { value: ps, configurable: true, writable: true });
      stdinPatched = true;
    } catch (e) {}
  }
  send({ id: rid, phase: "ready" });
  const cap = await capture(async () => vm.runInThisContext(req.code || "", { filename: "<marqdo>", breakOnSigint: true }));
  if (stdinPatched) {
    try { Object.defineProperty(process, "stdin", { value: oldStdin, configurable: true, writable: true }); } catch (e) {}
  }
  send({ id: rid, ok: !cap.error, stdout: cap.logs, stderr: cap.errs, result: cap.result, error: cap.error });
});
"#;

/// Internal protocol evidence, distinct from admitted language observations.
/// Includes failed requests and post-execution domain refusals, so set/input
/// dependencies cannot disappear from the recorded history.
#[derive(Debug, Clone)]
pub struct ProtocolStep {
    pub session: String,
    pub request: serde_json::Value,
    pub response: Result<serde_json::Value, String>,
}

pub struct Session {
    pub interrupt_handle: InterruptHandle,
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
        if let Some(_pid) = self.interrupt_handle.retire() {
            #[cfg(unix)]
            unsafe { libc::kill(-(_pid as i32), libc::SIGKILL); }
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
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

/// Session ids are opaque handles — text fabrications are refused.
fn as_session_id<'a>(id: &'a Value) -> Result<&'a str, String> {
    match id {
        Value::Handle(s) => Ok(s.as_str()),
        Value::Text(_) => Err("session id must be a handle (not text)".into()),
        _ => Err("session id must be a handle".into()),
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
    let session_timeout = parse_timeout(timeout, SESSION_TIMEOUT_SECS)?;
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
    let cmd_display = argv[0].clone();
    let session = Session {
        interrupt_handle: InterruptHandle::new(child.id()),
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
    Ok(Value::Handle(id))
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
    eval_with_stdin(ctx, id, code, source, input, None, timeout)
}

pub fn eval_with_stdin(
    ctx: &mut HostContext,
    id: &Value,
    code: Option<&Value>,
    source: Option<&Value>,
    input: Option<&Value>,
    stdin: Option<&str>,
    timeout: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    let id = as_session_id(id)?.to_string();
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
        Some(v) => { super::oracle::admit(v)?; value_to_json(v)? },
    };
    let default_timeout = ctx.foreign_sessions[&id].timeout.as_secs_f64();
    let dur = parse_timeout(timeout, default_timeout)?;
    let request = serde_json::json!({
        "id": 1,
        "op": "eval",
        "code": code,
        "input": input_json,
        "stdin": stdin,
    });
    let start = Instant::now();
    let (lang, cmd) = {
        let s = &ctx.foreign_sessions[&id];
        (s.lang.clone(), s.cmd.clone())
    };
    let reply = match send_and_wait(ctx, &id, &request, dur) {
        Ok(r) => r,
        Err(e) => {
            // Replay divergence is a language-level stop, not process death or
            // an ordinary failed-eval value. Keep the actual committed kernel.
            if ctx.foreign_replay_divergence.is_some() { return Err(e); }
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
    // Ω ∉ V_safe is a language fault, not a soft cell failure.
    if reply.get("domain_error").and_then(|v| v.as_bool()) == Some(true) {
        return Err(error);
    }
    let result = reply
        .get("result")
        .map(json_to_value)
        .transpose()?
        .unwrap_or(Value::None);
    super::oracle::admit(&result)?;
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
    if ctx.foreign_replay_expected.is_some() {
        if let Some(error) = &ctx.foreign_replay_divergence { return Err(error.clone()); }
    }
    let permit = validation::prepare(ctx.foreign_replay_expected.as_mut(), id, request)
        .map_err(|failure| {
            let error = failure.to_string();
            ctx.foreign_replay_divergence = Some(error.clone());
            error
        })?;
    let control = ctx.foreign_sessions.get(id).map(|s| s.interrupt_handle.clone());
    if let Some(handle) = &control { handle.begin(request.get("op").and_then(|v| v.as_str()) == Some("eval")); }
    let response = send_and_wait_inner(ctx, permit, timeout);
    if let Some(handle) = control { handle.finish(); }
    ctx.foreign_protocol_steps.push(ProtocolStep {
        session: id.to_string(), request: request.clone(), response: response.clone(),
    });
    if let Some(steps) = &mut ctx.foreign_replay_expected {
        if let Err(failure) = steps.accept_response(&response) {
            let error = failure.to_string();
            ctx.foreign_replay_divergence = Some(error.clone());
            return Err(error);
        }
    }
    response
}

fn send_control(
    ctx: &mut HostContext, id: &str, request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let response = send_and_wait(ctx, id, request, Duration::from_secs(10));
    if response.is_err() && ctx.foreign_replay_divergence.is_none() { remove_session(ctx, id); }
    response
}

fn send_and_wait_inner(
    ctx: &mut HostContext,
    permit: validation::DispatchPermit,
    timeout: Duration,
) -> Result<serde_json::Value, String> {
    let id = permit.session();
    let request = permit.request();
    let line = serde_json::to_string(request).map_err(|e| format!("session request: {e}"))?;
    {
        let session = ctx
            .foreign_sessions
            .get_mut(id)
            .ok_or_else(|| format!("foreign session `{id}` not found"))?;
        if let Ok(Some(_)) = session.interrupt_handle.try_wait(&mut session.child) {
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
                    if v.get("phase").and_then(|v| v.as_str()) == Some("ready") {
                        session.interrupt_handle.ready()?;
                        continue;
                    }
                    return Ok(v);
                }
                // Unexpected non-protocol output: keep waiting until the deadline.
                continue;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                session.interrupt_handle.retry()?;
                if let Ok(Some(_)) = session.interrupt_handle.try_wait(&mut session.child) {
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
        if let Some(_pid) = s.interrupt_handle.retire() {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(_pid as i32), libc::SIGKILL);
            }
            let _ = s.child.kill();
        }
        let _ = s.child.wait();
        s.alive = false;
    }
}

/// Interrupt the current evaluation without destroying the kernel. Python
/// reports KeyboardInterrupt through the normal result frame.
pub fn interrupt(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = as_session_id(id)?.to_string();
    let session = ctx
        .foreign_sessions
        .get_mut(&id)
        .ok_or_else(|| format!("foreign session `{id}` not found"))?;
    session.interrupt_handle.request()?;
    Ok(Value::Bool(true))
}

pub fn set(
    ctx: &mut HostContext,
    id: &Value,
    name: &Value,
    value: &Value,
) -> Result<Value, String> {
    let id = as_session_id(id)?.to_string();
    if !ctx.foreign_sessions.contains_key(&id) {
        return Err(format!("foreign session `{id}` not found"));
    }
    let name = as_text(name, "name")?.to_string();
    if !valid_name(&name) {
        return Err("session_set: name must be an identifier".into());
    }
    super::oracle::admit(value)?;
    let json = value_to_json(value)?;
    let request = serde_json::json!({"id": 1, "op": "set", "name": name, "value": json});
    let reply = send_control(ctx, &id, &request)?;
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
    let id = as_session_id(id)?.to_string();
    if !ctx.foreign_sessions.contains_key(&id) {
        return Err(format!("foreign session `{id}` not found"));
    }
    let name = as_text(name, "name")?.to_string();
    if !valid_name(&name) {
        return Err("session_get: name must be an identifier".into());
    }
    let request = serde_json::json!({"id": 1, "op": "get", "name": name});
    let reply = send_control(ctx, &id, &request)?;
    if !reply.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err(reply
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("session_get failed")
            .to_string());
    }
    let value = match reply.get("result") {
        Some(v) => json_to_value(v),
        None => Ok(Value::None),
    }?;
    super::oracle::admit(&value)?;
    Ok(value)
}

pub fn names(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = as_session_id(id)?.to_string();
    if !ctx.foreign_sessions.contains_key(&id) {
        return Err(format!("foreign session `{id}` not found"));
    }
    let request = serde_json::json!({"id": 1, "op": "names"});
    let reply = send_control(ctx, &id, &request)?;
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
    let id = as_session_id(id)?.to_string();
    // Affine close: consuming an absent / already-closed handle is a fault
    // (same surface as eval/set/get after close).
    if !ctx.foreign_sessions.contains_key(&id) {
        return Err(format!("foreign session `{id}` not found"));
    }
    kill_session(ctx, &id);
    Ok(Value::Bool(true))
}

pub fn list(ctx: &HostContext) -> Result<Value, String> {
    let mut sessions: Vec<&Session> = ctx.foreign_sessions.values().collect();
    sessions.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = Vec::new();
    for s in sessions {
        out.push(Value::Map(vec![
            ("id".into(), Value::Handle(s.id.clone())),
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

pub(crate) fn valid_name(name: &str) -> bool {
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
    #[test]
    fn bridge_live_boundary_and_refusal_preserves_input_store() {
        let edge = 9_007_199_254_740_991i64;
        for lang in ["python", "node"] {
            let mut ctx = HostContext::default();
            let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
            let name = Value::Text("payload".into());
            for n in [i64::MIN, -edge-1, -edge, -edge+1, -1, 0, 1, edge-1, edge, edge+1, i64::MAX] {
                set(&mut ctx, &id, &name, &Value::Int(17)).unwrap();
                let input = Value::List(vec![Value::Text("中文🧪".into()), Value::List(vec![Value::Int(n)])]);
                let result = set(&mut ctx, &id, &name, &input);
                if (-edge..=edge).contains(&n) {
                    result.unwrap();
                    assert_eq!(get(&mut ctx, &id, &name).unwrap(), input, "{lang} {n}");
                } else {
                    assert!(result.unwrap_err().contains("outside V_safe"));
                    assert_eq!(get(&mut ctx, &id, &name).unwrap(), Value::Int(17));
                }
                println!("E4 {lang} integer {n}: verified");
            }
            for source in if lang == "python" {
                ["2**53", "float('nan')", "float('inf')"]
            } else { ["2**53", "NaN", "Infinity"] } {
                let error = eval(&mut ctx, &id, None, Some(&Value::Text(source.into())), None, None).unwrap_err();
                assert!(error.contains("outside V_safe"), "{lang} {source}: {error}");
                // A refused response must remain a protocol response, not kill Node.
                assert_eq!(get(&mut ctx, &id, &name).unwrap(), Value::Int(17));
                println!("E4 {lang} response {source}: refused, session responsive");
            }
            let alias = id.clone();
            close(&mut ctx, &id).unwrap();
            // Missing handle wins over an invalid name and an OOD payload.
            let err = set(&mut ctx, &alias, &Value::Text("bad-name".into()), &Value::Int(i64::MAX)).unwrap_err();
            assert!(err.contains("not found"), "{err}");
            assert!(get(&mut ctx, &alias, &name).is_err());
            assert!(close(&mut ctx, &alias).is_err());
            assert!(ctx.foreign_sessions.is_empty());
        }
    }

    #[test]
    fn generated_two_session_model_traces() {
        for lang in ["python", "node"] {
            let mut ctx = HostContext::default();
            let mut ids = Vec::new();
            let mut live = [true; 2];
            let mut expected = [0i64; 2];
            let name = Value::Text("counter".into());
            for _ in 0..2 {
                let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
                set(&mut ctx, &id, &name, &Value::Int(0)).unwrap();
                ids.push(id);
            }
            let mut seed = 0x52455155495245u64;
            for step in 0..256 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let slot = ((seed >> 32) & 1) as usize;
                let op = (seed >> 40) % 5;
                match op {
                    0 => {
                        let value = step as i64 - 128;
                        let result = set(&mut ctx, &ids[slot], &name, &Value::Int(value));
                        assert_eq!(result.is_ok(), live[slot]);
                        if live[slot] { expected[slot] = value; }
                    }
                    1 => {
                        let result = get(&mut ctx, &ids[slot], &name);
                        if live[slot] { assert_eq!(result.unwrap(), Value::Int(expected[slot])); }
                        else { assert!(result.unwrap_err().contains("not found")); }
                    }
                    2 => {
                        let src = if lang == "python" { "counter += 1\ncounter" } else { "counter += 1; counter" };
                        let result = eval(&mut ctx, &ids[slot], None, Some(&Value::Text(src.into())), None, None);
                        assert_eq!(result.is_ok(), live[slot]);
                        if live[slot] { expected[slot] += 1; }
                    }
                    3 => {
                        assert_eq!(close(&mut ctx, &ids[slot]).is_ok(), live[slot]);
                        live[slot] = false;
                    }
                    _ => {
                        if live[slot] { close(&mut ctx, &ids[slot]).unwrap(); }
                        let stale = ids[slot].clone();
                        ids[slot] = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
                        assert_ne!(stale, ids[slot]);
                        assert!(get(&mut ctx, &stale, &name).is_err());
                        set(&mut ctx, &ids[slot], &name, &Value::Int(0)).unwrap();
                        expected[slot] = 0;
                        live[slot] = true;
                    }
                }
                // Independent model predicts every remaining store after every command.
                for j in 0..2 {
                    assert_eq!(ctx.foreign_sessions.contains_key(as_session_id(&ids[j]).unwrap()), live[j]);
                    if live[j] { assert_eq!(get(&mut ctx, &ids[j], &name).unwrap(), Value::Int(expected[j]), "{lang} step {step} slot {j}"); }
                }
            }
            shutdown(&mut ctx);
            assert!(ctx.foreign_sessions.is_empty());
            println!("E3 {lang}: seed 0x52455155495245, 256 commands verified");
        }
    }

    #[test]
    fn failure_process_exit_timeout_and_reopen() {
        for lang in ["python", "node"] {
            let mut ctx = HostContext::default();
            for source in if lang == "python" { ["import os; os._exit(7)", "while True: pass"] }
                else { ["process.exit(7)", "while(true) {}"] } {
                let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
                let result = eval(&mut ctx, &id, None, Some(&Value::Text(source.into())), None, Some(&Value::Num(0.1)));
                assert!(result.is_ok(), "process failure should be a failure record: {result:?}");
                let Value::Map(m) = result.unwrap() else { panic!() };
                assert!(m.iter().any(|(k,v)| k == "ok" && v == &Value::Bool(false)));
                assert!(ctx.foreign_sessions.is_empty(), "dead session must be retired");
                assert!(get(&mut ctx, &id, &Value::Text("x".into())).is_err());
                println!("E5 {lang} {source}: recorded and retired");
            }
            let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
            set(&mut ctx, &id, &Value::Text("x".into()), &Value::Int(41)).unwrap();
            assert_eq!(get(&mut ctx, &id, &Value::Text("x".into())).unwrap(), Value::Int(41));
            shutdown(&mut ctx);
        }
    }

    #[test]
    fn input_history_and_post_refusal_effects_are_preserved() {
        for lang in ["python", "node"] {
            let mut ctx = HostContext::default();
            let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
            let name = Value::Text("n".into());
            set(&mut ctx, &id, &name, &Value::Int(1)).unwrap();
            let code = if lang == "python" { "n += marqdo_input\nn" } else { "n += marqdo_input; n" };
            let result = eval(&mut ctx, &id, None, Some(&Value::Text(code.into())), Some(&Value::Int(3)), None).unwrap();
            let Value::Map(fields) = result else { panic!() };
            assert_eq!(fields.iter().find(|(k,_)| k == "result").unwrap().1, Value::Int(4));
            let sent = ctx.foreign_protocol_steps.last().unwrap();
            assert_eq!(sent.session, as_session_id(&id).unwrap());
            assert_eq!(sent.request["op"], "eval");
            assert_eq!(sent.request["input"], 3);
            assert_eq!(sent.request["code"], code);
            let before = ctx.foreign_protocol_steps.len();
            assert!(set(&mut ctx, &id, &name, &Value::Int(i64::MAX)).is_err());
            assert_eq!(ctx.foreign_protocol_steps.len(), before, "invalid input must not query oracle");
            assert_eq!(get(&mut ctx, &id, &name).unwrap(), Value::Int(4));
            let bad = if lang == "python" { "n = 99\n2**53" } else { "n = 99; 2**53" };
            assert!(eval(&mut ctx, &id, None, Some(&Value::Text(bad.into())), None, None).unwrap_err().contains("outside V_safe"));
            let fault = ctx.foreign_protocol_steps.last().unwrap();
            assert_eq!(fault.response.as_ref().unwrap()["domain_error"], true);
            assert_eq!(get(&mut ctx, &id, &name).unwrap(), Value::Int(99), "foreign effects must not be claimed to roll back");
            let ordinary_error = if lang == "python" { "raise ValueError('outside V_safe')" }
                else { "throw new Error('outside V_safe')" };
            let record = eval(&mut ctx, &id, None, Some(&Value::Text(ordinary_error.into())), None, None).unwrap();
            let Value::Map(fields) = record else { panic!() };
            assert_eq!(fields.iter().find(|(k,_)| k == "ok").unwrap().1, Value::Bool(false));
            let unsupported = if lang == "python" { "object()" } else { "(function(){})" };
            assert!(eval(&mut ctx, &id, None, Some(&Value::Text(unsupported.into())), None, None).unwrap_err().contains("not JSON-safe"));
            assert_eq!(get(&mut ctx, &id, &name).unwrap(), Value::Int(99));
            close(&mut ctx, &id).unwrap();
            println!("E6 {lang}: complete inputs recorded; pre-send refusal has no request; post-send refusal retains effects; structured domain classification verified");
        }
    }

    #[test]
    fn numeric_extension_policy_agrees_on_python_and_node() {
        for lang in ["python", "node"] {
            let mut ctx = HostContext::default();
            let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
            let key = Value::Text("number".into());
            for value in [1.5, -0.0, 9_007_199_254_740_991.0, -9_007_199_254_740_991.0] {
                set(&mut ctx, &id, &key, &Value::Num(value)).unwrap();
                match get(&mut ctx, &id, &key).unwrap() {
                    Value::Num(n) => assert_eq!(n, value),
                    Value::Int(n) => assert_eq!(n as f64, value),
                    other => panic!("{lang}: {other:?}"),
                }
            }
            for value in [9_007_199_254_740_992.0, -9_007_199_254_740_992.0, 1e100] {
                let before = ctx.foreign_protocol_steps.len();
                assert!(set(&mut ctx, &id, &key, &Value::Num(value)).unwrap_err().contains("outside V_safe"));
                assert_eq!(ctx.foreign_protocol_steps.len(), before);
            }
            let source = if lang == "python" { "float(2**53)" } else { "Number(2**53)" };
            assert!(eval(&mut ctx, &id, None, Some(&Value::Text(source.into())), None, None).unwrap_err().contains("outside V_safe"));
            close(&mut ctx, &id).unwrap();
            println!("E7 {lang}: finite fractions admitted, integral float/scientific-notation bypass refused");
        }
    }

    #[test]
    fn node_non_json_containers_and_lossy_properties_are_refused_without_coercion() {
        let mut ctx = HostContext::default();
        let id = open(&mut ctx, &Value::Text("node".into()), None, None, None, None).unwrap();
        let expressions = [
            "new Map([['k',1]])", "new Set([1,2])", "new Date(0)", "/x/",
            "new Uint8Array([1,2])", "new Number(7)",
            "Object.assign(Object.create({inherited:1}),{own:2})",
            "Object.defineProperty({},'hidden',{value:7})", "({[Symbol('k')]:7})",
            "Object.defineProperty({},'data',{enumerable:true,get(){retained=123;return 7}})",
            "new Array(2)", "Object.assign([1,2],{tag:3})", "[undefined]",
            "(()=>{var x={};x.self=x;return x;})()",
        ];
        for expression in expressions {
            let code = Value::Text(format!("retained = 99; {expression}"));
            let result = eval(&mut ctx,&id,None,Some(&code),None,None);
            assert!(result.as_ref().is_err_and(|e|e.contains("not JSON-safe")),"silently coerced {expression}: {result:?}");
            let reply=ctx.foreign_protocol_steps.last().unwrap().response.as_ref().unwrap().clone();
            assert_eq!(reply["domain_error"],serde_json::json!(true));
            assert_eq!(get(&mut ctx,&id,&Value::Text("retained".into())).unwrap(),Value::Int(99),"getter must not run during refusal");
            println!("NODE_DOMAIN_REFUSAL:{}",serde_json::json!({"expression":expression,"reply":reply.clone(),"retained":99}));
        }
        // Null prototypes, special string keys and shared acyclic subvalues remain records/lists.
        let code=Value::Text("var item = Object.assign(Object.create(null),{constructor:7}); [item,item]".into());
        let result=eval(&mut ctx,&id,None,Some(&code),None,None).unwrap();
        let Value::Map(fields)=result else {panic!()};
        let observed=&fields.iter().find(|(k,_)|k=="result").unwrap().1;
        assert_eq!(observed,&Value::List(vec![Value::Map(vec![("constructor".into(),Value::Int(7))]);2]));
        close(&mut ctx,&id).unwrap();
    }

    #[test]
    fn recursive_values_and_object_keys_cross_without_coercion() {
        fn payload(seed: u64, depth: usize) -> Value {
            if depth == 0 { return match seed % 4 {
                0 => Value::None, 1 => Value::Bool(seed & 4 != 0),
                2 => Value::Int((seed % 2001) as i64 - 1000),
                _ => Value::Text(format!("中文🧪\\\"\n{}", seed % 10)),
            }; }
            match seed % 3 {
                0 => Value::List(vec![payload(seed.wrapping_mul(17).wrapping_add(1), depth-1), payload(seed.wrapping_mul(31).wrapping_add(2), depth-1)]),
                1 => Value::Map(vec![("data".into(), payload(seed.wrapping_mul(43).wrapping_add(3), depth-1)), ("tag".into(), Value::Text("record".into()))]),
                _ => payload(seed, 0),
            }
        }
        for lang in ["python", "node"] {
            let mut ctx = HostContext::default();
            let id = open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
            let name = Value::Text("payload".into());
            for seed in 0..64 {
                let v = payload(seed, 4);
                set(&mut ctx, &id, &name, &v).unwrap();
                assert_eq!(get(&mut ctx, &id, &name).unwrap(), v, "{lang} seed {seed}");
            }
            let special = Value::Map(vec![
                ("__proto__".into(), Value::Map(vec![("polluted".into(), Value::Bool(true))])),
                ("constructor".into(), Value::Text("data".into())),
            ]);
            set(&mut ctx, &id, &name, &special).unwrap();
            assert_eq!(get(&mut ctx, &id, &name).unwrap(), special, "{lang} valid special keys lost");
            for bad in [Value::Map(vec![("a".into(), Value::Int(1)), ("a".into(), Value::Int(2))]),
                Value::Map(vec![("a".into(), Value::List(vec![])), ("b".into(), Value::List(vec![Value::Int(1)]))])] {
                let before = ctx.foreign_protocol_steps.len();
                assert!(set(&mut ctx, &id, &name, &bad).is_err());
                assert_eq!(ctx.foreign_protocol_steps.len(), before);
                assert_eq!(get(&mut ctx, &id, &name).unwrap(), special);
            }
            if lang == "python" {
                let code = Value::Text("{1: 'integer key', '1': 'string key'}".into());
                assert!(eval(&mut ctx, &id, None, Some(&code), None, None).unwrap_err().contains("keys must be strings"));
            } else {
                let code = Value::Text("typeof ({}).polluted === 'undefined'".into());
                let Value::Map(m) = eval(&mut ctx, &id, None, Some(&code), None, None).unwrap() else { panic!() };
                assert_eq!(m.iter().find(|(k,_)| k == "result").unwrap().1, Value::Bool(true));
            }
            close(&mut ctx, &id).unwrap();
            println!("E8 {lang}: 64 nested round-trips, special keys preserved, invalid key/shape inputs refused without requests");
        }
    }

    #[cfg(unix)]
    #[test]
    fn bootstrap_interrupts_busy_eval_and_preserves_idle_state(){
        for lang in ["python","node"]{
            let mut ctx=HostContext::default();let id=open(&mut ctx,&Value::Text(lang.into()),None,None,None,None).unwrap();
            let name=Value::Text("n".into());set(&mut ctx,&id,&name,&Value::Int(8)).unwrap();
            interrupt(&mut ctx,&id).unwrap();assert_eq!(get(&mut ctx,&id,&name).unwrap(),Value::Int(8));
            let marker=ctx.cwd.join(format!("target/interrupt-marker-{}-{lang}",std::process::id()));let _=std::fs::remove_file(&marker);
            let literal=serde_json::to_string(&marker.display().to_string()).unwrap();
            let code=if lang=="python"{format!("with open({literal}, 'w') as marker:\n marker.write('ready')\nwhile True: pass")}
                else{format!("require('fs').writeFileSync({literal}, 'ready'); while(true) {{}}")};
            let pid=ctx.foreign_sessions[as_session_id(&id).unwrap()].child.id();let child_id=id.clone();
            let worker=std::thread::spawn(move||{let out=eval(&mut ctx,&child_id,None,Some(&Value::Text(code)),None,Some(&Value::Num(3.0)));(ctx,out)});
            let deadline=Instant::now()+Duration::from_secs(2);
            while !marker.exists()&&Instant::now()<deadline{std::thread::sleep(Duration::from_millis(1));}
            let ready=marker.exists();
            if ready{assert_eq!(unsafe{libc::kill(-(pid as i32),libc::SIGINT)},0);}
            let (mut ctx,out)=worker.join().unwrap();assert!(ready,"{lang}: code did not start");
            let Value::Map(record)=out.unwrap() else{panic!()};
            assert_eq!(record.iter().find(|(k,_)|k=="ok").unwrap().1,Value::Bool(false));
            assert_eq!(record.iter().find(|(k,_)|k=="alive").unwrap().1,Value::Bool(true));
            let error=record.iter().find(|(k,_)|k=="error").unwrap().1.as_display();
            assert!(error.contains("KeyboardInterrupt")||error.contains("interrupted"),"{lang}: {error}");
            assert_eq!(get(&mut ctx,&id,&name).unwrap(),Value::Int(8));close(&mut ctx,&id).unwrap();std::fs::remove_file(marker).unwrap();
            println!("E10 {lang}: idle interrupt is safe; busy synchronous eval interrupted with kernel and store alive");
        }
    }

}

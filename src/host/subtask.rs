//! Concurrent subtasks: OS file children, in-process functions, foreign subprocesses.
//! Parent `HostContext` drop kills/waits file & foreign children; function threads detach.

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Cap for file-child stdout/stderr returned on `wait` (parent observation).
const FILE_IO_CAP: usize = 8 * 1024;

use crate::host::foreign;
use crate::host::HostContext;
use crate::interp::Interpreter;
use crate::load::load_module;
use crate::value::{CodeBlock, Value};

/// Stored subtask handle (tagged by spawn kind).
pub(crate) enum Handle {
    File(FileTask),
    Foreign(ForeignTask),
    Function(FunctionTask),
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Handle::File(_) => f.write_str("Subtask::File"),
            Handle::Foreign(_) => f.write_str("Subtask::Foreign"),
            Handle::Function(_) => f.write_str("Subtask::Function"),
        }
    }
}

pub(crate) struct FileTask {
    child: Child,
    result_path: PathBuf,
    /// Present when `quiet=True` (piped capture). Absent when inheriting TTY.
    stdout: Option<JoinHandle<String>>,
    stderr: Option<JoinHandle<String>>,
}

pub(crate) struct ForeignTask {
    child: Child,
    temp_script: PathBuf,
    stdout: JoinHandle<String>,
    stderr: JoinHandle<String>,
}

enum FnState {
    Running,
    Done(Value),
    Failed(String),
    Killed,
}

pub(crate) struct FunctionTask {
    state: Arc<Mutex<FnState>>,
    handle: Option<JoinHandle<()>>,
}

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

fn as_code(v: &Value) -> Result<&CodeBlock, String> {
    match v {
        Value::Code(c) => Ok(c),
        _ => Err("code must be a bound ```lang fence value".into()),
    }
}

fn opt_text(v: Option<&Value>) -> Result<Option<&str>, String> {
    match v {
        None | Some(Value::None) => Ok(None),
        Some(v) => Ok(Some(as_text(v, "value")?)),
    }
}

fn is_set(v: Option<&Value>) -> bool {
    match v {
        None => false,
        Some(Value::None) => false,
        Some(Value::Text(s)) if s.is_empty() => false,
        Some(_) => true,
    }
}

fn child_id(id: &Value) -> Result<u64, String> {
    match id {
        Value::Int(n) if *n > 0 => Ok(*n as u64),
        _ => Err("subtask id must be positive int".into()),
    }
}

fn next_id(ctx: &mut HostContext) -> u64 {
    ctx.subtask_seq = ctx.subtask_seq.wrapping_add(1);
    ctx.subtask_seq
}

fn status_map(status: &str, extra: Vec<(String, Value)>) -> Value {
    let mut pairs = vec![("status".into(), Value::Text(status.into()))];
    pairs.extend(extra);
    Value::Map(pairs)
}

fn fn_args_from_value(args: Option<&Value>) -> Result<Vec<(String, Value)>, String> {
    let Some(args) = args else {
        return Ok(Vec::new());
    };
    match args {
        Value::None => Ok(Vec::new()),
        Value::Map(pairs) => Ok(pairs.clone()),
        Value::List(items) => {
            let mut out = Vec::new();
            for (i, item) in items.iter().enumerate() {
                out.push((format!("arg{i}"), item.clone()));
            }
            Ok(out)
        }
        other => Err(format!(
            "fn args must be map or list, got {}",
            other.as_display()
        )),
    }
}

pub fn kill_all(ctx: &mut HostContext) {
    for (_, handle) in ctx.subtasks.drain() {
        match handle {
            Handle::File(mut task) => {
                let _ = task.child.kill();
                let _ = task.child.wait();
                if let Some(h) = task.stdout.take() {
                    let _ = h.join();
                }
                if let Some(h) = task.stderr.take() {
                    let _ = h.join();
                }
                let _ = std::fs::remove_file(&task.result_path);
            }
            Handle::Foreign(mut task) => {
                let _ = task.child.kill();
                let _ = task.child.wait();
                let _ = std::fs::remove_file(&task.temp_script);
                let _ = task.stdout.join();
                let _ = task.stderr.join();
            }
            Handle::Function(mut task) => {
                // Join so fire-and-forget writers (e.g. agent writeback) finish before exit.
                if let Some(h) = task.handle.take() {
                    let _ = h.join();
                }
            }
        }
    }
}

pub fn spawn(ctx: &mut HostContext, bound: &HashMap<String, Value>) -> Result<Value, String> {
    let has_path = is_set(bound.get("path"));
    let has_fn = is_set(bound.get("fn"));
    let has_code = is_set(bound.get("code"));
    let has_lang = is_set(bound.get("lang")) && is_set(bound.get("source"));
    let kinds = [has_path, has_fn, has_code, has_lang]
        .into_iter()
        .filter(|b| *b)
        .count();
    if kinds != 1 {
        return Err(
            "spawn: specify exactly one of path=, fn=, code=, or lang=+source=".into(),
        );
    }
    if has_path {
        spawn_file(
            ctx,
            require(bound, "path")?,
            bound.get("args"),
            bound.get("quiet"),
        )
    } else if has_fn {
        spawn_fn(ctx, require(bound, "fn")?, bound.get("args"))
    } else if has_code {
        spawn_code(ctx, require(bound, "code")?, bound.get("stdin"))
    } else {
        spawn_lang(
            ctx,
            require(bound, "lang")?,
            require(bound, "source")?,
            bound.get("stdin"),
        )
    }
}

fn spawn_file(
    ctx: &mut HostContext,
    path: &Value,
    args: Option<&Value>,
    quiet: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.caps.exec {
        return Err("subtask spawn disabled (exec capability off)".into());
    }
    let rel = as_text(path, "path")?;
    let joined = if std::path::Path::new(rel).is_absolute() {
        std::path::PathBuf::from(rel)
    } else {
        ctx.cwd.join(rel)
    };
    // Absolutize so the child `for_run` cwd/parent resolution does not double-prefix
    // a relative path like `tests/ext/.marqdo/...` when current_dir is already `tests/ext`.
    let file = std::fs::canonicalize(&joined).unwrap_or_else(|_| {
        if joined.is_absolute() {
            joined.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(&joined)
        }
    });
    let exe = std::env::current_exe().map_err(|e| format!("subtask: current_exe: {e}"))?;
    let id = next_id(ctx);
    let result_path = std::env::temp_dir().join(format!(
        "marqdo-subtask-{}-{}.json",
        std::process::id(),
        id
    ));
    let mut cmd = Command::new(&exe);
    cmd.arg("run").arg(&file);
    cmd.arg("--emit-result").arg(&result_path);
    cmd.current_dir(&ctx.cwd);
    cmd.stdin(Stdio::null());
    let quiet = match quiet {
        None | Some(Value::None) => true,
        Some(v) => v.truthy(),
    };
    // quiet: pipe-capture for parent observation (no TTY noise).
    // !quiet: inherit parent TTY; wait map omits stdout/stderr bodies.
    if quiet {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
    }
    if let Some(Value::List(items)) = args {
        for item in items {
            cmd.arg(as_text(item, "arg")?);
        }
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("subtask spawn {}: {e}", file.display()))?;
    let (stdout, stderr) = if quiet {
        let mut stdout_pipe = child.stdout.take();
        let mut stderr_pipe = child.stderr.take();
        let stdout = thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(ref mut r) = stdout_pipe {
                let _ = r.read_to_end(&mut buf);
            }
            String::from_utf8_lossy(&buf).into_owned()
        });
        let stderr = thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(ref mut r) = stderr_pipe {
                let _ = r.read_to_end(&mut buf);
            }
            String::from_utf8_lossy(&buf).into_owned()
        });
        (Some(stdout), Some(stderr))
    } else {
        (None, None)
    };
    ctx.subtasks.insert(
        id,
        Handle::File(FileTask {
            child,
            result_path,
            stdout,
            stderr,
        }),
    );
    Ok(Value::Int(id as i64))
}

fn spawn_fn(
    ctx: &mut HostContext,
    name: &Value,
    args: Option<&Value>,
) -> Result<Value, String> {
    let name = as_text(name, "fn")?.to_string();
    let call_args = fn_args_from_value(args)?;
    let module_path = ctx
        .entry_path
        .clone()
        .ok_or_else(|| "subtask fn: no entry module (run a .mq.md file first)".to_string())?;
    let cwd = ctx.cwd.clone();
    let caps = ctx.caps.clone();
    let foreign_cmds = ctx.foreign_cmds.clone();
    let fs_root = ctx.fs_root.clone();
    let argv = ctx.argv.clone();

    let state = Arc::new(Mutex::new(FnState::Running));
    let state_bg = Arc::clone(&state);
    let handle = thread::spawn(move || {
        let result = (|| -> Result<Value, String> {
            let module = load_module(&module_path).map_err(|e| e.to_string())?;
            let source = std::fs::read_to_string(&module_path)
                .map_err(|e| format!("subtask fn: read {}: {e}", module_path.display()))?;
            let mut host = HostContext::for_capture(Some(&module_path), caps);
            host.cwd = cwd;
            host.fs_root = fs_root;
            host.argv = argv;
            host.foreign_cmds = foreign_cmds;
            host.set_entry_source(Some(&module_path), &source);
            let mut interp = Interpreter::with_capture(Some(&module_path), false).with_host(host);
            interp
                .invoke_function(&module, &name, &call_args)
                .map_err(|e| e.to_string())
        })();
        if let Ok(mut st) = state_bg.lock() {
            if !matches!(*st, FnState::Killed) {
                *st = match result {
                    Ok(v) => FnState::Done(v),
                    Err(e) => FnState::Failed(e),
                };
            }
        }
    });

    let id = next_id(ctx);
    ctx.subtasks.insert(
        id,
        Handle::Function(FunctionTask {
            state,
            handle: Some(handle),
        }),
    );
    Ok(Value::Int(id as i64))
}

fn spawn_code(
    ctx: &mut HostContext,
    code: &Value,
    stdin: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.caps.exec {
        return Err("subtask spawn disabled (exec capability off)".into());
    }
    let block = as_code(code)?;
    let stdin = opt_text(stdin)?;
    spawn_foreign(ctx, &block.lang, &block.source, stdin)
}

fn spawn_lang(
    ctx: &mut HostContext,
    lang: &Value,
    source: &Value,
    stdin: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.caps.exec {
        return Err("subtask spawn disabled (exec capability off)".into());
    }
    let lang = as_text(lang, "lang")?;
    let source = as_text(source, "source")?;
    let stdin = opt_text(stdin)?;
    spawn_foreign(ctx, lang, source, stdin)
}

fn spawn_foreign(
    ctx: &mut HostContext,
    lang: &str,
    source: &str,
    stdin: Option<&str>,
) -> Result<Value, String> {
    let spawned = foreign::spawn_source(ctx, lang, source, stdin)?;
    let id = next_id(ctx);
    ctx.subtasks.insert(
        id,
        Handle::Foreign(ForeignTask {
            child: spawned.child,
            temp_script: spawned.script_path,
            stdout: spawned.stdout,
            stderr: spawned.stderr,
        }),
    );
    Ok(Value::Int(id as i64))
}

pub fn poll(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = child_id(id)?;
    let handle = ctx
        .subtasks
        .get_mut(&id)
        .ok_or_else(|| format!("subtask: unknown id {id}"))?;
    match handle {
        Handle::File(task) => poll_child(&mut task.child),
        Handle::Foreign(task) => poll_child(&mut task.child),
        Handle::Function(task) => poll_function(&task.state),
    }
}

fn poll_child(child: &mut Child) -> Result<Value, String> {
    match child.try_wait().map_err(|e| format!("subtask poll: {e}"))? {
        None => Ok(status_map("running", vec![])),
        Some(status) => {
            let code = status.code().unwrap_or(-1);
            let st = if status.success() { "done" } else { "failed" };
            Ok(status_map(
                st,
                vec![("code".into(), Value::Int(code as i64))],
            ))
        }
    }
}

fn poll_function(state: &Arc<Mutex<FnState>>) -> Result<Value, String> {
    let st = state
        .lock()
        .map_err(|_| "subtask fn: lock poisoned".to_string())?;
    match &*st {
        FnState::Running => Ok(status_map("running", vec![])),
        FnState::Done(v) => Ok(status_map(
            "done",
            vec![("value".into(), v.clone())],
        )),
        FnState::Failed(e) => Ok(status_map(
            "failed",
            vec![("error".into(), Value::Text(e.clone()))],
        )),
        FnState::Killed => Ok(status_map("failed", vec![("error".into(), Value::Text("killed".into()))])),
    }
}

pub fn join(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = child_id(id)?;
    let handle = ctx
        .subtasks
        .remove(&id)
        .ok_or_else(|| format!("subtask: unknown id {id}"))?;
    match handle {
        Handle::File(mut task) => {
            let status = task
                .child
                .wait()
                .map_err(|e| format!("subtask join: {e}"))?;
            let code = status.code().unwrap_or(-1) as i64;
            let value = read_emitted_result(&task.result_path);
            let _ = std::fs::remove_file(&task.result_path);
            let stdout = task
                .stdout
                .take()
                .map(|h| h.join().unwrap_or_default())
                .unwrap_or_default();
            let stderr = task
                .stderr
                .take()
                .map(|h| h.join().unwrap_or_default())
                .unwrap_or_default();
            let mut pairs = vec![
                ("code".into(), Value::Int(code)),
                ("value".into(), value),
            ];
            if !stdout.is_empty() {
                pairs.push(("stdout".into(), Value::Text(clip_io(&stdout))));
            }
            if !stderr.is_empty() {
                pairs.push(("stderr".into(), Value::Text(clip_io(&stderr))));
            }
            Ok(Value::Map(pairs))
        }
        Handle::Foreign(task) => join_foreign(task),
        Handle::Function(task) => join_function(task),
    }
}

fn read_emitted_result(path: &PathBuf) -> Value {
    match std::fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<serde_json::Value>(text.trim()) {
            Ok(j) => crate::host::json::json_to_value(&j).unwrap_or(Value::None),
            Err(_) => Value::None,
        },
        Err(_) => Value::None,
    }
}

fn join_foreign(mut task: ForeignTask) -> Result<Value, String> {
    let status = task
        .child
        .wait()
        .map_err(|e| format!("subtask foreign join: {e}"))?;
    let stdout = task.stdout.join().unwrap_or_default();
    let stderr = task.stderr.join().unwrap_or_default();
    let _ = std::fs::remove_file(&task.temp_script);
    if stdout.len() > foreign::MAX_OUTPUT || stderr.len() > foreign::MAX_OUTPUT {
        return Err("subtask foreign output too large".into());
    }
    if !status.success() {
        let code = status.code().unwrap_or(1);
        let mut msg = format!("subtask foreign failed (exit={code})");
        if !stderr.trim().is_empty() {
            msg.push_str(": ");
            msg.push_str(stderr.trim());
        }
        return Err(msg);
    }
    let mut out = stdout;
    if out.ends_with('\n') {
        out.pop();
        if out.ends_with('\r') {
            out.pop();
        }
    }
    Ok(Value::Text(out))
}

fn join_function(mut task: FunctionTask) -> Result<Value, String> {
    if let Some(handle) = task.handle.take() {
        let _ = handle.join();
    }
    let st = task
        .state
        .lock()
        .map_err(|_| "subtask fn: lock poisoned".to_string())?;
    match &*st {
        FnState::Done(v) => Ok(v.clone()),
        FnState::Failed(e) => Err(e.clone()),
        FnState::Killed => Err("subtask fn killed".into()),
        FnState::Running => Err("subtask fn still running".into()),
    }
}

pub fn kill(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = child_id(id)?;
    let handle = ctx
        .subtasks
        .remove(&id)
        .ok_or_else(|| format!("subtask: unknown id {id}"))?;
    match handle {
        Handle::File(mut task) => {
            task.child
                .kill()
                .map_err(|e| format!("subtask kill: {e}"))?;
            let _ = task.child.wait();
            if let Some(h) = task.stdout.take() {
                let _ = h.join();
            }
            if let Some(h) = task.stderr.take() {
                let _ = h.join();
            }
            let _ = std::fs::remove_file(&task.result_path);
        }
        Handle::Foreign(mut task) => {
            task.child.kill().map_err(|e| format!("subtask kill: {e}"))?;
            let _ = task.child.wait();
            let _ = std::fs::remove_file(&task.temp_script);
            let _ = task.stdout.join();
            let _ = task.stderr.join();
        }
        Handle::Function(mut task) => {
            if let Ok(mut st) = task.state.lock() {
                if matches!(*st, FnState::Running) {
                    *st = FnState::Killed;
                }
            }
            drop(task.handle.take());
        }
    }
    Ok(Value::None)
}

pub fn wait_all(ctx: &mut HostContext) -> Result<Value, String> {
    let ids: Vec<u64> = ctx.subtasks.keys().copied().collect();
    let mut results = Vec::new();
    for id in ids {
        results.push(join(ctx, &Value::Int(id as i64))?);
    }
    Ok(Value::List(results))
}

fn clip_io(s: &str) -> String {
    let mut it = s.chars();
    let head: String = it.by_ref().take(FILE_IO_CAP).collect();
    if it.next().is_some() {
        format!("{head}\n…(truncated)")
    } else {
        head
    }
}

fn require<'a>(bound: &'a HashMap<String, Value>, key: &str) -> Result<&'a Value, String> {
    bound
        .get(key)
        .ok_or_else(|| format!("host call missing `{key}`"))
}

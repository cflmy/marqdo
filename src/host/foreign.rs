//! Foreign language runners (subprocess). Import L1 `lib/foreign` to use.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::foreign;
use crate::host::HostContext;
use crate::value::{CodeBlock, Value};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const MAX_OUTPUT: usize = 1_048_576;

fn as_text<'a>(v: &'a Value, what: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{what} must be text")),
    }
}

fn opt_text(v: Option<&Value>) -> Result<Option<&str>, String> {
    match v {
        None | Some(Value::None) => Ok(None),
        Some(v) => Ok(Some(as_text(v, "value")?)),
    }
}

fn as_code(v: &Value) -> Result<&CodeBlock, String> {
    match v {
        Value::Code(c) => Ok(c),
        _ => Err("run needs a code value (bind with `name` = then a ```lang fence)".into()),
    }
}

/// Register interpreter argv for `lang` (e.g. venv python).
pub fn set_cmd(
    ctx: &mut HostContext,
    lang: &Value,
    cmd: &Value,
    args: Option<&Value>,
) -> Result<Value, String> {
    let lang = as_text(lang, "lang")?.to_ascii_lowercase();
    let cmd = as_text(cmd, "cmd")?;
    let mut argv = vec![cmd.to_string()];
    if let Some(a) = args {
        match a {
            Value::List(xs) => {
                for x in xs {
                    argv.push(x.as_display());
                }
            }
            Value::Text(s) => argv.push(s.clone()),
            Value::None => {}
            _ => return Err("set_cmd args must be list or text".into()),
        }
    }
    ctx.foreign_cmds.insert(lang, argv);
    Ok(Value::None)
}

pub fn run(
    ctx: &mut HostContext,
    code: &Value,
    stdin: Option<&Value>,
) -> Result<Value, String> {
    let block = as_code(code)?;
    let stdin = opt_text(stdin)?;
    run_source(ctx, &block.lang, &block.source, stdin)
}

pub fn run_lang(
    ctx: &mut HostContext,
    lang: &Value,
    source: &Value,
    stdin: Option<&Value>,
) -> Result<Value, String> {
    let lang = as_text(lang, "lang")?;
    let source = as_text(source, "source")?;
    let stdin = opt_text(stdin)?;
    run_source(ctx, lang, source, stdin)
}

pub fn langs(ctx: &HostContext) -> Result<Value, String> {
    let mut names: Vec<String> = ctx.foreign_cmds.keys().cloned().collect();
    if !names.iter().any(|n| n == "python" || n == "python3") {
        names.push("python".into());
    }
    names.sort();
    names.dedup();
    Ok(Value::List(names.into_iter().map(Value::Text).collect()))
}

/// Shared entry for CLI host and live view `/api/foreign-run`.
pub fn run_source(
    ctx: &HostContext,
    lang: &str,
    source: &str,
    stdin: Option<&str>,
) -> Result<Value, String> {
    let lang_key = lang.to_ascii_lowercase();
    let argv = resolve_argv(ctx, &lang_key)?;
    let script_path = write_temp_script(&lang_key, source)?;
    let result = run_argv(ctx, &argv, &script_path, stdin, &lang_key);
    let _ = std::fs::remove_file(&script_path);
    result
}

/// Resolve argv with optional one-shot cmd override (view command box).
pub fn resolve_argv_with_override(
    ctx: &HostContext,
    lang: &str,
    cmd_override: Option<&str>,
) -> Result<Vec<String>, String> {
    if let Some(cmd) = cmd_override.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(split_cmd_line(cmd));
    }
    resolve_argv(ctx, lang)
}

fn resolve_argv(ctx: &HostContext, lang: &str) -> Result<Vec<String>, String> {
    if let Some(argv) = ctx.foreign_cmds.get(lang) {
        return Ok(argv.clone());
    }
    if lang == "python3" {
        if let Some(argv) = ctx.foreign_cmds.get("python") {
            return Ok(argv.clone());
        }
    }
    if lang == "python" {
        if let Some(argv) = ctx.foreign_cmds.get("python3") {
            return Ok(argv.clone());
        }
    }

    let env_key = format!("MARQDO_FOREIGN_{}", lang.to_ascii_uppercase());
    if let Ok(v) = std::env::var(&env_key) {
        let v = v.trim();
        if !v.is_empty() {
            return Ok(split_cmd_line(v));
        }
    }
    if lang == "python" || lang == "python3" || lang == "py" {
        if let Ok(v) = std::env::var("MARQDO_FOREIGN_PYTHON") {
            let v = v.trim();
            if !v.is_empty() {
                return Ok(split_cmd_line(v));
            }
        }
    }
    if let Some(argv) = foreign::default_argv(lang) {
        return Ok(argv);
    }

    Err(format!(
        "no interpreter configured for lang=`{lang}` — set_cmd lang={lang} cmd=… \
         or env {env_key} (check your environment)"
    ))
}

fn split_cmd_line(s: &str) -> Vec<String> {
    s.split_whitespace().map(|p| p.to_string()).collect()
}

fn write_temp_script(lang: &str, source: &str) -> Result<PathBuf, String> {
    let ext = match lang {
        "python" | "python3" | "py" => "py",
        "javascript" | "js" | "node" => "js",
        other => {
            if other.len() <= 8 && other.chars().all(|c| c.is_ascii_alphanumeric()) {
                other
            } else {
                "txt"
            }
        }
    };
    let body = if matches!(lang, "python" | "python3" | "py") {
        format!("# -*- coding: utf-8 -*-\n{source}")
    } else {
        source.to_string()
    };
    let name = format!(
        "marqdo-foreign-{}-{}.{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        ext
    );
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, body.as_bytes())
        .map_err(|e| format!("failed to write temp foreign script: {e}"))?;
    Ok(path)
}

/// Background foreign subprocess (for `lib/subtask`); caller removes `script_path` after join.
pub struct SpawnedForeign {
    pub child: std::process::Child,
    pub script_path: PathBuf,
    pub stdout: thread::JoinHandle<String>,
    pub stderr: thread::JoinHandle<String>,
}

pub fn spawn_source(
    ctx: &HostContext,
    lang: &str,
    source: &str,
    stdin: Option<&str>,
) -> Result<SpawnedForeign, String> {
    let lang_key = lang.to_ascii_lowercase();
    let argv = resolve_argv(ctx, &lang_key)?;
    let script_path = write_temp_script(&lang_key, source)?;
    match spawn_argv(ctx, &argv, &script_path, stdin, &lang_key) {
        Ok(spawned) => Ok(spawned),
        Err(e) => {
            let _ = std::fs::remove_file(&script_path);
            Err(e)
        }
    }
}

fn build_foreign_command(
    ctx: &HostContext,
    argv: &[String],
    script: &Path,
) -> Result<Command, String> {
    if argv.is_empty() {
        return Err("foreign cmd argv is empty — check set_cmd / env".into());
    }
    let mut command = Command::new(&argv[0]);
    for a in &argv[1..] {
        command.arg(a);
    }
    command.arg(script);
    command.current_dir(&ctx.cwd);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.env("PYTHONIOENCODING", "utf-8");
    command.env("PYTHONUTF8", "1");
    Ok(command)
}

fn spawn_argv(
    ctx: &HostContext,
    argv: &[String],
    script: &Path,
    stdin: Option<&str>,
    lang: &str,
) -> Result<SpawnedForeign, String> {
    let mut child = build_foreign_command(ctx, argv, script)?
        .spawn()
        .map_err(|e| {
            format!(
                "foreign spawn failed (lang={lang}, cmd={}): {e} — check your interpreter / \
                 MARQDO_FOREIGN_* / set_cmd",
                argv.join(" ")
            )
        })?;

    if let Some(data) = stdin {
        if let Some(mut pipe) = child.stdin.take() {
            let _ = pipe.write_all(data.as_bytes());
        }
    } else {
        drop(child.stdin.take());
    }

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

    Ok(SpawnedForeign {
        child,
        script_path: script.to_path_buf(),
        stdout,
        stderr,
    })
}

fn run_argv(
    ctx: &HostContext,
    argv: &[String],
    script: &Path,
    stdin: Option<&str>,
    lang: &str,
) -> Result<Value, String> {
    let spawned = spawn_argv(ctx, argv, script, stdin, lang)?;
    let SpawnedForeign {
        mut child,
        stdout: t_out,
        stderr: t_err,
        ..
    } = spawned;

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if start.elapsed() > DEFAULT_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "foreign run timed out after {}s (lang={lang}, cmd={}) — check your script / environment",
                    DEFAULT_TIMEOUT.as_secs(),
                    argv.join(" ")
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(e) => {
                return Err(format!(
                    "foreign run failed (lang={lang}, cmd={}): {e}",
                    argv.join(" ")
                ));
            }
        }
    };

    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();
    if stdout.len() > MAX_OUTPUT || stderr.len() > MAX_OUTPUT {
        return Err(format!(
            "foreign output too large (lang={lang}) — truncate script output"
        ));
    }

    if !status.success() {
        let code = status.code().unwrap_or(1);
        let mut msg = format!(
            "foreign run failed (lang={lang}, cmd={}, exit={code})",
            argv.join(" ")
        );
        if !stderr.trim().is_empty() {
            msg.push_str(": ");
            msg.push_str(stderr.trim());
        }
        msg.push_str(" — check your interpreter / MARQDO_FOREIGN_* / set_cmd / script");
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

/// View/API helper: run with optional command override string.
pub fn run_with_cmd_override(
    cwd: &Path,
    lang: &str,
    source: &str,
    cmd_override: Option<&str>,
) -> Result<String, String> {
    let mut ctx = HostContext::default();
    ctx.cwd = cwd.to_path_buf();
    let argv = resolve_argv_with_override(&ctx, lang, cmd_override)?;
    let script_path = write_temp_script(&lang.to_ascii_lowercase(), source)?;
    let result = run_argv(&ctx, &argv, &script_path, None, lang);
    let _ = std::fs::remove_file(&script_path);
    match result {
        Ok(Value::Text(s)) => Ok(s),
        Ok(v) => Ok(v.as_display()),
        Err(e) => Err(e),
    }
}

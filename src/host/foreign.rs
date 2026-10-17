//! Foreign language runners (subprocess) and persistent kernels.
//!
//! F0: safe one-shot execution + structured `invoke` records.
//! Later phases add language registry, JSON bridge, parallel fan-out and sessions.

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::foreign::registry::{self, LangSpec};
use crate::foreign::split_cmd_line;
use crate::host::json::{json_to_value, value_to_json};
use crate::host::{HostCaps, HostContext};
use crate::value::{CodeBlock, Value};

const DEFAULT_TIMEOUT_SECS: f64 = 10.0;
const MAX_TIMEOUT_SECS: f64 = 3600.0;
pub(crate) const MAX_OUTPUT: usize = 1_048_576;
const MIN_OUTPUT: usize = 1024;
const MAX_OUTPUT_LIMIT: usize = 64 * 1024 * 1024;
const INPUT_ENV_CAP: usize = 8 * 1024;
const FAILURE_TAIL: usize = 4096;

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

fn parse_env(v: Option<&Value>) -> Result<Vec<(String, String)>, String> {
    match v {
        None | Some(Value::None) => Ok(Vec::new()),
        Some(Value::Map(entries)) => Ok(entries
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    Value::Secret(s) => s.clone(),
                    other => other.as_display(),
                };
                (k.clone(), value)
            })
            .collect()),
        Some(_) => Err("env must be a map".into()),
    }
}

fn parse_args(v: Option<&Value>) -> Result<Vec<String>, String> {
    match v {
        None | Some(Value::None) => Ok(Vec::new()),
        Some(Value::List(xs)) => Ok(xs.iter().map(|x| x.as_display()).collect()),
        Some(Value::Text(s)) => Ok(vec![s.clone()]),
        Some(_) => Err("args must be list or text".into()),
    }
}

fn parse_bool(v: Option<&Value>, what: &str, default: bool) -> Result<bool, String> {
    match v {
        None | Some(Value::None) => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("{what} must be bool")),
    }
}

fn parse_timeout(v: Option<&Value>) -> Result<Duration, String> {
    let secs = match v {
        None | Some(Value::None) => DEFAULT_TIMEOUT_SECS,
        Some(Value::Int(n)) => *n as f64,
        Some(Value::Num(n)) => *n,
        Some(_) => return Err("timeout must be a number of seconds".into()),
    };
    if !secs.is_finite() || secs <= 0.0 {
        return Err("timeout must be > 0 seconds".into());
    }
    if secs > MAX_TIMEOUT_SECS {
        return Err(format!("timeout must be <= {MAX_TIMEOUT_SECS} seconds"));
    }
    Ok(Duration::from_secs_f64(secs))
}

fn parse_max_output(v: Option<&Value>) -> Result<usize, String> {
    let n = match v {
        None | Some(Value::None) => MAX_OUTPUT,
        Some(Value::Int(n)) if *n > 0 => *n as usize,
        Some(Value::Int(_)) => return Err("max_output must be > 0".into()),
        Some(_) => return Err("max_output must be an integer byte count".into()),
    };
    if n < MIN_OUTPUT {
        return Err(format!("max_output must be >= {MIN_OUTPUT} bytes"));
    }
    if n > MAX_OUTPUT_LIMIT {
        return Err(format!("max_output must be <= {MAX_OUTPUT_LIMIT} bytes"));
    }
    Ok(n)
}

fn as_code(v: &Value) -> Result<&CodeBlock, String> {
    match v {
        Value::Code(c) => Ok(c),
        _ => Err("run needs a code value (bind with `name` = then a ```lang fence)".into()),
    }
}

/// One execution request. Owned so it can be sent to worker threads later.
#[derive(Debug, Clone)]
pub(crate) struct Job {
    pub lang: String,
    pub source: String,
    pub stdin: Option<String>,
    pub input_json: Option<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
    pub timeout: Duration,
    pub extra_args: Vec<String>,
    pub max_output: usize,
    pub keep: bool,
    pub cmd_override: Option<String>,
}

impl Job {
    fn simple(lang: &str, source: &str, stdin: Option<&str>) -> Self {
        Self {
            lang: lang.to_string(),
            source: source.to_string(),
            stdin: stdin.map(str::to_string),
            input_json: None,
            env: Vec::new(),
            cwd: None,
            timeout: Duration::from_secs_f64(DEFAULT_TIMEOUT_SECS),
            extra_args: Vec::new(),
            max_output: MAX_OUTPUT,
            keep: false,
            cmd_override: None,
        }
    }
}

/// Result of one subprocess execution. `ok` is the success bit; failures are data.
#[derive(Debug, Clone)]
pub(crate) struct Execution {
    pub ok: bool,
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub ms: u64,
    pub timed_out: bool,
    pub truncated: bool,
    pub error: String,
    pub lang: String,
    pub cmd: String,
    pub script: String,
}

impl Execution {
    fn failure_ok(&self) -> bool {
        self.ok
    }

    fn record(&self, result: Value, result_error: String) -> Value {
        Value::Map(vec![
            ("ok".into(), Value::Bool(self.failure_ok())),
            ("code".into(), Value::Int(self.code as i64)),
            ("stdout".into(), Value::Text(self.stdout.clone())),
            ("stderr".into(), Value::Text(self.stderr.clone())),
            ("ms".into(), Value::Int(self.ms as i64)),
            ("lang".into(), Value::Text(self.lang.clone())),
            ("cmd".into(), Value::Text(self.cmd.clone())),
            ("timed_out".into(), Value::Bool(self.timed_out)),
            ("truncated".into(), Value::Bool(self.truncated)),
            ("result".into(), result),
            ("result_error".into(), Value::Text(result_error)),
            ("error".into(), Value::Text(self.error.clone())),
            ("script".into(), Value::Text(self.script.clone())),
            ("cached".into(), Value::Bool(false)),
        ])
    }
}

/// Arguments for `foreign.invoke`. Kept as a struct so dispatch stays readable.
#[derive(Default)]
pub struct InvokeArgs<'a> {
    pub code: Option<&'a Value>,
    pub lang: Option<&'a Value>,
    pub source: Option<&'a Value>,
    pub stdin: Option<&'a Value>,
    pub input: Option<&'a Value>,
    pub env: Option<&'a Value>,
    pub cwd: Option<&'a Value>,
    pub timeout: Option<&'a Value>,
    pub args: Option<&'a Value>,
    pub output: Option<&'a Value>,
    pub max_output: Option<&'a Value>,
    pub cmd: Option<&'a Value>,
    pub keep: Option<&'a Value>,
}

/// Register interpreter argv for `lang` (e.g. venv python).
pub fn set_cmd(
    ctx: &mut HostContext,
    lang: &Value,
    cmd: &Value,
    args: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    let requested = as_text(lang, "lang")?.to_ascii_lowercase();
    let canon = registry::canonical(&requested);
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
    let spec = LangSpec {
        lang: canon.clone(),
        argv: argv.clone(),
        template: None,
        ext: None,
        env: Vec::new(),
        version_args: Vec::new(),
        session: registry::session_supported(&canon),
        base_dir: None,
    };
    ctx.foreign_specs.insert(canon.clone(), spec);
    ctx.foreign_cmds.insert(canon, argv);
    Ok(Value::None)
}

fn parse_template(v: Option<&Value>) -> Result<Option<Vec<String>>, String> {
    match v {
        None | Some(Value::None) => Ok(None),
        Some(Value::List(xs)) => Ok(Some(xs.iter().map(|x| x.as_display()).collect())),
        Some(Value::Text(s)) => Ok(Some(vec![s.clone()])),
        Some(_) => Err("template must be a list of text".into()),
    }
}

fn parse_version_args(v: Option<&Value>) -> Result<Vec<String>, String> {
    match v {
        None | Some(Value::None) => Ok(Vec::new()),
        Some(Value::List(xs)) => Ok(xs.iter().map(|x| x.as_display()).collect()),
        Some(Value::Text(s)) => Ok(vec![s.clone()]),
        Some(_) => Err("version_args must be a list of text".into()),
    }
}

/// Register a language invocation (`register`). `cmd` and `template` are exclusive.
pub fn register(
    ctx: &mut HostContext,
    lang: &Value,
    cmd: Option<&Value>,
    args: Option<&Value>,
    template: Option<&Value>,
    ext: Option<&Value>,
    env: Option<&Value>,
    version_args: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    let requested = as_text(lang, "lang")?.to_ascii_lowercase();
    let canon = registry::canonical(&requested);
    let cmd = opt_text(cmd)?;
    let template = parse_template(template)?;
    if cmd.is_some() && template.is_some() {
        return Err("register: cmd and template are mutually exclusive".into());
    }
    if cmd.is_none() && template.is_none() {
        return Err("register: provide cmd or template".into());
    }
    let mut argv = match &cmd {
        Some(c) => {
            let mut v = vec![c.clone()];
            v.extend(parse_args(args)?);
            v
        }
        None => vec![template
            .as_ref()
            .and_then(|t| t.first())
            .cloned()
            .ok_or_else(|| "register: template must not be empty".to_string())?],
    };
    if let Some(t) = &template {
        if t.is_empty() {
            return Err("register: template must not be empty".into());
        }
        if t.iter().any(|x| x == "{file}") && t.iter().any(|x| x == "{inline}") {
            return Err("register: {file} and {inline} cannot be used together".into());
        }
        // A template without file/inline gets the script path appended.
        if !t.iter().any(|x| x == "{file}" || x == "{inline}") {
            // leave as-is; prepare adds the script path
        }
    }
    if cmd.is_none() {
        argv = vec![template.as_ref().unwrap()[0].clone()];
    }
    let ext = match opt_text(ext)? {
        Some(e) => {
            let e = e.trim().trim_start_matches('.').to_string();
            if e.is_empty()
                || e.len() > 16
                || !e.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-')
            {
                return Err("register: ext must be 1..16 ASCII alphanumerics".into());
            }
            Some(e)
        }
        None => None,
    };
    let env = parse_env(env)?;
    let version_args = parse_version_args(version_args)?;
    let spec = LangSpec {
        lang: canon.clone(),
        argv: argv.clone(),
        template,
        ext,
        env,
        version_args,
        session: registry::session_supported(&canon),
        base_dir: None,
    };
    ctx.foreign_specs.insert(canon.clone(), spec);
    if ctx.foreign_specs.get(&canon).map(|s| s.template.is_none()).unwrap_or(false) {
        ctx.foreign_cmds.insert(canon, argv);
    }
    Ok(Value::None)
}

/// Remove a process-local registration; presets/config/env still apply.
pub fn unset(ctx: &mut HostContext, lang: &Value) -> Result<Value, String> {
    ensure_config(ctx);
    let requested = as_text(lang, "lang")?.to_ascii_lowercase();
    let canon = registry::canonical(&requested);
    let mut removed = ctx.foreign_specs.remove(&canon).is_some();
    removed |= ctx.foreign_specs.remove(&requested).is_some();
    removed |= ctx.foreign_cmds.remove(&canon).is_some();
    removed |= ctx.foreign_cmds.remove(&requested).is_some();
    Ok(Value::Bool(removed))
}

fn discover_config_path(ctx: &HostContext) -> Option<PathBuf> {
    if let Ok(v) = std::env::var("MARQDO_FOREIGN_CONFIG") {
        let v = v.trim();
        if !v.is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    let mut candidates = vec![ctx.cwd.join(".marqdo").join("foreign.toml")];
    if let Some(entry) = &ctx.entry_path {
        if let Some(dir) = entry.parent() {
            candidates.push(dir.join(".marqdo").join("foreign.toml"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            candidates.push(
                PathBuf::from(home)
                    .join(".config")
                    .join("marqdo")
                    .join("foreign.toml"),
            );
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Load project config once; malformed auto-config warns but does not block presets.
pub fn ensure_config(ctx: &mut HostContext) {
    if ctx.foreign_config_loaded {
        return;
    }
    ctx.foreign_config_loaded = true;
    if let Some(path) = discover_config_path(ctx) {
        if let Err(e) = load_config_path(ctx, &path) {
            eprintln!(
                "marqdo: foreign config {} ignored: {e}",
                path.display()
            );
        }
    }
}

fn load_config_path(ctx: &mut HostContext, path: &Path) -> Result<usize, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    let parsed = crate::host::toml_ops::parse(&Value::Text(source))
        .map_err(|e| format!("parse {}: {e}", path.display()))?;
    let map = match parsed {
        Value::Map(m) => m,
        _ => return Err("config must be a TOML table".into()),
    };
    let base = path.parent().map(|p| p.to_path_buf());
    let mut count = 0usize;
    for (lang, val) in map {
        let spec = spec_from_config(&lang, &val, base.as_deref())?;
        let canon = registry::canonical(&lang);
        // Runtime registrations win; config fills the lower layer.
        if !ctx.foreign_config_specs.contains_key(&canon) {
            ctx.foreign_config_specs.insert(canon.clone(), spec.clone());
        }
        if !ctx.foreign_specs.contains_key(&canon) && !ctx.foreign_cmds.contains_key(&canon) {
            if let Some(t) = &spec.template {
                if !t.iter().any(|x| x == "{inline}") {
                    ctx.foreign_cmds.insert(canon, spec.argv.clone());
                }
            } else {
                ctx.foreign_cmds.insert(canon, spec.argv.clone());
            }
        }
        count += 1;
    }
    Ok(count)
}

fn spec_from_config(lang: &str, v: &Value, base: Option<&Path>) -> Result<LangSpec, String> {
    let entries = match v {
        Value::Map(m) => m,
        _ => return Err(format!("[{lang}] must be a table")),
    };
    let mut cmd: Option<String> = None;
    let mut args: Vec<String> = Vec::new();
    let mut template: Option<Vec<String>> = None;
    let mut ext: Option<String> = None;
    let mut env: Vec<(String, String)> = Vec::new();
    let mut version_args: Vec<String> = Vec::new();
    let mut session = registry::session_supported(&registry::canonical(lang));
    for (k, val) in entries {
        match k.as_str() {
            "cmd" => cmd = Some(as_text(val, "cmd")?.to_string()),
            "args" => args = parse_args(Some(val))?,
            "template" => template = parse_template(Some(val))?,
            "ext" => ext = opt_text(Some(val))?,
            "env" => env = parse_env(Some(val))?,
            "version_args" => version_args = parse_version_args(Some(val))?,
            "session" => {
                session = parse_bool(Some(val), "session", session)?;
            }
            _ => {}
        }
    }
    if cmd.is_some() && template.is_some() {
        return Err(format!("[{lang}]: cmd and template are mutually exclusive"));
    }
    if cmd.is_none() && template.is_none() {
        return Err(format!("[{lang}]: provide cmd or template"));
    }
    let canon = registry::canonical(lang);
    let mut argv = if let Some(c) = &cmd {
        let mut v = vec![resolve_relative_cmd(c, base)];
        v.extend(args);
        v
    } else {
        vec![template.as_ref().unwrap()[0].clone()]
    };
    if let Some(t) = &template {
        if t.is_empty() {
            return Err(format!("[{lang}]: template must not be empty"));
        }
        argv = vec![t[0].clone()];
    }
    Ok(LangSpec {
        lang: canon,
        argv,
        template,
        ext,
        env,
        version_args,
        session,
        base_dir: base.map(|p| p.to_path_buf()),
    })
}

fn resolve_relative_cmd(cmd: &str, base: Option<&Path>) -> String {
    let p = Path::new(cmd);
    if p.is_absolute() || !cmd.contains('/') && !cmd.contains('\\') {
        return cmd.to_string();
    }
    let base = base.unwrap_or_else(|| Path::new("."));
    std::path::absolute(base.join(p))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| cmd.to_string())
}

/// Explicit config load; returns the number of language tables read.
pub fn load_config(ctx: &mut HostContext, path: Option<&Value>) -> Result<Value, String> {
    let explicit = opt_text(path)?;
    if let Some(p) = explicit {
        let p = PathBuf::from(p);
        if !p.is_file() {
            return Err(format!("foreign config not found: {}", p.display()));
        }
        let n = load_config_path(ctx, &p)?;
        ctx.foreign_config_loaded = true;
        return Ok(Value::Int(n as i64));
    }
    ensure_config(ctx);
    Ok(Value::Int(ctx.foreign_config_specs.len() as i64))
}

pub fn run(
    ctx: &mut HostContext,
    code: &Value,
    stdin: Option<&Value>,
) -> Result<Value, String> {
    run_with_options(ctx, code, stdin, &HashMap::new())
}

/// `run` with the F0 optional parameters (`stdin` is also accepted in `bound`).
pub fn run_with_options(
    ctx: &mut HostContext,
    code: &Value,
    stdin: Option<&Value>,
    bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    let block = as_code(code)?;
    let mut job = bound_job(ctx, &block.lang, &block.source, stdin, bound)?;
    job.lang = block.lang.clone();
    let exec = execute_job(ctx, &job)?;
    outcome_text(&exec)
}

pub fn run_lang(
    ctx: &mut HostContext,
    lang: &Value,
    source: &Value,
    stdin: Option<&Value>,
) -> Result<Value, String> {
    run_lang_with_options(ctx, lang, source, stdin, &HashMap::new())
}

pub fn run_lang_with_options(
    ctx: &mut HostContext,
    lang: &Value,
    source: &Value,
    stdin: Option<&Value>,
    bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    let lang = as_text(lang, "lang")?;
    let source = as_text(source, "source")?;
    let job = bound_job(ctx, lang, source, stdin, bound)?;
    let exec = execute_job(ctx, &job)?;
    outcome_text(&exec)
}

fn bound_job(
    ctx: &HostContext,
    lang: &str,
    source: &str,
    stdin: Option<&Value>,
    bound: &HashMap<String, Value>,
) -> Result<Job, String> {
    let mut job = Job::simple(lang, source, None);
    job.stdin = opt_text(stdin)?;
    job.env = parse_env(bound.get("env"))?;
    job.cwd = opt_text(bound.get("cwd"))?;
    job.timeout = parse_timeout(bound.get("timeout"))?;
    job.extra_args = parse_args(bound.get("args"))?;
    job.max_output = parse_max_output(bound.get("max_output"))?;
    let _ = ctx;
    Ok(job)
}

/// Structured one-shot execution. Never errors for process failures; only bad args/policy.
struct PreparedInvoke {
    job: Job,
    output: String,
}

fn prepare_invoke(args: &InvokeArgs<'_>) -> Result<PreparedInvoke, String> {
    let (lang, source) = resolve_invoke_source(args)?;
    let stdin = opt_text(args.stdin)?;
    let input_json = match args.input {
        None | Some(Value::None) => None,
        Some(v) => Some(
            serde_json::to_string(&value_to_json(v)?)
                .map_err(|e| format!("input JSON encode: {e}"))?,
        ),
    };
    if stdin.is_some() && input_json.is_some() {
        return Err("stdin and input are mutually exclusive".into());
    }
    let output = match opt_text(args.output)? {
        None => "auto".to_string(),
        Some(s) => s.to_ascii_lowercase(),
    };
    if !matches!(output.as_str(), "auto" | "text" | "json" | "lines") {
        return Err(format!("unknown output mode `{output}` (auto|text|json|lines)"));
    }
    let mut job = Job::simple(&lang, &source, None);
    job.stdin = stdin.or_else(|| input_json.clone());
    job.input_json = input_json;
    job.env = parse_env(args.env)?;
    job.cwd = opt_text(args.cwd)?;
    job.timeout = parse_timeout(args.timeout)?;
    job.extra_args = parse_args(args.args)?;
    job.max_output = parse_max_output(args.max_output)?;
    job.cmd_override = opt_text(args.cmd)?;
    job.keep = parse_bool(args.keep, "keep", false)?;
    Ok(PreparedInvoke { job, output })
}

pub fn invoke(ctx: &mut HostContext, args: InvokeArgs<'_>) -> Result<Value, String> {
    ensure_config(ctx);
    let prepared = prepare_invoke(&args)?;
    let exec = execute_job(ctx, &prepared.job)?;
    let (result, result_error) = parse_output(&exec.stdout, &prepared.output);
    Ok(exec.record(result, result_error))
}

/// JSON-in / JSON-out convenience: raises on process failure or invalid JSON.
pub fn call(ctx: &mut HostContext, args: InvokeArgs<'_>) -> Result<Value, String> {
    ensure_config(ctx);
    let mut prepared = prepare_invoke(&args)?;
    prepared.output = "json".into();
    let exec = execute_job(ctx, &prepared.job)?;
    if !exec.ok {
        return Err(format_failure(&exec));
    }
    if exec.stdout.trim().is_empty() {
        return Ok(Value::None);
    }
    match serde_json::from_str::<serde_json::Value>(&exec.stdout) {
        Ok(j) => json_to_value(&j),
        Err(e) => Err(format!(
            "foreign call: stdout is not JSON: {e}; stdout: {}",
            tail(exec.stdout.trim(), 512)
        )),
    }
}

fn present(v: Option<&Value>) -> Option<&Value> {
    match v {
        None | Some(Value::None) => None,
        Some(x) => Some(x),
    }
}

#[derive(Clone)]
struct ForeignEnv {
    cwd: PathBuf,
    fs_root: Option<PathBuf>,
    entry_path: Option<PathBuf>,
    caps: HostCaps,
    foreign_cmds: HashMap<String, Vec<String>>,
    foreign_specs: HashMap<String, LangSpec>,
    foreign_config_specs: HashMap<String, LangSpec>,
    foreign_config_loaded: bool,
    sandbox_session: Option<String>,
}

impl ForeignEnv {
    fn from_ctx(ctx: &HostContext) -> Self {
        Self {
            cwd: ctx.cwd.clone(),
            fs_root: ctx.fs_root.clone(),
            entry_path: ctx.entry_path.clone(),
            caps: ctx.caps.clone(),
            foreign_cmds: ctx.foreign_cmds.clone(),
            foreign_specs: ctx.foreign_specs.clone(),
            foreign_config_specs: ctx.foreign_config_specs.clone(),
            foreign_config_loaded: ctx.foreign_config_loaded,
            sandbox_session: ctx.sandbox_session.clone(),
        }
    }

    fn to_ctx(&self) -> HostContext {
        let mut c = HostContext::default();
        c.cwd = self.cwd.clone();
        c.fs_root = self.fs_root.clone();
        c.entry_path = self.entry_path.clone();
        c.caps = self.caps.clone();
        c.foreign_cmds = self.foreign_cmds.clone();
        c.foreign_specs = self.foreign_specs.clone();
        c.foreign_config_specs = self.foreign_config_specs.clone();
        c.foreign_config_loaded = self.foreign_config_loaded;
        c.sandbox_session = self.sandbox_session.clone();
        c
    }
}

struct PreparedMany {
    idx: usize,
    job: Job,
    output: String,
}

fn parse_concurrency(v: Option<&Value>) -> Result<usize, String> {
    let n = match v {
        None | Some(Value::None) => 4,
        Some(Value::Int(n)) if *n >= 0 => *n as usize,
        Some(_) => return Err("concurrency must be a non-negative integer".into()),
    };
    Ok(n.min(32))
}

fn map_get<'a>(m: &'a Value, key: &str) -> Option<&'a Value> {
    match m {
        Value::Map(entries) => entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .filter(|v| !matches!(v, Value::None)),
        _ => None,
    }
}

fn args_from_map<'a>(m: &'a Value) -> InvokeArgs<'a> {
    InvokeArgs {
        code: map_get(m, "code"),
        lang: map_get(m, "lang"),
        source: map_get(m, "source"),
        stdin: map_get(m, "stdin"),
        input: map_get(m, "input"),
        env: map_get(m, "env"),
        cwd: map_get(m, "cwd"),
        timeout: map_get(m, "timeout"),
        args: map_get(m, "args"),
        output: map_get(m, "output"),
        max_output: map_get(m, "max_output"),
        cmd: map_get(m, "cmd"),
        keep: map_get(m, "keep"),
    }
}

fn error_record(ctx: &HostContext, job: &Job, error: String) -> Value {
    Execution {
        ok: false,
        code: -1,
        stdout: String::new(),
        stderr: String::new(),
        ms: 0,
        timed_out: false,
        truncated: false,
        error,
        lang: job.lang.clone(),
        cmd: display_for_job(ctx, job, None),
        script: String::new(),
    }
    .record(Value::None, String::new())
}

fn run_prepared(env: &ForeignEnv, items: Vec<PreparedMany>, concurrency: usize) -> Vec<(usize, Value)> {
    if items.is_empty() {
        return Vec::new();
    }
    let workers = concurrency.max(1).min(items.len());
    let queue = Arc::new(Mutex::new(VecDeque::from(items)));
    let results: Arc<Mutex<Vec<(usize, Value)>>> = Arc::new(Mutex::new(Vec::new()));
    thread::scope(|scope| {
        for _ in 0..workers {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let env = env.clone();
            scope.spawn(move || loop {
                let item = {
                    let mut q = match queue.lock() {
                        Ok(q) => q,
                        Err(_) => break,
                    };
                    q.pop_front()
                };
                let Some(item) = item else { break };
                let ctx = env.to_ctx();
                let value = match catch_unwind(AssertUnwindSafe(|| {
                    execute_job(&ctx, &item.job)
                })) {
                    Ok(Ok(exec)) => {
                        let (result, result_error) = parse_output(&exec.stdout, &item.output);
                        exec.record(result, result_error)
                    }
                    Ok(Err(e)) => error_record(&ctx, &item.job, e),
                    Err(_) => error_record(&ctx, &item.job, "foreign worker panicked".into()),
                };
                if let Ok(mut out) = results.lock() {
                    out.push((item.idx, value));
                }
            });
        }
    });
    let mut out = match Arc::try_unwrap(results) {
        Ok(m) => m.into_inner().unwrap_or_default(),
        Err(shared) => shared.lock().map(|m| m.clone()).unwrap_or_default(),
    };
    out.sort_by_key(|(i, _)| *i);
    out
}

/// Build a prepared job for one `runs` item (code value or option map).
fn prepare_many_item(ctx: &HostContext, idx: usize, item: &Value) -> Result<PreparedMany, String> {
    match item {
        Value::Code(c) => Ok(PreparedMany {
            idx,
            job: Job::simple(&c.lang, &c.source, None),
            output: "auto".into(),
        }),
        Value::Map(_) => {
            let args = args_from_map(item);
            let prepared = prepare_invoke(&args)?;
            let _ = ctx;
            Ok(PreparedMany {
                idx,
                job: prepared.job,
                output: prepared.output,
            })
        }
        _ => Err(format!(
            "runs[{idx}] must be a code value or a map of invoke options"
        )),
    }
}

pub fn run_many(
    ctx: &mut HostContext,
    runs: &Value,
    concurrency: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    if !ctx.allow_exec() {
        return Err("foreign run_many denied by host policy".into());
    }
    let list = match runs {
        Value::List(xs) => xs,
        _ => return Err("run_many runs must be a list".into()),
    };
    let mut prepared = Vec::with_capacity(list.len());
    for (idx, item) in list.iter().enumerate() {
        prepared.push(prepare_many_item(ctx, idx, item)?);
    }
    let n = prepared.len();
    let requested = parse_concurrency(concurrency)?;
    let concurrency = if requested == 0 {
        n.min(8)
    } else {
        requested
    };
    let env = ForeignEnv::from_ctx(ctx);
    let results = run_prepared(&env, prepared, concurrency);
    Ok(Value::List(results.into_iter().map(|(_, v)| v).collect()))
}

pub fn call_many(
    ctx: &mut HostContext,
    args: InvokeArgs<'_>,
    inputs: &Value,
    concurrency: Option<&Value>,
) -> Result<Value, String> {
    ensure_config(ctx);
    if !ctx.allow_exec() {
        return Err("foreign call_many denied by host policy".into());
    }
    let list = match inputs {
        Value::List(xs) => xs,
        _ => return Err("call_many inputs must be a list".into()),
    };
    let base = prepare_invoke(&args)?;
    let n = list.len();
    let requested = parse_concurrency(concurrency)?;
    let concurrency = if requested == 0 {
        n.min(8)
    } else {
        requested
    };
    let mut prepared = Vec::with_capacity(n);
    for (idx, input) in list.iter().enumerate() {
        let json = serde_json::to_string(&value_to_json(input)?)
            .map_err(|e| format!("call_many input JSON encode: {e}"))?;
        let mut job = base.job.clone();
        job.stdin = Some(json.clone());
        job.input_json = Some(json);
        prepared.push(PreparedMany {
            idx,
            job,
            output: "json".into(),
        });
    }
    let env = ForeignEnv::from_ctx(ctx);
    let records = run_prepared(&env, prepared, concurrency);
    let mut results = Vec::with_capacity(records.len());
    let mut errors = Vec::new();
    for (idx, record) in records {
        let ok = matches!(map_get(&record, "ok"), Some(Value::Bool(true)));
        let result_error = map_get(&record, "result_error")
            .and_then(|v| v.reveal_text())
            .unwrap_or("");
        if !ok {
            let err = map_get(&record, "error")
                .and_then(|v| v.reveal_text())
                .or_else(|| map_get(&record, "stderr").and_then(|v| v.reveal_text()))
                .unwrap_or("foreign call failed");
            errors.push(format!("call_many[{idx}]: {err}"));
            continue;
        }
        if !result_error.is_empty() {
            errors.push(format!("call_many[{idx}]: {result_error}"));
            continue;
        }
        results.push(map_get(&record, "result").cloned().unwrap_or(Value::None));
    }
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    Ok(Value::List(results))
}

fn resolve_invoke_source(args: &InvokeArgs<'_>) -> Result<(String, String), String> {
    if let Some(code) = present(args.code) {
        if present(args.lang).is_some() || present(args.source).is_some() {
            return Err("invoke: pass either code or lang+source, not both".into());
        }
        let block = as_code(code)?;
        return Ok((block.lang.clone(), block.source.clone()));
    }
    let lang = present(args.lang).ok_or_else(|| "invoke needs code or lang+source".to_string())?;
    let source =
        present(args.source).ok_or_else(|| "invoke needs source when lang is given".to_string())?;
    Ok((
        as_text(lang, "lang")?.to_string(),
        as_text(source, "source")?.to_string(),
    ))
}

fn parse_output(stdout: &str, mode: &str) -> (Value, String) {
    match mode {
        "text" => (Value::None, String::new()),
        "lines" => (
            Value::List(stdout.lines().map(|l| Value::Text(l.to_string())).collect()),
            String::new(),
        ),
        "json" => {
            if stdout.trim().is_empty() {
                return (Value::None, String::new());
            }
            match serde_json::from_str::<serde_json::Value>(stdout) {
                Ok(j) => match json_to_value(&j) {
                    Ok(v) => (v, String::new()),
                    Err(e) => (Value::None, e),
                },
                Err(e) => (Value::None, format!("stdout is not JSON: {e}")),
            }
        }
        _ => {
            if stdout.trim().is_empty() {
                return (Value::None, String::new());
            }
            match serde_json::from_str::<serde_json::Value>(stdout) {
                Ok(j) => (json_to_value(&j).unwrap_or(Value::None), String::new()),
                Err(_) => (Value::None, String::new()),
            }
        }
    }
}

fn outcome_text(exec: &Execution) -> Result<Value, String> {
    if !exec.ok {
        return Err(format_failure(exec));
    }
    let mut out = exec.stdout.clone();
    if out.ends_with('\n') {
        out.pop();
        if out.ends_with('\r') {
            out.pop();
        }
    }
    Ok(Value::Text(out))
}

fn format_failure(exec: &Execution) -> String {
    let mut msg = format!(
        "foreign run failed (lang={}, cmd={}, exit={})",
        exec.lang, exec.cmd, exec.code
    );
    if exec.timed_out {
        msg.push_str(": timed out");
    }
    if !exec.error.is_empty() {
        msg.push_str(": ");
        msg.push_str(&exec.error);
    }
    let err = exec.stderr.trim();
    if !err.is_empty() {
        msg.push_str(": ");
        msg.push_str(&tail(err, FAILURE_TAIL));
    } else {
        let out = exec.stdout.trim();
        if !out.is_empty() {
            msg.push_str(": stdout: ");
            msg.push_str(&tail(out, FAILURE_TAIL));
        }
    }
    msg
}

fn tail(s: &str, n: usize) -> String {
    if s.len() <= n {
        return s.to_string();
    }
    let mut start = s.len() - n;
    while start < s.len() && !s.is_char_boundary(start) {
        start += 1;
    }
    format!("…{}", &s[start..])
}

fn known_langs(ctx: &HostContext) -> Vec<String> {
    let mut names: Vec<String> = registry::preset_ids();
    names.extend(ctx.foreign_specs.keys().cloned());
    names.extend(ctx.foreign_config_specs.keys().cloned());
    names.extend(ctx.foreign_cmds.keys().map(|k| registry::canonical(k)));
    names.sort();
    names.dedup();
    names
}

pub fn langs(ctx: &mut HostContext) -> Result<Value, String> {
    ensure_config(ctx);
    Ok(Value::List(
        known_langs(ctx).into_iter().map(Value::Text).collect(),
    ))
}

/// Probe one language: path, version, availability. Never panics on missing tools.
pub fn probe(ctx: &mut HostContext, lang: Option<&Value>) -> Result<Value, String> {
    ensure_config(ctx);
    if !ctx.allow_exec() {
        return Err("foreign probe denied by host policy".into());
    }
    match opt_text(lang)? {
        Some(l) => Ok(probe_one(ctx, &l)),
        None => {
            let ids = known_langs(ctx);
            Ok(Value::List(
                ids.iter().map(|l| probe_one(ctx, l)).collect(),
            ))
        }
    }
}

fn probe_one(ctx: &HostContext, lang: &str) -> Value {
    let spec = match resolve_spec(ctx, lang) {
        Ok(s) => s,
        Err(e) => {
            return Value::Map(vec![
                ("lang".into(), Value::Text(lang.to_string())),
                ("cmd".into(), Value::Text(String::new())),
                ("path".into(), Value::Text(String::new())),
                ("ok".into(), Value::Bool(false)),
                ("version".into(), Value::Text(String::new())),
                ("error".into(), Value::Text(e)),
            ])
        }
    };
    let exe = spec
        .argv
        .first()
        .cloned()
        .or_else(|| spec.template.as_ref().and_then(|t| t.first()).cloned())
        .unwrap_or_default();
    let path = resolve_executable(&spec, &exe, &ctx.cwd);
    let path_text = path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let version_args = if spec.version_args.is_empty() {
        vec!["--version".to_string()]
    } else {
        spec.version_args.clone()
    };
    let mut display = vec![exe.clone()];
    display.extend(version_args.iter().cloned());
    let cmd_text = display.join(" ");
    let Some(exe_path) = path else {
        return Value::Map(vec![
            ("lang".into(), Value::Text(lang.to_string())),
            ("cmd".into(), Value::Text(cmd_text)),
            ("path".into(), Value::Text(String::new())),
            ("ok".into(), Value::Bool(false)),
            ("version".into(), Value::Text(String::new())),
            ("error".into(), Value::Text(format!(
                "interpreter `{exe}` not found on PATH — register / set_cmd / MARQDO_FOREIGN_*"
            ))),
        ]);
    };
    let mut command = Command::new(&exe_path);
    command.args(&version_args);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.env("MARQDO_FOREIGN_LANG", &spec.lang);
    for (k, v) in &spec.env {
        command.env(k, v);
    }
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Value::Map(vec![
                ("lang".into(), Value::Text(lang.to_string())),
                ("cmd".into(), Value::Text(cmd_text)),
                ("path".into(), Value::Text(path_text)),
                ("ok".into(), Value::Bool(false)),
                ("version".into(), Value::Text(String::new())),
                ("error".into(), Value::Text(format!("spawn {exe}: {e}"))),
            ])
        }
    };
    let out = child.stdout.take();
    let err = child.stderr.take();
    let out_h = thread::spawn(move || read_capped(out, 64 * 1024));
    let err_h = thread::spawn(move || read_capped(err, 64 * 1024));
    let start = Instant::now();
    let mut ok = false;
    let mut error = String::new();
    loop {
        match child.try_wait() {
            Ok(Some(st)) => {
                ok = st.success();
                break;
            }
            Ok(None) if start.elapsed() > Duration::from_secs(3) => {
                let _ = child.kill();
                let _ = child.wait();
                error = "version probe timed out after 3s".into();
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(15)),
            Err(e) => {
                let _ = child.kill();
                error = format!("version probe failed: {e}");
                break;
            }
        }
    }
    let (stdout, _) = out_h.join().unwrap_or_default();
    let (stderr, _) = err_h.join().unwrap_or_default();
    let version = stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string();
    Value::Map(vec![
        ("lang".into(), Value::Text(lang.to_string())),
        ("cmd".into(), Value::Text(cmd_text)),
        ("path".into(), Value::Text(path_text)),
        ("ok".into(), Value::Bool(ok && error.is_empty())),
        ("version".into(), Value::Text(version)),
        ("error".into(), Value::Text(error)),
    ])
}

fn resolve_executable(spec: &LangSpec, exe: &str, cwd: &Path) -> Option<PathBuf> {
    if exe.is_empty() {
        return None;
    }
    let looks_path = exe.contains('/') || exe.contains('\\');
    if looks_path {
        let base = spec.base_dir.as_deref().unwrap_or(cwd);
        let p = if Path::new(exe).is_absolute() {
            PathBuf::from(exe)
        } else {
            base.join(exe)
        };
        return std::path::absolute(&p).ok().filter(|p| p.is_file());
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(exe);
        if p.is_file() {
            return std::path::absolute(&p).ok().or(Some(p));
        }
        #[cfg(windows)]
        {
            for ext in ["exe", "cmd", "bat", "com"] {
                let with = dir.join(format!("{exe}.{ext}"));
                if with.is_file() {
                    return Some(with);
                }
            }
        }
    }
    None
}

/// Shared entry for CLI host and live view `/api/foreign-run`.
pub fn run_source(
    ctx: &HostContext,
    lang: &str,
    source: &str,
    stdin: Option<&str>,
) -> Result<Value, String> {
    let job = Job::simple(lang, source, stdin);
    let exec = execute_job(ctx, &job)?;
    outcome_text(&exec)
}

/// Resolve argv with optional one-shot cmd override (view command box).
pub fn resolve_argv_with_override(
    ctx: &HostContext,
    lang: &str,
    cmd_override: Option<&str>,
) -> Result<Vec<String>, String> {
    if let Some(cmd) = cmd_override.map(str::trim).filter(|s| !s.is_empty()) {
        let argv = split_cmd_line(cmd);
        if argv.is_empty() {
            return Err("cmd override is empty".into());
        }
        return Ok(argv);
    }
    Ok(resolve_spec(ctx, lang)?.argv)
}

fn env_argv_for(lang: &str, canon: &str) -> Result<Option<Vec<String>>, String> {
    let mut keys = vec![format!("MARQDO_FOREIGN_{}", lang.to_ascii_uppercase())];
    if canon != lang {
        keys.push(format!("MARQDO_FOREIGN_{}", canon.to_ascii_uppercase()));
    }
    if canon == "python" {
        keys.push("MARQDO_FOREIGN_PYTHON".to_string());
    }
    for key in keys {
        if let Ok(v) = std::env::var(&key) {
            let v = v.trim();
            if !v.is_empty() {
                let argv = parse_env_argv(v)?;
                if !argv.is_empty() {
                    return Ok(Some(argv));
                }
            }
        }
    }
    Ok(None)
}

/// Resolve the effective language spec: register > env > config > preset.
fn resolve_spec(ctx: &HostContext, lang: &str) -> Result<LangSpec, String> {
    let requested = lang.to_ascii_lowercase();
    let canon = registry::canonical(&requested);
    if let Some(s) = ctx
        .foreign_specs
        .get(&requested)
        .or_else(|| ctx.foreign_specs.get(&canon))
    {
        return Ok(s.clone());
    }
    if let Some(argv) = env_argv_for(&requested, &canon)? {
        let mut spec = registry::preset_spec(&canon).unwrap_or_else(|| LangSpec {
            lang: canon.clone(),
            ..Default::default()
        });
        spec.argv = argv;
        spec.template = None;
        spec.lang = canon.clone();
        return Ok(spec);
    }
    if let Some(s) = ctx
        .foreign_config_specs
        .get(&requested)
        .or_else(|| ctx.foreign_config_specs.get(&canon))
    {
        return Ok(s.clone());
    }
    if let Some(s) = registry::preset_spec(&canon) {
        return Ok(s);
    }
    Err(format!(
        "no interpreter configured for lang=`{lang}` — register lang={lang} cmd=… \
         or env MARQDO_FOREIGN_{} (check your environment)",
        lang.to_ascii_uppercase()
    ))
}

pub(crate) fn resolve_session_spec(
    ctx: &HostContext,
    lang: &str,
) -> Result<LangSpec, String> {
    resolve_spec(ctx, lang)
}

struct Invocation {
    argv: Vec<String>,
    template: Option<Vec<String>>,
    ext: String,
    env: Vec<(String, String)>,
    inline: bool,
    #[allow(dead_code)]
    session: bool,
}

fn default_ext(lang: &str) -> String {
    if let Some(s) = registry::preset_spec(lang) {
        if let Some(e) = s.ext {
            return e;
        }
    }
    let l = lang.to_ascii_lowercase();
    if l.len() <= 8 && l.chars().all(|c| c.is_ascii_alphanumeric()) {
        l
    } else {
        "txt".to_string()
    }
}

fn resolve_invocation(ctx: &HostContext, job: &Job) -> Result<Invocation, String> {
    if let Some(cmd) = job.cmd_override.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let argv = split_cmd_line(cmd);
        if argv.is_empty() {
            return Err("cmd override is empty".into());
        }
        return Ok(Invocation {
            argv,
            template: None,
            ext: default_ext(&job.lang),
            env: Vec::new(),
            inline: false,
            session: registry::session_supported(&job.lang),
        });
    }
    let spec = resolve_spec(ctx, &job.lang)?;
    let mut template = spec.template.clone();
    if let Some(t) = &mut template {
        if t.is_empty() {
            return Err(format!("language `{}` has an empty template", spec.lang));
        }
        let has_file = t.iter().any(|x| x == "{file}");
        let has_inline = t.iter().any(|x| x == "{inline}");
        if has_file && has_inline {
            return Err(format!(
                "language `{}` template uses both {{file}} and {{inline}}",
                spec.lang
            ));
        }
        if !has_file && !has_inline {
            t.push("{file}".into());
        }
    }
    let inline = template
        .as_ref()
        .map(|t| t.iter().any(|x| x == "{inline}"))
        .unwrap_or(false);
    Ok(Invocation {
        argv: spec.argv,
        template,
        ext: spec.ext.unwrap_or_else(|| default_ext(&job.lang)),
        env: spec.env,
        inline,
        session: spec.session,
    })
}


/// `MARQDO_FOREIGN_<LANG>` accepts a shell-like line or a JSON array of strings.
fn parse_env_argv(v: &str) -> Result<Vec<String>, String> {
    if v.starts_with('[') {
        let parsed: serde_json::Value =
            serde_json::from_str(v).map_err(|e| format!("invalid JSON argv: {e}"))?;
        let arr = parsed
            .as_array()
            .ok_or_else(|| "argv JSON must be an array".to_string())?;
        let mut out = Vec::new();
        for x in arr {
            let s = x
                .as_str()
                .ok_or_else(|| "argv JSON entries must be strings".to_string())?;
            out.push(s.to_string());
        }
        return Ok(out);
    }
    Ok(split_cmd_line(v))
}

/// Shell-like split: whitespace, single/double quotes, backslash escapes.

fn is_python(lang: &str) -> bool {
    matches!(lang, "python" | "python3" | "py")
}

fn write_temp_script(
    ctx: &HostContext,
    lang: &str,
    source: &str,
    ext: &str,
) -> Result<PathBuf, String> {
    let body = if is_python(lang) {
        format!("# -*- coding: utf-8 -*-\n{source}")
    } else {
        source.to_string()
    };
    let base = {
        let preferred = temp_base(ctx);
        match std::fs::create_dir_all(&preferred) {
            Ok(()) => preferred,
            Err(_) => {
                let fallback = std::env::temp_dir();
                std::fs::create_dir_all(&fallback).map_err(|e| {
                    format!(
                        "failed to create foreign temp dir {} or fallback {}: {e}",
                        preferred.display(),
                        fallback.display()
                    )
                })?;
                fallback
            }
        }
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
    let path = base.join(name);
    std::fs::write(&path, body.as_bytes())
        .map_err(|e| format!("failed to write temp foreign script: {e}"))?;
    Ok(path)
}

fn temp_base(ctx: &HostContext) -> PathBuf {
    let p = if let Ok(v) = std::env::var("MARQDO_FOREIGN_TMPDIR") {
        let v = v.trim();
        if !v.is_empty() {
            PathBuf::from(v)
        } else {
            ctx.cwd.join(".marqdo").join("tmp")
        }
    } else {
        ctx.cwd.join(".marqdo").join("tmp")
    };
    std::path::absolute(&p).unwrap_or(p)
}

/// Background foreign subprocess (for `lib/subtask`); caller removes `script_path` after join.
pub struct SpawnedForeign {
    pub child: Child,
    pub script_path: PathBuf,
    pub stdout: thread::JoinHandle<(String, bool)>,
    pub stderr: thread::JoinHandle<(String, bool)>,
}

pub fn spawn_source(
    ctx: &mut HostContext,
    lang: &str,
    source: &str,
    stdin: Option<&str>,
) -> Result<SpawnedForeign, String> {
    ensure_config(ctx);
    let job = Job::simple(lang, source, stdin);
    let (child, script_path, stdout, stderr) = spawn_job(ctx, &job)?;
    Ok(SpawnedForeign {
        child,
        script_path,
        stdout,
        stderr,
    })
}

fn job_cwd(ctx: &HostContext, job: &Job) -> Result<PathBuf, String> {
    let cwd = match &job.cwd {
        Some(c) => ctx.resolve_path(c)?,
        None => ctx.cwd.clone(),
    };
    if !cwd.is_dir() {
        return Err(format!("foreign cwd is not a directory: {}", cwd.display()));
    }
    Ok(cwd)
}

fn expand_template(
    ctx: &HostContext,
    job: &Job,
    template: &[String],
    script: Option<&Path>,
    cwd: &Path,
) -> Vec<String> {
    let dir = script
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| temp_base(ctx));
    let file = script
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let base = script
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut out = Vec::with_capacity(template.len());
    for item in template {
        let mut s = item.clone();
        s = s.replace("{file}", &file);
        s = s.replace("{dir}", &dir.display().to_string());
        s = s.replace("{base}", &base);
        s = s.replace("{inline}", &job.source);
        s = s.replace("{lang}", &job.lang);
        s = s.replace("{cwd}", &cwd.display().to_string());
        out.push(s);
    }
    out
}

fn build_command(
    ctx: &HostContext,
    job: &Job,
    inv: &Invocation,
    script: Option<&Path>,
) -> Result<(Command, String), String> {
    let cwd = job_cwd(ctx, job)?;
    let argv = if let Some(template) = &inv.template {
        let mut v = expand_template(ctx, job, template, script, &cwd);
        v.extend(job.extra_args.iter().cloned());
        v
    } else {
        let mut v = inv.argv.clone();
        v.extend(job.extra_args.iter().cloned());
        if let Some(script) = script {
            v.push(script.display().to_string());
        }
        v
    };
    if argv.is_empty() {
        return Err("foreign cmd argv is empty — check register / env".into());
    }
    let mut command = Command::new(&argv[0]);
    for a in &argv[1..] {
        command.arg(a);
    }
    command.current_dir(&cwd);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    command.env("MARQDO_FOREIGN", "1");
    command.env("MARQDO_FOREIGN_LANG", &job.lang);
    if let Some(script) = script {
        command.env("MARQDO_FOREIGN_SCRIPT", script);
    }
    command.env(
        "MARQDO_FOREIGN_DIR",
        script
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| temp_base(ctx)),
    );
    command.env("MARQDO_FOREIGN_CWD", &cwd);
    if let Some(path) = &ctx.entry_path {
        command.env("MARQDO_ENTRY", path);
        if let Some(dir) = path.parent() {
            command.env("MARQDO_ENTRY_DIR", dir);
        }
    }
    if let Some(input) = &job.input_json {
        if input.len() <= INPUT_ENV_CAP {
            command.env("MARQDO_INPUT", input);
        }
    }
    for (k, v) in &inv.env {
        command.env(k, v);
    }
    for (k, v) in &job.env {
        command.env(k, v);
    }
    if is_python(&job.lang) {
        command.env("PYTHONUNBUFFERED", "1");
        command.env("PYTHONIOENCODING", "utf-8");
        command.env("PYTHONUTF8", "1");
        let sep = if cfg!(windows) { ";" } else { ":" };
        let mut paths = vec![cwd.to_string_lossy().into_owned()];
        if let Ok(existing) = std::env::var("PYTHONPATH") {
            if !existing.is_empty() {
                paths.push(existing);
            }
        }
        command.env("PYTHONPATH", paths.join(sep));
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    Ok((command, argv.join(" ")))
}

fn spawn_job(
    ctx: &HostContext,
    job: &Job,
) -> Result<
    (
        Child,
        PathBuf,
        thread::JoinHandle<(String, bool)>,
        thread::JoinHandle<(String, bool)>,
    ),
    String,
> {
    let inv = resolve_invocation(ctx, job)?;
    let script = if inv.inline {
        None
    } else {
        Some(write_temp_script(ctx, &job.lang, &job.source, &inv.ext)?)
    };
    let (mut command, display) = match build_command(ctx, job, &inv, script.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            if let Some(p) = &script {
                let _ = std::fs::remove_file(p);
            }
            return Err(e);
        }
    };
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            if let Some(p) = &script {
                let _ = std::fs::remove_file(p);
            }
            return Err(format!(
                "foreign spawn failed (lang={}, cmd={}): {e} — check your interpreter / \
                 MARQDO_FOREIGN_* / register",
                job.lang, display
            ));
        }
    };

    let mut stdin_pipe = child.stdin.take();
    let stdin_data = job.stdin.clone();
    if let Some(data) = stdin_data {
        thread::spawn(move || {
            if let Some(mut pipe) = stdin_pipe.take() {
                let _ = pipe.write_all(data.as_bytes());
            }
        });
    } else {
        drop(stdin_pipe.take());
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let max = job.max_output;
    let out_h = thread::spawn(move || read_capped(stdout, max));
    let err_h = thread::spawn(move || read_capped(stderr, max));
    Ok((child, script.unwrap_or_default(), out_h, err_h))
}

fn display_for_job(ctx: &HostContext, job: &Job, script: Option<&Path>) -> String {
    match resolve_invocation(ctx, job) {
        Ok(inv) => match build_command(ctx, job, &inv, script) {
            Ok((_, d)) => d,
            Err(_) => inv.argv.join(" "),
        },
        Err(_) => job.lang.clone(),
    }
}

fn read_capped<R: Read>(reader: Option<R>, max: usize) -> (String, bool) {
    let mut kept: Vec<u8> = Vec::new();
    let mut truncated = false;
    if let Some(mut r) = reader {
        let mut buf = [0u8; 8192];
        loop {
            match r.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if kept.len() < max {
                        let take = (max - kept.len()).min(n);
                        kept.extend_from_slice(&buf[..take]);
                        if take < n {
                            truncated = true;
                        }
                    } else {
                        truncated = true;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    }
    (String::from_utf8_lossy(&kept).into_owned(), truncated)
}

fn kill_tree(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .output();
    }
    let _ = child.kill();
}

fn execute_job(ctx: &HostContext, job: &Job) -> Result<Execution, String> {
    if !ctx.allow_exec() {
        if ctx.sandbox_session.is_some() {
            return Err("foreign unavailable in sandbox".into());
        }
        return Err("foreign denied by host policy".into());
    }
    let (mut child, script_path, out_h, err_h) = match spawn_job(ctx, job) {
        Ok(v) => v,
        Err(error) => {
            return Ok(Execution {
                ok: false,
                code: -1,
                stdout: String::new(),
                stderr: String::new(),
                ms: 0,
                timed_out: false,
                truncated: false,
                error,
                lang: job.lang.clone(),
                cmd: display_for_job(ctx, job, None),
                script: String::new(),
            });
        }
    };
    let start = Instant::now();
    let mut timed_out = false;
    let mut wait_error: Option<String> = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {
                if start.elapsed() >= job.timeout {
                    timed_out = true;
                    kill_tree(&mut child);
                    let _ = child.wait();
                    break None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                wait_error = Some(format!("foreign wait failed: {e}"));
                kill_tree(&mut child);
                let _ = child.wait();
                break None;
            }
        }
    };
    let (stdout, out_trunc) = out_h.join().unwrap_or_default();
    let (stderr, err_trunc) = err_h.join().unwrap_or_default();
    let ms = start.elapsed().as_millis() as u64;

    let script_is_empty = script_path.as_os_str().is_empty();
    let script_keep = if job.keep && !script_is_empty {
        script_path.display().to_string()
    } else {
        if !script_is_empty {
            let _ = std::fs::remove_file(&script_path);
        }
        String::new()
    };
    let code = status.and_then(|s| s.code()).unwrap_or(-1);
    let ok = status.map(|s| s.success()).unwrap_or(false) && !timed_out && wait_error.is_none();
    let error = if let Some(e) = wait_error {
        e
    } else if timed_out {
        format!("timed out after {:.3}s", job.timeout.as_secs_f64())
    } else {
        String::new()
    };
    let cmd = display_for_job(
        ctx,
        job,
        if script_is_empty {
            None
        } else {
            Some(script_path.as_path())
        },
    );
    Ok(Execution {
        ok,
        code,
        stdout,
        stderr,
        ms,
        timed_out,
        truncated: out_trunc || err_trunc,
        error,
        lang: job.lang.clone(),
        cmd,
        script: script_keep,
    })
}

/// View helper: execute one fence with the command-box override + full record.
pub fn invoke_for_view(
    root: &Path,
    lang: &str,
    source: &str,
    cmd_override: Option<&str>,
    stdin: Option<&str>,
    timeout_secs: Option<f64>,
) -> Result<Value, String> {
    let mut ctx = HostContext::default();
    ctx.cwd = root.to_path_buf();
    ctx.fs_root = Some(root.to_path_buf());
    ensure_config(&mut ctx);
    let timeout = match timeout_secs {
        Some(t) if t > 0.0 => Duration::from_secs_f64(t.min(MAX_TIMEOUT_SECS)),
        _ => Duration::from_secs_f64(DEFAULT_TIMEOUT_SECS),
    };
    let job = Job {
        lang: lang.to_string(),
        source: source.to_string(),
        stdin: stdin.map(str::to_string),
        input_json: None,
        env: Vec::new(),
        cwd: None,
        timeout,
        extra_args: Vec::new(),
        max_output: MAX_OUTPUT,
        keep: false,
        cmd_override: cmd_override.map(str::to_string),
    };
    let exec = execute_job(&ctx, &job)?;
    let (result, result_error) = parse_output(&exec.stdout, "auto");
    Ok(exec.record(result, result_error))
}

#[cfg(test)]
mod tests {
    use crate::foreign;
    use super::*;

    #[test]
    fn fence_lang_python() {
        assert_eq!(foreign::fence_lang("```python").as_deref(), Some("python"));
        assert_eq!(
            foreign::fence_lang("```python name=x").as_deref(),
            Some("python")
        );
        assert!(foreign::fence_lang("```").is_none());
    }

    #[test]
    fn split_quotes() {
        assert_eq!(
            split_cmd_line("python3 -u -c 'a b'"),
            vec!["python3", "-u", "-c", "a b"]
        );
        assert_eq!(
            split_cmd_line(r#"node -e "console.log(\"x\")""#),
            vec!["node", "-e", "console.log(\"x\")"]
        );
        assert_eq!(
            split_cmd_line(r#"/opt/my python/bin/python -u"#),
            vec!["/opt/my", "python/bin/python", "-u"]
        );
        assert_eq!(
            split_cmd_line(r#""/opt/my python/bin/python" -u"#),
            vec!["/opt/my python/bin/python", "-u"]
        );
    }

    #[test]
    fn env_argv_json() {
        assert_eq!(
            parse_env_argv(r#"["python3","-u"]"#).unwrap(),
            vec!["python3", "-u"]
        );
    }

    #[test]
    fn output_modes() {
        let (v, e) = parse_output("42", "auto");
        assert!(e.is_empty());
        assert_eq!(v, Value::Int(42));
        let (v, e) = parse_output("a\nb\n", "lines");
        assert!(e.is_empty());
        assert_eq!(v, Value::List(vec![Value::Text("a".into()), Value::Text("b".into())]));
        let (v, e) = parse_output("not json", "json");
        assert!(v == Value::None);
        assert!(!e.is_empty());
    }

    #[test]
    fn exec_cap_denies() {
        let mut ctx = HostContext::default();
        ctx.caps.exec = false;
        let job = Job::simple("node", "console.log(1)", None);
        let err = execute_job(&ctx, &job).unwrap_err();
        assert!(err.contains("denied"), "{err}");
    }

    #[test]
    fn missing_interpreter_is_record() {
        let mut ctx = HostContext::default();
        ctx.foreign_cmds.insert(
            "nope".into(),
            vec!["/definitely/not/a/real/interpreter".into()],
        );
        let job = Job::simple("nope", "x", None);
        let exec = execute_job(&ctx, &job).unwrap();
        assert!(!exec.ok);
        assert_eq!(exec.code, -1);
        assert!(!exec.error.is_empty(), "spawn failure should be recorded");
    }

    #[test]
    fn read_cap_marks_truncated() {
        let data = b"0123456789".to_vec();
        let (text, truncated) = read_capped(Some(std::io::Cursor::new(data)), 4);
        assert_eq!(text, "0123");
        assert!(truncated);
    }

    fn node_available() -> bool {
        Command::new("node")
            .args(["-e", "process.exit(0)"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn run_many_keeps_order_and_isolates_failures() {
        if !node_available() {
            return;
        }
        let mut ctx = HostContext::default();
        let runs = Value::List(vec![
            Value::Code(CodeBlock {
                lang: "node".into(),
                source: "console.log('a')".into(),
            }),
            Value::Code(CodeBlock {
                lang: "node".into(),
                source: "process.exit(2)".into(),
            }),
            Value::Code(CodeBlock {
                lang: "node".into(),
                source: "console.log('c')".into(),
            }),
        ]);
        let out = run_many(&mut ctx, &runs, Some(&Value::Int(2))).unwrap();
        let Value::List(records) = out else {
            panic!("expected list");
        };
        assert_eq!(records.len(), 3);
        assert_eq!(map_get(&records[0], "stdout").and_then(|v| v.reveal_text()), Some("a\n"));
        assert_eq!(map_get(&records[1], "ok"), Some(&Value::Bool(false)));
        assert_eq!(map_get(&records[2], "stdout").and_then(|v| v.reveal_text()), Some("c\n"));
    }
}

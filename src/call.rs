//! `marqdo call`: run one already-bound function in a `.mq.md` file.
//!
//! Location, effects, and execution go through the existing compiler and runtime.
//! The command does not scan headings, and it does not run the program entry when
//! the named function cannot be compiled.

use std::path::Path;

use anyhow::Result;

use crate::ast::Function;
use crate::bytecode::{compile_for_call, Vm};
use crate::diagnostics::{Diagnostic, Span};
use crate::host::json::json_to_value;
use crate::host::{HostCaps, HostContext};
use crate::interp::Interpreter;
use crate::load::load_module;
use crate::sema::{compile_file_closure, SemanticArtifact, SemanticProject};
use crate::value::Value;
use crate::Backend;

/// Explicit effect grants. Writes imply reads; unrelated effects stay denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CallEffects(u32);

impl CallEffects {
    #[allow(non_upper_case_globals)]
    pub const None: Self = Self(0);
    #[allow(non_upper_case_globals)]
    pub const Plugin: Self = Self((1 << 2) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 11));
    const NAMES: &'static [&'static str] = &[
        "fs.read",
        "fs.write",
        "plugin.call",
        "network.read",
        "network.write",
        "process.exec",
        "output.write",
        "time.read",
        "random.read",
        "environment.read",
        "environment.write",
        "process.control",
        "user.input",
    ];
    pub fn parse(s: &str) -> Result<Self> {
        let mut out = Self::None;
        for name in s.split(',').map(str::trim) {
            match name {
                "none" | "" => {}
                "plugin" => out.0 |= Self::Plugin.0,
                "read" => out.0 |= 1,
                "write" => out.0 |= 3,
                _ => {
                    let index = Self::NAMES.iter().position(|v| *v == name)
                        .ok_or_else(|| call_diag(None, 0, "call.bad_effects",
                            format!("unknown effect `{name}`; use none, plugin or comma-separated effect names")))?;
                    out.0 |= 1 << index;
                }
            }
        }
        if out.0 & (1 << 1) != 0 {
            out.0 |= 1;
        }
        if out.0 & (1 << 4) != 0 {
            out.0 |= 1 << 3;
        }
        Ok(out)
    }
    pub fn as_str(self) -> String {
        if self == Self::None {
            return "none".into();
        }
        if self == Self::Plugin {
            return "plugin".into();
        }
        Self::NAMES
            .iter()
            .enumerate()
            .filter(|(i, _)| self.0 & (1 << i) != 0)
            .map(|(_, name)| *name)
            .collect::<Vec<_>>()
            .join(",")
    }
    /// File/network/process writes must never be repeated to rebuild a handle.
    pub(crate) fn needs_snapshot(self) -> bool {
        [
            "fs.write",
            "network.write",
            "process.exec",
            "environment.write",
        ]
        .iter()
        .any(|effect| self.allows(effect))
    }
    fn host_caps(self) -> HostCaps {
        HostCaps {
            fs_write: self.allows("fs.write"),
            exec: self.allows("process.exec"),
            net: self.allows("network.read") || self.allows("network.write"),
            plugin: self.allows("plugin.call"),
        }
    }
    fn allows(self, effect: &str) -> bool {
        effect == "pure"
            || Self::NAMES
                .iter()
                .position(|v| *v == effect)
                .is_some_and(|i| self.0 & (1 << i) != 0)
    }
}

/// Options for one external call.
#[derive(Debug, Clone, Default)]
pub struct CallOptions {
    pub effects: CallEffects,
    pub fs_root: Option<std::path::PathBuf>,
    pub session: Option<String>,
    pub toget: Option<String>,
}

/// Call `function` in `path` with named arguments (pure-only, no session).
pub fn call_file(
    path: &Path,
    function: &str,
    args: &[(String, Value)],
    backend: Backend,
) -> Result<Value> {
    call_file_with(path, function, args, backend, CallOptions::default())
}

/// Call with effects / session / toget.
pub fn call_file_with(
    path: &Path,
    function: &str,
    args: &[(String, Value)],
    backend: Backend,
    opts: CallOptions,
) -> Result<Value> {
    let path = path.canonicalize().map_err(|e| {
        call_diag(
            None,
            1,
            "call.compile",
            format!("failed to read {}: {e}", path.display()),
        )
    })?;

    let mut session = if opts.session.is_some() || opts.toget.is_some() || args_need_session(args) {
        let id = opts
            .session
            .clone()
            .unwrap_or_else(crate::call_session::default_session_id);
        Some(crate::call_session::load_session(&id)?)
    } else {
        None
    };

    let resolved = if let Some(ref mut sess) = session {
        crate::call_session::resolve_args(sess, args, backend)?
    } else {
        args.to_vec()
    };

    // Record uses original arg texts (with $refs) so replay can re-resolve.
    let record_args = args.to_vec();

    let value = call_file_raw_with_root(
        &path,
        function,
        &resolved,
        opts.effects,
        backend,
        opts.fs_root.as_deref(),
    )?;

    if let Some(ref mut sess) = session {
        crate::call_session::record_step(
            sess,
            &path,
            function,
            &record_args,
            opts.effects,
            opts.fs_root.as_deref(),
            opts.toget.as_deref(),
            &value,
        )?;
        crate::call_session::save_session(sess)?;
    }

    Ok(value)
}

/// Execute without session bookkeeping (used by journal replay).
pub fn call_file_raw(
    path: &Path,
    function: &str,
    args: &[(String, Value)],
    effects: CallEffects,
    backend: Backend,
) -> Result<Value> {
    call_file_raw_with_root(path, function, args, effects, backend, None)
}

pub(crate) fn call_file_raw_with_root(
    path: &Path,
    function: &str,
    args: &[(String, Value)],
    effects: CallEffects,
    backend: Backend,
    fs_root: Option<&Path>,
) -> Result<Value> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        path.canonicalize().map_err(|e| {
            call_diag(
                None,
                1,
                "call.compile",
                format!("failed to read {}: {e}", path.display()),
            )
        })?
    };
    let compilation = compile_file_closure(&path).map_err(|e| preserve_diag(&path, e))?;
    let artifact = locate_artifact(&compilation.semantics, &path, function)?;
    deny_unsafe(&path, artifact, effects)?;
    execute_source_function_with_root(
        &path,
        artifact.source.line,
        &artifact.display_name,
        args,
        effects.host_caps(),
        backend,
        true,
        fs_root,
    )
    .map(|(value, _stdout)| value)
}

fn args_need_session(args: &[(String, Value)]) -> bool {
    args.iter()
        .any(|(_, v)| matches!(v, Value::Text(t) if t.starts_with('$') || t.starts_with('@')))
}

/// Run the function declared at `line` in `path`.
pub(crate) fn execute_source_function(
    path: &Path,
    line: u32,
    display_name: &str,
    args: &[(String, Value)],
    caps: HostCaps,
    backend: Backend,
    strict_args: bool,
) -> Result<(Value, String)> {
    execute_source_function_with_root(
        path,
        line,
        display_name,
        args,
        caps,
        backend,
        strict_args,
        None,
    )
}

fn execute_source_function_with_root(
    path: &Path,
    line: u32,
    display_name: &str,
    args: &[(String, Value)],
    caps: HostCaps,
    backend: Backend,
    strict_args: bool,
    fs_root: Option<&Path>,
) -> Result<(Value, String)> {
    let module = load_module(path)?;
    let fun = find_function_at_line(&module.functions, line).ok_or_else(|| {
        call_diag(
            Some(path),
            line,
            "artifact.unresolved",
            format!("无法在 {}:{} 定位函数 {display_name}", path.display(), line),
        )
    })?;
    let expected = display_name.rsplit('.').next().unwrap_or(display_name);
    if fun.name != expected {
        return Err(call_diag(
            Some(path),
            line,
            "artifact.unresolved",
            format!(
                "源码行 {line} 上的函数是 `{}`，与 `{display_name}` 不一致",
                fun.name
            ),
        ));
    }
    let mut host = HostContext::for_capture(Some(path), caps);
    if let Some(root) = fs_root {
        let root = root.canonicalize()?;
        if !root.is_dir() {
            anyhow::bail!("call.fs_root: root must be a directory");
        }
        host.cwd = root.clone();
        host.fs_root = Some(root);
    }
    let mut interp = Interpreter::with_capture(Some(path), false).with_host(host);
    let bound;
    let invoke_args: &[(String, Value)] = if strict_args {
        bound = interp.bind_external_args(&module, fun, args)?;
        &bound
    } else {
        args
    };
    match backend {
        Backend::Tree => {
            let value = interp.invoke_bound_function(&module, fun, invoke_args)?;
            Ok((value, interp.captured_stdout))
        }
        Backend::Bytecode => {
            let program = compile_for_call(Some(path), &module)?;
            let idx = program
                .functions
                .iter()
                .position(|chunk| {
                    chunk.module_scope == 0
                        && chunk.declaration_span == fun.span
                        && chunk.name == fun.name
                        && chunk.source.sha256 == module.source.sha256
                })
                .ok_or_else(|| {
                    call_diag(
                        Some(path),
                        line,
                        "artifact.unresolved",
                        format!("bytecode 没有编译出 `{display_name}`；拒绝改去执行程序入口"),
                    )
                })?;
            let mut host = HostContext::for_capture(Some(path), caps);
            if let Some(root) = fs_root {
                let root = root.canonicalize()?;
                host.cwd = root.clone();
                host.fs_root = Some(root);
            }
            let mut vm = Vm::with_capture(Some(path)).with_host(host);
            let value = vm.call_function(&program, idx, invoke_args)?;
            Ok((value, vm.captured_stdout))
        }
    }
}

/// Split trailing `toget <name>` from a clap `args` vector.
pub fn split_toget(tokens: &[String]) -> Result<(Vec<String>, Option<String>)> {
    if tokens.len() >= 2 {
        let n = tokens.len();
        if tokens[n - 2] == "toget" {
            let name = tokens[n - 1].clone();
            if name.contains('=') {
                return Err(call_diag(
                    None,
                    0,
                    "call.bad_toget",
                    "toget 名字不能含 `=`".into(),
                ));
            }
            return Ok((tokens[..n - 2].to_vec(), Some(name)));
        }
    }
    Ok((tokens.to_vec(), None))
}

/// Parse `name=value` tokens. Values are integers, decimals, quoted text, True/False/None,
/// or JSON when the raw value starts with `[` or `{`.
pub fn parse_call_args(tokens: &[String]) -> Result<Vec<(String, Value)>> {
    let mut out = Vec::new();
    for token in tokens {
        let Some((name, raw)) = token.split_once('=') else {
            return Err(call_diag(
                None,
                0,
                "call.bad_arg",
                format!("参数 `{token}` 需要写成 名字=值"),
            ));
        };
        if name.is_empty() {
            return Err(call_diag(
                None,
                0,
                "call.bad_arg",
                format!("参数 `{token}` 缺少名字"),
            ));
        }
        let value = parse_literal(raw).map_err(|message| {
            call_diag(
                None,
                0,
                "call.bad_value",
                format!("参数 `{name}`: {message}"),
            )
        })?;
        out.push((name.to_string(), value));
    }
    Ok(out)
}

fn locate_artifact<'a>(
    project: &'a SemanticProject,
    entry: &Path,
    query: &str,
) -> Result<&'a SemanticArtifact> {
    let in_entry: Vec<&SemanticArtifact> = project
        .artifacts
        .iter()
        .filter(|artifact| same_file(&project.root, &artifact.source.file, entry))
        .collect();
    let exact: Vec<&SemanticArtifact> = in_entry
        .iter()
        .copied()
        .filter(|artifact| {
            artifact.display_name == query
                || artifact
                    .symbol_id
                    .as_ref()
                    .is_some_and(|id| id.as_str() == query)
                || artifact.artifact_id.0 == query
        })
        .collect();
    let hits = if !exact.is_empty() {
        exact
    } else if !query.contains('.') {
        in_entry
            .iter()
            .copied()
            .filter(|artifact| {
                artifact.kind == "function"
                    && artifact.display_name.rsplit('.').next() == Some(query)
            })
            .collect()
    } else {
        // Nested sub-tools: `outer.helper` matches display path `main.outer.helper`.
        let suffix = format!(".{query}");
        in_entry
            .iter()
            .copied()
            .filter(|artifact| {
                artifact.kind == "function"
                    && (artifact.display_name.ends_with(&suffix) || artifact.display_name == query)
            })
            .collect()
    };
    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err(call_diag(
            Some(entry),
            1,
            "artifact.unresolved",
            format!("无法唯一绑定 `{query}`"),
        )),
        _ => Err(call_diag(
            Some(entry),
            1,
            "artifact.ambiguous",
            format!("无法唯一绑定 `{query}`"),
        )),
    }
}

fn deny_unsafe(path: &Path, artifact: &SemanticArtifact, effects: CallEffects) -> Result<()> {
    if artifact.kind != "function" || artifact.callable.as_deref() == Some("instance_method") {
        return Err(call_diag(
            Some(path),
            artifact.source.line,
            "artifact.not_executable",
            "只有非实例函数可由 call 直接执行".to_string(),
        ));
    }
    let denied: Vec<String> = artifact
        .effects
        .values()
        .into_iter()
        .filter(|e| !effects.allows(e))
        .collect();
    if !denied.is_empty() {
        return Err(call_diag(
            Some(path),
            artifact.source.line,
            "artifact.effect_denied",
            format!(
                "call --effects {} 拒绝效应 {}",
                effects.as_str(),
                denied.join(", ")
            ),
        ));
    }
    Ok(())
}

fn same_file(root: &Path, rel: &str, entry: &Path) -> bool {
    let abs = root.join(rel);
    match (abs.canonicalize(), entry.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn find_function_at_line(functions: &[Function], line: u32) -> Option<&Function> {
    for fun in functions {
        if fun.span.line == line {
            return Some(fun);
        }
        if let Some(found) = find_function_at_line(&fun.children, line) {
            return Some(found);
        }
    }
    None
}

fn parse_literal(raw: &str) -> Result<Value, String> {
    match raw {
        "True" | "true" => return Ok(Value::Bool(true)),
        "False" | "false" => return Ok(Value::Bool(false)),
        "None" | "none" => return Ok(Value::None),
        _ => {}
    }
    if raw.starts_with('$') || raw.starts_with('@') {
        return Ok(Value::Text(raw.to_string()));
    }
    if raw.len() >= 2 {
        let bytes = raw.as_bytes();
        if (bytes[0] == b'"' && bytes[raw.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[raw.len() - 1] == b'\'')
        {
            return Ok(Value::Text(raw[1..raw.len() - 1].to_string()));
        }
    }
    if raw.starts_with('[') || raw.starts_with('{') {
        let json: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| format!("JSON 解析失败: {e}"))?;
        return json_to_value(&json);
    }
    if let Ok(n) = raw.parse::<i64>() {
        return Ok(Value::Int(n));
    }
    if raw.contains('.') {
        if let Ok(n) = raw.parse::<f64>() {
            return Ok(Value::Num(n));
        }
    }
    Err(format!(
        "无法解析 `{raw}`（整数、小数、引号文本、True/False/None、JSON、`$name`）"
    ))
}

fn preserve_diag(path: &Path, err: anyhow::Error) -> anyhow::Error {
    if Diagnostic::find(&err).is_some() {
        err
    } else {
        call_diag(Some(path), 1, "call.compile", format!("{err:#}"))
    }
}

fn call_diag(path: Option<&Path>, line: u32, code: &str, message: String) -> anyhow::Error {
    anyhow::Error::new(Diagnostic::new(path, Span::new(line, 1), message).with_code(code))
}

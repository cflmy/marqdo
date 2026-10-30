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
use crate::host::{HostCaps, HostContext};
use crate::interp::Interpreter;
use crate::load::load_module;
use crate::sema::{compile_file_closure, SemanticArtifact, SemanticProject};
use crate::value::Value;
use crate::Backend;

/// Call `function` in `path` with named arguments.
///
/// Non-pure effects are refused before the function runs. Host write, exec, net,
/// and plugin capabilities stay off for the call.
pub fn call_file(
    path: &Path,
    function: &str,
    args: &[(String, Value)],
    backend: Backend,
) -> Result<Value> {
    let path = path
        .canonicalize()
        .map_err(|e| call_diag(None, 1, "call.compile", format!("failed to read {}: {e}", path.display())))?;
    let compilation = compile_file_closure(&path).map_err(|e| preserve_diag(&path, e))?;
    let artifact = locate_artifact(&compilation.semantics, &path, function)?;
    deny_unsafe(&path, artifact)?;
    execute_source_function(
        &path,
        artifact.source.line,
        &artifact.display_name,
        args,
        tight_caps(),
        backend,
        true,
    )
    .map(|(value, _stdout)| value)
}

/// Run the function declared at `line` in `path`.
///
/// `display_name` is the declaration path (`计算器.加法`). The query the caller
/// typed is not used as a heading search and is not used as the bytecode path,
/// so a unique short name or SymbolId still executes that function.
/// `caps` are the host capabilities for this run. `call` passes them all off.
pub(crate) fn execute_source_function(
    path: &Path,
    line: u32,
    display_name: &str,
    args: &[(String, Value)],
    caps: HostCaps,
    backend: Backend,
    strict_args: bool,
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
    let expected = display_name
        .rsplit('.')
        .next()
        .unwrap_or(display_name);
    if fun.name != expected {
        return Err(call_diag(
            Some(path),
            line,
            "artifact.unresolved",
            format!("源码行 {line} 上的函数是 `{}`，与 `{display_name}` 不一致", fun.name),
        ));
    }
    let mut interp = Interpreter::with_capture(Some(path), false)
        .with_host(HostContext::for_capture(Some(path), caps));
    // `call` checks the signature. `artifact.execute` keeps its previous
    // environment injection so an input the body reads is not rejected as unknown.
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
            let key: Vec<String> = display_name
                .split('.')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let idx = program.path_index.get(&key).copied().ok_or_else(|| {
                call_diag(
                    Some(path),
                    line,
                    "artifact.unresolved",
                    format!("bytecode 没有编译出 `{display_name}`；拒绝改去执行程序入口"),
                )
            })?;
            let mut vm = Vm::with_capture(Some(path))
                .with_host(HostContext::for_capture(Some(path), caps));
            let value = vm.call_function(&program, idx, invoke_args)?;
            Ok((value, vm.captured_stdout))
        }
    }
}

/// Parse `name=value` tokens. Values are integers, decimals, quoted text, or True/False/None.
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
            call_diag(None, 0, "call.bad_value", format!("参数 `{name}`: {message}"))
        })?;
        out.push((name.to_string(), value));
    }
    Ok(out)
}

fn tight_caps() -> HostCaps {
    HostCaps {
        fs_write: false,
        exec: false,
        net: false,
        plugin: false,
    }
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
        Vec::new()
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

fn deny_unsafe(path: &Path, artifact: &SemanticArtifact) -> Result<()> {
    if artifact.kind != "function" || artifact.callable.as_deref() == Some("instance_method") {
        return Err(call_diag(
            Some(path),
            artifact.source.line,
            "artifact.not_executable",
            "只有非实例函数可由 call 直接执行".to_string(),
        ));
    }
    if !artifact.effects.is_pure() {
        return Err(call_diag(
            Some(path),
            artifact.source.line,
            "artifact.effect_denied",
            format!(
                "call 默认只执行纯函数，拒绝效应 {}",
                artifact.effects.values().join(", ")
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
    if raw.len() >= 2 {
        let bytes = raw.as_bytes();
        if (bytes[0] == b'"' && bytes[raw.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[raw.len() - 1] == b'\'')
        {
            return Ok(Value::Text(raw[1..raw.len() - 1].to_string()));
        }
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
        "无法解析 `{raw}`（整数、小数、引号文本、True/False/None）"
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

//! Marqdo reference interpreter (Phase I + M5 bytecode prototype).
//!
//! Pipeline: load → line classify → parse → tree-walk eval | bytecode VM.
//! CLI also offers `view` for AST-backed browsing.

pub mod aliases;
pub mod ast;
pub mod builtin;
pub mod bytecode;
pub mod capture;
pub mod catalog;
pub mod debug;
pub mod diagnostics;
pub mod foreign;
pub mod formula;
pub mod host;
pub mod input_feed;
pub mod interp;
pub mod lex;
pub mod load;
pub mod parse;
pub mod value;
pub mod view;

use std::path::Path;

use anyhow::{bail, Result};

use crate::host::{flush_auto_plots, HostCaps, HostContext};
use crate::ast::format_ast_dump;
use crate::bytecode::{compile_module, Vm};
use crate::capture::RunCapture;
use crate::interp::Interpreter;
use crate::lex::{classify_source, format_lines_dump};
use crate::load::load_module;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    #[default]
    Tree,
    Bytecode,
}

impl Backend {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "tree" => Ok(Self::Tree),
            "bytecode" | "bc" => Ok(Self::Bytecode),
            other => bail!("unknown backend `{other}` (expected tree|bytecode)"),
        }
    }
}

/// Options for a single `run` / dump invocation.
#[derive(Debug, Clone)]
pub struct RunOptions {
    pub dump_lines: bool,
    pub dump_tokens: bool,
    pub dump_ast: bool,
    pub dump_sema: bool,
    pub dump_bytecode: bool,
    pub trace_eval: bool,
    pub backend: Backend,
    /// Preset lines consumed by `input` (from `--stdin-file` or view).
    pub stdin_lines: Vec<String>,
    pub allow_fs_write: bool,
    pub allow_exec: bool,
    pub allow_net: bool,
    /// Exposed via `host_args` / `args`.
    pub argv: Vec<String>,
    /// Override filesystem sandbox root (default: source file's directory).
    pub fs_root: Option<std::path::PathBuf>,
    /// Sleep clamp when capturing (None = host default; view export uses `Some(0)`).
    pub sleep_limit_ms: Option<u64>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            dump_lines: false,
            dump_tokens: false,
            dump_ast: false,
            dump_sema: false,
            dump_bytecode: false,
            trace_eval: false,
            backend: Backend::default(),
            stdin_lines: Vec::new(),
            allow_fs_write: true,
            allow_exec: true,
            allow_net: true,
            argv: Vec::new(),
            fs_root: None,
            sleep_limit_ms: None,
        }
    }
}

impl RunOptions {
    pub fn dump_all() -> Self {
        Self {
            dump_lines: true,
            dump_tokens: true,
            dump_ast: true,
            dump_sema: true,
            dump_bytecode: true,
            trace_eval: true,
            ..Self::default()
        }
    }

    pub fn any_dump(&self) -> bool {
        self.dump_lines
            || self.dump_tokens
            || self.dump_ast
            || self.dump_sema
            || self.dump_bytecode
            || self.trace_eval
    }

    pub fn host_caps(&self) -> HostCaps {
        HostCaps {
            fs_write: self.allow_fs_write,
            exec: self.allow_exec,
            net: self.allow_net,
            plugin: true,
        }
    }
}

/// Run a `.mq.md` program (prints to real stdout).
pub fn run_file(path: &Path, opts: &RunOptions) -> Result<i32> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;

    if source.trim().is_empty() {
        bail!("{} is empty", path.display());
    }

    let stdin_lines = crate::input_feed::effective_stdin(&source, &opts.stdin_lines);
    let path_label = path.display().to_string();

    if opts.dump_lines {
        let lines = classify_source(&source);
        print!("{}", format_lines_dump(&path_label, &lines));
    }
    if opts.dump_tokens {
        println!("=== marqdo: tokens ({path_label}) ===");
        println!("(fine-grained tokens not implemented yet — use --dump-lines)");
        println!("=== marqdo: end tokens ===");
    }

    let module = load_module(path)?;

    if opts.dump_ast {
        print!("{}", format_ast_dump(&path_label, &module));
    }
    if opts.dump_sema {
        println!("=== marqdo: sema ({path_label}) ===");
        println!(
            "functions: {:?}",
            module
                .functions
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>()
        );
        println!("imports: {:?}", module.imports);
        println!("=== marqdo: end sema ===");
    }

    match opts.backend {
        Backend::Tree => {
            let host = HostContext::for_run(Some(path), opts.host_caps(), opts.argv.clone());
            let mut interp = Interpreter::new(Some(path), opts.trace_eval)
                .with_stdin(stdin_lines.clone())
                .with_host(host);
            if opts.trace_eval {
                eprintln!("=== marqdo: trace-eval ({path_label}) ===");
            }
            interp.run_module(&module)?;
            if opts.trace_eval {
                eprintln!("=== marqdo: end trace-eval ===");
            }
            let plots = interp.take_plots();
            flush_auto_plots(Some(path), &plots).map_err(|e| anyhow::anyhow!(e))?;
        }
        Backend::Bytecode => {
            let program = compile_module(Some(path), &module)?;
            if opts.dump_bytecode {
                print!("{}", program.disassemble());
            }
            let host = HostContext::for_run(Some(path), opts.host_caps(), opts.argv.clone());
            let mut vm = Vm::new(Some(path))
                .with_stdin(stdin_lines)
                .with_trace(opts.trace_eval)
                .with_host(host);
            if opts.trace_eval {
                eprintln!("=== marqdo: trace-eval ({path_label}) ===");
            }
            vm.run(&program)?;
            if opts.trace_eval {
                eprintln!("=== marqdo: end trace-eval ===");
            }
            let plots = vm.take_plots();
            flush_auto_plots(Some(path), &plots).map_err(|e| anyhow::anyhow!(e))?;
        }
    }

    // dump bytecode even on tree backend if requested
    if opts.backend == Backend::Tree && opts.dump_bytecode {
        let program = compile_module(Some(path), &module)?;
        print!("{}", program.disassemble());
    }

    Ok(0)
}

/// Run and capture stdout (used by `view` and tests).
pub fn run_file_capture(path: &Path, opts: &RunOptions) -> Result<RunCapture> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
    let stdin_lines = crate::input_feed::effective_stdin(&source, &opts.stdin_lines);
    let module = load_module(path)?;
    match opts.backend {
        Backend::Tree => {
            let mut host = HostContext::for_capture(Some(path), opts.host_caps());
            host.argv = opts.argv.clone();
            if let Some(root) = &opts.fs_root {
                host.fs_root = Some(root.clone());
            }
            if let Some(lim) = opts.sleep_limit_ms {
                host.sleep_limit_ms = Some(lim);
            }
            let mut interp = Interpreter::with_capture(Some(path), false)
                .with_stdin(stdin_lines)
                .with_host(host);
            let value = interp.run_module(&module)?;
            let plots = interp.take_plots();
            Ok(RunCapture {
                stdout: interp.captured_stdout,
                value,
                plots,
            })
        }
        Backend::Bytecode => {
            let program = compile_module(Some(path), &module)?;
            let mut host = HostContext::for_capture(Some(path), opts.host_caps());
            host.argv = opts.argv.clone();
            if let Some(root) = &opts.fs_root {
                host.fs_root = Some(root.clone());
            }
            if let Some(lim) = opts.sleep_limit_ms {
                host.sleep_limit_ms = Some(lim);
            }
            let mut vm = Vm::with_capture(Some(path))
                .with_stdin(stdin_lines)
                .with_host(host);
            let value = vm.run(&program)?;
            let plots = vm.take_plots();
            Ok(RunCapture {
                stdout: vm.captured_stdout,
                value,
                plots,
            })
        }
    }
}

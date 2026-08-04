//! Marqdo reference interpreter (Phase I + M5 bytecode prototype).
//!
//! Pipeline: load → line classify → parse → tree-walk eval | bytecode VM.
//! CLI also offers `view` for AST-backed browsing.

pub mod ast;
pub mod builtin;
pub mod bytecode;
pub mod capture;
pub mod catalog;
pub mod debug;
pub mod diagnostics;
pub mod input_feed;
pub mod interp;
pub mod lex;
pub mod load;
pub mod parse;
pub mod value;
pub mod view;

use std::path::Path;

use anyhow::{bail, Result};

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
#[derive(Debug, Clone, Default)]
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
            backend: Backend::Tree,
            stdin_lines: Vec::new(),
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
}

/// Run a `.mq.md` program (prints to real stdout).
pub fn run_file(path: &Path, opts: &RunOptions) -> Result<i32> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;

    if source.trim().is_empty() {
        bail!("{} is empty", path.display());
    }

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
            let mut interp = Interpreter::new(Some(path), opts.trace_eval)
                .with_stdin(opts.stdin_lines.clone());
            if opts.trace_eval {
                eprintln!("=== marqdo: trace-eval ({path_label}) ===");
            }
            interp.run_module(&module)?;
            if opts.trace_eval {
                eprintln!("=== marqdo: end trace-eval ===");
            }
        }
        Backend::Bytecode => {
            let program = compile_module(Some(path), &module)?;
            if opts.dump_bytecode {
                print!("{}", program.disassemble());
            }
            let mut vm = Vm::new(Some(path))
                .with_stdin(opts.stdin_lines.clone())
                .with_trace(opts.trace_eval);
            if opts.trace_eval {
                eprintln!("=== marqdo: trace-eval ({path_label}) ===");
            }
            vm.run(&program)?;
            if opts.trace_eval {
                eprintln!("=== marqdo: end trace-eval ===");
            }
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
    let module = load_module(path)?;
    match opts.backend {
        Backend::Tree => {
            let mut interp = Interpreter::with_capture(Some(path), false)
                .with_stdin(opts.stdin_lines.clone());
            let value = interp.run_module(&module)?;
            Ok(RunCapture {
                stdout: interp.captured_stdout,
                value,
            })
        }
        Backend::Bytecode => {
            let program = compile_module(Some(path), &module)?;
            let mut vm = Vm::with_capture(Some(path)).with_stdin(opts.stdin_lines.clone());
            let value = vm.run(&program)?;
            Ok(RunCapture {
                stdout: vm.captured_stdout,
                value,
            })
        }
    }
}

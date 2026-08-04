//! Marqdo reference interpreter (Phase I).
//!
//! Pipeline: load → line classify → parse → tree-walk eval.

pub mod ast;
pub mod debug;
pub mod diagnostics;
pub mod interp;
pub mod lex;
pub mod load;
pub mod parse;
pub mod value;

use std::path::Path;

use anyhow::{bail, Result};

use crate::ast::format_ast_dump;
use crate::interp::Interpreter;
use crate::lex::{classify_source, format_lines_dump};
use crate::load::load_module;

/// Options for a single `run` / dump invocation.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub dump_lines: bool,
    pub dump_tokens: bool,
    pub dump_ast: bool,
    pub dump_sema: bool,
    pub trace_eval: bool,
}

impl RunOptions {
    pub fn dump_all() -> Self {
        Self {
            dump_lines: true,
            dump_tokens: true,
            dump_ast: true,
            dump_sema: true,
            trace_eval: true,
        }
    }

    pub fn any_dump(&self) -> bool {
        self.dump_lines || self.dump_tokens || self.dump_ast || self.dump_sema || self.trace_eval
    }
}

/// Run a `.mq.md` program.
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

    let mut interp = Interpreter::new(opts.trace_eval);
    if opts.trace_eval {
        eprintln!("=== marqdo: trace-eval ({path_label}) ===");
    }
    interp.run_module(&module)?;
    if opts.trace_eval {
        eprintln!("=== marqdo: end trace-eval ===");
    }
    Ok(0)
}

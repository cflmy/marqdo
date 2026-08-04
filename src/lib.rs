//! Marqdo reference interpreter (Phase I).
//!
//! Pipeline: load → line classify / lex → parse → sema → tree-walk eval.

pub mod debug;
pub mod diagnostics;
pub mod lex;
pub mod parse;

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::lex::{classify_source, format_lines_dump};

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

/// Run a `.mq.md` program (M1: line classify + dumps; eval still TODO).
pub fn run_file(path: &Path, opts: &RunOptions) -> Result<i32> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let path_label = path.display().to_string();
    let lines = classify_source(&source);

    if opts.dump_lines {
        print!("{}", format_lines_dump(&path_label, &lines));
    }
    if opts.dump_tokens {
        println!("=== marqdo: tokens ({path_label}) ===");
        println!("(tokens not implemented yet — M1+)");
        println!("=== marqdo: end tokens ===");
    }
    if opts.dump_ast {
        println!("=== marqdo: ast ({path_label}) ===");
        println!("(ast not implemented yet — M2)");
        println!("=== marqdo: end ast ===");
    }
    if opts.dump_sema {
        println!("=== marqdo: sema ({path_label}) ===");
        println!("(sema not implemented yet — M3+)");
        println!("=== marqdo: end sema ===");
    }
    if opts.trace_eval {
        println!("=== marqdo: trace-eval ({path_label}) ===");
        println!("(eval not implemented yet — M2+)");
        println!("=== marqdo: end trace-eval ===");
    }

    if source.is_empty() {
        bail!("{} is empty", path.display());
    }

    // Until eval exists: dumps are useful; run still fails loudly.
    bail!("evaluation not implemented — see doc/roadmap/interpreter.md (M2+)")
}

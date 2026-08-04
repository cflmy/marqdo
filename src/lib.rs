//! Marqdo reference interpreter (Phase I scaffolding).
//!
//! Pipeline (see `doc/roadmap/interpreter.md`):
//! load → line classify / lex → parse → sema → tree-walk eval.

pub mod diagnostics;
pub mod lex;
pub mod parse;

use std::path::Path;

use anyhow::{bail, Context, Result};

/// Run a `.mq.md` program. Phase I: not yet implemented beyond scaffolding.
pub fn run_file(path: &Path) -> Result<i32> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    if source.is_empty() {
        bail!("{} is empty", path.display());
    }

    // Placeholder: real lex/parse/eval lands in subsequent milestones.
    eprintln!(
        "marqdo: loaded {} ({} bytes); interpreter not implemented yet (M0 scaffold)",
        path.display(),
        source.len()
    );
    bail!("evaluation not implemented — see doc/roadmap/interpreter.md (M1+)")
}

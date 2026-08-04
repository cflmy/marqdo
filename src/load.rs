//! Load `.mq.md` files and merge frontmatter imports.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::ast::{Function, Module};
use crate::diagnostics::Diagnostic;
use crate::parse::parse_source;

/// Load `path` and recursively merge imported modules' top-level functions.
pub fn load_module(path: &Path) -> Result<Module> {
    let mut visited = HashSet::new();
    load_module_inner(path, &mut visited)
}

fn load_module_inner(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<Module> {
    let canon = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canon.clone()) {
        bail!("circular import involving {}", path.display());
    }

    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut module = parse_source(&source).map_err(|e| attach_path(path, e))?;

    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let imports = module.imports.clone();
    for rel in imports {
        let dep_path = base.join(&rel);
        let dep = load_module_inner(&dep_path, visited)?;
        for fun in dep.functions {
            merge_top_level(&mut module.functions, fun);
        }
    }

    Ok(module)
}

fn attach_path(path: &Path, err: anyhow::Error) -> anyhow::Error {
    if let Some(d) = err.downcast_ref::<Diagnostic>() {
        if d.path.is_none() {
            return Diagnostic::at(path, d.span, d.message.clone()).into();
        }
        return err;
    }
    // Legacy `line:col: message` from parse bail!
    let msg = err.to_string();
    if let Some((loc, rest)) = msg.split_once(": ") {
        if let Some((line_s, col_s)) = loc.split_once(':') {
            if let (Ok(line), Ok(col)) = (line_s.parse::<u32>(), col_s.parse::<u32>()) {
                return Diagnostic::at(path, crate::diagnostics::Span::new(line, col), rest).into();
            }
        }
    }
    err.context(format!("in {}", path.display()))
}

fn merge_top_level(into: &mut Vec<Function>, fun: Function) {
    if let Some(existing) = into.iter_mut().find(|f| f.name == fun.name) {
        *existing = fun;
    } else {
        into.push(fun);
    }
}

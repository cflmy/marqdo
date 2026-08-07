//! Load `.mq.md` files and merge frontmatter imports.

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::ast::{Function, Module};
use crate::diagnostics::Diagnostic;
use crate::embedded_lib;
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

    let source = read_module_source(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut module = parse_source(&source).map_err(|e| attach_path(path, e))?;

    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let imports = module.imports.clone();
    for rel in imports {
        let dep_path = resolve_import(base, &rel)?;
        let dep = load_module_inner(&dep_path, visited)?;
        for fun in dep.functions {
            merge_top_level(&mut module.functions, fun);
        }
    }

    Ok(module)
}

fn read_module_source(path: &Path) -> Result<String> {
    if path.is_file() {
        return std::fs::read_to_string(path).map_err(Into::into);
    }
    if let Some(source) = read_embedded_for_path(path) {
        return Ok(source);
    }
    std::fs::read_to_string(path).map_err(Into::into)
}

fn read_embedded_for_path(path: &Path) -> Option<String> {
    lib_remainder(path).as_deref().and_then(embedded_lib::read_file)
}

fn lib_remainder(path: &Path) -> Option<String> {
    let s = path.to_string_lossy().replace('\\', "/");
    if let Some(rest) = s.strip_prefix("lib/") {
        return Some(rest.to_string());
    }
    // Absolute or relative paths ending in lib/foo.mq.md
    let parts: Vec<&str> = s.split('/').collect();
    if let Some(pos) = parts.iter().position(|&p| p == "lib") {
        if pos + 1 < parts.len() {
            return Some(parts[pos + 1..].join("/"));
        }
    }
    None
}

/// Resolve an import path: relative to the importer first; `lib/…` / `std/…`
/// and `ext/…` also search official roots (`MARQDO_LIB` / `MARQDO_EXT`, cwd, near the binary),
/// then the embedded stdlib baked into the binary.
pub fn resolve_import(from_dir: &Path, rel: &str) -> Result<PathBuf> {
    let rel = rel.replace('\\', "/");
    let direct = from_dir.join(&rel);
    if direct.is_file() {
        return Ok(direct);
    }

    let normalized = if let Some(rest) = rel.strip_prefix("std/") {
        format!("lib/{rest}")
    } else {
        rel.clone()
    };

    if let Some(remainder) = normalized.strip_prefix("lib/") {
        for root in lib_search_roots() {
            let as_lib_dir = root.join(remainder);
            if as_lib_dir.is_file() {
                return Ok(as_lib_dir);
            }
            let as_repo_root = root.join("lib").join(remainder);
            if as_repo_root.is_file() {
                return Ok(as_repo_root);
            }
        }
        if embedded_lib::has_file(remainder) {
            return Ok(PathBuf::from("lib").join(remainder));
        }
        bail!(
            "cannot resolve library import `{rel}` (set MARQDO_LIB, keep lib/ next to the project, or use a binary with embedded stdlib)"
        );
    }

    if let Some(remainder) = normalized.strip_prefix("ext/") {
        for root in ext_search_roots() {
            let as_ext_dir = root.join(remainder);
            if as_ext_dir.is_file() {
                return Ok(as_ext_dir);
            }
            let as_repo_root = root.join("ext").join(remainder);
            if as_repo_root.is_file() {
                return Ok(as_repo_root);
            }
        }
        bail!(
            "cannot resolve extension import `{rel}` (set MARQDO_EXT or keep an ext/ next to the project)"
        );
    }

    Ok(direct)
}

fn lib_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(h) = env::var("MARQDO_LIB") {
        roots.push(PathBuf::from(h));
    }
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.join("lib"));
        roots.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("lib"));
            roots.push(dir.join("../lib"));
            roots.push(dir.join("../../lib"));
            roots.push(dir.join("../../../lib"));
        }
    }
    roots
}

fn ext_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(h) = env::var("MARQDO_EXT") {
        roots.push(PathBuf::from(h));
    }
    // Official installer default (`marqdo ext add`)
    roots.push(crate::ext_cli::default_user_ext_dir());
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.join("ext"));
        roots.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("ext"));
            roots.push(dir.join("../ext"));
            roots.push(dir.join("../../ext"));
            roots.push(dir.join("../../../ext"));
        }
    }
    roots
}

fn attach_path(path: &Path, err: anyhow::Error) -> anyhow::Error {
    if let Some(d) = err.downcast_ref::<Diagnostic>() {
        if d.path.is_none() {
            return Diagnostic::at(path, d.span, d.message.clone()).into();
        }
        return err;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_lib_from_cwd() {
        let cwd = env::current_dir().unwrap();
        let lib_text = cwd.join("lib").join("text.mq.md");
        if !lib_text.is_file() {
            return;
        }
        let p = resolve_import(Path::new("tests/keywords"), "lib/text.mq.md").unwrap();
        assert!(p.ends_with("text.mq.md"));
        let p2 = resolve_import(Path::new("tests/keywords"), "std/text.mq.md").unwrap();
        assert!(p2.ends_with("text.mq.md"));
    }

    #[test]
    fn resolve_embedded_lib_source() {
        let p = resolve_import(Path::new("/nonexistent"), "lib/writeback.mq.md").unwrap();
        assert!(p.to_string_lossy().replace('\\', "/").contains("lib/writeback.mq.md"));
        let src = read_module_source(&p).unwrap();
        assert!(src.contains("host_writeback"));
    }
}

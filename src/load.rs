//! Load `.mq.md` files and bind frontmatter imports as library modules.

use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::ast::{Function, Module};
use crate::diagnostics::Diagnostic;
use crate::embedded_lib;
use crate::parse::parse_source;

/// Load `path` and recursively bind imported modules (no flat merge).
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

    let result = (|| {
        let source = read_module_source(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut module = parse_source(&source).map_err(|e| attach_path(path, e))?;

        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let imports = module.imports.clone();
        let mut import_modules = HashMap::new();
        for imp in imports {
            let dep_path = resolve_import(base, &imp.path)?;
            let dep = load_module_inner(&dep_path, visited)?;
            if import_modules.contains_key(&imp.bind) {
                bail!(
                    "duplicate import bind `{}` while loading {}",
                    imp.bind,
                    path.display()
                );
            }
            import_modules.insert(imp.bind, dep);
        }
        module.import_modules = import_modules;

        Ok(module)
    })();

    visited.remove(&canon);
    result
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
    if let Some(source) = read_embedded_for_path(Path::new(&rel)) {
        let _ = source;
        // Prefer real file under search roots; else return a virtual lib/ path for embedded read.
    }
    if rel.starts_with("lib/") || rel.starts_with("std/") {
        let rest = rel
            .strip_prefix("lib/")
            .or_else(|| rel.strip_prefix("std/"))
            .unwrap();
        for root in lib_search_roots() {
            let p = root.join(rest);
            if p.is_file() {
                return Ok(p);
            }
        }
        // Embedded stdlib: synthesize a path under lib/ for read_module_source.
        return Ok(PathBuf::from("lib").join(rest));
    }
    if rel.starts_with("ext/") {
        for root in ext_search_roots() {
            let p = root.join(rel.strip_prefix("ext/").unwrap_or(&rel));
            // also try full ext/rel under root
            let p2 = root.join(&rel);
            if p.is_file() {
                return Ok(p);
            }
            if p2.is_file() {
                return Ok(p2);
            }
            // MARQDO_EXT is the ext root itself (contains ai/)
            let under = root.join(rel.strip_prefix("ext/").unwrap());
            if under.is_file() {
                return Ok(under);
            }
        }
    }
    // Relative again with normalize
    if direct.exists() {
        return Ok(direct);
    }
    bail!("cannot resolve import `{rel}` from {}", from_dir.display())
}

fn lib_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(h) = env::var("MARQDO_LIB") {
        roots.push(PathBuf::from(h));
    }
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.join("lib"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("lib"));
            roots.push(dir.join("..").join("lib"));
            roots.push(dir.join("..").join("..").join("lib"));
        }
    }
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(manifest).join("lib"));
    }
    roots
}

fn ext_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(h) = env::var("MARQDO_EXT") {
        roots.push(PathBuf::from(h));
    }
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.join("ext"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("ext"));
            roots.push(dir.join("..").join("ext"));
            roots.push(dir.join("..").join("..").join("ext"));
        }
    }
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(manifest).join("ext"));
    }
    roots
}

fn attach_path(path: &Path, err: anyhow::Error) -> anyhow::Error {
    match err.downcast::<Diagnostic>() {
        Ok(mut d) => {
            if d.path.is_none() {
                d.path = Some(path.to_path_buf());
            }
            d.into()
        }
        Err(e) => anyhow::anyhow!("{}: {e}", path.display()),
    }
}

/// Kept for tests / tooling that expect a flat list (local tops only).
#[allow(dead_code)]
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
}

//! Official extension installer (`marqdo ext list|add|remove`).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Official extension catalog entry.
#[derive(Debug, Clone)]
pub struct ExtPackage {
    pub id: &'static str,
    pub description: &'static str,
    /// Files relative to a source `ext/` directory (UTF-8 paths).
    pub mq_files: &'static [&'static str],
    /// If set, also install native plugin named by [`native_lib_filename`].
    pub native_crate: Option<&'static str>,
}

pub const CATALOG: &[ExtPackage] = &[
    ExtPackage {
        id: "llm",
        description: "OpenAI-compatible chat object (ext/llm)",
        mq_files: &["llm.mq.md", "大模型.mq.md"],
        native_crate: None,
    },
    ExtPackage {
        id: "agent",
        description: "Agent development framework (ext/agent + ABI plugin)",
        mq_files: &["agent.mq.md", "智能体.mq.md"],
        native_crate: Some("marqdo_plugin_agent"),
    },
];

pub fn find_package(id: &str) -> Option<&'static ExtPackage> {
    let id = id.trim();
    CATALOG.iter().find(|p| p.id == id)
}

/// Default install root: `MARQDO_EXT` if set, else `~/.marqdo/ext`.
pub fn install_root() -> Result<PathBuf> {
    if let Ok(h) = env::var("MARQDO_EXT") {
        let p = PathBuf::from(h);
        if !p.as_os_str().is_empty() {
            return Ok(p);
        }
    }
    Ok(default_user_ext_dir())
}

pub fn default_user_ext_dir() -> PathBuf {
    dirs_next_home()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".marqdo")
        .join("ext")
}

fn dirs_next_home() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Trusted roots for native plugins (sandbox bypass for `host_plugin_load`).
pub fn trusted_plugin_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(h) = env::var("MARQDO_EXT") {
        roots.push(PathBuf::from(h));
    }
    roots.push(default_user_ext_dir());
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("ext"));
        }
    }
    roots
}

pub fn native_lib_filename() -> &'static str {
    if cfg!(windows) {
        "agent.dll"
    } else if cfg!(target_os = "macos") {
        "libagent.dylib"
    } else {
        "libagent.so"
    }
}

pub fn is_installed(root: &Path, pkg: &ExtPackage) -> bool {
    pkg.mq_files
        .iter()
        .all(|f| root.join(f).is_file())
}

pub fn list_ext() -> Result<()> {
    let root = install_root()?;
    println!("Install root: {}", root.display());
    println!();
    println!(
        "{:<10} {:<8} {}",
        "ID", "STATUS", "DESCRIPTION"
    );
    println!("{:-<10} {:-<8} {:-<40}", "", "", "");
    for pkg in CATALOG {
        let status = if is_installed(&root, pkg) {
            "yes"
        } else {
            "no"
        };
        println!("{:<10} {:<8} {}", pkg.id, status, pkg.description);
    }
    Ok(())
}

/// Directories that may contain source `ext/*.mq.md` for `add` (dev / release layout).
pub fn source_ext_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(h) = env::var("MARQDO_EXT_SOURCE") {
        roots.push(PathBuf::from(h));
    }
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.join("ext"));
        // cargo test / run from repo root
        roots.push(cwd.join("..").join("ext"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("ext"));
            roots.push(dir.join("..").join("ext"));
            roots.push(dir.join("..").join("..").join("ext"));
            roots.push(dir.join("..").join("..").join("..").join("ext"));
            // target/debug → repo ext
            roots.push(dir.join("..").join("..").join("..").join("ext"));
        }
    }
    roots
}

fn find_source_file(name: &str) -> Result<PathBuf> {
    for root in source_ext_roots() {
        let p = root.join(name);
        if p.is_file() {
            return Ok(p);
        }
    }
    bail!(
        "cannot find source `{name}` for install (set MARQDO_EXT_SOURCE to an ext/ directory, or run from the Marqdo repo)"
    );
}

fn find_native_plugin() -> Result<PathBuf> {
    let name = native_lib_filename();
    let mut candidates = Vec::new();
    if let Ok(h) = env::var("MARQDO_EXT_SOURCE") {
        let base = PathBuf::from(h);
        candidates.push(base.join("native").join(name));
        candidates.push(base.join(name));
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("target").join("debug").join(name));
        candidates.push(cwd.join("target").join("release").join(name));
        candidates.push(cwd.join("ext").join("native").join(name));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("ext").join("native").join(name));
            candidates.push(dir.join(name));
            // target/debug/deps → target/debug
            candidates.push(dir.join(name));
            candidates.push(dir.join("..").join(name));
        }
    }
    for p in candidates {
        if p.is_file() {
            return Ok(p);
        }
    }
    bail!(
        "cannot find native plugin `{name}` (run `cargo build -p marqdo_plugin_agent`, or place it under MARQDO_EXT_SOURCE/native/)"
    );
}

pub fn add_ext(id: &str) -> Result<()> {
    let pkg = find_package(id).with_context(|| {
        format!(
            "unknown extension `{id}` (official: {})",
            CATALOG
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    let root = install_root()?;
    fs::create_dir_all(&root)
        .with_context(|| format!("create install root {}", root.display()))?;

    for f in pkg.mq_files {
        let src = find_source_file(f)?;
        let dest = root.join(f);
        fs::copy(&src, &dest)
            .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
        println!("installed {}", dest.display());
    }

    if pkg.native_crate.is_some() {
        let src = find_native_plugin()?;
        let native_dir = root.join("native");
        fs::create_dir_all(&native_dir)?;
        let dest = native_dir.join(native_lib_filename());
        fs::copy(&src, &dest)
            .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
        println!("installed {}", dest.display());
        // Absolute path hint for loaders / env
        let abs = dest
            .canonicalize()
            .unwrap_or_else(|_| dest.clone());
        let hint = root.join("agent.plugin");
        fs::write(&hint, abs.to_string_lossy().as_bytes())?;
        println!("wrote {}", hint.display());
        println!(
            "hint: import works via install root; set MARQDO_AGENT_PLUGIN={} if load_native needs it",
            abs.display()
        );
    }

    if env::var_os("MARQDO_EXT").is_none() {
        println!(
            "hint: imports resolve via {} automatically; or set MARQDO_EXT to this directory",
            root.display()
        );
    }
    Ok(())
}

pub fn remove_ext(id: &str) -> Result<()> {
    let pkg = find_package(id).with_context(|| format!("unknown extension `{id}`"))?;
    let root = install_root()?;
    let mut removed = 0usize;
    for f in pkg.mq_files {
        let dest = root.join(f);
        if dest.is_file() {
            fs::remove_file(&dest)
                .with_context(|| format!("remove {}", dest.display()))?;
            println!("removed {}", dest.display());
            removed += 1;
        }
    }
    if pkg.native_crate.is_some() {
        let dest = root.join("native").join(native_lib_filename());
        if dest.is_file() {
            fs::remove_file(&dest)?;
            println!("removed {}", dest.display());
            removed += 1;
        }
        let hint = root.join("agent.plugin");
        if hint.is_file() {
            fs::remove_file(&hint)?;
            removed += 1;
        }
    }
    if removed == 0 {
        println!("nothing to remove for `{id}` under {}", root.display());
    }
    Ok(())
}

/// Resolve installed native plugin path for `name` (currently only `agent`).
pub fn installed_native_path(name: &str) -> Option<PathBuf> {
    if name != "agent" {
        return None;
    }
    for root in trusted_plugin_roots() {
        let hint = root.join("agent.plugin");
        if let Ok(s) = fs::read_to_string(&hint) {
            let p = PathBuf::from(s.trim());
            if p.is_file() {
                return Some(p);
            }
        }
        let p = root.join("native").join(native_lib_filename());
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

pub fn path_is_trusted_plugin(path: &Path) -> bool {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    for root in trusted_plugin_roots() {
        let root = root.canonicalize().unwrap_or(root);
        if crate::host::path_under_root(&path, &root) {
            return true;
        }
        let native = root.join("native");
        let native = native.canonicalize().unwrap_or(native);
        if crate::host::path_under_root(&path, &native) {
            return true;
        }
    }
    false
}

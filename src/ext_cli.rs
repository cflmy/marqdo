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
        description: "OpenAI-compatible chat object (ext/ai/llm)",
        mq_files: &["ai/llm.mq.md", "ai/大模型.mq.md"],
        native_crate: None,
    },
    ExtPackage {
        id: "agent",
        description: "Agent development framework (ext/ai/agent)",
        mq_files: &["ai/agent.mq.md", "ai/智能体.mq.md"],
        native_crate: Some("marqdo_plugin_agent"),
    },
    ExtPackage {
        id: "web",
        description: "Dynamic website toolkit (ext/web) — page/db/bind/admin",
        mq_files: &["web/web.mq.md", "web/网页.mq.md"],
        native_crate: Some("marqdo_plugin_web"),
    },
    ExtPackage {
        id: "quantum",
        description: "Quantum circuit simulator (ext/quantum) — gates/state vector",
        mq_files: &["quantum/quantum.mq.md", "quantum/量子.mq.md"],
        native_crate: Some("marqdo_plugin_quantum"),
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

pub fn native_short_name(crate_or_id: &str) -> &str {
    match crate_or_id {
        "marqdo_plugin_agent" | "agent" => "agent",
        "marqdo_plugin_web" | "web" => "web",
        "marqdo_plugin_quantum" | "quantum" => "quantum",
        other => other,
    }
}

pub fn native_lib_filename(short: &str) -> String {
    let short = native_short_name(short);
    if cfg!(windows) {
        format!("{short}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{short}.dylib")
    } else {
        format!("lib{short}.so")
    }
}

pub fn native_env_var(short: &str) -> &'static str {
    match native_short_name(short) {
        "web" => "MARQDO_WEB_PLUGIN",
        "quantum" => "MARQDO_QUANTUM_PLUGIN",
        _ => "MARQDO_AGENT_PLUGIN",
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

fn find_native_plugin(short: &str) -> Result<PathBuf> {
    let short = native_short_name(short);
    let name = native_lib_filename(short);
    let env_key = native_env_var(short);
    let mut candidates = Vec::new();
    if let Ok(p) = env::var(env_key) {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Ok(p);
        }
        candidates.push(p);
    }
    if let Ok(h) = env::var("MARQDO_EXT_SOURCE") {
        let base = PathBuf::from(h);
        candidates.push(base.join("native").join(&name));
        candidates.push(base.join(&name));
        // MARQDO_EXT_SOURCE often points at repo `ext/`; native libs live in sibling target/.
        if let Some(repo) = base.parent() {
            candidates.push(repo.join("target").join("debug").join(&name));
            candidates.push(repo.join("target").join("release").join(&name));
        }
    }
    // Honor CARGO_TARGET_DIR (sandbox / custom target roots) before cwd/target.
    if let Ok(td) = env::var("CARGO_TARGET_DIR") {
        let td = PathBuf::from(td);
        candidates.push(td.join("debug").join(&name));
        candidates.push(td.join("release").join(&name));
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("target").join("debug").join(&name));
        candidates.push(cwd.join("target").join("release").join(&name));
        candidates.push(cwd.join("ext").join("native").join(&name));
        // Walk up looking for a Cargo workspace target/ (run from a subdir).
        let mut dir = cwd.clone();
        for _ in 0..6 {
            candidates.push(dir.join("target").join("debug").join(&name));
            candidates.push(dir.join("target").join("release").join(&name));
            if !dir.pop() {
                break;
            }
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("ext").join("native").join(&name));
            // Same directory as the marqdo binary (typical: target/debug/).
            candidates.push(dir.join(&name));
            // target/debug/deps → target/debug
            candidates.push(dir.join("..").join(&name));
            candidates.push(dir.join("..").join("release").join(&name));
        }
    }
    for p in &candidates {
        if p.is_file() {
            return Ok(p.clone());
        }
    }
    bail!(
        "cannot find native plugin `{name}` (run `cargo build -p marqdo_plugin_{short}`, or set {env_key})"
    )
}

/// Locate a built native plugin, or `cargo build -p …` once then look again.
fn ensure_native_plugin(crate_name: &str) -> Result<PathBuf> {
    let short = native_short_name(crate_name);
    if let Ok(p) = find_native_plugin(short) {
        return Ok(p);
    }
    println!(
        "native plugin for `{short}` not found; building `{crate_name}` (debug)…"
    );
    let status = std::process::Command::new("cargo")
        .args(["build", "-p", crate_name])
        .status()
        .with_context(|| format!("spawn cargo build -p {crate_name}"))?;
    if !status.success() {
        bail!(
            "cargo build -p {crate_name} failed (status {status}); build the plugin then re-run `marqdo ext add {short}`"
        );
    }
    find_native_plugin(short).with_context(|| {
        format!(
            "built `{crate_name}` but still cannot find {}; set {} to the .so/.dll path",
            native_lib_filename(short),
            native_env_var(short)
        )
    })
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

    // Preflight native binary *before* copying .mq.md so a missing .so does not
    // leave a half-installed tree that fails at `plugin.native_path` / load time.
    let native_src = if let Some(crate_name) = pkg.native_crate {
        Some(ensure_native_plugin(crate_name)?)
    } else {
        None
    };

    for f in pkg.mq_files {
        let src = find_source_file(f)?;
        let dest = root.join(f);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        fs::copy(&src, &dest)
            .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
        println!("installed {}", dest.display());
    }

    if let (Some(crate_name), Some(src)) = (pkg.native_crate, native_src) {
        let short = native_short_name(crate_name);
        let lib_name = native_lib_filename(short);
        let native_dir = root.join("native");
        fs::create_dir_all(&native_dir)?;
        let dest = native_dir.join(&lib_name);
        fs::copy(&src, &dest)
            .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
        println!("installed {}", dest.display());
        // Absolute path hint for loaders / env
        let abs = dest
            .canonicalize()
            .unwrap_or_else(|_| dest.clone());
        let hint = root.join(format!("{short}.plugin"));
        fs::write(&hint, abs.to_string_lossy().as_bytes())?;
        println!("wrote {}", hint.display());
        println!(
            "hint: `plugin.native_path name={short}` resolves via {}; or set {}={}",
            dest.display(),
            native_env_var(short),
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
    if let Some(crate_name) = pkg.native_crate {
        let short = native_short_name(crate_name);
        let dest = root.join("native").join(native_lib_filename(short));
        if dest.is_file() {
            fs::remove_file(&dest)?;
            println!("removed {}", dest.display());
            removed += 1;
        }
        let hint = root.join(format!("{short}.plugin"));
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

/// Resolve installed native plugin path for `name` (`agent`, `web`, …).
/// Also falls back to cargo `target/{debug,release}` artifacts for local runs.
pub fn installed_native_path(name: &str) -> Option<PathBuf> {
    let short = native_short_name(name);
    if !matches!(short, "agent" | "web" | "quantum") {
        return None;
    }
    let lib_name = native_lib_filename(short);
    for root in trusted_plugin_roots() {
        let hint = root.join(format!("{short}.plugin"));
        if let Ok(s) = fs::read_to_string(&hint) {
            let p = PathBuf::from(s.trim());
            if p.is_file() {
                return Some(p);
            }
        }
        let p = root.join("native").join(&lib_name);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(p) = env::var(native_env_var(short)) {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    find_native_plugin(short).ok()
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
    for key in ["MARQDO_AGENT_PLUGIN", "MARQDO_WEB_PLUGIN", "MARQDO_QUANTUM_PLUGIN"] {
        if let Ok(p) = env::var(key) {
            let p = PathBuf::from(p);
            let p = p.canonicalize().unwrap_or(p);
            if p == path {
                return true;
            }
        }
    }
    for short in ["agent", "web", "quantum"] {
        if let Ok(found) = find_native_plugin(short) {
            let found = found.canonicalize().unwrap_or(found);
            if found == path {
                return true;
            }
        }
    }
    false
}

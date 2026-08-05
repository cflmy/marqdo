//! L0.5 host primitives for official stdlib (`lib/fs`, `time`, …).
//!
//! User docs should import L1 wrappers; these `host_*` names are the Rust surface.
//! Capabilities default **on** (importing a lib means you intend to use it).
//! Soft side-effects (no process::exit; sleep clamp) apply under view / capture only.

mod dispatch;
mod fs;
mod json;
mod net;
mod sys;
mod time;

pub use dispatch::{call_host, HostFn};
pub use fs::path_under_root;

use std::path::{Path, PathBuf};

/// Host capability flags. Default: all enabled.
#[derive(Debug, Clone)]
pub struct HostCaps {
    pub fs_write: bool,
    pub exec: bool,
    pub net: bool,
}

impl Default for HostCaps {
    fn default() -> Self {
        Self {
            fs_write: true,
            exec: true,
            net: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostContext {
    pub caps: HostCaps,
    pub fs_root: Option<PathBuf>,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    /// Soft mode: do not `process::exit`; honor sleep_limit (view / capture / export).
    pub soft_side_effects: bool,
    pub sleep_limit_ms: Option<u64>,
}

impl Default for HostContext {
    fn default() -> Self {
        Self {
            caps: HostCaps::default(),
            fs_root: None,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            argv: Vec::new(),
            soft_side_effects: false,
            sleep_limit_ms: Some(30_000),
        }
    }
}

impl HostContext {
    pub fn for_run(path: Option<&Path>, caps: HostCaps, argv: Vec<String>) -> Self {
        let mut ctx = Self::default();
        ctx.caps = caps;
        ctx.argv = argv;
        if let Some(p) = path {
            if let Some(parent) = p.parent() {
                let parent = if parent.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    parent.to_path_buf()
                };
                ctx.cwd = parent.clone();
                ctx.fs_root = Some(parent);
            }
        }
        ctx
    }

    /// Capture stdout / view / export: soft exit + sleep clamp; same caps as CLI by default.
    pub fn for_capture(path: Option<&Path>, caps: HostCaps) -> Self {
        let mut ctx = Self::for_run(path, caps, Vec::new());
        ctx.soft_side_effects = true;
        ctx.sleep_limit_ms = Some(0);
        ctx
    }

    pub fn allow_fs_write(&self) -> bool {
        self.caps.fs_write
    }

    pub fn allow_exec(&self) -> bool {
        self.caps.exec
    }

    pub fn allow_net(&self) -> bool {
        self.caps.net
    }

    pub fn resolve_path(&self, rel: &str) -> Result<PathBuf, String> {
        let p = Path::new(rel);
        let abs = if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.cwd.join(p)
        };
        if let Some(root) = &self.fs_root {
            let root_canon = root.canonicalize().unwrap_or_else(|_| root.clone());
            let check = abs.canonicalize().unwrap_or_else(|_| abs.clone());
            if !path_under_root(&check, &root_canon) && !path_under_root(&abs, root) {
                if let Some(parent) = abs.parent() {
                    let parent_ok = parent
                        .canonicalize()
                        .map(|p| path_under_root(&p, &root_canon))
                        .unwrap_or_else(|_| path_under_root(parent, root));
                    if !parent_ok {
                        return Err(format!("path escapes sandbox root: {}", abs.display()));
                    }
                } else {
                    return Err(format!("path escapes sandbox root: {}", abs.display()));
                }
            }
        }
        Ok(abs)
    }
}

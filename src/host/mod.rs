//! L0.5 host primitives for official stdlib (`lib/fs`, `time`, `math`, …).
//!
//! User docs should import L1 wrappers; these `host_*` names are the Rust surface.
//! Capabilities default **on** (importing a lib means you intend to use it).
//! Soft side-effects (no process::exit; sleep clamp) apply under view / capture only.

pub mod agent_rt;
mod dispatch;
pub mod foreign;
mod fs;
mod json;
pub mod math;
mod net;
pub mod plugin;
pub mod subtask;
mod sys;
mod time;
pub mod writeback;

pub use dispatch::{call_host, HostFn};
pub use fs::path_under_root;
pub use plugin::PluginState;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::value::Value;

/// Host capability flags. Default: all enabled.
#[derive(Debug, Clone)]
pub struct HostCaps {
    pub fs_write: bool,
    pub exec: bool,
    pub net: bool,
    pub plugin: bool,
}

impl Default for HostCaps {
    fn default() -> Self {
        Self {
            fs_write: true,
            exec: true,
            net: true,
            plugin: true,
        }
    }
}

/// SVG plot produced during a run (for CLI auto-write / view embed).
#[derive(Debug, Clone)]
pub struct PlotArtifact {
    /// User-requested path, if any.
    pub path: Option<String>,
    pub svg: String,
}

#[derive(Debug)]
pub struct HostContext {
    pub caps: HostCaps,
    pub fs_root: Option<PathBuf>,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    /// Soft mode: do not `process::exit`; honor sleep_limit (view / capture / export).
    pub soft_side_effects: bool,
    pub sleep_limit_ms: Option<u64>,
    /// LCG state for `random` / `random_int`.
    pub rng: u64,
    pub plots: Vec<PlotArtifact>,
    /// lang → interpreter argv (from `set_cmd`).
    pub foreign_cmds: HashMap<String, Vec<String>>,
    /// Loaded native plugins (ABI v1).
    pub plugins: PluginState,
    /// Entry `.mq.md` source text (for agent context injection).
    pub entry_source: Option<String>,
    pub entry_path: Option<PathBuf>,
    /// Active function names (outer → inner) for call-site injection.
    pub call_stack: Vec<String>,
    /// Per-agent conversation history (id → turns).
    pub agent_histories: HashMap<String, Vec<Value>>,
    pub agent_seq: u64,
    /// Last statement line (updated by interpreters for `host_call_site`).
    pub current_line: u32,
    /// Entry-file line of each active user call (for writeback anchoring).
    pub call_site_lines: Vec<u32>,
    /// Concurrent subtasks (`lib/subtask`): file / function / foreign.
    pub subtasks: HashMap<u64, subtask::Handle>,
    pub subtask_seq: u64,
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
            rng: 0xC0FF_EE42_CAFE_BABE,
            plots: Vec::new(),
            foreign_cmds: HashMap::new(),
            plugins: PluginState::default(),
            entry_source: None,
            entry_path: None,
            call_stack: Vec::new(),
            agent_histories: HashMap::new(),
            agent_seq: 0,
            current_line: 1,
            call_site_lines: Vec::new(),
            subtasks: HashMap::new(),
            subtask_seq: 0,
        }
    }
}

impl Drop for HostContext {
    fn drop(&mut self) {
        subtask::kill_all(self);
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

    pub fn push_call_site_line(&mut self, line: u32) {
        self.call_site_lines.push(line);
    }

    pub fn pop_call_site_line(&mut self) {
        self.call_site_lines.pop();
    }

    pub fn allow_plugin(&self) -> bool {
        self.caps.plugin
    }

    pub fn next_u64(&mut self) -> u64 {
        // SplitMix64-ish LCG step
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    pub fn next_f64(&mut self) -> f64 {
        let x = self.next_u64() >> 11;
        (x as f64) / ((1u64 << 53) as f64)
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

/// Write auto-named plot files for CLI runs when `path` was omitted.
pub fn flush_auto_plots(source_path: Option<&Path>, plots: &[PlotArtifact]) -> Result<(), String> {
    let mut auto_i = 0usize;
    for plot in plots {
        if plot.path.is_some() {
            continue;
        }
        auto_i += 1;
        let stem = source_path
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("out");
        let stem = stem.trim_end_matches(".mq");
        let name = format!("{stem}-plot-{auto_i}.svg");
        let dir = source_path
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new("."));
        let dest = dir.join(&name);
        std::fs::write(&dest, plot.svg.as_bytes())
            .map_err(|e| format!("failed to write plot {}: {e}", dest.display()))?;
        println!("plot: {}", dest.display());
    }
    Ok(())
}

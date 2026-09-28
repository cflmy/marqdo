//! Marqdo Engineering Knowledge Compiler (EKC).
//!
//! Source of truth remains `*.mq.md`. This module projects L0–L4 catalog,
//! knowledge graph, reuse preflight, and agent context packs under `.marqdo/`.

pub mod capability;
pub mod compile;
pub mod duplicate;
pub mod extract;
pub mod find;
pub mod fingerprint;
pub mod graph;
pub mod impact;
pub mod ir;
pub mod learn;
pub mod lifecycle;
pub mod metrics;
pub mod policy;
pub mod preflight;
pub mod reuse;
pub mod util;

pub use compile::{compile, ensure_compiled, load_graph, KnowledgeOptions};
pub use duplicate::{find_duplicates, format_duplicates_text};
pub use find::{find, find_json, format_find_text};
pub use impact::{format_impact_text, impact, impact_json};
pub use ir::{KnowledgeGraph, ReuseDecision, ReuseResult};
pub use learn::{learn, LearnInput};
pub use lifecycle::{
    conflicts_json, find_conflicts, find_stale, format_conflicts_text, format_stale_text,
    format_verify_text, stale_json, verify, verify_json,
};
pub use metrics::{load_metrics, record_decision, save_metrics, ReuseMetrics};
pub use policy::compile_policy;
pub use preflight::{format_preflight_yaml, preflight, preflight_json, PreflightResult};
pub use reuse::{format_reuse_text, resolve, reuse_json, ReuseBudget};

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Run knowledge compile for CLI / catalog hook.
pub fn write_knowledge(opts: KnowledgeOptions) -> Result<KnowledgeGraph> {
    compile(opts)
}

/// Default out dir helper.
pub fn default_out(path: &Path) -> PathBuf {
    if path.is_file() {
        path.parent()
            .unwrap_or(Path::new("."))
            .join(".marqdo")
    } else {
        path.join(".marqdo")
    }
}

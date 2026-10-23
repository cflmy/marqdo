//! Compiler semantic analysis for executable knowledge.
//!
//! Syntax remains in `ast`/`parse`; this module owns project-wide identity and
//! binding facts so runtime, EKC, view, and agents do not independently infer
//! the meaning of the same source text.

pub mod ir;
pub mod lock;
pub mod symbols;

pub use ir::{
    artifact_json, compile_project, write_semantic_projections, ArtifactId, CompileOptions, Compilation,
    Digest, EffectSet, LockMode, SemanticArtifact, SemanticProject,
};
pub use lock::{check_links_lock, impact as semantic_impact, update_links_lock, LockReport};
pub use symbols::{CallableKind, ModuleId, ProjectIndex, SymbolDecl, SymbolId, SymbolKind};

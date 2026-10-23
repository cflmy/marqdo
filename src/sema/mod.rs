//! Compiler semantic analysis for executable knowledge.
//!
//! Syntax remains in `ast`/`parse`; this module owns project-wide identity and
//! binding facts so runtime, EKC, view, and agents do not independently infer
//! the meaning of the same source text.

pub mod symbols;

pub use symbols::{CallableKind, ModuleId, ProjectIndex, SymbolDecl, SymbolId, SymbolKind};

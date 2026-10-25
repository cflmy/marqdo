//! Workbench runtime contracts (M00–M05): schemas, capabilities, DocumentView,
//! KnowledgeLocator, ModelSelection / ModelCallUsage / RuntimeEvent / RunContext.
//!
//! Spec: `doc/design/qdagent-workbench-runtime-and-view-2026-10-25.md`.
//! These are v1 contracts distinct from KnowledgePresentation `schema_version:1`.

pub mod analysis_cache;
pub mod capabilities;
#[cfg(feature = "view")]
pub mod document_view;
pub mod event_sink;
pub mod locator;
pub mod run_context;
pub mod selection;
pub mod usage;

pub use analysis_cache::{cache_keys_isolated, note_analysis, project_source_digest};
pub use capabilities::probe_capabilities;
#[cfg(feature = "view")]
pub use document_view::{build_document_view, DocumentViewRequest};
pub use event_sink::{bind_event_sink, clear_event_sink, emit_event, emit_message_delta};
pub use locator::{resolve_reference, KnowledgeLocator};
pub use run_context::{cancel_requested, RunContext};
pub use selection::{parse_model_selection, ModelSelection};
pub use usage::{
    model_call_usage_from_provider, model_call_usage_from_provider_rev, replace_usage_if_newer,
    runtime_event, token_or_null,
};

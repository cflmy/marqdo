//! Pipeline dump helpers + interactive debug (see doc/design/pipeline-debug.md, view-debug).

mod ctrl;
mod trace;

pub use crate::lex::format_lines_dump;
pub use ctrl::{
    snapshot_json, DebugAction, DebugController, DebugPause, DebugSnapshot,
};
pub use trace::emit_trace;

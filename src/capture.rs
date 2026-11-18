//! Captured program run (for tests and `view`).

use crate::host::PlotArtifact;
use crate::value::Value;

/// Silent execution trace for view annotations (not stderr `trace_eval`).
#[derive(Debug, Clone, Default)]
pub struct ViewTrace {
    pub branches: Vec<BranchHit>,
    pub loops: Vec<LoopHit>,
    /// Per-statement post-state / control events (tree capture).
    pub events: Vec<StmtEvent>,
    /// User-function invocations (tree capture).
    pub calls: Vec<CallFrame>,
    /// Foreach / while loop frames with per-iteration item values.
    pub loop_frames: Vec<LoopFrame>,
}

#[derive(Debug, Clone)]
pub struct BranchHit {
    pub line: u32,
    pub arm_index: usize,
    pub count: u32,
    /// Call frame that took this arm (`0` = entry / `# main`).
    pub call_id: u32,
}

#[derive(Debug, Clone)]
pub struct LoopHit {
    pub line: u32,
    pub iterations: u32,
    pub last_item: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoopFrame {
    pub loop_id: u32,
    pub line: u32,
    pub item_name: Option<String>,
    pub collection: Option<String>,
    pub iterations: Vec<LoopIter>,
}

#[derive(Debug, Clone)]
pub struct LoopIter {
    /// 0-based iteration index.
    pub index: u32,
    pub item_value: String,
    pub call_id: u32,
}

#[derive(Debug, Clone)]
pub struct StmtEvent {
    pub line: u32,
    pub kind: StmtEventKind,
    pub name: Option<String>,
    pub value_after: Option<String>,
    pub arm_index: Option<usize>,
    /// 0-based iteration index when inside a loop.
    pub iter: Option<u32>,
    pub call_id: u32,
    pub loop_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmtEventKind {
    Assign,
    Return,
    Call,
    Branch,
    Loop,
    BindRef,
}

impl StmtEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            StmtEventKind::Assign => "assign",
            StmtEventKind::Return => "return",
            StmtEventKind::Call => "call",
            StmtEventKind::Branch => "branch",
            StmtEventKind::Loop => "loop",
            StmtEventKind::BindRef => "bind_ref",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub call_id: u32,
    pub fn_name: String,
    /// Definition line of the callee.
    pub fn_line: u32,
    /// Call site line in the executing document (0 for the entry call).
    pub call_line: u32,
    pub ordinal: u32,
    pub args_brief: String,
}

#[derive(Debug, Clone)]
pub struct RunCapture {
    pub stdout: String,
    pub value: Value,
    pub plots: Vec<PlotArtifact>,
    /// Entry-function bindings after a successful tree-backend run (empty for bytecode).
    pub bindings: std::collections::HashMap<String, Value>,
    /// Branch/loop/stmt hits collected during tree capture (empty for bytecode).
    pub view_trace: ViewTrace,
}

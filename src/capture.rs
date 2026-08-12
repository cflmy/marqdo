//! Captured program run (for tests and `view`).

use crate::value::Value;
use crate::host::PlotArtifact;

#[derive(Debug, Clone)]
pub struct RunCapture {
    pub stdout: String,
    pub value: Value,
    pub plots: Vec<PlotArtifact>,
    /// Entry-function bindings after a successful tree-backend run (empty for bytecode).
    pub bindings: std::collections::HashMap<String, Value>,
}

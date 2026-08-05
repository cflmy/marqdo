//! Captured program run (for tests and `view`).

use crate::value::Value;
use crate::host::PlotArtifact;

#[derive(Debug, Clone)]
pub struct RunCapture {
    pub stdout: String,
    pub value: Value,
    pub plots: Vec<PlotArtifact>,
}

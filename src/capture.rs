//! Captured program run (for tests and `view`).

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct RunCapture {
    pub stdout: String,
    pub value: Value,
}

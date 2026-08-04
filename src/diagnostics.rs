//! Diagnostics with file/line/column.
//! Expanded in M1+.

use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

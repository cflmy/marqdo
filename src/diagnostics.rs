//! Diagnostics with file/line/column.

use std::fmt;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Unified diagnostic: `path:line:col: message` (path omitted if unknown).
#[derive(Debug, Error, Clone)]
#[error("{}", self.format_message())]
pub struct Diagnostic {
    pub path: Option<PathBuf>,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn new(path: Option<&Path>, span: Span, message: impl Into<String>) -> Self {
        Self {
            path: path.map(|p| p.to_path_buf()),
            span,
            message: message.into(),
        }
    }

    pub fn at(path: &Path, span: Span, message: impl Into<String>) -> Self {
        Self::new(Some(path), span, message)
    }

    pub fn format_message(&self) -> String {
        match &self.path {
            Some(p) => format!("{}:{}: {}", p.display(), self.span, self.message),
            None => format!("{}: {}", self.span, self.message),
        }
    }
}

/// Convenience for `anyhow` sites that already have a span.
pub fn bail_at(path: Option<&Path>, span: Span, message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(Diagnostic::new(path, span, message))
}

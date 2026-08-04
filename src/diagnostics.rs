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

/// Format a path for user-facing diagnostics.
///
/// Strips the Windows extended-length prefix (`\\?\` / `\\?\UNC\`) that
/// `canonicalize` often adds, so messages stay readable.
pub fn display_path(path: &Path) -> String {
    let raw = path.display().to_string();
    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = raw.strip_prefix(r"\\?\") {
        rest.to_string()
    } else if let Some(rest) = raw.strip_prefix("//?/") {
        // Some tools emit forward-slash forms
        rest.to_string()
    } else {
        raw
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
            Some(p) => format!("{}:{}: {}", display_path(p), self.span, self.message),
            None => format!("{}: {}", self.span, self.message),
        }
    }
}

/// Convenience for `anyhow` sites that already have a span.
pub fn bail_at(path: Option<&Path>, span: Span, message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(Diagnostic::new(path, span, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_windows_extended_prefix() {
        let p = PathBuf::from(r"\\?\E:\marqdo\examples\errors\bad-arity.mq.md");
        assert_eq!(
            display_path(&p),
            r"E:\marqdo\examples\errors\bad-arity.mq.md"
        );
    }

    #[test]
    fn strips_unc_extended_prefix() {
        let p = PathBuf::from(r"\\?\UNC\server\share\file.mq.md");
        assert_eq!(display_path(&p), r"\\server\share\file.mq.md");
    }

    #[test]
    fn diagnostic_message_uses_clean_path() {
        let d = Diagnostic::at(
            Path::new(r"\\?\E:\marqdo\x.mq.md"),
            Span::new(7, 1),
            "missing argument for parameter `x`",
        );
        let msg = d.format_message();
        assert!(!msg.contains(r"\\?\"));
        assert!(msg.contains(r"E:\marqdo\x.mq.md:7:1:"));
    }
}

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

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Contract reference (Phase 2 填值；schema 源头 = 契约表 / 说明段落).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractRef {
    pub unit: String,
    pub kind: String,
    pub name: String,
    pub declared: String,
}

/// Unified diagnostic: `path:line:col: message` (path omitted if unknown).
///
/// 人类文本（`Display`）与机器面（[`Diagnostic::to_json`]）同源：凡可定位必有
/// `span`；凡契约错必带 `doc_anchor` / `doc_quote` / `contract_ref`
/// （见 `doc/roadmap/three-problems.md` §3.3 诊断协议）。
#[derive(Debug, Error, Clone)]
#[error("{}", self.format_message())]
pub struct Diagnostic {
    pub path: Option<PathBuf>,
    pub span: Span,
    pub message: String,
    /// Stable machine code (`parse.*` / `contract.*` / `runtime.*` …).
    pub code: String,
    pub severity: Severity,
    pub suggestion: Option<String>,
    pub doc_anchor: Option<String>,
    pub doc_quote: Option<String>,
    pub contract_ref: Option<ContractRef>,
}

impl Diagnostic {
    pub fn new(path: Option<&Path>, span: Span, message: impl Into<String>) -> Self {
        Self {
            path: path.map(|p| p.to_path_buf()),
            span,
            message: message.into(),
            code: "marqdo.error".to_string(),
            severity: Severity::Error,
            suggestion: None,
            doc_anchor: None,
            doc_quote: None,
            contract_ref: None,
        }
    }

    pub fn at(path: &Path, span: Span, message: impl Into<String>) -> Self {
        Self::new(Some(path), span, message)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_doc_anchor(mut self, anchor: impl Into<String>) -> Self {
        self.doc_anchor = Some(anchor.into());
        self
    }

    pub fn with_doc_quote(mut self, quote: impl Into<String>) -> Self {
        self.doc_quote = Some(quote.into());
        self
    }

    pub fn with_contract_ref(mut self, contract_ref: ContractRef) -> Self {
        self.contract_ref = Some(contract_ref);
        self
    }

    pub fn format_message(&self) -> String {
        match &self.path {
            Some(p) => format!("{}:{}: {}", display_path(p), self.span, self.message),
            None => format!("{}: {}", self.span, self.message),
        }
    }

    /// Machine-readable form（AI/MLSP 面；`run --json` 出口）。
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "code": self.code,
            "severity": match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            "message": self.message,
            "span": {
                "file": self.path.as_ref().map(|p| display_path(p)),
                "line": self.span.line,
                "col": self.span.col,
            },
            "doc_anchor": self.doc_anchor,
            "doc_quote": self.doc_quote,
            "contract_ref": self.contract_ref.as_ref().map(|c| json!({
                "unit": c.unit,
                "kind": c.kind,
                "name": c.name,
                "declared": c.declared,
            })),
            "suggestion": self.suggestion,
        })
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

    #[test]
    fn json_shape_defaults_nonempty_code_and_span() {
        let d = Diagnostic::at(Path::new("x.mq.md"), Span::new(3, 5), "boom");
        let v = d.to_json();
        assert_eq!(v["code"], "marqdo.error");
        assert_eq!(v["severity"], "error");
        assert_eq!(v["span"]["line"], 3);
        assert_eq!(v["span"]["col"], 5);
        assert_eq!(v["span"]["file"], "x.mq.md");
    }

    #[test]
    fn json_carries_doc_anchor_and_contract_ref() {
        let d = Diagnostic::at(
            Path::new("demo.mq.md"),
            Span::new(12, 3),
            "参数 `n` 期望 number，实际 text",
        )
        .with_code("contract.arg_mismatch")
        .with_doc_anchor("demo.mq.md#L4-L7")
        .with_doc_quote("| `n` | number | 输入值 |")
        .with_suggestion("把实参改为数字")
        .with_contract_ref(ContractRef {
            unit: "加一".to_string(),
            kind: "param".to_string(),
            name: "n".to_string(),
            declared: "number".to_string(),
        });
        let v = d.to_json();
        assert_eq!(v["code"], "contract.arg_mismatch");
        assert_eq!(v["doc_anchor"], "demo.mq.md#L4-L7");
        assert_eq!(v["contract_ref"]["unit"], "加一");
        assert_eq!(v["suggestion"], "把实参改为数字");
        // Display 零回归
        assert!(d.format_message().contains("demo.mq.md:12:3:"));
    }
}

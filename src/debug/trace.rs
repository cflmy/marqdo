//! Structured eval trace (P3-D1): JSON lines on stderr.

use std::path::Path;

use crate::diagnostics::{display_path, Span};

/// Emit one JSON object line to stderr when tracing.
pub fn emit_trace(path: Option<&Path>, span: Option<Span>, event: &str, fields: &[(&str, &str)]) {
    let mut out = String::from("{");
    out.push_str(&format!("\"event\":{}", json_str(event)));
    if let Some(p) = path {
        out.push_str(&format!(",\"path\":{}", json_str(&display_path(p))));
    }
    if let Some(sp) = span {
        out.push_str(&format!(",\"span\":{}", json_str(&sp.to_string())));
    }
    for (k, v) in fields {
        out.push_str(&format!(",\"{}\":{}", k, json_str(v)));
    }
    out.push('}');
    eprintln!("{out}");
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_str_escapes() {
        assert_eq!(json_str("a\"b"), r#""a\"b""#);
    }
}

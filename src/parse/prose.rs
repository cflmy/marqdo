//! Inline prose scan for Markup v0.3 (叙述中的 `` `名` `` / `**代码**` / `*返回*`).

use crate::lex::{matching_bold_close_inline, matching_italic_close_inline};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProseBit {
    /// `` `ident` `` or `` `ident`=default `` (default string is raw RHS text).
    Decl {
        name: String,
        default: Option<String>,
        /// Byte offset in the line (for diagnostics).
        at: usize,
    },
    /// Inner text of `**…**` (code segment).
    BoldCode { inner: String, at: usize },
    /// Inner text of `*…*` (return), when not starting a bold.
    ItalicReturn { inner: String, at: usize },
}

/// Scan a narrative (or mixed) line for v0.3 inline bits, left to right.
pub fn scan_prose_line(line: &str) -> Vec<ProseBit> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'`' {
            if let Some((bit, next)) = scan_backtick_decl(line, i) {
                out.push(bit);
                i = next;
                continue;
            }
            i += 1;
            continue;
        }
        if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let rest = &line[i..];
            if let Some(close_rel) = matching_bold_close_inline(rest) {
                // close_rel is index within `rest` of first `*` of closing `**`
                let inner = rest[2..close_rel].to_string();
                out.push(ProseBit::BoldCode {
                    inner,
                    at: i,
                });
                i += close_rel + 2;
                continue;
            }
            i += 1;
            continue;
        }
        if c == b'*' {
            let rest = &line[i..];
            if rest.starts_with("**") {
                i += 1;
                continue;
            }
            if let Some(close_rel) = matching_italic_close_inline(rest) {
                let inner = rest[1..close_rel].to_string();
                out.push(ProseBit::ItalicReturn {
                    inner,
                    at: i,
                });
                i += close_rel + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn scan_backtick_decl(line: &str, start: usize) -> Option<(ProseBit, usize)> {
    let rest = &line[start + 1..];
    let end = rest.find('`')?;
    let name = rest[..end].to_string();
    if name.is_empty() || name.chars().any(|c| c.is_whitespace()) {
        return None;
    }
    let after_tick = start + 1 + end + 1;
    let after = line.get(after_tick..)?;
    // `` `名`=默认 `` — optional default glued after closing tick
    if let Some(stripped) = after.strip_prefix('=') {
        // default runs until whitespace or CJK punctuation-ish break for prose
        let def_end = stripped
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(c, '，' | '。' | '；' | '、' | ',' | ';' | '*' | '`')
            })
            .unwrap_or(stripped.len());
        let default = stripped[..def_end].to_string();
        let next = after_tick + 1 + def_end;
        return Some((
            ProseBit::Decl {
                name,
                default: Some(default),
                at: start,
            },
            next,
        ));
    }
    Some((
        ProseBit::Decl {
            name,
            default: None,
            at: start,
        },
        after_tick,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_inc_prose_line() {
        let line = "对于输入变量`n`,我们执行操作**n=n+1**接着返回*n*。";
        let bits = scan_prose_line(line);
        assert_eq!(bits.len(), 3);
        match &bits[0] {
            ProseBit::Decl { name, default, .. } => {
                assert_eq!(name, "n");
                assert!(default.is_none());
            }
            _ => panic!("{:?}", bits[0]),
        }
        match &bits[1] {
            ProseBit::BoldCode { inner, .. } => assert_eq!(inner, "n=n+1"),
            _ => panic!("{:?}", bits[1]),
        }
        match &bits[2] {
            ProseBit::ItalicReturn { inner, .. } => assert_eq!(inner, "n"),
            _ => panic!("{:?}", bits[2]),
        }
    }

    #[test]
    fn scan_default_decl() {
        let line = "默认`礼貌`=False。";
        let bits = scan_prose_line(line);
        assert_eq!(bits.len(), 1);
        match &bits[0] {
            ProseBit::Decl { name, default, .. } => {
                assert_eq!(name, "礼貌");
                assert_eq!(default.as_deref(), Some("False"));
            }
            _ => panic!("{:?}", bits[0]),
        }
    }
}

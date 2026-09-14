//! Inline prose scan for Markup v0.3 (叙述中的 `` `名` `` / `**代码**` / `*返回*`).
//!
//! Soft Markdown emphasis that does not look like code/return is skipped (G2 —
//! [code-as-docs-gaps.md](../../../doc/design/code-as-docs-gaps.md)).

use crate::lex::{matching_bold_close_inline, matching_italic_close_inline};

pub use crate::lex::looks_like_bold_code;

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
            // Non-decl span (e.g. `` `OPENAI_*` ``): skip whole ticks so inner `*`
            // does not open italic/bold.
            if let Some(rel) = line[i + 1..].find('`') {
                i = i + 1 + rel + 1;
                continue;
            }
            i += 1;
            continue;
        }
        if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let rest = &line[i..];
            if let Some(close_rel) = matching_bold_close_inline(rest) {
                let inner = rest[2..close_rel].to_string();
                let next = i + close_rel + 2;
                if looks_like_bold_code(&inner) {
                    out.push(ProseBit::BoldCode { inner, at: i });
                }
                i = next;
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
                let next = i + close_rel + 1;
                if looks_like_italic_return(line, i, next, &inner) {
                    out.push(ProseBit::ItalicReturn { inner, at: i });
                }
                i = next;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Whether `*inner*` is a return (vs mid-prose English/CJK emphasis).
fn looks_like_italic_return(line: &str, at: usize, after: usize, inner: &str) -> bool {
    let t = inner.trim();
    if t.is_empty()
        || matches!(
            t,
            "None" | "无" | "空" | "True" | "False" | "真" | "假" | "true" | "false"
        )
    {
        return true;
    }
    if t.starts_with('>')
        || t.contains('=')
        || t.chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '[' | ']'))
    {
        return true;
    }
    // Whole-line return: only ws outside the italic span.
    let before = line[..at].trim();
    let after_s = line.get(after..).unwrap_or("").trim();
    if before.is_empty() && after_s.is_empty() {
        return true;
    }
    // Narration "返回*n*" / "return *n*"
    if before.ends_with("返回") || before.ends_with("return") || before.ends_with("Return") {
        return true;
    }
    // Mid-prose simple word → soft emphasis (G2).
    if is_simple_emphasis_word(t) {
        return false;
    }
    // Multi-token or dotted id mid-line: keep as return (rare).
    true
}

fn is_simple_emphasis_word(t: &str) -> bool {
    if t.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    t.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
}

fn scan_backtick_decl(line: &str, start: usize) -> Option<(ProseBit, usize)> {
    let rest = &line[start + 1..];
    let end = rest.find('`')?;
    let name = rest[..end].to_string();
    if name.is_empty() || !is_prose_decl_name(&name) {
        return None;
    }
    let after_tick = start + 1 + end + 1;
    let after = line.get(after_tick..)?;
    if let Some(stripped) = after.strip_prefix('=') {
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

/// Identifiers only: letters / digits / `_` / non-ASCII (CJK). Rejects `` `n+1` ``.
fn is_prose_decl_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_' || !first.is_ascii()) {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
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

    #[test]
    fn soft_bold_emphasis_skipped() {
        let bits = scan_prose_line("本文件既是**说明**也是**程序**。");
        assert!(bits.is_empty(), "{bits:?}");
        assert!(!looks_like_bold_code("说明"));
        assert!(looks_like_bold_code("n=n+1"));
        assert!(looks_like_bold_code("加一 37"));
    }

    #[test]
    fn soft_italic_emphasis_skipped() {
        let bits = scan_prose_line("When soft=True, *only* FIND continues.");
        assert!(bits.is_empty(), "{bits:?}");
        let bits = scan_prose_line("*总额*");
        assert!(
            matches!(&bits[..], [ProseBit::ItalicReturn { inner, .. }] if inner == "总额"),
            "{bits:?}"
        );
    }

    #[test]
    fn reject_n_plus_one_as_decl() {
        let bits = scan_prose_line("输入`n+1`然后继续。");
        assert!(bits.is_empty(), "`` `n+1` `` must not Decl: {bits:?}");
    }

    #[test]
    fn skip_glob_backticks_without_italic() {
        let bits = scan_prose_line("from `OPENAI_*` / `MARQDO_LLM_*`.");
        assert!(
            bits.is_empty(),
            "glob-like ticks must not yield italic: {bits:?}"
        );
    }
}

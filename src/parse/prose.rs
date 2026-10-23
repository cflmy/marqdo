//! Inline prose scan for Markup v0.3 (叙述中的 `` `名` `` / `**代码**` / `*返回*`).
//!
//! Soft Markdown emphasis that does not look like code/return is skipped (G2 —
//! [code-as-docs-gaps.md](../../../doc/design/code-as-docs-gaps.md)).
//!
//! Comment-line Markdown links `[label](target)` are NarrativeRef / navigation
//! sites (G03); they are never executed.

use crate::lex::{matching_bold_close_inline, matching_italic_close_inline, LineKind};

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

/// Why a comment-line `[]()` is not an executable BoundCall (G03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrativeLinkKind {
    /// In-project symbol / binding / `#anchor` — inert explanation.
    Narrate,
    /// URL or file path — navigation only; never fetched by compile/run.
    Navigation,
}

impl NarrativeLinkKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Narrate => "narrate",
            Self::Navigation => "navigation",
        }
    }
}

/// One Markdown link found on a comment (narrative) line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeLink {
    pub label: String,
    pub target: String,
    pub kind: NarrativeLinkKind,
    /// 1-based source line.
    pub line: u32,
    pub at: usize,
}

/// Classify a link target without parsing it as an expression.
pub fn classify_narrative_target(target: &str) -> NarrativeLinkKind {
    let t = target.trim();
    if t.starts_with("http://")
        || t.starts_with("https://")
        || t.starts_with("mailto:")
        || t.starts_with("//")
    {
        return NarrativeLinkKind::Navigation;
    }
    if t.contains('/') || t.ends_with(".md") || t.ends_with(".mq.md") || t.ends_with(".html")
    {
        return NarrativeLinkKind::Navigation;
    }
    NarrativeLinkKind::Narrate
}

/// Scan whole-source comment lines for Markdown links; skip fences and code lines.
pub fn scan_narrative_links(source: &str) -> Vec<NarrativeLink> {
    let classified = crate::lex::classify_source(source);
    let mut out = Vec::new();
    let mut in_fence = false;
    for (i, cl) in classified.iter().enumerate() {
        let line_no = (i + 1) as u32;
        let trimmed = cl.text.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if cl.kind != LineKind::Comment {
            continue;
        }
        out.extend(scan_md_links_in_line(&cl.text, line_no));
    }
    out
}

/// Scan one line for `[label](target)`, skipping `` `inline` `` spans.
pub fn scan_md_links_in_line(line: &str, line_no: u32) -> Vec<NarrativeLink> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        if bytes[i] == b'`' {
            if let Some(rel) = line[i + 1..].find('`') {
                i = i + 1 + rel + 1;
                continue;
            }
            i += 1;
            continue;
        }
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        let Some((label, target, next)) = parse_md_link(line, i) else {
            i += 1;
            continue;
        };
        let kind = classify_narrative_target(&target);
        out.push(NarrativeLink {
            label,
            target,
            kind,
            line: line_no,
            at: i,
        });
        i = next;
    }
    out
}

fn parse_md_link(line: &str, start: usize) -> Option<(String, String, usize)> {
    if !line[start..].starts_with('[') {
        return None;
    }
    let mut depth = 0i32;
    let mut label_end = None;
    for (rel, ch) in line[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    label_end = Some(start + rel);
                    break;
                }
            }
            _ => {}
        }
    }
    let label_end = label_end?;
    let after = line.get(label_end + 1..)?;
    if !after.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    let mut target_end = None;
    for (rel, ch) in after.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    target_end = Some(rel);
                    break;
                }
            }
            _ => {}
        }
    }
    let target_end = target_end?;
    let label = line[start + 1..label_end].to_string();
    let target = after[1..target_end].trim().to_string();
    if label.is_empty() || target.is_empty() {
        return None;
    }
    // Executable occlusion payloads look like calls (`foo(…)` / `a.b(…)`); those
    // live on Code lines already. If a comment link target is expression-shaped
    // with `(`, still treat as narrate/navigation by target class — do not eval.
    let next = label_end + 1 + target_end + 1;
    Some((label, target, next))
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
///
/// Design §12 #5: mid-line italic is return only for whole-line, `返回*…*`,
/// or an expression-shaped span — not `*very important*` / `*foo.bar*`.
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
    // Whole-line return: only ws outside the italic span (`*n*` / `*总额*`).
    let before = line[..at].trim();
    let after_s = line.get(after..).unwrap_or("").trim();
    if before.is_empty() && after_s.is_empty() {
        return true;
    }
    // Narration "返回*n*" / "return *n*"
    if before.ends_with("返回") || before.ends_with("return") || before.ends_with("Return") {
        return true;
    }
    // Mid-prose: expression-shaped only (`*n + 1*` / `*x=1*`); not hyphenated emphasis.
    if is_hyphenated_emphasis(t) {
        return false;
    }
    t.starts_with('>')
        || t.contains('=')
        || t.chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '[' | ']'))
}

fn is_hyphenated_emphasis(t: &str) -> bool {
    !t.chars().any(char::is_whitespace)
        && t.contains('-')
        && !t
            .chars()
            .any(|c| matches!(c, '+' | '*' | '/' | '(' | ')' | '[' | ']' | '='))
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
        let bits = scan_prose_line("This is *very important* documentation.");
        assert!(bits.is_empty(), "multi-word italic is emphasis: {bits:?}");
        let bits = scan_prose_line("See *foo.bar* in the text.");
        assert!(bits.is_empty(), "dotted italic is emphasis: {bits:?}");
        let bits = scan_prose_line("Use *code-as-docs* style.");
        assert!(bits.is_empty(), "hyphenated italic is emphasis: {bits:?}");
        let bits = scan_prose_line("*总额*");
        assert!(
            matches!(&bits[..], [ProseBit::ItalicReturn { inner, .. }] if inner == "总额"),
            "{bits:?}"
        );
        let bits = scan_prose_line("接着返回*n*。");
        assert!(
            matches!(&bits[..], [ProseBit::ItalicReturn { inner, .. }] if inner == "n"),
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

    #[test]
    fn narrative_links_skip_fence_and_classify() {
        let src = "---\nmarkup: occlusion\n---\n\n# main\n\n见[文档](https://example.com/a)与[总额](总额)。\n\n```\n[ignored](https://x.test)\n```\n\n行内跳过 `[not](x)` 标记。\n";
        let links = scan_narrative_links(src);
        assert!(
            links.iter().any(|l| {
                l.kind == NarrativeLinkKind::Navigation && l.target.contains("example.com")
            }),
            "{links:?}"
        );
        assert!(
            links
                .iter()
                .any(|l| l.kind == NarrativeLinkKind::Narrate && l.target == "总额"),
            "{links:?}"
        );
        assert!(
            !links.iter().any(|l| l.target.contains("x.test")),
            "fenced link must not become a step: {links:?}"
        );
        assert!(
            !links.iter().any(|l| l.target == "x"),
            "inline-code link must not become a step: {links:?}"
        );
    }
}

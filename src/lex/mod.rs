//! Line classification (M1) — no Flex.
//!
//! Narrative comments are paragraph-scoped: blank lines separate paragraphs.
//! After a comment line, subsequent non-blank lines stay comments until a blank
//! (so mid-paragraph lines starting with `` ` `` are not treated as code).

use std::fmt;

/// Kind of a source line after skipping leading indentation for the test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Blank,
    Comment,
    /// Jupyter-style persisted output (`<!-- marqdo-out … -->`).
    Writeback,
    Code,
}

impl fmt::Display for LineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineKind::Blank => write!(f, "Blank"),
            LineKind::Comment => write!(f, "Comment"),
            LineKind::Writeback => write!(f, "Writeback"),
            LineKind::Code => write!(f, "Code"),
        }
    }
}

/// One classified source line (1-based line number).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedLine {
    pub line_no: u32,
    pub kind: LineKind,
    pub text: String,
}

/// Characters that start a code line when they appear as the first non-ws char.
fn is_code_starter(c: char) -> bool {
    matches!(
        c,
        '#' | '*' | '>' | '+' | '-' | '`' | '|' | '~' | '$'
    ) || c.is_ascii_digit()
}

/// `*` / `**` lines that are Marqdo executable (statement / return / else), not narrative Markdown.
fn is_marqdo_star_line(trimmed: &str) -> bool {
    if trimmed == "*" || trimmed == "****" {
        return true;
    }
    if trimmed.starts_with("**") {
        if trimmed.trim() == "** **" {
            return true;
        }
        if let Some(rest) = trimmed.strip_prefix("**") {
            if let Some(pos) = rest.find("**") {
                return rest[pos + 2..].trim().is_empty();
            }
        }
        return false;
    }
    trimmed.starts_with('*') && trimmed.ends_with('*') && trimmed.len() >= 2
}

/// Classify a single logical line (no trailing newline required).
///
/// This is the **paragraph-start** rule only. Prefer [`classify_source`] for
/// full-file classification (paragraph continuation).
pub fn classify_line(text: &str) -> LineKind {
    let trimmed = text.trim_end_matches(['\r', '\n']);
    if trimmed.chars().all(|c| c.is_whitespace()) {
        return LineKind::Blank;
    }
    let trimmed = text.trim();
    let first = trimmed.chars().find(|c| !c.is_whitespace());
    match first {
        None => LineKind::Blank,
        Some('*') if !is_marqdo_star_line(trimmed) => LineKind::Comment,
        Some(c) if is_code_starter(c) => LineKind::Code,
        Some(_) => LineKind::Comment,
    }
}

/// Classify every line of a source file (blank-line paragraph comments).
///
/// Structural end markers (`---` / `***` / empty `****`) always stay Code and
/// break a comment paragraph, so frontmatter closers and function ends work
/// even when adjacent to narrative.
pub fn classify_source(source: &str) -> Vec<ClassifiedLine> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = if source.is_empty() {
        Vec::new()
    } else {
        source.lines().collect()
    };

    let mut out = Vec::with_capacity(lines.len());
    let mut in_comment_paragraph = false;
    let mut in_math_fence = false;
    let mut in_md_fence = false;
    let mut in_writeback = false;

    for (i, text) in lines.into_iter().enumerate() {
        let base = classify_line(text);
        let trimmed = text.trim();
        let structural = is_structural_code_line(trimmed);
        let kind = if in_writeback {
            if trimmed.contains("-->") {
                in_writeback = false;
            }
            LineKind::Writeback
        } else if in_math_fence {
            if trimmed == "$$" {
                in_math_fence = false;
            }
            LineKind::Code
        } else if in_md_fence {
            // Markdown ``` fences are narrative / foreign sources — not Marqdo stmts.
            if is_md_fence_line(trimmed) {
                in_md_fence = false;
            }
            LineKind::Comment
        } else if is_md_fence_opener(trimmed) {
            in_md_fence = true;
            in_comment_paragraph = false;
            LineKind::Comment
        } else if trimmed.starts_with("<!--") && trimmed.contains("marqdo-out") {
            in_writeback = true;
            in_comment_paragraph = false;
            if trimmed.contains("-->") {
                in_writeback = false;
            }
            LineKind::Writeback
        } else {
            match base {
                LineKind::Blank => {
                    in_comment_paragraph = false;
                    LineKind::Blank
                }
                LineKind::Comment => {
                    in_comment_paragraph = true;
                    LineKind::Comment
                }
                LineKind::Code if structural => {
                    in_comment_paragraph = false;
                    LineKind::Code
                }
                LineKind::Code if in_comment_paragraph => LineKind::Comment,
                LineKind::Writeback => LineKind::Writeback,
                LineKind::Code => {
                    // Multi-line math fence opener (not same-line `$$…$$`).
                    if trimmed == "$$" {
                        in_math_fence = true;
                    }
                    LineKind::Code
                }
            }
        };
        out.push(ClassifiedLine {
            line_no: (i + 1) as u32,
            kind,
            text: text.to_string(),
        });
    }
    out
}

fn is_md_fence_opener(trimmed: &str) -> bool {
    trimmed.starts_with("```")
        && !trimmed
            .chars()
            .skip_while(|c| *c == '`')
            .all(|c| c.is_whitespace())
}

fn is_md_fence_line(trimmed: &str) -> bool {
    trimmed.starts_with("```")
}

/// Function-end / frontmatter HR or empty bold return — never swallowed by paragraphs.
fn is_structural_code_line(trimmed: &str) -> bool {
    if trimmed.len() >= 3
        && (trimmed.chars().all(|c| c == '-') || trimmed.chars().all(|c| c == '*'))
    {
        return true;
    }
    false
}

/// Human-readable dump for `--dump-lines`.
pub fn format_lines_dump(path: &str, lines: &[ClassifiedLine]) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== marqdo: lines ({path}) ===\n"));
    for line in lines {
        let preview = if line.text.len() > 80 {
            format!("{}…", &line.text.chars().take(80).collect::<String>())
        } else {
            line.text.clone()
        };
        match line.kind {
            LineKind::Blank => {
                out.push_str(&format!("{:4}  {:7}\n", line.line_no, line.kind));
            }
            _ => {
                out.push_str(&format!(
                    "{:4}  {:7}  | {}\n",
                    line.line_no, line.kind, preview
                ));
            }
        }
    }
    out.push_str("=== marqdo: end lines ===\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_and_comment_and_code() {
        assert_eq!(classify_line(""), LineKind::Blank);
        assert_eq!(classify_line("   "), LineKind::Blank);
        assert_eq!(classify_line("hello world"), LineKind::Comment);
        assert_eq!(classify_line("  中文说明"), LineKind::Comment);
        assert_eq!(classify_line("> print text=hi"), LineKind::Code);
        assert_eq!(classify_line("# main"), LineKind::Code);
        assert_eq!(classify_line("*`x` = 1*"), LineKind::Code);
        assert_eq!(classify_line("**ret**"), LineKind::Code);
        assert_eq!(classify_line("+ `x` > 0"), LineKind::Code);
        assert_eq!(classify_line("- item"), LineKind::Code);
        assert_eq!(classify_line("1. arm"), LineKind::Code);
        assert_eq!(classify_line("| a |"), LineKind::Code);
        assert_eq!(classify_line("`x` = 1"), LineKind::Code);
    }

    #[test]
    fn narrative_bold_in_comment_paragraph() {
        let src = "说明如下：\n\n**强调信息**是什么什么\n\n> print text=ok\n";
        let lines = classify_source(src);
        assert_eq!(lines[0].kind, LineKind::Comment);
        assert_eq!(lines[1].kind, LineKind::Blank);
        assert_eq!(lines[2].kind, LineKind::Comment, "bold prose is comment");
        assert_eq!(lines[3].kind, LineKind::Blank);
        assert_eq!(lines[4].kind, LineKind::Code);
    }

    #[test]
    fn writeback_block_classified() {
        let src = "*`x` = 1*\n<!-- marqdo-out @line=1\n3\n-->\n";
        let lines = classify_source(src);
        assert_eq!(lines[0].kind, LineKind::Code);
        assert_eq!(lines[1].kind, LineKind::Writeback);
        assert_eq!(lines[2].kind, LineKind::Writeback);
        assert_eq!(lines[3].kind, LineKind::Writeback);
    }

    #[test]
    fn paragraph_comment_keeps_backtick_continuation() {
        let src = "采用了\n`.mq.md`的双层后缀\n\n> print text=ok\n";
        let lines = classify_source(src);
        assert_eq!(lines[0].kind, LineKind::Comment);
        assert_eq!(lines[1].kind, LineKind::Comment);
        assert_eq!(lines[2].kind, LineKind::Blank);
        assert_eq!(lines[3].kind, LineKind::Code);
    }

    #[test]
    fn frontmatter_closing_hr_stays_code() {
        let src = "---\ntitle: x\n---\n\n# main\n";
        let lines = classify_source(src);
        assert_eq!(lines[0].kind, LineKind::Code);
        assert_eq!(lines[1].kind, LineKind::Comment);
        assert_eq!(lines[2].kind, LineKind::Code);
        assert_eq!(lines[2].text.trim(), "---");
    }

    #[test]
    fn classify_hello_fixture() {
        let src = include_str!("../../tests/structure/hello.mq.md");
        let lines = classify_source(src);
        assert!(lines.iter().any(|l| l.kind == LineKind::Comment));
        assert!(lines.iter().any(|l| l.kind == LineKind::Code));
    }

    #[test]
    fn math_fence_body_stays_code() {
        let lines = classify_source("`f` =\n$$\nx^2 - 2\n$$\n");
        assert!(
            lines
                .iter()
                .all(|l| l.kind == LineKind::Code || l.kind == LineKind::Blank),
            "expected fence body as Code, got {lines:?}"
        );
    }

    #[test]
    fn star_line_classification_is_star_terminated() {
        // A `*…*` Marqdo statement line must END with `*` to be treated as code.
        // If it accidentally ends with a backtick (e.g. `table="t"`), the line
        // falls back to narrative Comment and swallows the following paragraph.
        // Regression: `store.count` calls after a blank line were being dropped.
        let star_term = "*c = > `store`.count table=\"t\"*";
        assert!(
            is_marqdo_star_line(star_term),
            "star-terminated assignment must be code"
        );
        assert_eq!(classify_line(star_term), LineKind::Code);

        let backtick_term = "*c = > `store`.count table=\"t\"`";
        assert!(
            !is_marqdo_star_line(backtick_term),
            "backtick-terminated assignment must NOT be marqdo star line"
        );
        assert_eq!(classify_line(backtick_term), LineKind::Comment);

        // In paragraph context the misplaced line must not stay Code.
        let src = "> `store`.insert table=t rows=`数据`\n\n*c = > `store`.count table=\"t\"`\n> print text=count1-done\n";
        let lines = classify_source(src);
        assert_eq!(lines[2].kind, LineKind::Comment, "bad terminator → comment");
    }
}

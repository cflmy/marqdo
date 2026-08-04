//! Line classification (M1) — no Flex.
//!
//! Unmarked lines are comments; lines whose first non-whitespace character
//! opens a Marqdo/Markdown marker are code.

use std::fmt;

/// Kind of a source line after skipping leading indentation for the test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Blank,
    Comment,
    Code,
}

impl fmt::Display for LineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineKind::Blank => write!(f, "Blank"),
            LineKind::Comment => write!(f, "Comment"),
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
        '#' | '*' | '>' | '+' | '-' | '`' | '|' | '~'
    ) || c.is_ascii_digit()
}

/// Classify a single logical line (no trailing newline required).
pub fn classify_line(text: &str) -> LineKind {
    let trimmed = text.trim_end_matches(['\r', '\n']);
    if trimmed.chars().all(|c| c.is_whitespace()) {
        return LineKind::Blank;
    }
    let first = trimmed.chars().find(|c| !c.is_whitespace());
    match first {
        None => LineKind::Blank,
        Some(c) if is_code_starter(c) => LineKind::Code,
        Some(_) => LineKind::Comment,
    }
}

/// Classify every line of a source file.
pub fn classify_source(source: &str) -> Vec<ClassifiedLine> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = if source.is_empty() {
        Vec::new()
    } else {
        source.lines().collect()
    };

    lines
        .into_iter()
        .enumerate()
        .map(|(i, text)| ClassifiedLine {
            line_no: (i + 1) as u32,
            kind: classify_line(text),
            text: text.to_string(),
        })
        .collect()
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
    fn classify_hello_fixture() {
        let src = include_str!("../../examples/structure/hello.mq.md");
        let lines = classify_source(src);
        assert!(lines.iter().any(|l| l.kind == LineKind::Code && l.text.contains("print")));
        assert!(lines.iter().any(|l| l.kind == LineKind::Comment));
    }
}

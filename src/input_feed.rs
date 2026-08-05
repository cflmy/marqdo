//! Preset stdin lines for `input` (CLI `--stdin-file`, view form, frontmatter, tests).

use std::collections::VecDeque;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use anyhow::{Context, Result};

/// Split text into input lines (strips `\n` / `\r\n`; empty → no lines).
pub fn split_stdin_text(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    text.lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect()
}

pub fn load_stdin_file(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read stdin file {}", path.display()))?;
    Ok(split_stdin_text(&text))
}

/// Prefer explicit override (CLI / view form); else frontmatter `stdin:` / `输入:`.
pub fn effective_stdin(source: &str, override_lines: &[String]) -> Vec<String> {
    if !override_lines.is_empty() {
        return override_lines.to_vec();
    }
    extract_frontmatter_stdin(source)
}

/// Read demo / preset lines from YAML-ish frontmatter.
///
/// Supported shapes:
/// - `stdin: Alice` / `输入: 小明`
/// - block scalar:
///   ```text
///   stdin: |
///     Alice
///     Bob
///   ```
/// - list:
///   ```text
///   stdin:
///     - Alice
///     - Bob
///   ```
pub fn extract_frontmatter_stdin(source: &str) -> Vec<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return Vec::new();
    }

    let mut i = 1usize;
    while i < lines.len() {
        let raw = lines[i];
        let t = raw.trim();
        if t == "---" {
            break;
        }

        let (key, rest) = match split_fm_key(t) {
            Some(pair) => pair,
            None => {
                i += 1;
                continue;
            }
        };
        if key != "stdin" && key != "输入" {
            i += 1;
            continue;
        }

        let rest = rest.trim();
        if rest == "|" || rest == "|-" || rest == "|+" {
            i += 1;
            let mut out = Vec::new();
            while i < lines.len() {
                let body = lines[i];
                let bt = body.trim();
                if bt == "---" || (!body.starts_with(' ') && !body.starts_with('\t') && !bt.is_empty())
                {
                    break;
                }
                if bt.is_empty() && (body.is_empty() || body.chars().all(|c| c == ' ' || c == '\t')) {
                    out.push(String::new());
                } else {
                    out.push(strip_block_indent(body));
                }
                i += 1;
            }
            return out;
        }
        if rest.is_empty() {
            i += 1;
            let mut out = Vec::new();
            while i < lines.len() {
                let body = lines[i];
                let bt = body.trim();
                if bt == "---" {
                    break;
                }
                if let Some(item) = bt.strip_prefix("- ") {
                    out.push(item.trim().trim_matches('"').to_string());
                    i += 1;
                    continue;
                }
                if bt == "-" {
                    out.push(String::new());
                    i += 1;
                    continue;
                }
                // end of list (next key or blank then key)
                if !body.starts_with(' ') && !body.starts_with('\t') && !bt.is_empty() {
                    break;
                }
                if bt.is_empty() {
                    i += 1;
                    continue;
                }
                break;
            }
            return out;
        }
        return vec![rest.trim_matches('"').to_string()];
    }
    Vec::new()
}

fn split_fm_key(t: &str) -> Option<(&str, &str)> {
    let (k, v) = t.split_once(':')?;
    let k = k.trim();
    if k.is_empty() || k.starts_with('>') {
        return None;
    }
    Some((k, v))
}

fn strip_block_indent(line: &str) -> String {
    let s = line.strip_prefix("  ").or_else(|| line.strip_prefix('\t')).unwrap_or(line);
    s.trim_end_matches('\r').to_string()
}

#[derive(Debug, Clone, Default)]
pub struct InputFeed {
    queue: VecDeque<String>,
    capture: bool,
}

impl InputFeed {
    pub fn new(capture: bool, lines: impl Into<Vec<String>>) -> Self {
        Self {
            queue: lines.into().into(),
            capture,
        }
    }

    /// Next line from the preset queue, or real stdin when not capturing.
    pub fn read_line(&mut self) -> Result<String> {
        if let Some(line) = self.queue.pop_front() {
            return Ok(line);
        }
        if self.capture {
            anyhow::bail!(
                "input needs a line (frontmatter stdin:/输入:, view Preset input, or --stdin-file)"
            );
        }
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Ok(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{run_file_capture, RunOptions};

    #[test]
    fn capture_accepts_preset_stdin() {
        let cap = run_file_capture(
            Path::new("tests/keywords/input.mq.md"),
            &RunOptions {
                stdin_lines: vec!["Alice".into()],
                ..RunOptions::default()
            },
        )
        .expect("capture with stdin");
        assert_eq!(cap.stdout.trim_end(), "Name:Hello Alice!");
    }

    #[test]
    fn capture_uses_frontmatter_stdin() {
        let src = "---\ntitle: t\nstdin: Ada\n---\n\n# main\n\n*`n` = > input*\n\n> print text=`n`\n";
        let dir = std::env::temp_dir().join("marqdo-fm-stdin-test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("fm.mq.md");
        fs::write(&path, src).unwrap();
        let cap = run_file_capture(&path, &RunOptions::default()).expect("fm stdin");
        assert_eq!(cap.stdout.trim_end(), "Ada");
    }

    #[test]
    fn extract_block_and_list() {
        assert_eq!(
            extract_frontmatter_stdin("---\nstdin: |\n  a\n  b\n---\n"),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            extract_frontmatter_stdin("---\n输入:\n  - 甲\n  - 乙\n---\n"),
            vec!["甲".to_string(), "乙".to_string()]
        );
        assert_eq!(
            extract_frontmatter_stdin("---\nstdin: Zoe\n---\n"),
            vec!["Zoe".to_string()]
        );
    }

    #[test]
    fn split_keeps_blank_middle_lines() {
        assert_eq!(
            split_stdin_text("a\n\nb\n"),
            vec!["a".to_string(), String::new(), "b".to_string()]
        );
    }
}

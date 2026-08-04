//! Preset stdin lines for `input` (CLI `--stdin-file`, view form, tests).

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
            anyhow::bail!("input is not available under capture / view");
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
    fn split_keeps_blank_middle_lines() {
        assert_eq!(
            split_stdin_text("a\n\nb\n"),
            vec!["a".to_string(), String::new(), "b".to_string()]
        );
    }
}

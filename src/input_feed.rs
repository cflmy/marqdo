//! Preset stdin lines for `input` (CLI `--stdin-file`, view form, frontmatter, tests).
//! Interactive TTY path uses Unicode-aware line editing (char backspace + redraw).

use std::collections::VecDeque;
use std::fs;
use std::io::{self, BufRead, Write};
#[cfg(all(unix, feature = "tty"))]
use std::io::Read;
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

pub fn stdin_is_tty() -> bool {
    #[cfg(all(unix, feature = "tty"))]
    {
        unsafe { libc::isatty(libc::STDIN_FILENO) == 1 }
    }
    #[cfg(not(all(unix, feature = "tty")))]
    {
        false
    }
}

/// Piped / non-TTY: read raw bytes to newline (avoids mid-frame UTF-8 panic), lossy decode.
fn read_stdin_line_piped() -> Result<String> {
    let mut buf = Vec::new();
    io::stdin().lock().read_until(b'\n', &mut buf)?;
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Ensure prompt has a trailing space so typed text does not glue to the last glyph.
fn normalize_prompt(prompt: &str) -> String {
    if prompt.is_empty() {
        return String::new();
    }
    if prompt.ends_with(|c: char| c.is_whitespace()) {
        prompt.to_string()
    } else {
        format!("{prompt} ")
    }
}

#[cfg(all(unix, feature = "tty"))]
struct TermGuard {
    fd: i32,
    original: libc::termios,
}

#[cfg(all(unix, feature = "tty"))]
impl TermGuard {
    fn enter_cbreak() -> Result<Self> {
        let fd = libc::STDIN_FILENO;
        let mut original = unsafe { std::mem::zeroed::<libc::termios>() };
        if unsafe { libc::tcgetattr(fd, &mut original) } != 0 {
            return Err(io::Error::last_os_error()).context("tcgetattr");
        }
        let mut raw = original;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error()).context("tcsetattr");
        }
        Ok(Self { fd, original })
    }
}

#[cfg(all(unix, feature = "tty"))]
impl Drop for TermGuard {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSANOW, &self.original);
        }
    }
}

#[cfg(all(unix, feature = "tty"))]
fn redraw_line(prompt: &str, buf: &str) -> Result<()> {
    let mut out = io::stdout().lock();
    // CR + clear whole line, then reprint (fixes CJK backspace display desync).
    write!(out, "\r\x1b[2K{prompt}{buf}")?;
    out.flush()?;
    Ok(())
}

/// Interactive line editor: UTF-8 char backspace + full-line redraw.
#[cfg(all(unix, feature = "tty"))]
fn read_stdin_line_interactive(prompt: &str) -> Result<String> {
    let prompt = normalize_prompt(prompt);
    let _guard = TermGuard::enter_cbreak()?;
    redraw_line(&prompt, "")?;

    let mut buf = String::new();
    let mut pending = Vec::new();
    let mut stdin = io::stdin().lock();
    let mut byte = [0u8; 1];

    loop {
        let n = stdin.read(&mut byte)?;
        if n == 0 {
            // EOF
            writeln!(io::stdout())?;
            return Ok(buf);
        }
        let b = byte[0];
        match b {
            b'\n' | b'\r' => {
                writeln!(io::stdout())?;
                return Ok(buf);
            }
            0x7f | 0x08 => {
                // Backspace / BS — delete one Unicode scalar, not one byte.
                if buf.pop().is_some() {
                    redraw_line(&prompt, &buf)?;
                }
            }
            0x03 => {
                // Ctrl-C
                writeln!(io::stdout())?;
                anyhow::bail!("interrupted");
            }
            0x04 => {
                // Ctrl-D
                if buf.is_empty() && pending.is_empty() {
                    writeln!(io::stdout())?;
                    return Ok(buf);
                }
            }
            b if b < 0x20 => {
                // ignore other controls
            }
            b => {
                pending.push(b);
                match std::str::from_utf8(&pending) {
                    Ok(s) => {
                        buf.push_str(s);
                        pending.clear();
                        redraw_line(&prompt, &buf)?;
                    }
                    Err(e) if e.error_len().is_none() => {
                        // incomplete UTF-8 sequence — wait for more bytes
                    }
                    Err(_) => {
                        // invalid — drop and continue
                        pending.clear();
                    }
                }
            }
        }
    }
}

#[cfg(not(all(unix, feature = "tty")))]
fn read_stdin_line_interactive(prompt: &str) -> Result<String> {
    let prompt = normalize_prompt(prompt);
    print!("{prompt}");
    io::stdout().flush()?;
    read_stdin_line_piped()
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
    /// Prefer [`Self::read_line_with_prompt`] for interactive TTY (Unicode backspace).
    pub fn read_line(&mut self) -> Result<String> {
        if let Some(line) = self.queue.pop_front() {
            return Ok(line);
        }
        if self.capture {
            anyhow::bail!(
                "input needs a line (frontmatter stdin:/输入:, view Preset input, or --stdin-file)"
            );
        }
        read_stdin_line_piped()
    }

    /// Read a line; on a TTY, run Unicode-aware editing and draw `prompt` itself.
    /// Returns `Ok((line, painted_prompt))` where `painted_prompt` is true when this
    /// function already printed the prompt (caller must not `emit_prompt` again).
    pub fn read_line_with_prompt(&mut self, prompt: &str) -> Result<(String, bool)> {
        if let Some(line) = self.queue.pop_front() {
            return Ok((line, false));
        }
        if self.capture {
            anyhow::bail!(
                "input needs a line (frontmatter stdin:/输入:, view Preset input, or --stdin-file)"
            );
        }
        let line = read_stdin_line_interactive(prompt)?;
        Ok((line, true))
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

    #[test]
    fn normalize_prompt_adds_space() {
        assert_eq!(normalize_prompt("问？"), "问？ ");
        assert_eq!(normalize_prompt("问？ "), "问？ ");
        assert_eq!(normalize_prompt(""), "");
    }

    #[test]
    fn utf8_pop_removes_full_char() {
        let mut s = String::from("帮我规划");
        assert_eq!(s.pop(), Some('划'));
        assert_eq!(s.pop(), Some('规'));
        assert_eq!(s, "帮我");
    }
}

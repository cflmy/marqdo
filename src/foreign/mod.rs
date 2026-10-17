//! Foreign / code-block helpers (外联).

pub mod registry;

pub use registry::{default_argv, default_cmd_display, default_python_argv};

/// Shell-like split: whitespace, single/double quotes, backslash escapes.
pub fn split_cmd_line(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else if c == '\\' && q == '"' {
                    if let Some(n) = chars.next() {
                        cur.push(n);
                    }
                } else {
                    cur.push(c);
                }
            }
            None => {
                if c.is_whitespace() {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                } else if c == '\'' || c == '"' {
                    quote = Some(c);
                } else if c == '\\' {
                    if let Some(n) = chars.next() {
                        cur.push(n);
                    }
                } else {
                    cur.push(c);
                }
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Parse ```lang … opener → language id (first token).
pub fn fence_lang(opener_trimmed: &str) -> Option<String> {
    let rest = opener_trimmed.strip_prefix("```")?;
    if rest.is_empty() || rest.chars().all(|c| c == '`') {
        return None;
    }
    let lang = rest.split_whitespace().next()?.to_string();
    if lang.is_empty() {
        None
    } else {
        Some(lang)
    }
}

pub fn is_fence_opener(trimmed: &str) -> bool {
    fence_lang(trimmed).is_some()
}

pub fn is_fence_closer(trimmed: &str) -> bool {
    trimmed.starts_with("```")
        && trimmed
            .chars()
            .skip_while(|c| *c == '`')
            .all(|c| c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_lang_python() {
        assert_eq!(fence_lang("```python").as_deref(), Some("python"));
        assert_eq!(fence_lang("```python name=x").as_deref(), Some("python"));
        assert!(fence_lang("```").is_none());
    }
}

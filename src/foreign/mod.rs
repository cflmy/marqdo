//! Foreign / code-block helpers (外联).

/// Default interpreter argv for a language tag (before env / set_cmd).
pub fn default_argv(lang: &str) -> Option<Vec<String>> {
    match lang.to_ascii_lowercase().as_str() {
        "python" | "python3" | "py" => Some(default_python_argv()),
        "javascript" | "js" | "node" => Some(vec!["node".into()]),
        _ => None,
    }
}

pub fn default_python_argv() -> Vec<String> {
    #[cfg(windows)]
    {
        vec!["python".into()]
    }
    #[cfg(not(windows))]
    {
        vec!["python3".into()]
    }
}

/// Placeholder shown in the view command box.
pub fn default_cmd_display(lang: &str) -> String {
    default_argv(lang)
        .map(|a| a.join(" "))
        .unwrap_or_else(|| lang.to_string())
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

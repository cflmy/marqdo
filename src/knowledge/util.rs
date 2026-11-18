//! Shared frontmatter / prose helpers for knowledge extraction.

pub fn extract_fm_field(source: &str, key: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    let prefix = format!("{key}:");
    for line in lines.iter().skip(1) {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix(&prefix) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

pub fn extract_nested_by(source: &str, block: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    let mut in_block = false;
    for line in lines.iter().skip(1) {
        let t = line.trim_end();
        if t.trim() == "---" {
            break;
        }
        if t.trim() == format!("{block}:") {
            in_block = true;
            continue;
        }
        if in_block {
            let trimmed = t.trim();
            if trimmed.starts_with("by:") {
                return Some(
                    trimmed
                        .trim_start_matches("by:")
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            }
            if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
        }
    }
    None
}

pub fn extract_fm_list(source: &str, key: &str) -> Vec<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut in_list = false;
    let header = format!("{key}:");
    for line in lines.iter().skip(1) {
        let t = line.trim_end();
        if t.trim() == "---" {
            break;
        }
        if t.trim() == header || t.trim() == format!("{key}: []") {
            if t.trim().ends_with("[]") {
                return Vec::new();
            }
            in_list = true;
            continue;
        }
        if in_list {
            let trimmed = t.trim();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                out.push(rest.trim().trim_matches('"').to_string());
            } else if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
        }
    }
    out
}

/// Extract section body under `## Heading` until next `##` / `#` / EOF.
pub fn extract_section(source: &str, heading: &str) -> Option<String> {
    let markers = [
        format!("## {heading}"),
        format!("## {}", heading.to_ascii_lowercase()),
        format!("## {}", heading.to_ascii_uppercase()),
    ];
    let lines: Vec<&str> = source.lines().collect();
    let mut start = None;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if markers.iter().any(|m| t.eq_ignore_ascii_case(m)) {
            start = Some(i + 1);
            break;
        }
    }
    let start = start?;
    let mut body = Vec::new();
    for line in lines.iter().skip(start) {
        let t = line.trim();
        if t.starts_with("## ") || (t.starts_with("# ") && !t.starts_with("## ")) {
            break;
        }
        body.push(*line);
    }
    let s = body.join("\n").trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn first_paragraph(source: &str) -> String {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut after_fm = source;
    if let Some(rest) = source.strip_prefix("---") {
        if let Some(idx) = rest.find("\n---") {
            after_fm = &rest[idx + 4..];
        }
    }
    let mut paras = Vec::new();
    let mut cur = String::new();
    for line in after_fm.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            if !cur.is_empty() {
                break;
            }
            continue;
        }
        if t.is_empty() {
            if !cur.is_empty() {
                paras.push(cur.clone());
                break;
            }
            continue;
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(t);
    }
    if paras.is_empty() && !cur.is_empty() {
        paras.push(cur);
    }
    paras.first().cloned().unwrap_or_default()
}

pub fn yaml_escape(s: &str) -> String {
    if s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('\n')
        || s.starts_with(' ')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

pub fn module_stem(resource: &str) -> String {
    resource
        .replace('/', "__")
        .replace('\\', "__")
        .trim_end_matches(".mq.md")
        .trim_end_matches(".md")
        .to_string()
}

pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c == '_' || c == '-' || c == '.' {
            out.push(c);
        } else if c.is_whitespace() || c == '/' {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    out.trim_matches('-').to_string()
}

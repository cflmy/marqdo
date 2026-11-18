//! Markdown-heading-backed, bounded context projection for Marqdo SKILLs.
//! This is source selection, not execution: quoted text never becomes tool authority.
//! Markdown events ignore fenced-code headings; ambiguous functions fail closed.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
struct Section {
    level: u8,
    title: String,
    start: usize,
}

fn level_num(h: HeadingLevel) -> u8 {
    match h {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn headings(source: &str) -> Vec<Section> {
    let mut out = Vec::new();
    let mut pending: Option<Section> = None;
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                pending = Some(Section {
                    level: level_num(level),
                    title: String::new(),
                    start: range.start,
                });
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(section) = pending.as_mut() {
                    section.title.push_str(&text);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(mut section) = pending.take() {
                    section.title = section.title.trim().to_string();
                    out.push(section);
                }
            }
            _ => {}
        }
    }
    out
}

fn class_for(headings: &[Section], index: usize) -> Option<&str> {
    headings[..index]
        .iter()
        .rev()
        .find(|h| h.level == 1)
        .map(|h| h.title.as_str())
}

fn frontmatter_end(source: &str) -> usize {
    if !source.starts_with("---\n") {
        return 0;
    }
    let mut pos = 4;
    for line in source[4..].split_inclusive('\n') {
        pos += line.len();
        if line.trim() == "---" {
            return pos;
        }
    }
    0
}

fn truncate_chars(text: &str, max_chars: usize) -> (&str, bool) {
    let Some((end, _)) = text.char_indices().nth(max_chars) else {
        return (text, false);
    };
    (&text[..end], true)
}

pub fn project(args: &Value) -> Result<Value, String> {
    let source = match args.get("source").and_then(Value::as_str) {
        Some(s) => s.to_string(),
        None => crate::host_query_json("module_source")?
            .as_str()
            .ok_or_else(|| "module_source unavailable".to_string())?
            .to_string(),
    };
    let scope = args.get("scope").and_then(Value::as_str).unwrap_or("function");
    let class = args.get("class").and_then(Value::as_str);
    let function = args.get("function").and_then(Value::as_str);
    let maximum = args
        .get("max_chars")
        .and_then(Value::as_u64)
        .unwrap_or(1600)
        .clamp(32, 16000) as usize;
    // YAML frontmatter may look like a setext Markdown heading; never treat it
    // as a Marqdo class or function boundary.
    let front_end = frontmatter_end(&source);
    let marks: Vec<Section> = headings(&source)
        .into_iter()
        .filter(|heading| heading.start >= front_end)
        .collect();
    let results: Vec<(usize, usize)> = match scope {
        "file" => {
            let end = marks.first().map_or(source.len(), |h| h.start);
            let start = frontmatter_end(&source).min(end);
            vec![(start, end)]
        }
        "class" | "function" | "tool" => {
            let level = match scope {
                "class" => 1,
                "function" => 2,
                _ => 3,
            };
            let target = if level == 1 { class } else { function };
            let wanted = target.ok_or_else(|| format!("{scope} selector required"))?;
            marks
                .iter()
                .enumerate()
                .filter(|(i, h)| {
                    h.level == level
                        && h.title == wanted
                        && (level == 1 || class.is_none() || class_for(&marks, *i) == class)
                })
                .map(|(i, h)| {
                    let end = marks[i + 1..]
                        .iter()
                        .find(|next| next.level <= level)
                        .map_or(source.len(), |next| next.start);
                    let end = if level == 1 {
                        marks[i + 1..]
                            .iter()
                            .find(|next| next.level <= 2)
                            .map_or(end, |next| next.start)
                    } else {
                        end
                    };
                    (h.start, end)
                })
                .collect()
        }
        _ => return Err("scope must be file, class, function or tool".to_string()),
    };
    if results.len() != 1 {
        return Ok(json!({
            "ok": false,
            "status": if results.is_empty() { "not_found" } else { "ambiguous" },
            "scope": scope,
            "matches": results.len(),
            "text": null
        }));
    }
    let (start, end) = results[0];
    let (slice, truncated) = truncate_chars(source[start..end].trim(), maximum);
    let line_start = source[..start].bytes().filter(|c| *c == b'\n').count() + 1;
    Ok(json!({
        "ok": true, "status": "accepted", "scope": scope,
        "text": slice, "truncated": truncated,
        "start_byte": start, "end_byte": end,
        "start_line": line_start,
        "source_kind": "markdown_headings"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "---\ntitle: example\n---\nEntry instructions.\n\n# main\nClass instructions.\n\n## route\nReal body.\n```md\n## route\nfake\n```\n\n## other\nOther body.\n\n# helper\nHelper prose.\n\n## route\nSecond body.\n";

    #[test]
    fn scope_is_bounded_and_fenced_headings_ignored() {
        let v = project(&json!({"source": DOC, "scope": "function", "class": "main", "function": "route"})).unwrap();
        assert_eq!(v["ok"], true);
        assert!(v["text"].as_str().unwrap().contains("Real body."));
        assert!(v["text"].as_str().unwrap().contains("fake"));
        assert!(!v["text"].as_str().unwrap().contains("Other body."));
        assert!(!v["text"].as_str().unwrap().contains("Second body."));
    }

    #[test]
    fn ambiguous_function_fails_closed() {
        let v = project(&json!({"source": DOC, "scope": "function", "function": "route"})).unwrap();
        assert_eq!(v["status"], "ambiguous");
        assert_eq!(v["text"], Value::Null);
    }

    #[test]
    fn file_scope_removes_frontmatter_and_class_prose_excludes_functions() {
        let a = project(&json!({"source": DOC, "scope": "file"})).unwrap();
        assert_eq!(a["text"], "Entry instructions.");
        let b = project(&json!({"source": DOC, "scope": "class", "class": "main"})).unwrap();
        assert!(b["text"].as_str().unwrap().contains("Class instructions."));
        assert!(!b["text"].as_str().unwrap().contains("Real body."));
    }

    #[test]
    fn unicode_truncation_does_not_split_characters() {
        let v = project(&json!({"source": "汉".repeat(100), "scope": "file", "max_chars": 32})).unwrap();
        assert_eq!(v["text"].as_str().unwrap().chars().count(), 32);
        assert_eq!(v["truncated"], true);
    }
}

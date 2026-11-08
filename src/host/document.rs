//! Inert Markdown structure using the same pulldown-cmark parser as rendering.
//! Byte ranges always refer to the original UTF-8 source; nothing is executed.
use super::{as_text, result};
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
type Args = HashMap<String, Value>;
fn arg<'a>(b: &'a Args, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
fn text<'a>(b: &'a Args, k: &str) -> Result<&'a str, String> {
    as_text(arg(b, k)?, k)
}
fn map(values: Vec<(&str, Value)>) -> Value {
    Value::Map(values.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn string(v: impl Into<String>) -> Value {
    Value::Text(v.into())
}
fn num(v: usize) -> Value {
    Value::Int(v as i64)
}
#[derive(Debug)]
struct Section {
    title: String,
    level: usize,
    start: usize,
    body: usize,
    end: usize,
}
fn sections(text: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    let mut active = None;
    for (event, range) in Parser::new_ext(
        text,
        Options::ENABLE_TABLES | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
    )
    .into_offset_iter()
    {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let n = level as usize;
                for section in &mut out {
                    if section.end == text.len() && section.level >= n {
                        section.end = range.start
                    }
                }
                out.push(Section {
                    title: String::new(),
                    level: n,
                    start: range.start,
                    body: range.end,
                    end: text.len(),
                });
                active = Some(out.len() - 1);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(i) = active.take() {
                    out[i].body = range.end
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some(i) = active {
                    out[i].title.push_str(&t)
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(i) = active {
                    out[i].title.push(' ')
                }
            }
            _ => {}
        }
    }
    out
}
fn unique_section<'a>(sections: &'a [Section], title: &str) -> Result<&'a Section, String> {
    let hits: Vec<_> = sections.iter().filter(|s| s.title == title).collect();
    match hits.as_slice() {
        [s] => Ok(s),
        [] => Err(format!("section not found: {title}")),
        _ => Err(format!("ambiguous section: {title}")),
    }
}
#[derive(Default)]
struct Table {
    start: usize,
    end: usize,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}
fn tables(text: &str) -> Vec<Table> {
    let mut out = Vec::new();
    let mut table: Option<Table> = None;
    let mut header = false;
    let mut row = Vec::new();
    let mut cell: Option<String> = None;
    for (event, range) in Parser::new_ext(
        text,
        Options::ENABLE_TABLES | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
    )
    .into_offset_iter()
    {
        match event {
            Event::Start(Tag::Table(_)) => {
                table = Some(Table {
                    start: range.start,
                    ..Default::default()
                })
            }
            Event::Start(Tag::TableHead) => {
                header = true;
                row.clear()
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(t) = table.as_mut() {
                    t.headers = std::mem::take(&mut row)
                }
                header = false
            }
            Event::Start(Tag::TableRow) => row.clear(),
            Event::End(TagEnd::TableRow) => {
                if !header {
                    if let Some(t) = table.as_mut() {
                        t.rows.push(std::mem::take(&mut row));
                    }
                }
            }
            Event::Start(Tag::TableCell) => cell = Some(String::new()),
            Event::End(TagEnd::TableCell) => {
                if let Some(c) = cell.take() {
                    row.push(c)
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some(c) = cell.as_mut() {
                    c.push_str(&t)
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(c) = cell.as_mut() {
                    c.push('\n')
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(mut t) = table.take() {
                    t.end = range.end;
                    out.push(t)
                }
            }
            _ => {}
        }
    }
    out
}
fn check_digest(b: &Args, text: &str) -> Result<(), String> {
    if let Some(Value::Text(expected)) = b.get("expected_sha256") {
        if format!("{:x}", Sha256::digest(text.as_bytes())) != *expected {
            return Err("conflict: document content changed".into());
        }
    }
    Ok(())
}
pub(super) fn call(_ctx: &mut HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    let source = text(b, "text")?;
    match id {
        HostFn::DocumentSections => Ok(Value::List(
            sections(source)
                .into_iter()
                .map(|s| {
                    map(vec![
                        ("title", string(s.title)),
                        ("level", num(s.level)),
                        ("start", num(s.start)),
                        ("body_start", num(s.body)),
                        ("end", num(s.end)),
                        ("text", string(&source[s.body..s.end])),
                    ])
                })
                .collect(),
        )),
        HostFn::DocumentSection => {
            let sections = sections(source);
            let section = unique_section(&sections, text(b, "title")?)?;
            Ok(string(&source[section.body..section.end]))
        }
        HostFn::DocumentReplaceSection => {
            let change = (|| {
                check_digest(b, source)?;
                let sections = sections(source);
                let section = unique_section(&sections, text(b, "title")?)?;
                let mut out = source[..section.body].to_string();
                let body = text(b, "body")?;
                if !out.ends_with('\n') {
                    out.push('\n')
                }
                out.push('\n');
                out.push_str(body);
                if !body.ends_with('\n') {
                    out.push('\n')
                }
                out.push_str(&source[section.end..]);
                Ok(string(out))
            })();
            Ok(result::capture("document.replace_section", change))
        }
        HostFn::DocumentTables => Ok(Value::List(
            tables(source)
                .into_iter()
                .map(|t| {
                    map(vec![
                        ("start", num(t.start)),
                        ("end", num(t.end)),
                        (
                            "headers",
                            Value::List(t.headers.into_iter().map(string).collect()),
                        ),
                        (
                            "rows",
                            Value::List(
                                t.rows
                                    .into_iter()
                                    .map(|r| Value::List(r.into_iter().map(string).collect()))
                                    .collect(),
                            ),
                        ),
                    ])
                })
                .collect(),
        )),
        HostFn::DocumentReplaceTable => {
            let change = (|| {
                check_digest(b, source)?;
                let index = match arg(b, "index")? {
                    Value::Int(n) if *n >= 0 => *n as usize,
                    _ => return Err("table index is zero-based nonnegative integer".into()),
                };
                let tables = tables(source);
                let table = tables.get(index).ok_or("table index out of range")?;
                fn cells(v: &Value) -> Result<Vec<String>, String> {
                    let Value::List(xs) = v else {
                        return Err("cells must be list of text".into());
                    };
                    xs.iter()
                        .map(|v| {
                            as_text(v, "cell").map(|s| {
                                s.replace('\\', "\\\\")
                                    .replace('|', "\\|")
                                    .replace('\n', "<br>")
                            })
                        })
                        .collect()
                }
                let headers = cells(arg(b, "headers")?)?;
                if headers.is_empty() {
                    return Err("table needs headers".into());
                }
                let Value::List(rows) = arg(b, "rows")? else {
                    return Err("rows must be list".into());
                };
                let mut rendered = format!(
                    "| {} |\n| {} |\n",
                    headers.join(" | "),
                    vec!["---"; headers.len()].join(" | ")
                );
                for row in rows {
                    let row = cells(row)?;
                    if row.len() != headers.len() {
                        return Err("table row length differs from header".into());
                    }
                    rendered.push_str(&format!("| {} |\n", row.join(" | ")))
                }
                Ok(string(format!(
                    "{}{}{}",
                    &source[..table.start],
                    rendered,
                    &source[table.end..]
                )))
            })();
            Ok(result::capture("document.replace_table", change))
        }
        HostFn::DocumentLinks => {
            let mut out = Vec::new();
            let mut active: Vec<(String, String, usize)> = Vec::new();
            for (event, range) in Parser::new_ext(
                source,
                Options::ENABLE_TABLES | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
            )
            .into_offset_iter()
            {
                match event {
                    Event::Start(Tag::Link { dest_url, .. }) => {
                        active.push((dest_url.into_string(), String::new(), range.start))
                    }
                    Event::Text(s) | Event::Code(s) => {
                        if let Some((_, text, _)) = active.last_mut() {
                            text.push_str(&s)
                        }
                    }
                    Event::End(TagEnd::Link) => {
                        if let Some((url, text, start)) = active.pop() {
                            out.push(map(vec![
                                ("label", string(text)),
                                ("target", string(url)),
                                ("start", num(start)),
                                ("end", num(range.end)),
                            ]))
                        }
                    }
                    _ => {}
                }
            }
            Ok(Value::List(out))
        }
        HostFn::DocumentCodeBlocks => {
            let mut out = Vec::new();
            let mut active = None;
            for (event, range) in Parser::new_ext(
                source,
                Options::ENABLE_TABLES | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
            )
            .into_offset_iter()
            {
                match event {
                    Event::Start(Tag::CodeBlock(kind)) => {
                        let lang = match kind {
                            CodeBlockKind::Fenced(s) => s.into_string(),
                            CodeBlockKind::Indented => String::new(),
                        };
                        active = Some((lang, String::new(), range.start));
                    }
                    Event::Text(s) => {
                        if let Some((_, body, _)) = active.as_mut() {
                            body.push_str(&s)
                        }
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        if let Some((lang, body, start)) = active.take() {
                            out.push(map(vec![
                                ("language", string(lang)),
                                ("text", string(body)),
                                ("start", num(start)),
                                ("end", num(range.end)),
                            ]))
                        }
                    }
                    _ => {}
                }
            }
            Ok(Value::List(out))
        }
        _ => Err("not a document operation".into()),
    }
}

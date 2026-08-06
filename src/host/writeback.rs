//! Jupyter-style output persistence in `.mq.md` (`<!-- marqdo-out … -->`).

use std::collections::HashMap;

use crate::host::HostContext;
use crate::value::Value;

const MAGIC: &str = "marqdo-out";

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockSpan {
    start: usize,
    end: usize,
    body: String,
    start_line: u32,
}

fn display_value(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        other => other.as_display(),
    }
}

fn is_truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::None) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Int(n)) => *n != 0,
        _ => false,
    }
}

fn call_line(ctx: &HostContext) -> Result<u32, String> {
    ctx.call_site_lines
        .last()
        .copied()
        .ok_or_else(|| "writeback: no call site line".into())
}

fn line_number_at(source: &str, byte: usize) -> u32 {
    source[..byte.min(source.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count() as u32
        + 1
}

fn format_block(body: &str) -> String {
    format!("<!-- {MAGIC}\n{body}\n-->")
}

fn parse_one_block(block: &str, start_byte: usize, source: &str) -> Option<BlockSpan> {
    let inner = block
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    if !inner.contains(MAGIC) {
        return None;
    }
    let mut lines: Vec<&str> = inner.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let header = lines.remove(0).trim();
    if !header.split_whitespace().any(|t| t == MAGIC) {
        return None;
    }
    let body = lines.join("\n");
    let end = start_byte + block.len();
    Some(BlockSpan {
        start: start_byte,
        end,
        body,
        start_line: line_number_at(source, start_byte),
    })
}

fn scan_blocks(source: &str) -> Vec<BlockSpan> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < source.len() {
        let rest = &source[i..];
        let Some(start) = rest.find("<!--") else {
            break;
        };
        let abs = i + start;
        let after = &source[abs..];
        if !after.contains(MAGIC) {
            i = abs + 4;
            continue;
        }
        let Some(end_rel) = after.find("-->") else {
            break;
        };
        let block = &after[..end_rel + 3];
        if let Some(b) = parse_one_block(block, abs, source) {
            out.push(b);
        }
        i = abs + end_rel + 3;
    }
    out
}

fn strip_span(source: &str, start: usize, end: usize) -> String {
    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..start]);
    out.push_str(&source[end..]);
    out
}

fn line_start_byte(source: &str, line_no: u32) -> Option<usize> {
    if line_no == 0 {
        return None;
    }
    let mut cur = 1u32;
    for (i, b) in source.bytes().enumerate() {
        if cur == line_no {
            return Some(i);
        }
        if b == b'\n' {
            cur += 1;
        }
    }
    if cur == line_no {
        Some(source.len())
    } else {
        None
    }
}

fn line_end_byte(source: &str, line_no: u32) -> Option<usize> {
    let start = line_start_byte(source, line_no)?;
    let rest = &source[start..];
    match rest.find('\n') {
        Some(off) => Some(start + off + 1),
        None => Some(source.len()),
    }
}

fn is_blank_line(source: &str, line_no: u32) -> bool {
    let start = match line_start_byte(source, line_no) {
        Some(s) => s,
        None => return true,
    };
    let end = line_end_byte(source, line_no).unwrap_or(source.len());
    source[start..end].trim().is_empty()
}

fn find_adjacent_block(source: &str, after_line: u32) -> Option<BlockSpan> {
    let mut line = after_line.saturating_add(1);
    let total = source.bytes().filter(|b| *b == b'\n').count() as u32 + 1;
    while line <= total && is_blank_line(source, line) {
        line += 1;
    }
    for b in scan_blocks(source) {
        if b.start_line == line {
            return Some(b);
        }
    }
    None
}

fn is_eof_block(source: &str, block: &BlockSpan) -> bool {
    source[block.end..].trim().is_empty()
}

fn find_eof_block(source: &str) -> Option<BlockSpan> {
    scan_blocks(source)
        .into_iter()
        .rev()
        .find(|b| is_eof_block(source, b))
}

fn insert_after_line(source: &str, line_no: u32, block: &str) -> Result<String, String> {
    let insert_at = line_end_byte(source, line_no)
        .ok_or_else(|| format!("writeback: line {line_no} out of range"))?;
    let mut out = String::with_capacity(source.len() + block.len() + 2);
    out.push_str(&source[..insert_at]);
    if !block.starts_with('\n') && !source[..insert_at].ends_with('\n') {
        out.push('\n');
    }
    out.push_str(block);
    if !block.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&source[insert_at..]);
    Ok(out)
}

fn append_eof(source: &str, block: &str) -> String {
    let mut trimmed = source.trim_end().to_string();
    if !trimmed.is_empty() {
        trimmed.push('\n');
    }
    trimmed.push_str(block);
    if !block.ends_with('\n') {
        trimmed.push('\n');
    }
    trimmed
}

fn persist(ctx: &mut HostContext, source: &str) -> Result<(), String> {
    if !ctx.allow_fs_write() {
        return Err("writeback disabled (fs_write capability off)".into());
    }
    let path = ctx
        .entry_path
        .as_ref()
        .ok_or_else(|| "writeback: no entry file".to_string())?;
    std::fs::write(path, source).map_err(|e| format!("writeback write {}: {e}", path.display()))?;
    ctx.entry_source = Some(source.to_string());
    Ok(())
}

fn anchor_line_above(source: &str, block_start_line: u32) -> u32 {
    let mut line = block_start_line.saturating_sub(1);
    while line > 0 && is_blank_line(source, line) {
        line -= 1;
    }
    line.max(1)
}

pub fn record(
    ctx: &mut HostContext,
    value: &Value,
    at_end: Option<&Value>,
) -> Result<Value, String> {
    let body = display_value(value);
    let source = ctx
        .entry_source
        .as_deref()
        .ok_or_else(|| "writeback: no entry source".to_string())?;
    let next = if is_truthy(at_end) {
        let stripped = find_eof_block(source)
            .map(|b| strip_span(source, b.start, b.end))
            .unwrap_or_else(|| source.to_string());
        append_eof(&stripped, &format_block(&body))
    } else {
        let line = call_line(ctx)?;
        let stripped = find_adjacent_block(source, line)
            .map(|b| strip_span(source, b.start, b.end))
            .unwrap_or_else(|| source.to_string());
        insert_after_line(&stripped, line, &format_block(&body))?
    };
    persist(ctx, &next)?;
    Ok(Value::Text(body))
}

pub fn get(ctx: &HostContext, at_end: Option<&Value>) -> Result<Value, String> {
    let source = ctx
        .entry_source
        .as_deref()
        .ok_or_else(|| "writeback: no entry source".to_string())?;
    let body = if is_truthy(at_end) {
        find_eof_block(source).map(|b| b.body)
    } else {
        let line = call_line(ctx)?;
        find_adjacent_block(source, line).map(|b| b.body)
    };
    Ok(match body {
        Some(s) => Value::Text(s),
        None => Value::None,
    })
}

pub fn clear(ctx: &mut HostContext, at_end: Option<&Value>) -> Result<Value, String> {
    let source = ctx
        .entry_source
        .as_deref()
        .ok_or_else(|| "writeback: no entry source".to_string())?;
    let next = if is_truthy(at_end) {
        find_eof_block(source)
            .map(|b| strip_span(source, b.start, b.end))
            .unwrap_or_else(|| source.to_string())
    } else {
        let line = call_line(ctx)?;
        find_adjacent_block(source, line)
            .map(|b| strip_span(source, b.start, b.end))
            .unwrap_or_else(|| source.to_string())
    };
    if next != source {
        persist(ctx, &next)?;
    }
    Ok(Value::None)
}

pub fn list(ctx: &HostContext) -> Result<Value, String> {
    let source = ctx
        .entry_source
        .as_deref()
        .ok_or_else(|| "writeback: no entry source".to_string())?;
    let items: Vec<Value> = scan_blocks(source)
        .into_iter()
        .map(|b| {
            let anchor = anchor_line_above(source, b.start_line);
            let at_eof = is_eof_block(source, &b);
            Value::Map(vec![
                ("line".into(), Value::Int(anchor as i64)),
                ("body".into(), Value::Text(b.body)),
                ("at_end".into(), Value::Bool(at_eof)),
            ])
        })
        .collect();
    Ok(Value::List(items))
}

/// Map statement line → body for adjacent blocks; EOF blocks use key `0`.
pub fn writeback_map(source: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for b in scan_blocks(source) {
        if is_eof_block(source, &b) {
            map.insert(0, b.body);
        } else {
            map.insert(anchor_line_above(source, b.start_line), b.body);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_insert_adjacent() {
        let src = "# main\n\n*`x` = 1*\n";
        let block = format_block("1");
        let next = insert_after_line(src, 3, &block).unwrap();
        assert!(next.contains("marqdo-out"));
        assert!(!next.contains("@line"));
        let blocks = scan_blocks(&next);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].body, "1");
        let replaced = strip_span(&next, blocks[0].start, blocks[0].end);
        assert!(!replaced.contains("marqdo-out"));
    }

    #[test]
    fn writeback_map_uses_line_above() {
        let src = "# main\n\n*`x` = 1*\n<!-- marqdo-out\n42\n-->\n\n> print text=tail\n";
        let map = writeback_map(src);
        assert_eq!(map.get(&3), Some(&"42".to_string()));
    }
}

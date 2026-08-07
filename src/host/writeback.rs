//! Jupyter-style output persistence in `.mq.md` (`<!-- marqdo-out … -->`).
//!
//! Optional **named slots**: `<!-- marqdo-out ok -->` / `<!-- marqdo-out error -->`
//! so success and failure results do not overwrite each other.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::host::HostContext;
use crate::value::Value;

const MAGIC: &str = "marqdo-out";

/// Serialize all writeback file mutations (sync + subtask writers).
static WRITEBACK_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockSpan {
    start: usize,
    end: usize,
    body: String,
    start_line: u32,
    /// Named slot from header (`marqdo-out ok` → `ok`); `None` = legacy unkeyed.
    key: Option<String>,
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

fn opt_key(v: Option<&Value>) -> Result<Option<String>, String> {
    match v {
        None | Some(Value::None) => Ok(None),
        Some(Value::Text(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                Ok(Some(normalize_slot_key(t)))
            }
        }
        _ => Err("writeback key must be text".into()),
    }
}

fn opt_line(v: Option<&Value>) -> Result<Option<u32>, String> {
    match v {
        None | Some(Value::None) => Ok(None),
        Some(Value::Int(n)) if *n > 0 => Ok(Some(*n as u32)),
        Some(Value::Text(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                t.parse::<u32>()
                    .map(|n| if n > 0 { Some(n) } else { None })
                    .map_err(|_| "writeback line must be a positive int".into())
            }
        }
        _ => Err("writeback line must be a positive int".into()),
    }
}

/// Canonical slot ids used by the agent framework.
fn normalize_slot_key(raw: &str) -> String {
    match raw {
        "ok" | "success" | "成功" => "ok".into(),
        "error" | "failure" | "失败" => "error".into(),
        other => other.to_string(),
    }
}

fn call_line(ctx: &HostContext) -> Result<u32, String> {
    ctx.call_site_lines
        .last()
        .copied()
        .ok_or_else(|| "writeback: no call site line".into())
}

/// Line that should own adjacent output cards.
/// Prefer explicit `line=`, else for keyed slots the outermost call site (`step`),
/// else the parent/current call site.
fn resolve_anchor(
    ctx: &HostContext,
    keyed: bool,
    line: Option<&Value>,
) -> Result<u32, String> {
    if let Some(n) = opt_line(line)? {
        return Ok(n);
    }
    if keyed {
        if let Some(n) = ctx.call_site_lines.first().copied() {
            return Ok(n);
        }
        if ctx.call_site_lines.len() >= 2 {
            return Ok(ctx.call_site_lines[ctx.call_site_lines.len() - 2]);
        }
    }
    call_line(ctx)
}

fn line_number_at(source: &str, byte: usize) -> u32 {
    source[..byte.min(source.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count() as u32
        + 1
}

fn format_block(body: &str, key: Option<&str>) -> String {
    match key {
        Some(k) => format!("<!-- {MAGIC} {k}\n{body}\n-->"),
        None => format!("<!-- {MAGIC}\n{body}\n-->"),
    }
}

fn parse_header_key(header: &str) -> Option<String> {
    let mut parts = header.split_whitespace();
    let first = parts.next()?;
    if first != MAGIC {
        return None;
    }
    parts.next().map(normalize_slot_key)
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
    let key = parse_header_key(header);
    let body = lines.join("\n");
    let end = start_byte + block.len();
    Some(BlockSpan {
        start: start_byte,
        end,
        body,
        start_line: line_number_at(source, start_byte),
        key,
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

/// Byte end of a `<!-- … -->` span, including the single newline that terminates the `-->` line.
fn span_end_including_nl(source: &str, end: usize) -> usize {
    if source.as_bytes().get(end) == Some(&b'\n') {
        end + 1
    } else {
        end
    }
}

/// Write `block` (plus a terminating newline) over `[start, end)`, preserving blank lines after.
fn splice_block(source: &str, start: usize, end: usize, block: &str) -> String {
    let end = span_end_including_nl(source, end);
    let mut out = String::with_capacity(source.len() + block.len() + 1);
    out.push_str(&source[..start]);
    out.push_str(block);
    if !block.ends_with('\n') {
        out.push('\n');
    }
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
        if b.start_line == line && b.key.is_none() {
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
        .find(|b| b.key.is_none() && is_eof_block(source, b))
}

fn find_keyed_block(source: &str, key: &str) -> Option<BlockSpan> {
    let key = normalize_slot_key(key);
    scan_blocks(source)
        .into_iter()
        .rev()
        .find(|b| b.key.as_deref() == Some(key.as_str()))
}

/// Consecutive `marqdo-out` blocks immediately below `after_line` (blank lines allowed).
fn cluster_after_line(source: &str, after_line: u32) -> Vec<BlockSpan> {
    let total = source.bytes().filter(|b| *b == b'\n').count() as u32 + 1;
    let mut line = after_line.saturating_add(1);
    while line <= total && is_blank_line(source, line) {
        line += 1;
    }
    let blocks = scan_blocks(source);
    let mut cluster = Vec::new();
    while line <= total {
        if let Some(b) = blocks.iter().find(|b| b.start_line == line) {
            cluster.push(b.clone());
            let end_line = line_number_at(source, b.end.saturating_sub(1).max(b.start));
            line = end_line.saturating_add(1);
            while line <= total && is_blank_line(source, line) {
                line += 1;
            }
        } else {
            break;
        }
    }
    cluster
}

fn find_keyed_in_cluster(source: &str, after_line: u32, key: &str) -> Option<BlockSpan> {
    let key = normalize_slot_key(key);
    cluster_after_line(source, after_line)
        .into_iter()
        .find(|b| b.key.as_deref() == Some(key.as_str()))
}

/// Replace keyed block under `anchor` line, or insert into that cluster (not EOF).
fn replace_or_insert_keyed_at(source: &str, anchor: u32, key: &str, body: &str) -> Result<String, String> {
    let key = normalize_slot_key(key);
    let block = format_block(body, Some(&key));
    if let Some(existing) = find_keyed_in_cluster(source, anchor, &key) {
        return Ok(splice_block(source, existing.start, existing.end, &block));
    }
    let cluster = cluster_after_line(source, anchor);
    if let Some(last) = cluster.last() {
        // Insert after the last block's terminating newline (or after `-->` if none).
        let insert_at = span_end_including_nl(source, last.end);
        let mut out = String::with_capacity(source.len() + block.len() + 1);
        out.push_str(&source[..insert_at]);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&block);
        if !block.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&source[insert_at..]);
        Ok(out)
    } else {
        insert_after_line(source, anchor, &block)
    }
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

fn load_entry_source(ctx: &HostContext) -> Result<String, String> {
    if let Some(path) = &ctx.entry_path {
        if let Ok(text) = std::fs::read_to_string(path) {
            return Ok(text);
        }
    }
    ctx.entry_source
        .clone()
        .ok_or_else(|| "writeback: no entry source".to_string())
}

fn with_writeback_lock<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = WRITEBACK_LOCK
        .lock()
        .map_err(|_| "writeback lock poisoned".to_string())?;
    f()
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
    key: Option<&Value>,
    line: Option<&Value>,
) -> Result<Value, String> {
    let body = display_value(value);
    let key = opt_key(key)?;
    let at_end = is_truthy(at_end);
    let anchor = if key.is_some() || !at_end {
        Some(resolve_anchor(ctx, key.is_some(), line)?)
    } else {
        None
    };
    with_writeback_lock(|| {
        let source = load_entry_source(ctx)?;
        let next = if let Some(ref k) = key {
            let anchor = anchor.ok_or_else(|| "writeback: missing anchor line".to_string())?;
            replace_or_insert_keyed_at(&source, anchor, k, &body)?
        } else if at_end {
            let stripped = find_eof_block(&source)
                .map(|b| strip_span(&source, b.start, span_end_including_nl(&source, b.end)))
                .unwrap_or_else(|| source.clone());
            append_eof(&stripped, &format_block(&body, None))
        } else {
            let line = anchor.ok_or_else(|| "writeback: missing anchor line".to_string())?;
            let stripped = find_adjacent_block(&source, line)
                .map(|b| strip_span(&source, b.start, span_end_including_nl(&source, b.end)))
                .unwrap_or_else(|| source.clone());
            insert_after_line(&stripped, line, &format_block(&body, None))?
        };
        persist(ctx, &next)?;
        Ok(Value::Text(body.clone()))
    })
}

pub fn get(
    ctx: &HostContext,
    at_end: Option<&Value>,
    key: Option<&Value>,
    line: Option<&Value>,
) -> Result<Value, String> {
    let key = opt_key(key)?;
    let at_end = is_truthy(at_end);
    let anchor = if key.is_some() || !at_end {
        resolve_anchor(ctx, key.is_some(), line).ok()
    } else {
        None
    };
    with_writeback_lock(|| {
        let source = load_entry_source(ctx)?;
        let body = if let Some(ref k) = key {
            let found = anchor
                .and_then(|a| find_keyed_in_cluster(&source, a, k))
                .or_else(|| find_keyed_block(&source, k));
            found.map(|b| b.body)
        } else if at_end {
            find_eof_block(&source).map(|b| b.body)
        } else {
            let line = anchor.ok_or_else(|| "writeback: no call site line".to_string())?;
            find_adjacent_block(&source, line).map(|b| b.body)
        };
        Ok(match body {
            Some(s) => Value::Text(s),
            None => Value::None,
        })
    })
}

pub fn clear(
    ctx: &mut HostContext,
    at_end: Option<&Value>,
    key: Option<&Value>,
    line: Option<&Value>,
) -> Result<Value, String> {
    let key = opt_key(key)?;
    let at_end = is_truthy(at_end);
    let anchor = if key.is_some() || !at_end {
        resolve_anchor(ctx, key.is_some(), line).ok()
    } else {
        None
    };
    with_writeback_lock(|| {
        let source = load_entry_source(ctx)?;
        let next = if let Some(ref k) = key {
            let block = anchor
                .and_then(|a| find_keyed_in_cluster(&source, a, k))
                .or_else(|| find_keyed_block(&source, k));
            block
                .map(|b| strip_span(&source, b.start, span_end_including_nl(&source, b.end)))
                .unwrap_or_else(|| source.clone())
        } else if at_end {
            find_eof_block(&source)
                .map(|b| strip_span(&source, b.start, span_end_including_nl(&source, b.end)))
                .unwrap_or_else(|| source.clone())
        } else {
            let line = anchor.ok_or_else(|| "writeback: no call site line".to_string())?;
            find_adjacent_block(&source, line)
                .map(|b| strip_span(&source, b.start, span_end_including_nl(&source, b.end)))
                .unwrap_or_else(|| source.clone())
        };
        if next != source {
            persist(ctx, &next)?;
        }
        Ok(Value::None)
    })
}

pub fn list(ctx: &HostContext) -> Result<Value, String> {
    let source = ctx
        .entry_source
        .as_deref()
        .ok_or_else(|| "writeback: no entry source".to_string())?;
    Ok(slots_from_source(source))
}

/// Scan `path` for `<!-- marqdo-out … -->` slots (any file, not only the entry).
pub fn scan_path(ctx: &HostContext, path: &Value) -> Result<Value, String> {
    let rel = match path {
        Value::Text(s) => s.as_str(),
        _ => return Err("writeback scan_path: path must be text".into()),
    };
    let p = ctx.resolve_path(rel)?;
    let source = std::fs::read_to_string(&p)
        .map_err(|e| format!("writeback scan_path {}: {e}", p.display()))?;
    Ok(slots_from_source(&source))
}

fn slots_from_source(source: &str) -> Value {
    let items: Vec<Value> = scan_blocks(source)
        .into_iter()
        .map(|b| {
            let anchor = anchor_line_above(source, b.start_line);
            let at_eof = b.key.is_none() && is_eof_block(source, &b);
            let mut entries = vec![
                ("line".into(), Value::Int(anchor as i64)),
                ("body".into(), Value::Text(b.body)),
                ("at_end".into(), Value::Bool(at_eof)),
            ];
            if let Some(k) = b.key {
                entries.push(("key".into(), Value::Text(k)));
            }
            Value::Map(entries)
        })
        .collect();
    Value::List(items)
}

/// Ensure named slots exist with a placeholder body; **does not** overwrite non-empty bodies.
pub fn ensure_slot(
    ctx: &mut HostContext,
    key: &Value,
    placeholder: &Value,
    line: Option<&Value>,
) -> Result<Value, String> {
    let k = opt_key(Some(key))?.ok_or_else(|| "writeback ensure needs key".to_string())?;
    let body = display_value(placeholder);
    let anchor = resolve_anchor(ctx, true, line)?;
    with_writeback_lock(|| {
        let source = load_entry_source(ctx)?;
        if find_keyed_in_cluster(&source, anchor, &k).is_some() {
            return Ok(Value::Bool(false));
        }
        let next = replace_or_insert_keyed_at(&source, anchor, &k, &body)?;
        persist(ctx, &next)?;
        Ok(Value::Bool(true))
    })
}

/// Map statement line → body for adjacent blocks (keyed slots included under their anchor).
pub fn writeback_map(source: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for b in scan_blocks(source) {
        let anchor = anchor_line_above(source, b.start_line);
        let piece = match &b.key {
            Some(k) => format!("[{k}]\n{}", b.body),
            None => b.body.clone(),
        };
        if b.key.is_none() && is_eof_block(source, &b) {
            map.insert(0, piece);
        } else {
            map.entry(anchor)
                .and_modify(|e| {
                    e.push_str("\n\n");
                    e.push_str(&piece);
                })
                .or_insert(piece);
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
        let block = format_block("1", None);
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

    #[test]
    fn keyed_slots_sit_under_anchor_and_do_not_clobber() {
        let src = "# main\n\n*`r` = > step *\n\n> print text=after\n";
        let with_ok = replace_or_insert_keyed_at(src, 3, "ok", "success-1").unwrap();
        let with_both = replace_or_insert_keyed_at(&with_ok, 3, "error", "fail-1").unwrap();
        let step_at = with_both.find("`r` = > step").unwrap();
        let ok_at = with_both.find("marqdo-out ok").unwrap();
        let err_at = with_both.find("marqdo-out error").unwrap();
        let after_at = with_both.find("> print text=after").unwrap();
        assert!(ok_at > step_at && ok_at < after_at);
        assert!(err_at > ok_at && err_at < after_at);
        let updated = replace_or_insert_keyed_at(&with_both, 3, "error", "fail-2").unwrap();
        assert!(updated.contains("success-1"));
        assert!(updated.contains("fail-2"));
        assert!(!updated.contains("fail-1"));
    }

    #[test]
    fn keyed_replace_does_not_accumulate_blank_lines() {
        let src = "# main\n\n> writeback.record\n<!-- marqdo-out ok\nold\n-->\n\n2. *\n";
        let once = replace_or_insert_keyed_at(src, 3, "ok", "new-1").unwrap();
        let twice = replace_or_insert_keyed_at(&once, 3, "ok", "new-2").unwrap();
        let thrice = replace_or_insert_keyed_at(&twice, 3, "ok", "new-3").unwrap();
        assert_eq!(once.matches("\n\n\n").count(), src.matches("\n\n\n").count());
        assert_eq!(twice, thrice.replace("new-3", "new-2"));
        // Exactly one blank line between `-->` and the next statement.
        let after = thrice.split("-->\n").nth(1).unwrap();
        assert!(after.starts_with("\n2. *"), "got after -->: {after:?}");
        assert!(!after.starts_with("\n\n\n"), "blank lines accumulated");
    }

    #[test]
    fn unkeyed_replace_does_not_accumulate_blank_lines() {
        let src = "# main\n\n*`x` = 1*\n<!-- marqdo-out\nold\n-->\n\n> print text=tail\n";
        let line = 3u32;
        let strip_insert = |s: &str, body: &str| {
            let stripped = find_adjacent_block(s, line)
                .map(|b| strip_span(s, b.start, span_end_including_nl(s, b.end)))
                .unwrap_or_else(|| s.to_string());
            insert_after_line(&stripped, line, &format_block(body, None)).unwrap()
        };
        let once = strip_insert(src, "a");
        let twice = strip_insert(&once, "b");
        let thrice = strip_insert(&twice, "c");
        assert_eq!(
            once.matches("\n\n\n").count(),
            thrice.matches("\n\n\n").count()
        );
        let after = thrice.split("-->\n").nth(1).unwrap();
        assert!(after.starts_with("\n> print"), "got after -->: {after:?}");
    }
}

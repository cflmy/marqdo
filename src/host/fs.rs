//! Filesystem host primitives.

use std::fs;
use std::path::Path;

use crate::host::HostContext;
use crate::value::Value;

pub fn path_under_root(path: &Path, root: &Path) -> bool {
    let mut cur = path.to_path_buf();
    loop {
        if cur == *root {
            return true;
        }
        if !cur.pop() {
            return false;
        }
    }
}

fn text_path(v: &Value) -> Result<&str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err("path must be text".into()),
    }
}

fn text_body(v: &Value) -> Result<&str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err("text must be text".into()),
    }
}

pub fn read_text(ctx: &HostContext, path: &Value) -> Result<Value, String> {
    let p = ctx.resolve_path(text_path(path)?)?;
    let s = fs::read_to_string(&p).map_err(|e| format!("read_text {}: {e}", p.display()))?;
    Ok(Value::Text(s))
}

pub fn write_text(ctx: &HostContext, path: &Value, text: &Value) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("write_text denied by host policy".into());
    }
    let p = ctx.resolve_path(text_path(path)?)?;
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("write_text mkdir: {e}"))?;
    }
    fs::write(&p, text_body(text)?).map_err(|e| format!("write_text {}: {e}", p.display()))?;
    Ok(Value::None)
}

pub fn append_text(ctx: &HostContext, path: &Value, text: &Value) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("append_text denied by host policy".into());
    }
    let p = ctx.resolve_path(text_path(path)?)?;
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| format!("append_text {}: {e}", p.display()))?;
    f.write_all(text_body(text)?.as_bytes())
        .map_err(|e| format!("append_text {}: {e}", p.display()))?;
    Ok(Value::None)
}

pub fn exists(ctx: &HostContext, path: &Value) -> Result<Value, String> {
    let p = ctx.resolve_path(text_path(path)?)?;
    Ok(Value::Bool(p.exists()))
}

pub fn list_dir(ctx: &HostContext, path: &Value) -> Result<Value, String> {
    let p = ctx.resolve_path(text_path(path)?)?;
    let mut names = Vec::new();
    let rd = fs::read_dir(&p).map_err(|e| format!("list_dir {}: {e}", p.display()))?;
    for ent in rd {
        let ent = ent.map_err(|e| format!("list_dir: {e}"))?;
        names.push(Value::Text(
            ent.file_name().to_string_lossy().into_owned(),
        ));
    }
    names.sort_by(|a, b| a.as_display().cmp(&b.as_display()));
    Ok(Value::List(names))
}

pub fn make_dir(ctx: &HostContext, path: &Value) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("make_dir denied by host policy".into());
    }
    let p = ctx.resolve_path(text_path(path)?)?;
    fs::create_dir_all(&p).map_err(|e| format!("make_dir {}: {e}", p.display()))?;
    Ok(Value::None)
}

pub fn remove(ctx: &HostContext, path: &Value) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("remove denied by host policy".into());
    }
    let p = ctx.resolve_path(text_path(path)?)?;
    if p.is_dir() {
        fs::remove_dir_all(&p).map_err(|e| format!("remove {}: {e}", p.display()))?;
    } else {
        fs::remove_file(&p).map_err(|e| format!("remove {}: {e}", p.display()))?;
    }
    Ok(Value::None)
}

/// Exact UTF-8 FIND→REPLACE on a file. `find` must occur exactly once.
pub fn text_patch(ctx: &HostContext, path: &Value, find: &Value, replace: &Value) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("text_patch denied by host policy".into());
    }
    let p = ctx.resolve_path(text_path(path)?)?;
    let find = text_body(find)?;
    let replace = text_body(replace)?;
    if find.is_empty() {
        return Err("text_patch: find must be non-empty".into());
    }
    let src = fs::read_to_string(&p).map_err(|e| format!("text_patch {}: {e}", p.display()))?;
    let count = src.matches(find).count();
    if count == 0 {
        return Err(format!(
            "text_patch: find not found in {}",
            p.display()
        ));
    }
    if count > 1 {
        return Err(format!(
            "text_patch: find matched {count} times in {} (must be unique)",
            p.display()
        ));
    }
    let next = src.replacen(find, replace, 1);
    fs::write(&p, next).map_err(|e| format!("text_patch write {}: {e}", p.display()))?;
    Ok(Value::Int(1))
}

/// Apply one or more FIND/REPLACE blocks from plan-reply text.
///
/// Preferred blocks:
/// ```text
/// <<<
/// FIND
/// <old>
/// ===
/// REPLACE
/// <new>
/// >>>
/// ```
///
/// Also accepts fenced pairs (common model mistake):
/// ````text
/// ```find
/// <old>
/// ```
/// ```replace
/// <new>
/// ```
/// ````
pub fn apply_patch_blocks(
    ctx: &HostContext,
    path: &Value,
    text: &Value,
    soft: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("apply_patch_blocks denied by host policy".into());
    }
    let soft = match soft {
        None | Some(Value::None) => false,
        Some(v) => v.truthy(),
    };
    match apply_patch_blocks_inner(ctx, path, text) {
        Ok(n) => Ok(n),
        // soft only swallows unique-FIND miss — never parse / multi-match / whole-file.
        Err(e) if soft && e.contains("FIND not found") => Ok(Value::Int(0)),
        Err(e) => Err(e),
    }
}

/// Reject FIND that rewrites a non-trivial file wholesale (exact or ≥90% span).
/// Tiny files may be replaced entirely (e.g. two-line import rewrites).
fn find_spans_whole_file(src: &str, find: &str) -> bool {
    const MIN_BYTES: usize = 800;
    let st = src.trim();
    let ft = find.trim();
    if ft.is_empty() || src.len() < MIN_BYTES {
        return false;
    }
    if st == ft {
        return true;
    }
    if !src.contains(find) {
        return false;
    }
    find.len().saturating_mul(10) >= src.len().saturating_mul(9)
}

fn apply_patch_blocks_inner(
    ctx: &HostContext,
    path: &Value,
    text: &Value,
) -> Result<Value, String> {
    let blocks = parse_patch_blocks(text_body(text)?)?;
    if blocks.is_empty() {
        return Ok(Value::Int(0));
    }
    let p = ctx.resolve_path(text_path(path)?)?;
    let mut src = fs::read_to_string(&p).map_err(|e| format!("apply_patch_blocks {}: {e}", p.display()))?;
    let mut applied = 0i64;
    for (find, replace) in blocks {
        if find.is_empty() {
            return Err("apply_patch_blocks: empty FIND".into());
        }
        if find_spans_whole_file(&src, &find) {
            return Err(
                "apply_patch_blocks: FIND spans entire file (whole-file rewrite forbidden)"
                    .into(),
            );
        }
        let count = src.matches(&find).count();
        if count == 0 {
            return Err(format!(
                "apply_patch_blocks: FIND not found in {}",
                p.display()
            ));
        }
        if count > 1 {
            return Err(format!(
                "apply_patch_blocks: FIND matched {count} times in {} (must be unique)",
                p.display()
            ));
        }
        src = src.replacen(&find, &replace, 1);
        applied += 1;
    }
    fs::write(&p, src).map_err(|e| format!("apply_patch_blocks write {}: {e}", p.display()))?;
    Ok(Value::Int(applied))
}

fn parse_patch_blocks(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("<<<") {
        let after = &rest[start + 3..];
        let end = after
            .find(">>>")
            .ok_or_else(|| "apply_patch_blocks: unclosed <<< block".to_string())?;
        let body = after[..end].trim();
        rest = &after[end + 3..];
        let (find_part, replace_part) = body
            .split_once("===")
            .ok_or_else(|| "apply_patch_blocks: block missing ===".to_string())?;
        let find = strip_labeled(find_part, "FIND")?;
        let replace = strip_labeled(replace_part, "REPLACE")?;
        out.push((find, replace));
    }
    if out.is_empty() {
        parse_fence_find_replace(text, &mut out)?;
    }
    if out.is_empty() {
        parse_begin_patch_hunks(text, &mut out)?;
    }
    Ok(out)
}

/// ```find … ``` followed by ```replace … ``` (label case-insensitive).
fn parse_fence_find_replace(text: &str, out: &mut Vec<(String, String)>) -> Result<(), String> {
    let mut i = 0;
    while i < text.len() {
        let Some(rel) = text[i..].find("```") else {
            break;
        };
        let open = i + rel;
        let after_ticks = open + 3;
        let header_end = text[after_ticks..]
            .find('\n')
            .map(|n| after_ticks + n)
            .unwrap_or(text.len());
        let label = text[after_ticks..header_end]
            .trim()
            .trim_matches('`')
            .to_ascii_lowercase();
        if label != "find" {
            i = after_ticks;
            continue;
        }
        let body_start = if header_end < text.len() {
            header_end + 1
        } else {
            header_end
        };
        let Some(rel_close) = text[body_start..].find("```") else {
            return Err("apply_patch_blocks: unclosed ```find fence".into());
        };
        let find_end = body_start + rel_close;
        let find = text[body_start..find_end]
            .trim_end_matches(['\r', '\n'])
            .to_string();
        let mut j = find_end + 3;
        while j < text.len() && text.as_bytes()[j].is_ascii_whitespace() {
            j += 1;
        }
        if text[j..].starts_with("```") {
            j += 3;
        } else {
            return Err("apply_patch_blocks: ```find without following ```replace".into());
        }
        let rep_header_end = text[j..]
            .find('\n')
            .map(|n| j + n)
            .unwrap_or(text.len());
        let rep_label = text[j..rep_header_end]
            .trim()
            .trim_matches('`')
            .to_ascii_lowercase();
        if rep_label != "replace" {
            return Err("apply_patch_blocks: ```find without following ```replace".into());
        }
        let rep_body_start = if rep_header_end < text.len() {
            rep_header_end + 1
        } else {
            rep_header_end
        };
        let Some(rel_rep_close) = text[rep_body_start..].find("```") else {
            return Err("apply_patch_blocks: unclosed ```replace fence".into());
        };
        let rep_end = rep_body_start + rel_rep_close;
        let replace = text[rep_body_start..rep_end]
            .trim_end_matches(['\r', '\n'])
            .to_string();
        out.push((find, replace));
        i = rep_end + 3;
    }
    Ok(())
}

/// Minimal support for `*** Begin Patch` / `*** End Patch` with `-`/`+` line pairs
/// (same line count preferred; falls back to joining all `-` vs all `+` in the hunk).
fn parse_begin_patch_hunks(text: &str, out: &mut Vec<(String, String)>) -> Result<(), String> {
    const BEGIN: &str = "*** Begin Patch";
    const END: &str = "*** End Patch";
    let mut rest = text;
    while let Some(start) = rest.find(BEGIN) {
        let after = &rest[start + BEGIN.len()..];
        let end = after
            .find(END)
            .ok_or_else(|| "apply_patch_blocks: unclosed *** Begin Patch".to_string())?;
        let body = &after[..end];
        rest = &after[end + END.len()..];
        let mut minus: Vec<&str> = Vec::new();
        let mut plus: Vec<&str> = Vec::new();
        for line in body.lines() {
            if let Some(s) = line.strip_prefix('-') {
                if s.starts_with('-') {
                    continue; // skip --- / file headers like ---
                }
                // skip diff file headers "*** Update File" already not prefixed with single -
                minus.push(s.strip_prefix(' ').unwrap_or(s));
            } else if let Some(s) = line.strip_prefix('+') {
                if s.starts_with('+') {
                    continue;
                }
                plus.push(s.strip_prefix(' ').unwrap_or(s));
            }
        }
        if minus.is_empty() {
            continue;
        }
        let find = minus.join("\n");
        let replace = plus.join("\n");
        out.push((find, replace));
    }
    Ok(())
}

fn strip_labeled(chunk: &str, label: &str) -> Result<String, String> {
    let t = chunk.trim();
    let upper = t.to_ascii_uppercase();
    let label_u = label.to_ascii_uppercase();
    if upper.starts_with(&label_u) {
        let after = t[label.len()..].trim_start_matches([':', ' ', '\t']);
        let after = after.strip_prefix('\n').unwrap_or(after);
        Ok(after.to_string())
    } else {
        Ok(t.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{HostCaps, HostContext};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_ctx(dir: &std::path::Path) -> HostContext {
        let entry = dir.join("entry.mq.md");
        let _ = fs::write(&entry, "# main\n");
        let mut ctx = HostContext::for_run(Some(&entry), HostCaps::default(), vec![]);
        ctx.cwd = dir.to_path_buf();
        ctx.fs_root = Some(dir.to_path_buf());
        ctx
    }

    fn temp_workspace(tag: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("marqdo-fs-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn text_patch_unique() {
        let dir = temp_workspace("patch");
        let path = dir.join("a.md");
        fs::write(&path, "hello OLD world").unwrap();
        let ctx = write_ctx(&dir);
        let n = text_patch(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text("OLD".into()),
            &Value::Text("NEW".into()),
        )
        .unwrap();
        assert_eq!(n, Value::Int(1));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello NEW world");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_patch_blocks_two() {
        let dir = temp_workspace("blocks");
        let path = dir.join("a.md");
        fs::write(&path, "aa bb cc").unwrap();
        let ctx = write_ctx(&dir);
        let reply = r#"
DECISION: CONTINUE
PATCH:
<<<
FIND
aa
===
REPLACE
AA
>>>
<<<
FIND
cc
===
REPLACE
CC
>>>
"#;
        let n = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply.into()),
            None,
        )
        .unwrap();
        assert_eq!(n, Value::Int(2));
        assert_eq!(fs::read_to_string(&path).unwrap(), "AA bb CC");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_patch_blocks_fence_find_replace() {
        let dir = temp_workspace("fence");
        let path = dir.join("a.md");
        fs::write(&path, "old line\nkeep\n").unwrap();
        let ctx = write_ctx(&dir);
        let reply = "DECISION: CONTINUE\nPATCH:\n```find\nold line\n```\n```replace\nnew line\n```\n";
        let n = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply.into()),
            None,
        )
        .unwrap();
        assert_eq!(n, Value::Int(1));
        assert_eq!(fs::read_to_string(&path).unwrap(), "new line\nkeep\n");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_patch_blocks_begin_patch() {
        let dir = temp_workspace("beginpatch");
        let path = dir.join("a.md");
        fs::write(
            &path,
            "> ext/ai/llm.mq.md\n> lib/json.mq.md\n",
        )
        .unwrap();
        let ctx = write_ctx(&dir);
        let reply = r#"
DECISION: CONTINUE
PATCH:
```patch
*** Begin Patch
*** Update File: a.md
@@
-> ext/ai/llm.mq.md
-> lib/json.mq.md
+import llm:ext/ai/llm.mq.md
+import json:lib/json.mq.md
*** End Patch
```
"#;
        let n = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply.into()),
            None,
        )
        .unwrap();
        assert_eq!(n, Value::Int(1));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "import llm:ext/ai/llm.mq.md\nimport json:lib/json.mq.md\n"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_patch_blocks_soft_miss() {
        let dir = temp_workspace("soft");
        let path = dir.join("a.md");
        fs::write(&path, "hello").unwrap();
        let ctx = write_ctx(&dir);
        let reply = "<<<\nFIND\nmissing\n===\nREPLACE\nx\n>>>\n";
        let hard = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply.into()),
            None,
        );
        assert!(hard.is_err());
        let n = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply.into()),
            Some(&Value::Bool(true)),
        )
        .unwrap();
        assert_eq!(n, Value::Int(0));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_patch_blocks_rejects_whole_file_find() {
        let dir = temp_workspace("whole");
        let path = dir.join("a.md");
        let body = format!(
            "---\ntitle: workbook\n---\n\n# main\n\n{}",
            "line padding for size threshold\n".repeat(40)
        );
        assert!(body.len() >= 800, "fixture too small: {}", body.len());
        fs::write(&path, &body).unwrap();
        let ctx = write_ctx(&dir);
        let reply = format!(
            "<<<\nFIND\n{body}===\nREPLACE\n# main\n\n> print text=bye\n>>>\n"
        );
        let err = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply),
            None,
        )
        .unwrap_err();
        assert!(
            err.contains("whole-file rewrite forbidden"),
            "unexpected err={err}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), body);
        let soft_err = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(format!(
                "<<<\nFIND\n{body}===\nREPLACE\nx\n>>>\n"
            )),
            Some(&Value::Bool(true)),
        )
        .unwrap_err();
        assert!(soft_err.contains("whole-file rewrite forbidden"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_patch_blocks_soft_keeps_multi_match_hard() {
        let dir = temp_workspace("multisoft");
        let path = dir.join("a.md");
        fs::write(&path, "xx xx").unwrap();
        let ctx = write_ctx(&dir);
        let reply = "<<<\nFIND\nxx\n===\nREPLACE\nYY\n>>>\n";
        let err = apply_patch_blocks(
            &ctx,
            &Value::Text("a.md".into()),
            &Value::Text(reply.into()),
            Some(&Value::Bool(true)),
        )
        .unwrap_err();
        assert!(err.contains("matched 2 times"), "unexpected err={err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), "xx xx");
        let _ = fs::remove_dir_all(&dir);
    }
}

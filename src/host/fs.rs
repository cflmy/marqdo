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
/// Blocks:
/// ```text
/// <<<
/// FIND
/// <old>
/// ===
/// REPLACE
/// <new>
/// >>>
/// ```
pub fn apply_patch_blocks(ctx: &HostContext, path: &Value, text: &Value) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("apply_patch_blocks denied by host policy".into());
    }
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
    Ok(out)
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
        let n = apply_patch_blocks(&ctx, &Value::Text("a.md".into()), &Value::Text(reply.into())).unwrap();
        assert_eq!(n, Value::Int(2));
        assert_eq!(fs::read_to_string(&path).unwrap(), "AA bb CC");
        let _ = fs::remove_dir_all(&dir);
    }
}

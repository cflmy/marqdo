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

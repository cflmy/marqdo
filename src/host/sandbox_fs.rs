//! Application-level CoW workspace for `ext/ai/sandbox` (via host_query).
//!
//! lower = real root (read-through); upper = session bytes; whiteout = logical delete.
//! Not a symlink forest. Symlink targets are not followed as isolation primitives.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde_json::json;

use crate::host::HostContext;
use crate::value::Value;

#[derive(Debug, Default)]
pub struct CowSession {
    pub root: PathBuf,
    /// Relative path (forward slashes) → file bytes.
    pub upper: HashMap<String, Vec<u8>>,
    pub whiteout: HashSet<String>,
}

static SESSIONS: OnceLock<Mutex<HashMap<String, CowSession>>> = OnceLock::new();
static SEQ: OnceLock<Mutex<u64>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<String, CowSession>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> String {
    let mut g = SEQ
        .get_or_init(|| Mutex::new(0))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *g = g.wrapping_add(1);
    format!("sbx-{g}")
}

fn normalize_rel(rel: &str) -> Result<String, String> {
    let rel = rel.trim().trim_start_matches('/');
    if rel.is_empty() {
        return Err("sandbox path must not be empty".into());
    }
    let p = Path::new(rel);
    if p.is_absolute() {
        return Err("sandbox path must be relative to session root".into());
    }
    let mut out = Vec::new();
    for c in p.components() {
        match c {
            Component::Normal(s) => out.push(s.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err("sandbox path must not contain '..'".into());
            }
            _ => return Err(format!("sandbox path invalid component in `{rel}`")),
        }
    }
    if out.is_empty() {
        return Err("sandbox path must not be empty".into());
    }
    Ok(out.join("/"))
}

fn abs_under_root(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_n = normalize_rel(rel)?;
    let abs = root.join(Path::new(&rel_n));
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    // Parent may not exist yet for writes — check by prefix components.
    let mut check = abs.clone();
    loop {
        if check == *root || check == root_canon {
            return Ok(root.join(Path::new(&rel_n)));
        }
        if !check.pop() {
            break;
        }
        if let Ok(c) = check.canonicalize() {
            if crate::host::path_under_root(&c, &root_canon) || c == root_canon {
                return Ok(root.join(Path::new(&rel_n)));
            }
        }
    }
    if crate::host::path_under_root(&abs, root) {
        return Ok(abs);
    }
    Err(format!("path escapes sandbox root: {rel_n}"))
}

pub fn open(root: Option<&str>, ctx: &HostContext) -> Result<serde_json::Value, String> {
    let root_pb = if let Some(r) = root.filter(|s| !s.is_empty()) {
        let p = PathBuf::from(r);
        if p.is_absolute() {
            p
        } else {
            ctx.cwd.join(p)
        }
    } else if let Some(ep) = &ctx.entry_path {
        ep.parent()
            .unwrap_or(ep.as_path())
            .to_path_buf()
    } else {
        ctx.cwd.clone()
    };
    let root_pb = if root_pb.is_absolute() {
        root_pb
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(root_pb)
    };
    if !root_pb.is_dir() {
        return Err(format!(
            "sandbox_open: root is not a directory: {}",
            root_pb.display()
        ));
    }
    let id = next_id();
    let sess = CowSession {
        root: root_pb.clone(),
        upper: HashMap::new(),
        whiteout: HashSet::new(),
    };
    sessions()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(id.clone(), sess);
    Ok(json!({
        "id": id,
        "root": root_pb.to_string_lossy(),
    }))
}

fn with_session_mut<F, T>(id: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&mut CowSession) -> Result<T, String>,
{
    let mut map = sessions().lock().unwrap_or_else(|e| e.into_inner());
    let sess = map
        .get_mut(id)
        .ok_or_else(|| format!("sandbox: unknown session `{id}`"))?;
    f(sess)
}

fn with_session<F, T>(id: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&CowSession) -> Result<T, String>,
{
    let map = sessions().lock().unwrap_or_else(|e| e.into_inner());
    let sess = map
        .get(id)
        .ok_or_else(|| format!("sandbox: unknown session `{id}`"))?;
    f(sess)
}

pub fn write(id: &str, path: &str, text: &str) -> Result<serde_json::Value, String> {
    with_session_mut(id, |sess| {
        let rel = normalize_rel(path)?;
        let _ = abs_under_root(&sess.root, &rel)?;
        sess.whiteout.remove(&rel);
        sess.upper.insert(rel, text.as_bytes().to_vec());
        Ok(json!({ "ok": true }))
    })
}

pub fn read(id: &str, path: &str) -> Result<serde_json::Value, String> {
    with_session(id, |sess| {
        let rel = normalize_rel(path)?;
        if sess.whiteout.contains(&rel) {
            return Err(format!("sandbox_read: not found (whiteout): {rel}"));
        }
        if let Some(bytes) = sess.upper.get(&rel) {
            let s = String::from_utf8(bytes.clone())
                .map_err(|e| format!("sandbox_read: not utf-8: {e}"))?;
            return Ok(json!(s));
        }
        let abs = abs_under_root(&sess.root, &rel)?;
        let s = std::fs::read_to_string(&abs)
            .map_err(|e| format!("sandbox_read {}: {e}", abs.display()))?;
        Ok(json!(s))
    })
}

pub fn remove(id: &str, path: &str) -> Result<serde_json::Value, String> {
    with_session_mut(id, |sess| {
        let rel = normalize_rel(path)?;
        let _ = abs_under_root(&sess.root, &rel)?;
        sess.upper.remove(&rel);
        sess.whiteout.insert(rel);
        Ok(json!({ "ok": true }))
    })
}

pub fn diff(id: &str) -> Result<serde_json::Value, String> {
    with_session(id, |sess| {
        let mut items: Vec<serde_json::Value> = Vec::new();
        for p in sess.upper.keys() {
            items.push(json!({ "path": p, "kind": "write" }));
        }
        for p in &sess.whiteout {
            items.push(json!({ "path": p, "kind": "remove" }));
        }
        items.sort_by(|a, b| {
            let pa = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let pb = b.get("path").and_then(|v| v.as_str()).unwrap_or("");
            pa.cmp(pb)
        });
        Ok(json!(items))
    })
}

pub fn commit(id: &str) -> Result<serde_json::Value, String> {
    with_session_mut(id, |sess| {
        let mut applied = 0i64;
        let writes: Vec<(String, Vec<u8>)> = sess
            .upper
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let removes: Vec<String> = sess.whiteout.iter().cloned().collect();
        for (rel, bytes) in writes {
            let abs = abs_under_root(&sess.root, &rel)?;
            if let Some(parent) = abs.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("sandbox_commit mkdir: {e}"))?;
            }
            std::fs::write(&abs, &bytes)
                .map_err(|e| format!("sandbox_commit write {}: {e}", abs.display()))?;
            applied += 1;
        }
        for rel in removes {
            let abs = abs_under_root(&sess.root, &rel)?;
            if abs.exists() {
                if abs.is_dir() {
                    std::fs::remove_dir_all(&abs)
                        .map_err(|e| format!("sandbox_commit remove {}: {e}", abs.display()))?;
                } else {
                    std::fs::remove_file(&abs)
                        .map_err(|e| format!("sandbox_commit remove {}: {e}", abs.display()))?;
                }
                applied += 1;
            }
        }
        sess.upper.clear();
        sess.whiteout.clear();
        Ok(json!({ "ok": true, "applied": applied }))
    })
}

pub fn discard(id: &str) -> Result<serde_json::Value, String> {
    let mut map = sessions().lock().unwrap_or_else(|e| e.into_inner());
    if map.remove(id).is_none() {
        return Err(format!("sandbox: unknown session `{id}`"));
    }
    Ok(json!({ "ok": true }))
}

pub fn inspect(id: &str) -> Result<serde_json::Value, String> {
    with_session(id, |sess| {
        Ok(json!({
            "id": id,
            "root": sess.root.to_string_lossy(),
            "upper_count": sess.upper.len(),
            "whiteout_count": sess.whiteout.len(),
            "caps_default": {
                "fs_write": true,
                "exec": false,
                "net": false,
                "plugin": false
            },
            "notes": [
                "ext/web listen/DB/WS unavailable in sandbox",
                "plugin.load denied during sandbox.run",
                "subtask path/code/lang unavailable in sandbox"
            ]
        }))
    })
}

/// Read file bytes through CoW for an active host sandbox session (lib/fs).
pub fn cow_read_text(ctx: &HostContext, abs: &Path) -> Result<Option<String>, String> {
    let Some(id) = ctx.sandbox_session.as_deref() else {
        return Ok(None);
    };
    with_session(id, |sess| {
        let rel = rel_from_root(&sess.root, abs)?;
        if sess.whiteout.contains(&rel) {
            return Err(format!("read_text: not found (sandbox whiteout): {rel}"));
        }
        if let Some(bytes) = sess.upper.get(&rel) {
            let s = String::from_utf8(bytes.clone())
                .map_err(|e| format!("read_text: not utf-8: {e}"))?;
            return Ok(Some(s));
        }
        Ok(None)
    })
}

pub fn cow_write_text(ctx: &HostContext, abs: &Path, text: &str) -> Result<bool, String> {
    let Some(id) = ctx.sandbox_session.as_deref() else {
        return Ok(false);
    };
    with_session_mut(id, |sess| {
        let rel = rel_from_root(&sess.root, abs)?;
        sess.whiteout.remove(&rel);
        sess.upper.insert(rel, text.as_bytes().to_vec());
        Ok(true)
    })
}

pub fn cow_remove(ctx: &HostContext, abs: &Path) -> Result<bool, String> {
    let Some(id) = ctx.sandbox_session.as_deref() else {
        return Ok(false);
    };
    with_session_mut(id, |sess| {
        let rel = rel_from_root(&sess.root, abs)?;
        sess.upper.remove(&rel);
        sess.whiteout.insert(rel);
        Ok(true)
    })
}

pub fn cow_exists(ctx: &HostContext, abs: &Path) -> Result<Option<bool>, String> {
    let Some(id) = ctx.sandbox_session.as_deref() else {
        return Ok(None);
    };
    with_session(id, |sess| {
        let rel = rel_from_root(&sess.root, abs)?;
        if sess.whiteout.contains(&rel) {
            return Ok(Some(false));
        }
        if sess.upper.contains_key(&rel) {
            return Ok(Some(true));
        }
        Ok(None)
    })
}

fn rel_from_root(root: &Path, abs: &Path) -> Result<String, String> {
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let abs_canon = abs.canonicalize().unwrap_or_else(|_| abs.to_path_buf());
    let rel = abs_canon
        .strip_prefix(&root_canon)
        .or_else(|_| abs.strip_prefix(root))
        .map_err(|_| format!("path not under sandbox root: {}", abs.display()))?;
    normalize_rel(&rel.to_string_lossy().replace('\\', "/"))
}

pub fn run(
    id: &str,
    source: Option<&str>,
    path: Option<&str>,
) -> Result<serde_json::Value, String> {
    let (root, body) = with_session(id, |sess| {
        let body = if let Some(src) = source.filter(|s| !s.trim().is_empty()) {
            src.to_string()
        } else if let Some(p) = path.filter(|s| !s.is_empty()) {
            let rel = normalize_rel(p)?;
            if sess.whiteout.contains(&rel) {
                return Err(format!("sandbox_run: path whiteout: {rel}"));
            }
            if let Some(bytes) = sess.upper.get(&rel) {
                String::from_utf8(bytes.clone())
                    .map_err(|e| format!("sandbox_run: path not utf-8: {e}"))?
            } else {
                let abs = abs_under_root(&sess.root, &rel)?;
                std::fs::read_to_string(&abs)
                    .map_err(|e| format!("sandbox_run read {}: {e}", abs.display()))?
            }
        } else {
            return Err("sandbox_run requires `source` or `path`".into());
        };
        Ok((sess.root.clone(), body))
    })?;

    let opts = crate::RunOptions {
        allow_fs_write: true,
        allow_exec: false,
        allow_net: false,
        allow_plugin: false,
        fs_root: Some(root),
        sandbox_session: Some(id.to_string()),
        sleep_limit_ms: Some(0),
        ..crate::RunOptions::default()
    };

    match crate::run_source(&body, &opts) {
        Ok(cap) => {
            let value = crate::host::json::value_to_json(&cap.value)
                .unwrap_or(serde_json::Value::Null);
            Ok(json!({
                "ok": true,
                "value": value,
                "stdout": cap.stdout,
            }))
        }
        Err(e) => {
            let msg = e.to_string();
            Ok(json!({
                "ok": false,
                "error": msg,
                "value": null,
                "stdout": "",
            }))
        }
    }
}

/// Parse host_query JSON args helper used from plugin.rs.
pub fn query(name: &str, args: &serde_json::Value, ctx: &HostContext) -> Result<serde_json::Value, String> {
    let id = || {
        args.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{name}: missing `id`"))
    };
    match name {
        "sandbox_open" => {
            let root = args.get("root").and_then(|v| v.as_str());
            open(root, ctx)
        }
        "sandbox_write" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sandbox_write: missing `path`".to_string())?;
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sandbox_write: missing `text`".to_string())?;
            write(id()?, path, text)
        }
        "sandbox_read" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sandbox_read: missing `path`".to_string())?;
            read(id()?, path)
        }
        "sandbox_remove" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "sandbox_remove: missing `path`".to_string())?;
            remove(id()?, path)
        }
        "sandbox_diff" => diff(id()?),
        "sandbox_commit" => commit(id()?),
        "sandbox_discard" => discard(id()?),
        "sandbox_inspect" => inspect(id()?),
        "sandbox_run" => {
            let source = args.get("source").and_then(|v| v.as_str());
            let path = args.get("path").and_then(|v| v.as_str());
            run(id()?, source, path)
        }
        other => Err(format!("sandbox query unknown `{other}`")),
    }
}

/// Used by fs layer when session is active — map Value path through resolve then CoW.
pub fn try_read_value(ctx: &HostContext, path: &Value) -> Result<Option<Value>, String> {
    let rel = match path {
        Value::Text(s) => s.as_str(),
        _ => return Ok(None),
    };
    let abs = ctx.resolve_path(rel)?;
    match cow_read_text(ctx, &abs)? {
        Some(s) => Ok(Some(Value::Text(s))),
        None => Ok(None),
    }
}

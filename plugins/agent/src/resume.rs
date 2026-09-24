//! Agent plan/run resume checkpoints under `.marqdo/agent-resume/`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

const DEFAULT_RESUME_DIR: &str = ".marqdo/agent-resume";

fn resolve_path(rel: &str) -> PathBuf {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return p;
    }
    if p.exists() {
        return p;
    }
    let base = (|| {
        let q = crate::host_query_json("cwd")?;
        q.as_str()
            .map(PathBuf::from)
            .ok_or_else(|| "cwd not text".to_string())
    })()
    .or_else(|_| std::env::current_dir().map_err(|e| e.to_string()))
    .unwrap_or_else(|_| PathBuf::from("."));
    base.join(p)
}

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    opt_text(args, key).ok_or_else(|| format!("missing text `{key}`"))
}

fn opt_i64(args: &Value, key: &str, default: i64) -> i64 {
    match args.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(default),
        Some(Value::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

fn resume_root(args: &Value) -> PathBuf {
    let rel = opt_text(args, "resume_dir").unwrap_or(DEFAULT_RESUME_DIR);
    resolve_path(rel)
}

fn checkpoint_path(root: &Path, id: &str) -> PathBuf {
    let safe: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    root.join(format!("{safe}.json"))
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn ensure_dir(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("resume mkdir: {e}"))
}

pub fn resume_save(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let root = resume_root(args);
    ensure_dir(&root)?;
    let path = checkpoint_path(&root, id);
    let mut doc = json!({
        "id": id,
        "updated_at": now_rfc3339(),
    });
    if let Some(obj) = doc.as_object_mut() {
        for key in [
            "goal", "task", "status", "workbook", "events", "route", "result", "round", "meta",
        ] {
            if let Some(v) = args.get(key) {
                if !v.is_null() {
                    obj.insert(key.to_string(), v.clone());
                }
            }
        }
        if !obj.contains_key("round") {
            obj.insert("round".into(), json!(opt_i64(args, "round", 0)));
        }
        if !obj.contains_key("status") {
            obj.insert("status".into(), json!("running"));
        }
    }
    let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| format!("resume save: {e}"))?;
    Ok(json!({
        "ok": true,
        "id": id,
        "path": path.to_string_lossy(),
        "updated_at": doc.get("updated_at").cloned().unwrap_or(Value::Null),
    }))
}

pub fn resume_load(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let root = resume_root(args);
    let path = checkpoint_path(&root, id);
    if !path.is_file() {
        return Ok(json!({
            "ok": false,
            "found": false,
            "id": id,
            "path": path.to_string_lossy(),
        }));
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("resume load: {e}"))?;
    let mut doc: Value = serde_json::from_str(&text).map_err(|e| format!("resume json: {e}"))?;
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("ok".into(), json!(true));
        obj.insert("found".into(), json!(true));
        obj.insert("path".into(), json!(path.to_string_lossy()));
    }
    Ok(doc)
}

pub fn resume_clear(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let root = resume_root(args);
    let path = checkpoint_path(&root, id);
    let existed = path.is_file();
    if existed {
        fs::remove_file(&path).map_err(|e| format!("resume clear: {e}"))?;
    }
    Ok(json!({
        "ok": true,
        "id": id,
        "cleared": existed,
        "path": path.to_string_lossy(),
    }))
}

pub fn resume_list(args: &Value) -> Result<Value, String> {
    let root = resume_root(args);
    if !root.is_dir() {
        return Ok(json!({ "ok": true, "items": [], "resume_dir": root.to_string_lossy() }));
    }
    let mut items = Vec::new();
    let rd = fs::read_dir(&root).map_err(|e| format!("resume list: {e}"))?;
    for ent in rd.flatten() {
        let p = ent.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let id = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let meta = fs::metadata(&p).ok();
        items.push(json!({
            "id": id,
            "path": p.to_string_lossy(),
            "len": meta.map(|m| m.len()).unwrap_or(0),
        }));
    }
    items.sort_by(|a, b| {
        let ai = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let bi = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        ai.cmp(bi)
    });
    Ok(json!({
        "ok": true,
        "items": items,
        "resume_dir": root.to_string_lossy(),
    }))
}

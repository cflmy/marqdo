//! Agent plan/run resume checkpoints under `.marqdo/agent-resume/`.
//! M-A1: CheckpointV2 atomic write (temp→fsync→rename); legacy load → manual_review.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
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
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
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

/// Host-relative resume roots only — reject absolute client paths (M-A1).
fn resume_root(args: &Value) -> Result<PathBuf, String> {
    let rel = opt_text(args, "resume_dir").unwrap_or(DEFAULT_RESUME_DIR);
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return Err(
            "resume_dir must be host-relative; absolute client paths are rejected".into(),
        );
    }
    if rel.contains("..") {
        return Err("resume_dir must not contain '..'".into());
    }
    Ok(resolve_path(rel))
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

/// Atomic durable write: unique temp → write → flush/fsync → rename.
fn atomic_write_json(path: &Path, doc: &Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "checkpoint path has no parent".to_string())?;
    ensure_dir(parent)?;
    let tmp_name = format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("ckpt.json"),
        std::process::id(),
        now_rfc3339().replace(':', "")
    );
    let tmp = parent.join(tmp_name);
    let text = serde_json::to_string_pretty(doc).map_err(|e| e.to_string())?;
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|e| format!("resume temp open: {e}"))?;
        f.write_all(text.as_bytes())
            .map_err(|e| format!("resume temp write: {e}"))?;
        f.flush().map_err(|e| format!("resume temp flush: {e}"))?;
        f.sync_all()
            .map_err(|e| format!("resume temp fsync: {e}"))?;
    }
    // Best-effort directory fsync (platform may ignore).
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("resume atomic rename: {e}")
    })?;
    Ok(())
}

fn wants_checkpoint_v2(args: &Value) -> bool {
    matches!(
        args.get("schema").and_then(|v| v.as_str()),
        Some("CheckpointV2")
    ) || args.get("schema_version").and_then(|v| v.as_u64()) == Some(2)
        || args.get("checkpoint_v2").and_then(|v| v.as_bool()) == Some(true)
}

fn build_checkpoint_v2(args: &Value, id: &str) -> Value {
    let committed = now_rfc3339();
    let mut doc = json!({
        "schema": "CheckpointV2",
        "schema_version": 2,
        "checkpoint_id": args.get("checkpoint_id").cloned().unwrap_or_else(|| json!(id)),
        "logical_run_id": args.get("logical_run_id").cloned().unwrap_or_else(|| json!(id)),
        "task_id": args.get("task_id").cloned().unwrap_or_else(|| json!(id)),
        "attempt_id": args.get("attempt_id").cloned().unwrap_or_else(|| json!("attempt_1")),
        "step_id": args.get("step_id").cloned().unwrap_or(Value::Null),
        "parent_checkpoint_id": args.get("parent_checkpoint_id").cloned().unwrap_or(Value::Null),
        "principal_ref": args.get("principal_ref").cloned().unwrap_or(Value::Null),
        "workspace_id": args.get("workspace_id").cloned().unwrap_or(Value::Null),
        "session_id": args.get("session_id").cloned().unwrap_or(Value::Null),
        "completed_steps": args.get("completed_steps").cloned().unwrap_or_else(|| json!([])),
        "pending_operation": args.get("pending_operation").cloned().unwrap_or(Value::Null),
        "deadline": args.get("deadline").cloned().unwrap_or(Value::Null),
        "committed_at": args.get("committed_at").cloned().unwrap_or_else(|| json!(committed)),
        // Effect watermark — absent/null means no ledger yet (write auto-resume closed).
        "effect_ledger_watermark": args.get("effect_ledger_watermark").cloned().unwrap_or(Value::Null),
        "event_seq": args.get("event_seq").cloned().unwrap_or(Value::Null),
        "manifest_digest": args.get("manifest_digest").cloned().unwrap_or(Value::Null),
    });
    if let Some(obj) = doc.as_object_mut() {
        // Preserve legacy author fields for dual-read.
        for key in [
            "goal", "task", "status", "workbook", "events", "route", "result", "round", "meta",
            "id",
        ] {
            if let Some(v) = args.get(key) {
                if !v.is_null() {
                    obj.insert(key.to_string(), v.clone());
                }
            }
        }
        if !obj.contains_key("id") {
            obj.insert("id".into(), json!(id));
        }
        if !obj.contains_key("updated_at") {
            obj.insert("updated_at".into(), json!(committed.clone()));
        }
        if !obj.contains_key("round") {
            obj.insert("round".into(), json!(opt_i64(args, "round", 0)));
        }
        if !obj.contains_key("status") {
            obj.insert("status".into(), json!("running"));
        }
    }
    doc
}

fn build_legacy_doc(args: &Value, id: &str) -> Value {
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
    doc
}

pub fn resume_save(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let root = resume_root(args)?;
    ensure_dir(&root)?;
    let path = checkpoint_path(&root, id);
    let v2 = wants_checkpoint_v2(args);
    let doc = if v2 {
        build_checkpoint_v2(args, id)
    } else {
        build_legacy_doc(args, id)
    };
    atomic_write_json(&path, &doc)?;
    Ok(json!({
        "ok": true,
        "id": id,
        "path": path.to_string_lossy(),
        "schema": doc.get("schema").cloned().unwrap_or(Value::Null),
        "schema_version": doc.get("schema_version").cloned().unwrap_or(Value::Null),
        "updated_at": doc.get("updated_at").or_else(|| doc.get("committed_at")).cloned().unwrap_or(Value::Null),
        "atomic": true,
    }))
}

fn watermark_allows_write_auto(wm: &Value) -> bool {
    match wm {
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Object(obj) => {
            // Prefer explicit safety bit from effects_watermark; bare digest objects
            // without the bit are treated as unsafe (open unknown / pending writes).
            match obj.get("write_auto_resume_safe").and_then(|v| v.as_bool()) {
                Some(b) => b,
                None => obj
                    .get("watermark")
                    .and_then(|v| v.as_str())
                    .map(|s| !s.is_empty())
                    .unwrap_or(false)
                    && obj.get("open_unknown").and_then(|v| v.as_u64()).unwrap_or(1) == 0
                    && obj.get("pending_writes").and_then(|v| v.as_u64()).unwrap_or(1) == 0,
            }
        }
        _ => false,
    }
}

fn annotate_recovery(mut doc: Value) -> Value {
    let is_v2 = doc.get("schema").and_then(|v| v.as_str()) == Some("CheckpointV2")
        && doc.get("schema_version").and_then(|v| v.as_u64()) == Some(2);
    let wm = doc
        .get("effect_ledger_watermark")
        .cloned()
        .unwrap_or(Value::Null);
    let ledger_safe = watermark_allows_write_auto(&wm);
    let (recovery_mode, write_auto_resume) = if is_v2 && ledger_safe {
        ("checkpoint_v2", true)
    } else if is_v2 {
        // V2 without a safe ledger watermark: read resume OK; write auto-resume closed.
        ("checkpoint_v2_read_only", false)
    } else {
        // Legacy: no effect info → manual_review; never auto redo writes.
        ("manual_review", false)
    };
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("recovery_mode".into(), json!(recovery_mode));
        obj.insert("write_auto_resume".into(), json!(write_auto_resume));
        if !is_v2 {
            obj.insert("legacy_adaptation".into(), json!("manual_review"));
        }
    }
    doc
}

pub fn resume_load(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let root = resume_root(args)?;
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
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return Ok(json!({
                "ok": false,
                "found": true,
                "id": id,
                "path": path.to_string_lossy(),
                "error": {
                    "schema": "AgentErrorV1",
                    "schema_version": 1,
                    "code": "checkpoint.corrupt",
                    "category": "unknown_effect",
                    "stage": "checkpoint",
                    "message": format!("corrupt checkpoint json: {e}"),
                    "retryable": false,
                    "effect_state": "unknown",
                    "recovery_action": "manual_review",
                },
                "recovery_mode": "manual_review",
                "write_auto_resume": false,
            }));
        }
    };
    let mut doc = annotate_recovery(parsed);
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("ok".into(), json!(true));
        obj.insert("found".into(), json!(true));
        obj.insert("path".into(), json!(path.to_string_lossy()));
    }
    Ok(doc)
}

pub fn resume_clear(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let root = resume_root(args)?;
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
    let root = resume_root(args)?;
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
        // Skip leftover temp files from crashed writers.
        if p.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("marqdo-resume-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn absolute_resume_dir_rejected() {
        let err = resume_root(&json!({"resume_dir": "/tmp/evil"})).unwrap_err();
        assert!(err.contains("host-relative"));
    }

    #[test]
    fn atomic_v2_roundtrip_and_legacy_manual_review() {
        let dir = scratch("v2");
        // Use relative path via env cwd — chdir into scratch for this test.
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let rel = "ckpts";
        let saved = resume_save(&json!({
            "id": "run1",
            "schema": "CheckpointV2",
            "schema_version": 2,
            "goal": "ping",
            "status": "running",
            "round": 1,
            "resume_dir": rel,
            "attempt_id": "a1",
        }))
        .unwrap();
        assert_eq!(saved["ok"], true);
        assert_eq!(saved["atomic"], true);
        assert_eq!(saved["schema"], "CheckpointV2");

        let loaded = resume_load(&json!({"id": "run1", "resume_dir": rel})).unwrap();
        assert_eq!(loaded["found"], true);
        assert_eq!(loaded["schema"], "CheckpointV2");
        assert_eq!(loaded["recovery_mode"], "checkpoint_v2_read_only");
        assert_eq!(loaded["write_auto_resume"], false);

        // Legacy document without schema.
        let _ = resume_save(&json!({
            "id": "legacy1",
            "goal": "pong",
            "status": "interrupted",
            "round": 2,
            "resume_dir": rel,
        }))
        .unwrap();
        let legacy = resume_load(&json!({"id": "legacy1", "resume_dir": rel})).unwrap();
        assert_eq!(legacy["recovery_mode"], "manual_review");
        assert_eq!(legacy["write_auto_resume"], false);
        assert_eq!(legacy["legacy_adaptation"], "manual_review");

        // V2 + safe watermark → write_auto_resume opens.
        let _ = resume_save(&json!({
            "id": "run_safe",
            "schema": "CheckpointV2",
            "schema_version": 2,
            "status": "running",
            "round": 3,
            "resume_dir": rel,
            "attempt_id": "a2",
            "effect_ledger_watermark": {
                "schema": "EffectLedgerWatermarkV1",
                "watermark": "sha256-lite:abc",
                "write_auto_resume_safe": true,
                "open_unknown": 0,
                "pending_writes": 0
            }
        }))
        .unwrap();
        let safe = resume_load(&json!({"id": "run_safe", "resume_dir": rel})).unwrap();
        assert_eq!(safe["recovery_mode"], "checkpoint_v2");
        assert_eq!(safe["write_auto_resume"], true);

        // Unsafe watermark (open unknown) keeps write auto-resume closed.
        let _ = resume_save(&json!({
            "id": "run_unsafe",
            "schema": "CheckpointV2",
            "schema_version": 2,
            "status": "running",
            "round": 4,
            "resume_dir": rel,
            "attempt_id": "a3",
            "effect_ledger_watermark": {
                "watermark": "sha256-lite:def",
                "write_auto_resume_safe": false,
                "open_unknown": 1
            }
        }))
        .unwrap();
        let unsafe_ck = resume_load(&json!({"id": "run_unsafe", "resume_dir": rel})).unwrap();
        assert_eq!(unsafe_ck["recovery_mode"], "checkpoint_v2_read_only");
        assert_eq!(unsafe_ck["write_auto_resume"], false);

        std::env::set_current_dir(prev).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_checkpoint_reports_error() {
        let dir = scratch("corrupt");
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        fs::create_dir_all("ckpts").unwrap();
        fs::write("ckpts/bad.json", "{not-json").unwrap();
        let loaded = resume_load(&json!({"id": "bad", "resume_dir": "ckpts"})).unwrap();
        assert_eq!(loaded["found"], true);
        assert_eq!(loaded["ok"], false);
        assert_eq!(loaded["error"]["code"], "checkpoint.corrupt");
        assert_eq!(loaded["recovery_mode"], "manual_review");
        std::env::set_current_dir(prev).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}

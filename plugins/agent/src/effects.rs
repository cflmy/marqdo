//! EffectLedgerRecordV1 — operation receipts for write-safe resume (M-A5 subset).
//!
//! State machine (design §4.2):
//! prepared → started → committed
//!                  ↘ failed_no_effect
//!                  ↘ unknown → reconciled_committed | reconciled_no_effect | needs_input

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

const DEFAULT_LEDGER_DIR: &str = ".marqdo/agent-effects";

const STATES: &[&str] = &[
    "prepared",
    "started",
    "committed",
    "failed_no_effect",
    "unknown",
    "reconciled_committed",
    "reconciled_no_effect",
    "needs_input",
];

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    opt_text(args, key).ok_or_else(|| format!("missing text `{key}`"))
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

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

fn ledger_root(args: &Value) -> Result<PathBuf, String> {
    let rel = opt_text(args, "ledger_dir").unwrap_or(DEFAULT_LEDGER_DIR);
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return Err(
            "ledger_dir must be host-relative; absolute client paths are rejected".into(),
        );
    }
    if rel.contains("..") {
        return Err("ledger_dir must not contain '..'".into());
    }
    Ok(resolve_path(rel))
}

fn safe_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn record_path(root: &Path, operation_id: &str) -> PathBuf {
    root.join(format!("{}.json", safe_id(operation_id)))
}

fn ensure_dir(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("effects mkdir: {e}"))
}

fn atomic_write_json(path: &Path, doc: &Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "ledger path has no parent".to_string())?;
    ensure_dir(parent)?;
    let tmp_name = format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("op.json"),
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
            .map_err(|e| format!("effects temp open: {e}"))?;
        f.write_all(text.as_bytes())
            .map_err(|e| format!("effects temp write: {e}"))?;
        f.flush().map_err(|e| format!("effects temp flush: {e}"))?;
        f.sync_all()
            .map_err(|e| format!("effects temp fsync: {e}"))?;
    }
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("effects atomic rename: {e}")
    })?;
    Ok(())
}

fn load_record(path: &Path) -> Result<Option<Value>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| format!("effects read: {e}"))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("effects corrupt: {e}"))?;
    Ok(Some(doc))
}

fn valid_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("prepared", "started")
            | ("started", "committed")
            | ("started", "failed_no_effect")
            | ("started", "unknown")
            | ("unknown", "reconciled_committed")
            | ("unknown", "reconciled_no_effect")
            | ("unknown", "needs_input")
            // Idempotent no-ops / crash retries of the same terminal.
            | ("committed", "committed")
            | ("failed_no_effect", "failed_no_effect")
            | ("reconciled_committed", "reconciled_committed")
            | ("reconciled_no_effect", "reconciled_no_effect")
            | ("needs_input", "needs_input")
            // Re-prepare of same op is handled separately via idempotency.
            | ("prepared", "prepared")
    )
}

fn effect_class_ok(c: &str) -> bool {
    matches!(c, "read" | "write" | "execute" | "external_write" | "none")
}

/// Prepare (or return existing) an EffectLedgerRecordV1.
pub fn effects_prepare(args: &Value) -> Result<Value, String> {
    let operation_id = arg_text(args, "operation_id")?;
    let idempotency_key = arg_text(args, "idempotency_key")?;
    let args_digest = arg_text(args, "args_digest")?;
    let effect_class = opt_text(args, "effect_class").unwrap_or("write");
    if !effect_class_ok(effect_class) {
        return Err(format!("invalid effect_class={effect_class}"));
    }
    let root = ledger_root(args)?;
    ensure_dir(&root)?;
    let path = record_path(&root, operation_id);

    if let Some(existing) = load_record(&path)? {
        let same_key = existing
            .get("idempotency_key")
            .and_then(|v| v.as_str())
            == Some(idempotency_key);
        let same_digest = existing.get("args_digest").and_then(|v| v.as_str()) == Some(args_digest);
        if same_key && same_digest {
            return Ok(json!({
                "ok": true,
                "deduped": true,
                "record": existing,
            }));
        }
        if same_key && !same_digest {
            return Err(
                "idempotency_key collision: same key with different args_digest".into(),
            );
        }
        // Different operation_id path can't collide; same path different key is corrupt reuse.
        return Err("operation_id already exists with different idempotency_key".into());
    }

    // Also scan for same idempotency_key under other operation_ids in this ledger dir.
    if let Ok(rd) = fs::read_dir(&root) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(Some(doc)) = load_record(&p) {
                if doc.get("idempotency_key").and_then(|v| v.as_str()) == Some(idempotency_key) {
                    let dig = doc.get("args_digest").and_then(|v| v.as_str());
                    if dig == Some(args_digest) {
                        return Ok(json!({
                            "ok": true,
                            "deduped": true,
                            "record": doc,
                        }));
                    }
                    return Err(
                        "idempotency_key collision: same key with different args_digest".into(),
                    );
                }
            }
        }
    }

    let doc = json!({
        "schema": "EffectLedgerRecordV1",
        "schema_version": 1,
        "operation_id": operation_id,
        "idempotency_key": idempotency_key,
        "args_digest": args_digest,
        "resource_scope": args.get("resource_scope").cloned().unwrap_or(Value::Null),
        "expected_revision": args.get("expected_revision").cloned().unwrap_or(Value::Null),
        "effect_class": effect_class,
        "result_ref": args.get("result_ref").cloned().unwrap_or(Value::Null),
        "state": "prepared",
        "logical_run_id": args.get("logical_run_id").cloned().unwrap_or(Value::Null),
        "step_id": args.get("step_id").cloned().unwrap_or(Value::Null),
        "attempt_id": args.get("attempt_id").cloned().unwrap_or(Value::Null),
        "prepared_at": now_rfc3339(),
        "updated_at": now_rfc3339(),
    });
    atomic_write_json(&path, &doc)?;
    Ok(json!({
        "ok": true,
        "deduped": false,
        "record": doc,
        "path": path.to_string_lossy(),
    }))
}

/// Transition an existing record to a new state.
pub fn effects_transition(args: &Value) -> Result<Value, String> {
    let operation_id = arg_text(args, "operation_id")?;
    let to = arg_text(args, "state")?;
    if !STATES.contains(&to) {
        return Err(format!("invalid state={to}"));
    }
    let root = ledger_root(args)?;
    let path = record_path(&root, operation_id);
    let mut doc = load_record(&path)?
        .ok_or_else(|| format!("operation_id not found: {operation_id}"))?;
    let from = doc
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("prepared")
        .to_string();
    if !valid_transition(&from, to) {
        return Err(format!("illegal transition {from} → {to}"));
    }
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("state".into(), json!(to));
        obj.insert("updated_at".into(), json!(now_rfc3339()));
        if let Some(r) = args.get("result_ref") {
            if !r.is_null() {
                obj.insert("result_ref".into(), r.clone());
            }
        }
        if let Some(n) = args.get("note") {
            if !n.is_null() {
                obj.insert("note".into(), n.clone());
            }
        }
        if to == "unknown" || to.starts_with("reconciled") {
            obj.insert(
                "reconcile_hint".into(),
                args.get("reconcile_hint")
                    .cloned()
                    .unwrap_or_else(|| json!("check receipt / resource revision")),
            );
        }
    }
    atomic_write_json(&path, &doc)?;
    Ok(json!({
        "ok": true,
        "from": from,
        "to": to,
        "record": doc,
    }))
}

pub fn effects_get(args: &Value) -> Result<Value, String> {
    let operation_id = arg_text(args, "operation_id")?;
    let root = ledger_root(args)?;
    let path = record_path(&root, operation_id);
    match load_record(&path)? {
        Some(doc) => Ok(json!({
            "ok": true,
            "found": true,
            "record": doc,
            "path": path.to_string_lossy(),
        })),
        None => Ok(json!({
            "ok": true,
            "found": false,
            "operation_id": operation_id,
            "path": path.to_string_lossy(),
        })),
    }
}

pub fn effects_list(args: &Value) -> Result<Value, String> {
    let root = ledger_root(args)?;
    let mut records = Vec::new();
    if root.is_dir() {
        let mut paths: Vec<_> = fs::read_dir(&root)
            .map_err(|e| e.to_string())?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();
        paths.sort();
        for p in paths {
            if let Ok(Some(doc)) = load_record(&p) {
                if let Some(run) = opt_text(args, "logical_run_id") {
                    if doc.get("logical_run_id").and_then(|v| v.as_str()) != Some(run) {
                        continue;
                    }
                }
                if let Some(st) = opt_text(args, "state") {
                    if doc.get("state").and_then(|v| v.as_str()) != Some(st) {
                        continue;
                    }
                }
                records.push(doc);
            }
        }
    }
    Ok(json!({
        "ok": true,
        "count": records.len(),
        "records": records,
        "ledger_dir": root.to_string_lossy(),
    }))
}

/// Compute a watermark digest over committed / reconciled records for a run.
pub fn effects_watermark(args: &Value) -> Result<Value, String> {
    let listed = effects_list(args)?;
    let records = listed
        .get("records")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut parts = Vec::new();
    let mut open_unknown = 0u64;
    let mut pending_writes = 0u64;
    for r in &records {
        let state = r.get("state").and_then(|v| v.as_str()).unwrap_or("");
        let oid = r.get("operation_id").and_then(|v| v.as_str()).unwrap_or("");
        let cls = r.get("effect_class").and_then(|v| v.as_str()).unwrap_or("");
        parts.push(format!("{oid}:{state}"));
        if state == "unknown" || state == "needs_input" {
            open_unknown += 1;
        }
        if matches!(cls, "write" | "external_write" | "execute")
            && matches!(state, "prepared" | "started" | "unknown" | "needs_input")
        {
            pending_writes += 1;
        }
    }
    parts.sort();
    let joined = parts.join("|");
    let digest = format!("sha256-lite:{:x}", fnv64(&joined));
    let write_auto_resume_safe = open_unknown == 0 && pending_writes == 0;
    Ok(json!({
        "ok": true,
        "schema": "EffectLedgerWatermarkV1",
        "schema_version": 1,
        "watermark": digest,
        "record_count": records.len(),
        "open_unknown": open_unknown,
        "pending_writes": pending_writes,
        "write_auto_resume_safe": write_auto_resume_safe,
        "logical_run_id": args.get("logical_run_id").cloned().unwrap_or(Value::Null),
    }))
}

/// Reconcile an `unknown` record after inspecting the external receipt / revision.
pub fn effects_reconcile(args: &Value) -> Result<Value, String> {
    let outcome = arg_text(args, "outcome")?; // committed | no_effect | needs_input
    let state = match outcome {
        "committed" => "reconciled_committed",
        "no_effect" => "reconciled_no_effect",
        "needs_input" => "needs_input",
        other => return Err(format!("invalid reconcile outcome={other}")),
    };
    let mut targs = args.clone();
    if let Some(obj) = targs.as_object_mut() {
        obj.insert("state".into(), json!(state));
    }
    effects_transition(&targs)
}

fn fnv64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mq-effects-{}-{}-{}",
            tag,
            std::process::id(),
            now_rfc3339().replace(':', "")
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn prepare_dedupe_and_collision() {
        let dir = scratch("prep");
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let a = effects_prepare(&json!({
            "operation_id": "op1",
            "idempotency_key": "idem1",
            "args_digest": "sha256:a",
            "effect_class": "write",
            "ledger_dir": "led",
            "logical_run_id": "run1",
        }))
        .unwrap();
        assert_eq!(a["deduped"], false);
        assert_eq!(a["record"]["state"], "prepared");

        let again = effects_prepare(&json!({
            "operation_id": "op1",
            "idempotency_key": "idem1",
            "args_digest": "sha256:a",
            "ledger_dir": "led",
        }))
        .unwrap();
        assert_eq!(again["deduped"], true);

        let clash = effects_prepare(&json!({
            "operation_id": "op2",
            "idempotency_key": "idem1",
            "args_digest": "sha256:b",
            "ledger_dir": "led",
        }));
        assert!(clash.unwrap_err().contains("collision"));

        std::env::set_current_dir(prev).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn transition_and_watermark_gates_write_resume() {
        let dir = scratch("wm");
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        effects_prepare(&json!({
            "operation_id": "op_w",
            "idempotency_key": "idem_w",
            "args_digest": "sha256:w",
            "effect_class": "write",
            "ledger_dir": "led",
            "logical_run_id": "run1",
        }))
        .unwrap();
        effects_transition(&json!({
            "operation_id": "op_w",
            "state": "started",
            "ledger_dir": "led",
        }))
        .unwrap();
        // Crash window: unknown — write auto-resume must stay closed.
        effects_transition(&json!({
            "operation_id": "op_w",
            "state": "unknown",
            "ledger_dir": "led",
        }))
        .unwrap();
        let wm1 = effects_watermark(&json!({
            "ledger_dir": "led",
            "logical_run_id": "run1",
        }))
        .unwrap();
        assert_eq!(wm1["write_auto_resume_safe"], false);
        assert_eq!(wm1["open_unknown"], 1);

        effects_reconcile(&json!({
            "operation_id": "op_w",
            "outcome": "committed",
            "ledger_dir": "led",
            "result_ref": "receipt:1",
        }))
        .unwrap();
        let wm2 = effects_watermark(&json!({
            "ledger_dir": "led",
            "logical_run_id": "run1",
        }))
        .unwrap();
        assert_eq!(wm2["write_auto_resume_safe"], true);
        assert_eq!(wm2["open_unknown"], 0);
        assert!(wm2["watermark"].as_str().unwrap().starts_with("sha256-lite:"));

        std::env::set_current_dir(prev).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn illegal_skip_started() {
        let dir = scratch("bad");
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        effects_prepare(&json!({
            "operation_id": "op_x",
            "idempotency_key": "idem_x",
            "args_digest": "sha256:x",
            "ledger_dir": "led",
        }))
        .unwrap();
        let err = effects_transition(&json!({
            "operation_id": "op_x",
            "state": "committed",
            "ledger_dir": "led",
        }))
        .unwrap_err();
        assert!(err.contains("illegal transition"));
        std::env::set_current_dir(prev).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}

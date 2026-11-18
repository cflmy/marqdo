//! Durable input gates for document-native SKILLs.
//! Pausing is a committed *step boundary*, not serialization of a Rust stack.
//! The caller resumes the next documented step with the saved answer.

use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use serde_json::{json, Value};

use crate::resume;

struct GateLock(PathBuf);
impl Drop for GateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn text_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing text `{key}`"))
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

fn root(args: &Value) -> Result<PathBuf, String> {
    let name = args
        .get("resume_dir")
        .and_then(Value::as_str)
        .unwrap_or(".marqdo/agent-resume");
    let path = PathBuf::from(name);
    if path.is_absolute()
        || path.components().any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("resume_dir must be a relative path without '..'".to_string());
    }
    fs::create_dir_all(&path).map_err(|e| format!("input gate mkdir: {e}"))?;
    Ok(path)
}

fn lock(args: &Value, id: &str) -> Result<GateLock, String> {
    if !valid_id(id) {
        return Err("input gate id must be 1..=100 ASCII letters, numbers, '_' or '-'".into());
    }
    let path = root(args)?.join(format!("{id}.input-lock"));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("input gate is busy or needs reconciliation: {e}"))?;
    Ok(GateLock(path))
}

fn options(args: &Value) -> Result<Value, String> {
    let choices = args.get("options").and_then(Value::as_array)
        .ok_or_else(|| "options must be a list of 1..=32 short labels".to_string())?;
    if choices.is_empty() || choices.len() > 32 {
        return Err("options must contain 1..=32 choices".into());
    }
    for v in choices {
        let Some(s) = v.as_str() else {
            return Err("every option must be text".into());
        };
        if s.is_empty() || s.len() > 256 {
            return Err("each option must contain 1..=256 bytes".into());
        }
    }
    let empty = crate::choice::parse_choice(&json!({
        "options": choices, "reply": "__invalid_choice_probe__"
    }))?;
    // The parser checks duplicates and option size; the probe must not be accepted.
    if empty["ok"] == true {
        return Err("invalid reserved choice option".into());
    }
    Ok(Value::Array(choices.to_vec()))
}

/// Create a durable waiting-input checkpoint. Repeated open never resets a
/// previously answered gate. No external effect is performed.
pub fn input_open(args: &Value) -> Result<Value, String> {
    let id = text_arg(args, "id")?;
    let _lock = lock(args, id)?;
    let labels = options(args)?;
    let query = text_arg(args, "question")?;
    let version = text_arg(args, "source_revision")?;
    let read_args = json!({"id": id, "resume_dir": args.get("resume_dir")});
    let existing = resume::resume_load(&read_args)?;
    if existing["found"] == true {
        let meta = &existing["meta"];
        let same = meta["question"] == query
            && meta["options"] == labels
            && meta["source_revision"] == version
            && meta["session_id"] == args.get("session_id").unwrap_or(&Value::Null)
            && meta["principal_ref"] == args.get("principal_ref").unwrap_or(&Value::Null);
        if same && existing["ok"] == true {
            let status = existing["status"].as_str().unwrap_or("unknown");
            return Ok(json!({
                "ok": true, "id": id, "status": status,
                "value": existing["result"], "reused": true,
                "question": query, "options": labels
            }));
        }
        return Ok(json!({"ok":false,"id":id,"status":"conflict",
            "error":{"code":"input.stale_or_reused_gate"}}));
    }
    let saved = resume::resume_save(&json!({
        "id": id, "resume_dir": args.get("resume_dir"),
        "checkpoint_v2": true,
        "status": "waiting_input",
        "task": query,
        "meta": {
            "question": query,
            "options": labels,
            "source_revision": version,
            "session_id": args.get("session_id").unwrap_or(&Value::Null),
            "principal_ref": args.get("principal_ref").unwrap_or(&Value::Null),
            "freeform": args.get("freeform").and_then(Value::as_bool).unwrap_or(true)
        }
    }))?;
    if saved["ok"] != true {
        return Err("unable to save input checkpoint".into());
    }
    Ok(json!({"ok": true, "id":id, "status":"waiting_input", "value":null,
        "question":query,"options":labels,"reused":false}))
}

/// Resume a specific gate with a user response. Only a committed answer can be
/// replayed: different duplicate replies conflict instead of re-triggering work.
pub fn input_resume(args: &Value) -> Result<Value, String> {
    let id = text_arg(args, "id")?;
    let _lock = lock(args, id)?;
    let reply = text_arg(args, "reply")?;
    if reply.len() > 4096 {
        return Err("input reply must not exceed 4096 bytes".into());
    }
    let version = text_arg(args, "source_revision")?;
    let read_args = json!({"id": id, "resume_dir": args.get("resume_dir")});
    let existing = resume::resume_load(&read_args)?;
    if existing["ok"] != true || existing["schema"] != "CheckpointV2" {
        return Ok(json!({"ok":false,"status":"not_found_or_corrupt","id":id}));
    }
    let meta = &existing["meta"];
    if meta["source_revision"] != version
        || meta["session_id"] != *args.get("session_id").unwrap_or(&Value::Null)
        || meta["principal_ref"] != *args.get("principal_ref").unwrap_or(&Value::Null)
    {
        return Ok(json!({"ok":false,"status":"conflict","id":id,
            "error":{"code":"input.scope_or_revision_changed"}}));
    }
    if existing["status"] == "resumed" {
        if existing["result"] == reply {
            return Ok(json!({"ok":true,"status":"resumed","id":id,
                "value":reply,"reused":true}));
        }
        return Ok(json!({"ok":false,"status":"conflict","id":id,
            "error":{"code":"input.already_answered"}}));
    }
    if existing["status"] != "waiting_input" {
        return Ok(json!({"ok":false,"status":"conflict","id":id}));
    }
    let allowed = crate::choice::parse_choice(&json!({
        "options": meta["options"], "reply": reply
    }))?;
    let normalized = if allowed["ok"] == true {
        allowed["value"].clone()
    } else if meta["freeform"] == true && !reply.trim().is_empty() {
        json!(reply.trim())
    } else {
        return Ok(json!({"ok":false,"status":"invalid","id":id,
            "error":{"code":"input.invalid_choice"}}));
    };
    let saved = resume::resume_save(&json!({
        "id": id, "resume_dir": args.get("resume_dir"),
        "checkpoint_v2": true,
        "status": "resumed",
        "task": existing["task"],
        "meta": meta,
        "result": normalized
    }))?;
    if saved["ok"] != true {
        return Err("unable to commit resumed input".into());
    }
    Ok(json!({"ok":true,"status":"resumed","id":id,
        "value":normalized,"reused":false}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn fixture() -> (String, String) {
        let id = format!("gate_{}_{}",std::process::id(),SEQ.fetch_add(1,Ordering::Relaxed));
        let dir = format!("target/mq-gate-test-{id}");
        (id, dir)
    }

    #[test]
    fn persists_answer_without_duplicate_side_effect_or_changed_reply() {
        let (id, dir) = fixture();
        let initial = json!({"id":id,"resume_dir":dir,"question":"Choose?",
            "options":["chat","office"],"source_revision":"v1",
            "session_id":"s1","principal_ref":"u1","freeform":false});
        assert_eq!(input_open(&initial).unwrap()["status"], "waiting_input");
        assert_eq!(input_open(&initial).unwrap()["reused"], true);
        let v = json!({"id":id,"resume_dir":dir,"reply":"2",
            "source_revision":"v1","session_id":"s1","principal_ref":"u1"});
        assert_eq!(input_resume(&v).unwrap()["value"], "office");
        assert_eq!(input_resume(&json!({"id":id,"resume_dir":dir,
            "reply":"office","source_revision":"v1","session_id":"s1",
            "principal_ref":"u1"})).unwrap()["reused"], true);
        assert_eq!(input_resume(&json!({"id":id,"resume_dir":dir,
            "reply":"chat","source_revision":"v1","session_id":"s1",
            "principal_ref":"u1"})).unwrap()["status"], "conflict");
        assert_eq!(input_open(&initial).unwrap()["status"], "resumed");
    }

    #[test]
    fn prevents_changed_source_or_unauthorized_session() {
        let (id, dir) = fixture();
        let initial = json!({"id":id,"resume_dir":dir,"question":"Proceed?",
            "options":["yes","no"],"source_revision":"v1",
            "session_id":"s1","principal_ref":"u1"});
        input_open(&initial).unwrap();
        for (rev,session) in [("v2","s1"),("v1","other")] {
            assert_eq!(input_resume(&json!({"id":id,"resume_dir":dir,"reply":"yes",
                "source_revision":rev,"session_id":session,
                "principal_ref":"u1"})).unwrap()["status"], "conflict");
        }
    }

    #[test]
    fn invalid_choice_keeps_gate_open_and_no_freeform_is_enforced() {
        let (id, dir) = fixture();
        input_open(&json!({"id":id,"resume_dir":dir,"question":"Pick",
            "options":["a","b"],"freeform":false,
            "source_revision":"v1"})).unwrap();
        assert_eq!(input_resume(&json!({"id":id,"resume_dir":dir,"reply":"??",
            "source_revision":"v1"})).unwrap()["status"], "invalid");
        assert_eq!(resume::resume_load(&json!({"id":id,"resume_dir":dir})).unwrap()["status"], "waiting_input");
    }
}

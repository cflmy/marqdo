//! RunContextV1 — request-local run identity for selection / usage / cancel (M01 / M-R1).

use serde_json::{json, Map, Value};

/// Explicit request Context handles (M-R1). Paths are opaque grants owned by the
/// control plane; callers must not pass browser-supplied absolute paths unchecked.
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub workspace_id: Option<String>,
    pub snapshot_id: Option<String>,
    pub selection_ref: Option<String>,
    pub capabilities_ref: Option<String>,
    pub deadline_at: Option<String>,
    pub lease_id: Option<String>,
    pub attempt_id: Option<String>,
    pub epoch: Option<u64>,
}

impl RequestContext {
    pub fn from_value(v: &Value) -> Self {
        Self {
            workspace_id: str_opt(v, "workspace_id"),
            snapshot_id: str_opt(v, "snapshot_id"),
            selection_ref: str_opt(v, "selection_ref"),
            capabilities_ref: str_opt(v, "capabilities_ref"),
            deadline_at: str_opt(v, "deadline_at"),
            lease_id: str_opt(v, "lease_id").or_else(|| {
                v.get("lease")
                    .and_then(|l| l.get("id"))
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string())
            }),
            attempt_id: str_opt(v, "attempt_id"),
            epoch: v
                .get("epoch")
                .and_then(|x| x.as_u64())
                .or_else(|| {
                    v.get("lease")
                        .and_then(|l| l.get("epoch"))
                        .and_then(|x| x.as_u64())
                }),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "workspace_id": self.workspace_id,
            "snapshot_id": self.snapshot_id,
            "selection_ref": self.selection_ref,
            "capabilities_ref": self.capabilities_ref,
            "deadline_at": self.deadline_at,
            "lease_id": self.lease_id,
            "attempt_id": self.attempt_id,
            "epoch": self.epoch,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunContext {
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub turn_id: Option<String>,
    pub workspace_scope: Option<String>,
    pub selection_context: Map<String, Value>,
    pub usage_sink: Option<String>,
    pub event_sink: Option<String>,
    pub cancellation_token: Option<String>,
    /// Explicit Context object (M-R1); serialized alongside RunContextV1 fields.
    pub context: RequestContext,
}

impl RunContext {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let run_id = v
            .get("run_id")
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "run_context.missing:run_id".to_string())?
            .to_string();
        let context = v
            .get("context")
            .map(RequestContext::from_value)
            .unwrap_or_default();
        Ok(Self {
            run_id,
            parent_run_id: v
                .get("parent_run_id")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            turn_id: v
                .get("turn_id")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            workspace_scope: v
                .get("workspace_scope")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            selection_context: v
                .get("selection_context")
                .and_then(|x| x.as_object())
                .cloned()
                .unwrap_or_default(),
            usage_sink: v
                .get("usage_sink")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            event_sink: v
                .get("event_sink")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            cancellation_token: v
                .get("cancellation_token")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            context,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schema": "RunContextV1",
            "schema_version": 1,
            "run_id": self.run_id,
            "parent_run_id": self.parent_run_id,
            "turn_id": self.turn_id,
            "workspace_scope": self.workspace_scope,
            "selection_context": self.selection_context,
            "usage_sink": self.usage_sink,
            "event_sink": self.event_sink,
            "cancellation_token": self.cancellation_token,
            "context": self.context.to_json(),
        })
    }

    /// Child inherits selection/event/cancel handles; new run_id linked to parent.
    pub fn spawn_child(&self, child_run_id: &str) -> Self {
        Self {
            run_id: child_run_id.to_string(),
            parent_run_id: Some(self.run_id.clone()),
            turn_id: self.turn_id.clone(),
            workspace_scope: self.workspace_scope.clone(),
            selection_context: self.selection_context.clone(),
            usage_sink: self.usage_sink.clone(),
            event_sink: self.event_sink.clone(),
            cancellation_token: self.cancellation_token.clone(),
            context: self.context.clone(),
        }
    }

    /// Env pairs for *child* Command.env injection (never api_key / credential plaintext).
    /// Parent process must not `setenv` these to carry request state across tasks.
    pub fn selection_env(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        out.push(("MARQDO_RUN_ID".into(), self.run_id.clone()));
        if let Some(p) = &self.parent_run_id {
            out.push(("MARQDO_PARENT_RUN_ID".into(), p.clone()));
        }
        if let Some(t) = &self.turn_id {
            out.push(("MARQDO_TURN_ID".into(), t.clone()));
        }
        if let Some(w) = &self.workspace_scope {
            out.push(("MARQDO_WORKSPACE_SCOPE".into(), w.clone()));
        }
        if let Some(tok) = &self.cancellation_token {
            out.push(("MARQDO_CANCEL_TOKEN".into(), tok.clone()));
        }
        if let Some(sink) = &self.event_sink {
            out.push(("MARQDO_EVENT_FORWARD".into(), sink.clone()));
        }
        if let Some(usage) = &self.usage_sink {
            out.push(("MARQDO_USAGE_SINK".into(), usage.clone()));
        }
        if let Some(model) = self
            .selection_context
            .get("upstream_model_id")
            .or_else(|| self.selection_context.get("model"))
            .and_then(|v| v.as_str())
        {
            out.push(("MARQDO_LLM_MODEL".into(), model.to_string()));
            out.push(("OPENAI_MODEL".into(), model.to_string()));
        }
        if let Some(base) = self
            .selection_context
            .get("base_url")
            .and_then(|v| v.as_str())
        {
            out.push(("MARQDO_LLM_BASE_URL".into(), base.to_string()));
            out.push(("OPENAI_BASE_URL".into(), base.to_string()));
        }
        // credential_ref only — never expand api_key into child env here.
        if let Some(cred) = self
            .selection_context
            .get("credential_ref")
            .and_then(|v| v.as_str())
        {
            out.push(("MARQDO_CREDENTIAL_REF".into(), cred.to_string()));
        }
        if let Some(ws) = &self.context.workspace_id {
            out.push(("MARQDO_WORKSPACE_ID".into(), ws.clone()));
        }
        if let Some(snap) = &self.context.snapshot_id {
            out.push(("MARQDO_SNAPSHOT_ID".into(), snap.clone()));
        }
        if let Some(lease) = &self.context.lease_id {
            out.push(("MARQDO_LEASE_ID".into(), lease.clone()));
        }
        if let Some(attempt) = &self.context.attempt_id {
            out.push(("MARQDO_ATTEMPT_ID".into(), attempt.clone()));
        }
        out
    }

    /// Apply this run's handles onto a child Command without mutating parent env.
    pub fn apply_to_command(&self, cmd: &mut std::process::Command) {
        for (k, v) in self.selection_env() {
            cmd.env(k, v);
        }
    }
}

fn str_opt(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Unified cancel-token semantics (M-R1 / design §6):
/// - missing path / missing file → not cancelled
/// - empty body / `"0"` → not cancelled
/// - `"1"` / `"cancelled"` → cancelled
/// - read failure → not cancelled (caller may log); do not treat as cancel
///
/// Process-global `MARQDO_CANCEL` is **not** consulted when an explicit token
/// handle is provided, so parallel tasks cannot cancel each other via env.
pub fn cancel_requested(token: Option<&str>) -> bool {
    let Some(tok) = token.filter(|s| !s.is_empty()) else {
        return false;
    };
    let path = std::path::Path::new(tok);
    if !path.is_file() {
        return false;
    }
    match std::fs::read_to_string(path) {
        Ok(body) => {
            let t = body.trim();
            t == "1" || t.eq_ignore_ascii_case("cancelled")
        }
        Err(_) => false,
    }
}

/// Token-file body interpretation used by LLM plugin and control plane alike.
pub fn cancel_token_body_means_cancel(body: &str) -> bool {
    let t = body.trim();
    t == "1" || t.eq_ignore_ascii_case("cancelled")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn empty_and_zero_token_are_not_cancelled() {
        let dir = std::env::temp_dir().join(format!("mq-rc-empty-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        std::fs::File::create(&dir).unwrap();
        assert!(!cancel_requested(Some(dir.to_str().unwrap())));
        std::fs::write(&dir, "0").unwrap();
        assert!(!cancel_requested(Some(dir.to_str().unwrap())));
        std::fs::write(&dir, "cancelled").unwrap();
        assert!(cancel_requested(Some(dir.to_str().unwrap())));
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn explicit_token_ignores_global_marqdo_cancel() {
        let dir = std::env::temp_dir().join(format!("mq-rc-ign-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let mut f = std::fs::File::create(&dir).unwrap();
        f.write_all(b"").unwrap();
        drop(f);
        std::env::set_var("MARQDO_CANCEL", "1");
        assert!(
            !cancel_requested(Some(dir.to_str().unwrap())),
            "empty token must not cancel even if MARQDO_CANCEL=1"
        );
        std::env::remove_var("MARQDO_CANCEL");
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn context_round_trip() {
        let v = json!({
            "run_id": "r1",
            "cancellation_token": "/tmp/t",
            "context": {
                "workspace_id": "ws",
                "lease": {"id": "lease_1", "epoch": 3},
                "attempt_id": "a1"
            }
        });
        let rc = RunContext::from_value(&v).unwrap();
        assert_eq!(rc.context.lease_id.as_deref(), Some("lease_1"));
        assert_eq!(rc.context.epoch, Some(3));
        let child = rc.spawn_child("r2");
        assert_eq!(child.parent_run_id.as_deref(), Some("r1"));
        assert_eq!(child.context.lease_id.as_deref(), Some("lease_1"));
    }
}

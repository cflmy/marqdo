//! RunContextV1 — request-local run identity for selection / usage / cancel (M01).

use serde_json::{json, Map, Value};

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
}

impl RunContext {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let run_id = v
            .get("run_id")
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "run_context.missing:run_id".to_string())?
            .to_string();
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
        })
    }

    /// Child inherits selection/event/cancel; new run_id linked to parent.
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
        }
    }

    /// Env vars for subtask injection (never includes api_key).
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
        out
    }
}

/// Check whether a cancel token file/env indicates cancellation.
pub fn cancel_requested(token: Option<&str>) -> bool {
    let Some(tok) = token.filter(|s| !s.is_empty()) else {
        return false;
    };
    // Token may be a path: presence of file or contents "1"/"cancelled" means stop.
    let path = std::path::Path::new(tok);
    if path.is_file() {
        if let Ok(body) = std::fs::read_to_string(path) {
            let t = body.trim();
            return t.is_empty() || t == "1" || t.eq_ignore_ascii_case("cancelled");
        }
        return true;
    }
    std::env::var("MARQDO_CANCEL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

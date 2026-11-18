//! ModelSelectionV1 — request-level model identity (M01).

use serde_json::{json, Map, Value};

#[derive(Debug, Clone)]
pub struct ModelSelection {
    pub schema_version: u64,
    pub selection_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub provider_revision: Option<u64>,
    pub model_revision: Option<u64>,
    pub adapter: String,
    pub endpoint_ref: Option<String>,
    pub credential_ref: Option<String>,
    pub credential_version: Option<u64>,
    pub upstream_model_id: String,
    pub parameters: Map<String, Value>,
    pub fallback_policy: String,
    pub auth_mode: String,
    /// Resolved call-local endpoint (never serialize API keys into events).
    pub base_url: Option<String>,
    /// Call-local credential material — must not appear in public snapshots.
    pub api_key: Option<String>,
}

impl ModelSelection {
    pub fn public_snapshot(&self) -> Value {
        json!({
            "schema": "ModelSelectionV1",
            "schema_version": self.schema_version,
            "selection_id": self.selection_id,
            "provider_id": self.provider_id,
            "model_id": self.model_id,
            "provider_revision": self.provider_revision,
            "model_revision": self.model_revision,
            "adapter": self.adapter,
            "endpoint_ref": self.endpoint_ref,
            "credential_ref": self.credential_ref,
            "credential_version": self.credential_version,
            "upstream_model_id": self.upstream_model_id,
            "parameters": self.parameters,
            "fallback_policy": self.fallback_policy,
            "auth_mode": self.auth_mode,
        })
    }
}

/// Parse selection. In `strict` mode, missing required fields error (no env fallback).
pub fn parse_model_selection(v: &Value, strict: bool) -> Result<ModelSelection, String> {
    let schema_version = v
        .get("schema_version")
        .and_then(|x| x.as_u64())
        .unwrap_or(1);
    if schema_version != 1 {
        return Err(format!("unsupported ModelSelection schema_version={schema_version}"));
    }
    let require = |key: &str| -> Result<String, String> {
        match v.get(key).and_then(|x| x.as_str()).filter(|s| !s.is_empty()) {
            Some(s) => Ok(s.to_string()),
            None if strict => Err(format!("llm.selection_missing:{key}")),
            None => Ok(String::new()),
        }
    };
    let selection_id = require("selection_id")?;
    let provider_id = require("provider_id")?;
    let model_id = require("model_id")?;
    let adapter = v
        .get("adapter")
        .and_then(|x| x.as_str())
        .unwrap_or("openai-compatible")
        .to_string();
    let upstream = v
        .get("upstream_model_id")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            if model_id.is_empty() {
                None
            } else {
                Some(model_id.clone())
            }
        });
    let upstream_model_id = match upstream {
        Some(u) => u,
        None if strict => return Err("llm.selection_missing:upstream_model_id".into()),
        None => String::new(),
    };
    let auth_mode = v
        .get("auth_mode")
        .and_then(|x| x.as_str())
        .unwrap_or("api_key")
        .to_string();
    let fallback_policy = v
        .get("fallback_policy")
        .and_then(|x| x.as_str())
        .unwrap_or("disabled")
        .to_string();
    let parameters = v
        .get("parameters")
        .and_then(|x| x.as_object())
        .cloned()
        .unwrap_or_default();
    let base_url = v
        .get("base_url")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let api_key = v
        .get("api_key")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    if strict && auth_mode != "none" && api_key.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        // credential_ref may be resolved by host; allow missing api_key if credential_ref present
        if v.get("credential_ref").and_then(|x| x.as_str()).is_none() && base_url.is_some() {
            return Err("llm.selection_missing:api_key_or_credential_ref".into());
        }
    }
    if strict && selection_id.is_empty() {
        return Err("llm.selection_missing:selection_id".into());
    }
    Ok(ModelSelection {
        schema_version,
        selection_id,
        provider_id,
        model_id,
        provider_revision: v.get("provider_revision").and_then(|x| x.as_u64()),
        model_revision: v.get("model_revision").and_then(|x| x.as_u64()),
        adapter,
        endpoint_ref: v
            .get("endpoint_ref")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        credential_ref: v
            .get("credential_ref")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        credential_version: v.get("credential_version").and_then(|x| x.as_u64()),
        upstream_model_id,
        parameters,
        fallback_policy,
        auth_mode,
        base_url,
        api_key,
    })
}

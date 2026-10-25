//! ModelCallUsageV1 — null ≠ 0; measurement_source honesty (M02 / F04).

use serde_json::{json, Value};

/// Prefer null over zero when the provider omitted a field.
pub fn token_or_null(v: Option<i64>) -> Value {
    match v {
        Some(n) if n >= 0 => json!(n),
        _ => Value::Null,
    }
}

/// Build ModelCallUsageV1 from an OpenAI-shaped `usage` object (or Null).
pub fn model_call_usage_from_provider(
    call_id: &str,
    attempt_id: &str,
    run_id: &str,
    selection: &Value,
    provider_usage: &Value,
    status: &str,
) -> Value {
    model_call_usage_from_provider_rev(call_id, attempt_id, run_id, selection, provider_usage, status, 1)
}

/// Same as [`model_call_usage_from_provider`] with an explicit revision (F05).
pub fn model_call_usage_from_provider_rev(
    call_id: &str,
    attempt_id: &str,
    run_id: &str,
    selection: &Value,
    provider_usage: &Value,
    status: &str,
    revision: u64,
) -> Value {
    let has_usage = provider_usage.is_object();
    let input = provider_usage
        .get("prompt_tokens")
        .or_else(|| provider_usage.get("input_tokens"))
        .and_then(|v| v.as_i64());
    let output = provider_usage
        .get("completion_tokens")
        .or_else(|| provider_usage.get("output_tokens"))
        .and_then(|v| v.as_i64());
    let total = provider_usage
        .get("total_tokens")
        .and_then(|v| v.as_i64())
        .or_else(|| match (input, output) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        });
    let cached = provider_usage
        .get("cached_input_tokens")
        .or_else(|| {
            provider_usage
                .get("prompt_tokens_details")
                .and_then(|d| d.get("cached_tokens"))
        })
        .and_then(|v| v.as_i64());
    let reasoning = provider_usage
        .get("reasoning_output_tokens")
        .or_else(|| {
            provider_usage
                .get("completion_tokens_details")
                .and_then(|d| d.get("reasoning_tokens"))
        })
        .and_then(|v| v.as_i64());

    let (usage_state, measurement_source) = if !has_usage {
        ("unknown", "none")
    } else if input.is_none() && output.is_none() && total.is_none() {
        ("unknown", "none")
    } else {
        ("final", "provider")
    };

    let mut diagnostics = Vec::new();
    if let (Some(t), Some(i), Some(o)) = (total, input, output) {
        if t != i + o && cached.is_none() {
            diagnostics.push(json!({
                "code": "usage.inconsistent",
                "message": format!("total_tokens={t} != input+output={}", i + o)
            }));
        }
    }

    json!({
        "schema": "ModelCallUsageV1",
        "schema_version": 1,
        "call_id": call_id,
        "attempt_id": attempt_id,
        "parent_call_id": Value::Null,
        "run_id": run_id,
        "turn_id": selection.get("turn_id").cloned().unwrap_or(Value::Null),
        "workspace_scope": selection.get("workspace_scope").cloned().unwrap_or(Value::Null),
        "purpose": selection.get("purpose").cloned().unwrap_or(json!("answer")),
        "selection_id": selection.get("selection_id").cloned().unwrap_or(Value::Null),
        "provider_id": selection.get("provider_id").cloned().unwrap_or(Value::Null),
        "model_id": selection.get("model_id").cloned().unwrap_or(Value::Null),
        "requested_model": selection.get("upstream_model_id").cloned()
            .or_else(|| selection.get("model").cloned())
            .unwrap_or(Value::Null),
        "resolved_model": selection.get("resolved_model").cloned()
            .or_else(|| selection.get("upstream_model_id").cloned())
            .unwrap_or(Value::Null),
        "status": status,
        "usage_state": usage_state,
        "measurement_source": measurement_source,
        "input_tokens": token_or_null(input),
        "output_tokens": token_or_null(output),
        "total_tokens": token_or_null(total),
        "cached_input_tokens": token_or_null(cached),
        "reasoning_output_tokens": token_or_null(reasoning),
        "revision": revision.max(1),
        "provider_request_id": provider_usage.get("provider_request_id").cloned().unwrap_or(Value::Null),
        "diagnostics": diagnostics,
        // Legacy fields kept but marked — apps must not treat token_cost as money.
        "legacy": {
            "tokens": token_or_null(total),
            "token_cost": token_or_null(total),
            "deprecated": true,
            "unit": "tokens"
        }
    })
}

/// Replace prior usage for the same (call_id, attempt_id) when revision is newer (F05).
/// Returns the winning snapshot; does not sum cumulative stream counters.
pub fn replace_usage_if_newer(prior: &Value, next: &Value) -> Value {
    let same_call = prior.get("call_id") == next.get("call_id")
        && prior.get("attempt_id") == next.get("attempt_id");
    if !same_call {
        return next.clone();
    }
    let pr = prior.get("revision").and_then(|v| v.as_u64()).unwrap_or(0);
    let nr = next.get("revision").and_then(|v| v.as_u64()).unwrap_or(0);
    if nr >= pr {
        next.clone()
    } else {
        prior.clone()
    }
}

/// RuntimeEventV1 envelope helper.
pub fn runtime_event(
    event_id: &str,
    seq: u64,
    run_id: &str,
    event_type: &str,
    payload: Value,
) -> Value {
    json!({
        "schema": "RuntimeEventV1",
        "schema_version": 1,
        "event_id": event_id,
        "seq": seq,
        "run_id": run_id,
        "parent_run_id": Value::Null,
        "turn_id": Value::Null,
        "workspace_scope": Value::Null,
        "timestamp": chrono_like_now(),
        "type": event_type,
        "payload": payload
    })
}

fn chrono_like_now() -> String {
    // Avoid new chrono dep: RFC3339-ish from system time.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

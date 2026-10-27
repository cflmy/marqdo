//! AgentErrorV1 — classified errors and bounded recovery decisions (M-A1).

use serde_json::{json, Value};

/// Build an AgentErrorV1 document from caller fields (compat: missing → defaults).
pub fn make_error(args: &Value) -> Result<Value, String> {
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "missing text `code`".to_string())?;
    let category = args
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| infer_category(code));
    let stage = args
        .get("stage")
        .and_then(|v| v.as_str())
        .unwrap_or("agent");
    let message = args
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or(code);
    let retryable = args
        .get("retryable")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| default_retryable(category));
    let effect_state = args
        .get("effect_state")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let recovery_action = args
        .get("recovery_action")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| default_recovery(category, effect_state));

    let mut out = json!({
        "schema": "AgentErrorV1",
        "schema_version": 1,
        "code": code,
        "category": category,
        "stage": stage,
        "message": message,
        "retryable": retryable,
        "effect_state": effect_state,
        "recovery_action": recovery_action,
    });
    if let Some(obj) = out.as_object_mut() {
        if let Some(v) = args.get("retry_after_ms") {
            obj.insert("retry_after_ms".into(), v.clone());
        } else {
            obj.insert("retry_after_ms".into(), Value::Null);
        }
        for key in ["call_id", "details_ref", "fingerprint"] {
            if let Some(v) = args.get(key) {
                if !v.is_null() {
                    obj.insert(key.into(), v.clone());
                }
            }
        }
    }
    Ok(out)
}

/// Decide whether a step may auto-retry (read-only / no-effect only in M-A1).
///
/// Returns `{ok, action, reason, attempts_used, max_attempts}`.
pub fn decide_retry(args: &Value) -> Result<Value, String> {
    let error = args
        .get("error")
        .cloned()
        .ok_or_else(|| "missing map `error`".to_string())?;
    let attempts = args
        .get("attempts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let max_transport = args
        .get("max_transport_attempts")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as u32; // 1 initial + 2 retries
    let max_arg_repair = args
        .get("max_arg_repairs")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;
    let effect_class = args
        .get("effect_class")
        .and_then(|v| v.as_str())
        .unwrap_or("read");
    let cancelled = args
        .get("cancelled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let fingerprint_repeats = args
        .get("fingerprint_repeats")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    if cancelled {
        return Ok(decision("stop", "cancelled", attempts, max_transport));
    }

    let category = error
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let retryable = error
        .get("retryable")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let effect_state = error
        .get("effect_state")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let recovery = error
        .get("recovery_action")
        .and_then(|v| v.as_str())
        .unwrap_or("none");

    // Write / unknown effect: never auto-retry in M-A1 (ledger required).
    if effect_class != "read" || matches!(effect_state, "unknown" | "started" | "committed") {
        return Ok(decision(
            "manual_review",
            "write_or_unknown_effect_requires_ledger",
            attempts,
            max_transport,
        ));
    }

    if fingerprint_repeats >= 2 {
        return Ok(decision(
            "stop",
            "error_fingerprint_loop",
            attempts,
            max_transport,
        ));
    }

    match category {
        "transient" if retryable && attempts + 1 < max_transport => Ok(decision(
            "retry_step",
            "transient_read_safe",
            attempts,
            max_transport,
        )),
        "invalid_arguments" if recovery == "repair_args" && attempts < max_arg_repair => Ok(
            decision("repair_args", "one_arg_repair", attempts, max_arg_repair),
        ),
        "auth" | "forbidden" | "policy_denied" => Ok(decision(
            "needs_input",
            category,
            attempts,
            max_transport,
        )),
        "budget_exhausted" => Ok(decision("partial", category, attempts, max_transport)),
        "cancelled" => Ok(decision("stop", "cancelled", attempts, max_transport)),
        "unknown_effect" | "worker_crash" => Ok(decision(
            "manual_review",
            category,
            attempts,
            max_transport,
        )),
        _ => Ok(decision("stop", "not_auto_retryable", attempts, max_transport)),
    }
}

fn decision(action: &str, reason: &str, attempts: u32, max_attempts: u32) -> Value {
    json!({
        "ok": true,
        "schema": "AgentRetryDecisionV1",
        "schema_version": 1,
        "action": action,
        "reason": reason,
        "attempts_used": attempts,
        "max_attempts": max_attempts,
        "auto": matches!(action, "retry_step" | "repair_args"),
    })
}

fn infer_category(code: &str) -> &'static str {
    let c = code.to_ascii_lowercase();
    if c.contains("timeout") || c.contains("429") || c.contains("5xx") || c.contains("connect") {
        "transient"
    } else if c.contains("schema") || c.contains("argument") {
        "invalid_arguments"
    } else if c.contains("auth") {
        "auth"
    } else if c.contains("forbid") {
        "forbidden"
    } else if c.contains("policy") {
        "policy_denied"
    } else if c.contains("budget") {
        "budget_exhausted"
    } else if c.contains("cancel") {
        "cancelled"
    } else if c.contains("conflict") || c.contains("stale") {
        "conflict"
    } else if c.contains("effect") || c.contains("crash") {
        "unknown_effect"
    } else {
        "unknown"
    }
}

fn default_retryable(category: &str) -> bool {
    matches!(category, "transient")
}

fn default_recovery(category: &str, effect_state: &str) -> &'static str {
    if effect_state == "unknown" {
        return "reconcile";
    }
    match category {
        "transient" => "retry_step",
        "invalid_arguments" => "repair_args",
        "auth" | "forbidden" | "policy_denied" | "needs_input" => "needs_input",
        "budget_exhausted" => "return_partial",
        "cancelled" => "stop",
        "unknown_effect" | "worker_crash" => "manual_review",
        _ => "stop",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_error_defaults() {
        let err = make_error(&json!({"code": "tool.timeout", "message": "t"})).unwrap();
        assert_eq!(err["schema"], "AgentErrorV1");
        assert_eq!(err["category"], "transient");
        assert_eq!(err["retryable"], true);
        assert_eq!(err["recovery_action"], "retry_step");
    }

    #[test]
    fn read_transient_retries_once() {
        let err = make_error(&json!({"code": "tool.timeout"})).unwrap();
        let d = decide_retry(&json!({
            "error": err,
            "attempts": 0,
            "effect_class": "read"
        }))
        .unwrap();
        assert_eq!(d["action"], "retry_step");
        assert_eq!(d["auto"], true);
    }

    #[test]
    fn write_never_auto_retries() {
        let err = make_error(&json!({"code": "tool.timeout"})).unwrap();
        let d = decide_retry(&json!({
            "error": err,
            "attempts": 0,
            "effect_class": "write"
        }))
        .unwrap();
        assert_eq!(d["action"], "manual_review");
        assert_eq!(d["auto"], false);
    }

    #[test]
    fn fingerprint_loop_stops() {
        let err = make_error(&json!({"code": "tool.timeout"})).unwrap();
        let d = decide_retry(&json!({
            "error": err,
            "attempts": 0,
            "effect_class": "read",
            "fingerprint_repeats": 2
        }))
        .unwrap();
        assert_eq!(d["action"], "stop");
    }
}

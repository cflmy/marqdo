//! Deterministic fan-out results: never discard a successful sibling and never
//! infer success from a completed wait protocol alone.

use serde_json::{json, Value};

pub fn report(args: &Value) -> Result<Value, String> {
    let results = args
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| "results must be ordered subtask result list".to_string())?;
    if results.len() > 10_000 {
        return Err("parallel result count exceeds scheduler limit".into());
    }
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut unknown = 0usize;
    let mut entries = Vec::with_capacity(results.len());
    for (index, child) in results.iter().enumerate() {
        let transport = child.get("ok").and_then(Value::as_bool);
        let body = child.get("value").unwrap_or(&Value::Null);
        let status = match (transport, body.get("code").and_then(Value::as_i64)) {
            (Some(true), Some(0)) => { passed += 1; "succeeded" }
            (Some(true), Some(_)) | (Some(false), _) => { failed += 1; "failed" }
            _ => { unknown += 1; "uncertain" }
        };
        // Raw child result is kept as data; the model sees only an optional
        // bounded summary. The host may store detailed output as an artifact.
        let detail = child.get("error")
            .and_then(Value::as_str)
            .or_else(|| body.get("stderr").and_then(Value::as_str))
            .unwrap_or("");
        let brief: String = detail.chars().take(280).collect();
        entries.push(json!({
            "index": index,
            "status": status,
            "summary": brief,
            "result": child
        }));
    }
    let status = if unknown != 0 {
        "uncertain"
    } else if failed == 0 {
        "succeeded"
    } else if passed > 0 {
        "partial"
    } else {
        "failed"
    };
    Ok(json!({
        "ok": failed == 0 && unknown == 0,
        "schema": "AgentParallelResultV1",
        "status": status,
        "succeeded": passed,
        "failed": failed,
        "uncertain": unknown,
        "items": entries
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_success_and_exposes_failed_child() {
        let result = report(&json!({"results":[
            {"ok":true,"value":{"code":0,"value":"good"}},
            {"ok":true,"value":{"code":2,"stderr":"failed"}},
            {"ok":false,"error":"timeout"}
        ]})).unwrap();
        assert_eq!(result["status"], "partial");
        assert_eq!(result["succeeded"], 1);
        assert_eq!(result["failed"], 2);
        assert_eq!(result["items"][0]["status"], "succeeded");
    }
    #[test]
    fn unknown_child_cannot_be_mislabeled_successful() {
        let value = report(&json!({"results":[{"ok":true,"value":{"value":"no code"}}]})).unwrap();
        assert_eq!(value["status"], "uncertain");
        assert_eq!(value["ok"], false);
    }
}

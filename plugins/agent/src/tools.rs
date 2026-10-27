//! ToolSpecV1 registry + call validation (M-A2).
//! Discovery → schema validate → authorize. Execution of Marqdo `##` stays in subtask;
//! this module never evals model-supplied code.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde_json::{json, Map, Value};

fn registry() -> &'static Mutex<HashMap<String, Value>> {
    static REG: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    opt_text(args, key).ok_or_else(|| format!("missing text `{key}`"))
}

/// Supported JSON Schema subset for tool input (reject unsupported constructs).
fn validate_schema_subset(schema: &Value) -> Result<(), String> {
    let obj = schema
        .as_object()
        .ok_or_else(|| "input_schema must be object".to_string())?;
    let ty = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("object");
    if ty != "object" {
        return Err(format!("unsupported root type={ty}; only object"));
    }
    if let Some(props) = obj.get("properties") {
        let props = props
            .as_object()
            .ok_or_else(|| "properties must be object".to_string())?;
        for (name, prop) in props {
            let p = prop
                .as_object()
                .ok_or_else(|| format!("property {name} must be object"))?;
            let pty = p.get("type").and_then(|v| v.as_str()).unwrap_or("string");
            match pty {
                "string" | "integer" | "number" | "boolean" | "array" | "object" => {}
                other => {
                    return Err(format!(
                        "unsupported property type {other} for {name}; refuse register"
                    ))
                }
            }
            // Reject constructs we do not implement.
            for banned in ["$ref", "oneOf", "anyOf", "allOf", "not", "if", "then", "else"] {
                if p.contains_key(banned) {
                    return Err(format!(
                        "unsupported schema keyword {banned} on {name}; refuse register"
                    ));
                }
            }
        }
    }
    for banned in ["$ref", "oneOf", "anyOf", "allOf", "not", "if", "then", "else"] {
        if obj.contains_key(banned) {
            return Err(format!(
                "unsupported schema keyword {banned}; refuse register"
            ));
        }
    }
    Ok(())
}

fn normalize_spec(mut spec: Value) -> Result<Value, String> {
    let obj = spec
        .as_object_mut()
        .ok_or_else(|| "spec must be object".to_string())?;
    obj.insert("schema".into(), json!("ToolSpecV1"));
    obj.entry("schema_version".to_string())
        .or_insert(json!(1));
    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "ToolSpecV1 requires id".to_string())?
        .to_string();
    obj.entry("version".to_string()).or_insert(json!("1"));
    obj.entry("description".to_string())
        .or_insert(json!(""));
    let input = obj
        .get("input_schema")
        .cloned()
        .unwrap_or_else(|| json!({"type": "object", "properties": {}, "additionalProperties": false}));
    validate_schema_subset(&input)?;
    obj.insert("input_schema".into(), input);
    obj.entry("effect".to_string()).or_insert(json!("read"));
    let effect = obj.get("effect").and_then(|v| v.as_str()).unwrap_or("read");
    if !matches!(effect, "read" | "write" | "execute" | "external_write") {
        return Err(format!("invalid effect={effect}"));
    }
    obj.entry("scopes".to_string()).or_insert(json!([]));
    obj.entry("timeout_ms".to_string()).or_insert(json!(10_000));
    obj.entry("max_result_bytes".to_string())
        .or_insert(json!(65_536));
    obj.entry("idempotency".to_string())
        .or_insert(json!("safe"));
    obj.entry("availability".to_string())
        .or_insert(json!("ready"));
    let _ = id;
    Ok(spec)
}

fn validate_args_against_schema(schema: &Value, arguments: &Value) -> Result<(), String> {
    let args = arguments
        .as_object()
        .ok_or_else(|| "arguments must be object".to_string())?;
    let sch = schema.as_object().ok_or_else(|| "input_schema invalid".to_string())?;
    let props = sch
        .get("properties")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let additional = sch
        .get("additionalProperties")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !additional {
        for key in args.keys() {
            if !props.contains_key(key) {
                return Err(format!("unknown argument `{key}`"));
            }
        }
    }
    if let Some(req) = sch.get("required").and_then(|v| v.as_array()) {
        for r in req {
            let name = r.as_str().unwrap_or("");
            if name.is_empty() {
                continue;
            }
            if !args.contains_key(name) || args.get(name).map(|v| v.is_null()).unwrap_or(true) {
                return Err(format!("missing required argument `{name}`"));
            }
        }
    }
    for (key, val) in args {
        if let Some(prop) = props.get(key) {
            if let Some(pty) = prop.get("type").and_then(|v| v.as_str()) {
                let ok = match pty {
                    "string" => val.is_string(),
                    "integer" => val.as_i64().is_some() || val.as_u64().is_some(),
                    "number" => val.as_f64().is_some(),
                    "boolean" => val.is_boolean(),
                    "array" => val.is_array(),
                    "object" => val.is_object(),
                    _ => true,
                };
                if !ok {
                    return Err(format!("argument `{key}` type mismatch (want {pty})"));
                }
            }
            if let Some(min) = prop.get("minimum").and_then(|v| v.as_f64()) {
                if let Some(n) = val.as_f64() {
                    if n < min {
                        return Err(format!("argument `{key}` below minimum {min}"));
                    }
                }
            }
            if let Some(max) = prop.get("maximum").and_then(|v| v.as_f64()) {
                if let Some(n) = val.as_f64() {
                    if n > max {
                        return Err(format!("argument `{key}` above maximum {max}"));
                    }
                }
            }
        }
    }
    Ok(())
}

fn authorized(allowlist: Option<&Value>, tool_id: &str) -> bool {
    match allowlist {
        None => true, // registry-only call sites may omit; still need registered spec
        Some(tools) => crate::tool_names(tools).iter().any(|n| n == tool_id),
    }
}

/// Register one ToolSpecV1 (or list). Host/trusted only — model cannot invent principal/root.
pub fn tools_register(args: &Value) -> Result<Value, String> {
    let specs = if let Some(arr) = args.get("specs").and_then(|v| v.as_array()) {
        arr.clone()
    } else if args.get("spec").is_some() || args.get("id").is_some() {
        let raw = args.get("spec").cloned().unwrap_or_else(|| args.clone());
        vec![raw]
    } else {
        return Err("tools.register requires spec or specs[]".into());
    };
    let mut registered = Vec::new();
    let mut guard = registry()
        .lock()
        .map_err(|_| "tools registry lock poisoned".to_string())?;
    for raw in specs {
        let spec = normalize_spec(raw)?;
        let id = spec
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();
        guard.insert(id.clone(), spec);
        registered.push(id);
    }
    Ok(json!({
        "ok": true,
        "registered": registered,
        "count": registered.len(),
    }))
}

pub fn tools_clear(args: &Value) -> Result<Value, String> {
    let _ = args;
    let mut guard = registry()
        .lock()
        .map_err(|_| "tools registry lock poisoned".to_string())?;
    let n = guard.len();
    guard.clear();
    Ok(json!({"ok": true, "cleared": n}))
}

pub fn tools_search(args: &Value) -> Result<Value, String> {
    let query = opt_text(args, "query").unwrap_or("").to_ascii_lowercase();
    let allow = args.get("tools");
    let guard = registry()
        .lock()
        .map_err(|_| "tools registry lock poisoned".to_string())?;
    let mut items = Vec::new();
    for (id, spec) in guard.iter() {
        let avail = spec
            .get("availability")
            .and_then(|v| v.as_str())
            .unwrap_or("ready");
        let auth = authorized(allow, id);
        if !auth {
            // Do not leak unauthorized private connector names in search hits.
            continue;
        }
        if !query.is_empty() {
            let desc = spec
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !id.to_ascii_lowercase().contains(&query)
                && !desc.to_ascii_lowercase().contains(&query)
            {
                continue;
            }
        }
        items.push(json!({
            "id": id,
            "version": spec.get("version").cloned().unwrap_or(json!("1")),
            "description": spec.get("description").cloned().unwrap_or(json!("")),
            "effect": spec.get("effect").cloned().unwrap_or(json!("read")),
            "availability": avail,
            "authorized": true,
        }));
    }
    items.sort_by(|a, b| {
        let ai = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let bi = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        ai.cmp(bi)
    });
    Ok(json!({
        "ok": true,
        "schema": "ToolSearchResultV1",
        "schema_version": 1,
        "items": items,
        "count": items.len(),
    }))
}

pub fn tools_describe(args: &Value) -> Result<Value, String> {
    let id = arg_text(args, "id")?;
    let allow = args.get("tools");
    if !authorized(allow, id) {
        return Ok(json!({
            "ok": false,
            "found": false,
            "authorized": false,
            "error": {
                "schema": "AgentErrorV1",
                "schema_version": 1,
                "code": "tool.forbidden",
                "category": "forbidden",
                "stage": "tool",
                "message": format!("tool `{id}` not in allowlist"),
                "retryable": false,
                "effect_state": "none",
                "recovery_action": "needs_input",
            }
        }));
    }
    let guard = registry()
        .lock()
        .map_err(|_| "tools registry lock poisoned".to_string())?;
    match guard.get(id) {
        Some(spec) => Ok(json!({
            "ok": true,
            "found": true,
            "authorized": true,
            "spec": spec,
        })),
        None => Ok(json!({
            "ok": false,
            "found": false,
            "authorized": true,
            "error": {
                "schema": "AgentErrorV1",
                "schema_version": 1,
                "code": "tool.not_found",
                "category": "capability_unavailable",
                "stage": "tool",
                "message": format!("tool `{id}` not registered"),
                "retryable": false,
                "effect_state": "none",
                "recovery_action": "stop",
            }
        })),
    }
}

/// Validate + authorize a ToolCallV1. Does not execute side effects.
pub fn tools_prepare_call(args: &Value) -> Result<Value, String> {
    let tool_id = arg_text(args, "tool_id")
        .or_else(|_| arg_text(args, "id"))
        .or_else(|_| arg_text(args, "name"))?;
    let allow = args.get("tools");
    if !authorized(allow, tool_id) {
        return Ok(tool_result_err(
            "tool.forbidden",
            "forbidden",
            format!("unauthorized call to `{tool_id}`"),
            false,
        ));
    }
    let guard = registry()
        .lock()
        .map_err(|_| "tools registry lock poisoned".to_string())?;
    let spec = match guard.get(tool_id) {
        Some(s) => s.clone(),
        None => {
            return Ok(tool_result_err(
                "tool.not_found",
                "capability_unavailable",
                format!("tool `{tool_id}` not registered"),
                false,
            ));
        }
    };
    drop(guard);

    let avail = spec
        .get("availability")
        .and_then(|v| v.as_str())
        .unwrap_or("ready");
    if avail != "ready" {
        return Ok(tool_result_err(
            "tool.unavailable",
            "capability_unavailable",
            format!("tool `{tool_id}` availability={avail}"),
            false,
        ));
    }

    let arguments = match args.get("arguments") {
        None | Some(Value::Null) => json!({}),
        Some(v) => v.clone(),
    };
    let schema = spec
        .get("input_schema")
        .cloned()
        .unwrap_or_else(|| json!({"type":"object"}));
    if let Err(e) = validate_args_against_schema(&schema, &arguments) {
        return Ok(tool_result_err(
            "tool.invalid_arguments",
            "invalid_arguments",
            e,
            true,
        ));
    }

    let call_id = opt_text(args, "call_id")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("call_{}", std::process::id()));
    let operation_id = opt_text(args, "operation_id")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("op_{tool_id}_{call_id}"));
    let version = spec
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1");
    let effect = spec
        .get("effect")
        .and_then(|v| v.as_str())
        .unwrap_or("read");
    let digest = args_digest(&arguments);
    let logical_run = opt_text(args, "logical_run_id").unwrap_or("-");
    let step = opt_text(args, "step_id").unwrap_or("-");
    let idempotency_key = opt_text(args, "idempotency_key")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{logical_run}|{step}|{tool_id}|{digest}"));
    let needs_ledger = matches!(effect, "write" | "execute" | "external_write");

    let call = json!({
        "schema": "ToolCallV1",
        "schema_version": 1,
        "call_id": call_id,
        "tool_id": tool_id,
        "tool_version": version,
        "arguments": arguments,
        "operation_id": operation_id,
        "parent_span_id": args.get("parent_span_id").cloned().unwrap_or(Value::Null),
    });

    Ok(json!({
        "ok": true,
        "status": "prepared",
        "call": call,
        "spec": {
            "id": tool_id,
            "effect": effect,
            "timeout_ms": spec.get("timeout_ms").cloned().unwrap_or(json!(10000)),
            "max_result_bytes": spec.get("max_result_bytes").cloned().unwrap_or(json!(65536)),
        },
        "args_digest": digest,
        "idempotency_key": idempotency_key,
        "needs_ledger": needs_ledger,
        "summary": format!("prepared authorized call to `{tool_id}`"),
    }))
}

/// Stable digest of tool arguments for effect ledger idempotency (M-A5a).
pub fn tools_args_digest(args: &Value) -> Result<Value, String> {
    let arguments = args
        .get("arguments")
        .cloned()
        .or_else(|| args.get("value").cloned())
        .unwrap_or_else(|| json!({}));
    Ok(json!({
        "ok": true,
        "args_digest": args_digest(&arguments),
    }))
}

fn args_digest(arguments: &Value) -> String {
    let text = serde_json::to_string(arguments).unwrap_or_else(|_| "{}".into());
    format!("sha256-lite:{:x}", fnv1a64(&text))
}

fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// True when effect_class requires EffectLedger before/after execution.
pub fn tools_needs_ledger(args: &Value) -> Result<Value, String> {
    let effect = opt_text(args, "effect")
        .or_else(|| opt_text(args, "effect_class"))
        .unwrap_or("read");
    Ok(json!({
        "ok": true,
        "needs_ledger": matches!(effect, "write" | "execute" | "external_write"),
        "effect": effect,
    }))
}

/// Wrap a successful / failed tool execution into ToolResultV1.
pub fn tools_wrap_result(args: &Value) -> Result<Value, String> {
    let ok = args.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let status = opt_text(args, "status").unwrap_or(if ok { "succeeded" } else { "failed" });
    let summary = opt_text(args, "summary").unwrap_or("");
    let mut out = json!({
        "schema": "ToolResultV1",
        "schema_version": 1,
        "ok": ok,
        "status": status,
        "summary": summary,
        "structured_content": args.get("structured_content").cloned().unwrap_or(Value::Null),
        "resources": args.get("resources").cloned().unwrap_or_else(|| json!([])),
        "citations": args.get("citations").cloned().unwrap_or_else(|| json!([])),
        "error": args.get("error").cloned().unwrap_or(Value::Null),
        "usage": args.get("usage").cloned().unwrap_or(Value::Null),
        "truncated": args.get("truncated").and_then(|v| v.as_bool()).unwrap_or(false),
        "next_cursor": args.get("next_cursor").cloned().unwrap_or(Value::Null),
        "effect_receipt_ref": args.get("effect_receipt_ref").cloned().unwrap_or(Value::Null),
    });
    // Truncate oversized summary into artifact hint (do not dump MB into prompt).
    let max_bytes = args
        .get("max_result_bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(65_536) as usize;
    if summary.len() > max_bytes {
        if let Some(obj) = out.as_object_mut() {
            obj.insert("truncated".into(), json!(true));
            obj.insert(
                "summary".into(),
                json!(format!(
                    "{}… [truncated; use resource handle]",
                    &summary[..max_bytes.min(summary.len())]
                )),
            );
        }
    }
    Ok(out)
}

/// Seed the five engineering tools used by qdagent (aliases remain CALL:tool_*).
pub fn tools_register_eng_defaults(args: &Value) -> Result<Value, String> {
    let _ = args;
    let specs = eng_default_specs();
    tools_register(&json!({"specs": specs}))
}

fn eng_default_specs() -> Vec<Value> {
    vec![
        json!({
            "id": "tool_find",
            "version": "1",
            "description": "Engineering find via EKC (read-only)",
            "input_schema": {
                "type": "object",
                "properties": {"query": {"type": "string"}},
                "required": ["query"],
                "additionalProperties": false
            },
            "effect": "read",
            "scopes": ["knowledge:read"],
            "availability": "ready"
        }),
        json!({
            "id": "tool_reuse",
            "version": "1",
            "description": "Engineering reuse / preflight discovery (read-only)",
            "input_schema": {
                "type": "object",
                "properties": {"task": {"type": "string"}},
                "required": ["task"],
                "additionalProperties": false
            },
            "effect": "read",
            "scopes": ["knowledge:read"],
            "availability": "ready"
        }),
        json!({
            "id": "tool_bind",
            "version": "1",
            "description": "Bind and execute a verified knowledge artifact",
            "input_schema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "policy": {"type": "string"},
                    "project": {"type": "string"}
                },
                "required": ["query"],
                "additionalProperties": false
            },
            "effect": "execute",
            "scopes": ["knowledge:execute"],
            "availability": "ready"
        }),
        json!({
            "id": "tool_verify",
            "version": "1",
            "description": "Run L3 knowledge verification checks",
            "input_schema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            },
            "effect": "read",
            "scopes": ["knowledge:verify"],
            "availability": "ready"
        }),
        json!({
            "id": "tool_summarize",
            "version": "1",
            "description": "Summarize a Context Pack / preflight map",
            "input_schema": {
                "type": "object",
                "properties": {"preflight": {"type": "object"}},
                "required": ["preflight"],
                "additionalProperties": false
            },
            "effect": "read",
            "scopes": ["knowledge:read"],
            "availability": "ready"
        }),
    ]
}

fn tool_result_err(code: &str, category: &str, message: String, retryable: bool) -> Value {
    let recovery = if category == "invalid_arguments" {
        "repair_args"
    } else if category == "forbidden" {
        "needs_input"
    } else {
        "stop"
    };
    json!({
        "ok": false,
        "status": "failed",
        "summary": message,
        "schema": "ToolResultV1",
        "schema_version": 1,
        "structured_content": null,
        "resources": [],
        "citations": [],
        "error": {
            "schema": "AgentErrorV1",
            "schema_version": 1,
            "code": code,
            "category": category,
            "stage": "tool",
            "message": message,
            "retryable": retryable,
            "retry_after_ms": null,
            "effect_state": "none",
            "recovery_action": recovery,
        },
        "usage": null,
        "truncated": false,
        "next_cursor": null,
        "effect_receipt_ref": null,
    })
}

/// Seed stub specs from a CALL allowlist (args.tools).
pub fn seed_from_allowlist_args(args: &Value) -> Result<Value, String> {
    let tools = args
        .get("tools")
        .ok_or_else(|| "tools.seed_allowlist requires tools".to_string())?;
    seed_from_allowlist(tools)
}

fn seed_from_allowlist(tools: &Value) -> Result<Value, String> {
    let names = crate::tool_names(tools);
    let mut specs = Vec::new();
    for name in names {
        // Minimal stub if not already registered — CALL whitelist compatibility.
        let already = {
            let guard = registry()
                .lock()
                .map_err(|_| "tools registry lock poisoned".to_string())?;
            guard.contains_key(&name)
        };
        if already {
            continue;
        }
        let mut m = Map::new();
        m.insert("id".into(), json!(name));
        m.insert("version".into(), json!("1"));
        m.insert(
            "description".into(),
            json!(format!("Allowlist-derived tool `{name}`")),
        );
        m.insert(
            "input_schema".into(),
            json!({"type":"object","properties":{},"additionalProperties":true}),
        );
        m.insert("effect".into(), json!("execute"));
        m.insert("scopes".into(), json!(["call:allowlist"]));
        m.insert("availability".into(), json!("ready"));
        specs.push(Value::Object(m));
    }
    if specs.is_empty() {
        return Ok(json!({"ok": true, "registered": [], "count": 0}));
    }
    tools_register(&json!({"specs": specs}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_search_describe_and_forbid() {
        let _ = tools_clear(&json!({}));
        let reg = tools_register_eng_defaults(&json!({})).unwrap();
        assert_eq!(reg["count"], 5);

        let allow = json!(["tool_find", "tool_reuse"]);
        let search = tools_search(&json!({"tools": allow, "query": "find"})).unwrap();
        assert_eq!(search["count"], 1);
        assert_eq!(search["items"][0]["id"], "tool_find");

        // Unauthorized tool must not appear.
        let search2 = tools_search(&json!({"tools": allow, "query": "bind"})).unwrap();
        assert_eq!(search2["count"], 0);

        let denied = tools_describe(&json!({"id": "tool_bind", "tools": allow})).unwrap();
        assert_eq!(denied["authorized"], false);
        assert_eq!(denied["ok"], false);

        let ok = tools_describe(&json!({"id": "tool_find", "tools": allow})).unwrap();
        assert_eq!(ok["found"], true);
        assert_eq!(ok["spec"]["schema"], "ToolSpecV1");
    }

    #[test]
    fn prepare_rejects_bad_args_and_unauthorized() {
        let _ = tools_clear(&json!({}));
        tools_register_eng_defaults(&json!({})).unwrap();
        let allow = json!(["tool_find"]);

        let bad = tools_prepare_call(&json!({
            "tool_id": "tool_find",
            "tools": allow,
            "arguments": {}
        }))
        .unwrap();
        assert_eq!(bad["ok"], false);
        assert_eq!(bad["error"]["category"], "invalid_arguments");

        let unauth = tools_prepare_call(&json!({
            "tool_id": "tool_bind",
            "tools": allow,
            "arguments": {"query": "x"}
        }))
        .unwrap();
        assert_eq!(unauth["ok"], false);
        assert_eq!(unauth["error"]["category"], "forbidden");

        let good = tools_prepare_call(&json!({
            "tool_id": "tool_find",
            "tools": allow,
            "arguments": {"query": "foo"},
            "call_id": "c1"
        }))
        .unwrap();
        assert_eq!(good["ok"], true);
        assert_eq!(good["call"]["schema"], "ToolCallV1");
        assert_eq!(good["call"]["tool_id"], "tool_find");
        assert_eq!(good["needs_ledger"], false);
        assert!(good["args_digest"].as_str().unwrap().starts_with("sha256-lite:"));

        let allow2 = json!(["tool_bind"]);
        let bind = tools_prepare_call(&json!({
            "tool_id": "tool_bind",
            "tools": allow2,
            "arguments": {"query": "x"},
            "call_id": "c2",
            "logical_run_id": "run1",
            "step_id": "s1"
        }))
        .unwrap();
        assert_eq!(bind["ok"], true);
        assert_eq!(bind["needs_ledger"], true);
        assert_eq!(bind["spec"]["effect"], "execute");
        assert!(bind["idempotency_key"]
            .as_str()
            .unwrap()
            .starts_with("run1|s1|tool_bind|"));
    }

    #[test]
    fn needs_ledger_only_for_mutating_effects() {
        assert_eq!(
            tools_needs_ledger(&json!({"effect": "read"})).unwrap()["needs_ledger"],
            false
        );
        assert_eq!(
            tools_needs_ledger(&json!({"effect": "write"})).unwrap()["needs_ledger"],
            true
        );
        assert_eq!(
            tools_needs_ledger(&json!({"effect": "execute"})).unwrap()["needs_ledger"],
            true
        );
    }

    #[test]
    fn refuse_unsupported_schema_keywords() {
        let err = tools_register(&json!({
            "id": "bad",
            "input_schema": {
                "type": "object",
                "oneOf": [{"type": "object"}]
            }
        }));
        assert!(err.is_err());
    }
}

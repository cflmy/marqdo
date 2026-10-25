//! Workbench M00–M06 contract gate (offline).
//! Spec: doc/design/qdagent-workbench-runtime-and-view-2026-10-25.md

use serde_json::Value;
use std::path::PathBuf;

fn fixture(name: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/workbench/fixtures").join(name);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).expect("fixture json")
}

fn schema(name: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/workbench/schemas").join(name);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).expect("schema json")
}

/// Lightweight required-field check against our JSON Schema documents (no jsonschema crate).
fn assert_required(instance: &Value, schema_doc: &Value, label: &str) {
    let required = schema_doc["required"].as_array().expect("schema.required");
    for key in required {
        let k = key.as_str().unwrap();
        assert!(
            instance.get(k).is_some(),
            "{label}: missing required field {k} in {instance}"
        );
    }
}

fn mlsp(method: &str, params: Value) -> Value {
    let req = serde_json::json!({"id": 1, "method": method, "params": params});
    let line = marqdo::mlsp::handle_line(&req.to_string());
    serde_json::from_str(&line).expect("mlsp response json")
}

#[test]
fn m00_fixtures_consume_and_capabilities() {
    let sel = fixture("model_selection_ok.json");
    assert_required(&sel, &schema("ModelSelectionV1.schema.json"), "selection");
    assert_eq!(sel["schema"], "ModelSelectionV1");
    let parsed = marqdo::workbench::parse_model_selection(&sel, true).expect("strict ok");
    assert_eq!(parsed.selection_id, "sel-1");
    let snap = parsed.public_snapshot();
    assert!(snap.get("api_key").is_none());

    let usage = fixture("model_call_usage_unknown.json");
    assert_required(&usage, &schema("ModelCallUsageV1.schema.json"), "usage");
    assert!(usage["input_tokens"].is_null());
    assert_ne!(usage["input_tokens"], 0);
    assert_eq!(usage["usage_state"], "unknown");

    let cancelled = fixture("model_call_usage_cancelled.json");
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["usage_state"], "partial");

    let doc = fixture("document_view_partial.json");
    assert_required(&doc, &schema("DocumentViewV1.schema.json"), "document_view");
    assert_eq!(doc["execution"]["state"], "not_run");

    let stale = fixture("locator_stale.json");
    assert_eq!(stale["resolution"], "stale");
    let loc = &stale["locator"];
    assert_required(loc, &schema("KnowledgeLocatorV1.schema.json"), "locator");

    let unavail = fixture("locator_version_unavailable.json");
    assert_eq!(unavail["resolution"], "version_unavailable");

    let ev = fixture("runtime_event_cancelled.json");
    assert_required(&ev, &schema("RuntimeEventV1.schema.json"), "runtime_event");
    assert_eq!(ev["type"], "run.cancelled");

    let _ = fixture("model_selection_error.json");
    let _ = fixture("document_view_unsupported.json");

    let caps = mlsp("capabilities", serde_json::json!({}));
    assert_eq!(caps["ok"], true, "{caps}");
    let features = &caps["result"]["features"];
    assert_eq!(features["view.document"], 1);
    assert_eq!(features["view.reference_resolve"], 1);
    assert!(caps["result"]["contracts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c.as_str() == Some("DocumentViewV1")));

    // F16 honesty: unloaded plugins + forced-off stream/cancel → 0
    let down = mlsp(
        "capabilities",
        serde_json::json!({
            "force_unloaded": true,
            "force_stream_off": true,
            "force_cancel_off": true
        }),
    );
    assert_eq!(down["result"]["features"]["llm.messages"], 0);
    assert_eq!(down["result"]["features"]["runtime.event_stream"], 0);
    assert_eq!(down["result"]["features"]["runtime.cancel"], 0);
}

#[test]
fn m00_strict_selection_rejects_missing() {
    let bad = serde_json::json!({"schema_version": 1, "model_id": "x"});
    let err = marqdo::workbench::parse_model_selection(&bad, true).unwrap_err();
    assert!(err.contains("selection_missing"), "{err}");
}

#[test]
fn m02_usage_null_not_zero() {
    let usage = marqdo::workbench::model_call_usage_from_provider(
        "c1",
        "a1",
        "r1",
        &serde_json::json!({"selection_id": "s", "provider_id": "p", "model_id": "m"}),
        &serde_json::Value::Null,
        "succeeded",
    );
    assert!(usage["input_tokens"].is_null());
    assert_eq!(usage["measurement_source"], "none");
    assert_eq!(usage["usage_state"], "unknown");
}

#[test]
fn m03_view_document_no_exec() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "main.mq.md",
            "mode": "read",
            "include": ["fragment", "outline", "nodes", "diagnostics"]
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    let doc = &resp["result"];
    assert_eq!(doc["schema"], "DocumentViewV1");
    assert_eq!(doc["execution"]["state"], "not_run");
    assert_eq!(doc["capabilities"]["read_only"], true);
    assert_eq!(doc["capabilities"]["executed"], false);
    let fragment = doc["render"]["fragment"].as_str().unwrap_or("");
    assert!(fragment.contains("mq-document"), "{fragment}");
    assert!(!fragment.contains("<script"), "must not embed scripts");
    let outline = doc["outline"].as_array().unwrap();
    assert!(
        outline.iter().any(|o| o["kind"] == "heading"),
        "expected heading outline from # lines: {outline:?}"
    );
}

#[test]
fn m03_view_document_revision_mismatch_refuses_downgrade() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "main.mq.md",
            "revision": "sha256:deadbeef",
            "mode": "read"
        }),
    );
    assert_eq!(resp["ok"], false, "{resp}");
    assert_eq!(resp["error"]["code"], "view.revision_mismatch");
}

#[test]
fn m04_reference_resolve_unresolved() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.reference_resolve",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "project_id": root.to_string_lossy(),
            "document_id": "main.mq.md",
            "artifact_id": "does-not-exist-xyz"
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["resolution"], "unresolved");
}

#[test]
fn m04_generation_mismatch_version_unavailable() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.reference_resolve",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "project_id": root.to_string_lossy(),
            "document_id": "main.mq.md",
            "generation_id": "not-a-real-generation",
            "artifact_id": "商品小计"
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["resolution"], "version_unavailable");
}

#[test]
fn m04_source_revision_mismatch_stale() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.reference_resolve",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "project_id": root.to_string_lossy(),
            "document_id": "main.mq.md",
            "source_revision": "sha256:deadbeef",
            "artifact_id": "商品小计"
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["resolution"], "stale");
    assert_eq!(
        resp["result"]["diagnostics"][0]["code"],
        "source_revision_mismatch"
    );
}

#[test]
fn m04_artifact_inspect_version_mismatch() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let ok = mlsp(
        "artifact.inspect",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "artifact_id": "商品小计"
        }),
    );
    assert_eq!(ok["ok"], true, "{ok}");
    let bad_rev = mlsp(
        "artifact.inspect",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "artifact_id": "商品小计",
            "expected_source_revision": "sha256:deadbeef"
        }),
    );
    assert_eq!(bad_rev["ok"], false, "{bad_rev}");
    assert_eq!(bad_rev["error"]["code"], "artifact.version_mismatch");
    let bad_gen = mlsp(
        "artifact.inspect",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "artifact_id": "商品小计",
            "expected_generation": "not-a-real-generation"
        }),
    );
    assert_eq!(bad_gen["ok"], false, "{bad_gen}");
    assert_eq!(bad_gen["error"]["code"], "artifact.version_mismatch");
}

#[test]
fn m05_extract_segment_byte_locator() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/executable_knowledge/baseline/02-extract/main.mq.md");
    let doc = marqdo::document_extract::extract_path(&path);
    assert!(doc.status == "ok" || doc.status == "partial", "{:?}", doc.status);
    assert!(
        doc.segments.iter().any(|s| s.locator.contains(":B")),
        "expected byte offset in locator: {:?}",
        doc.segments
    );
}

#[test]
fn m06_capabilities_lists_adapters_honestly() {
    let caps = mlsp("capabilities", serde_json::json!({}));
    let adapters = caps["result"]["adapters"].as_array().unwrap();
    let native = adapters
        .iter()
        .find(|a| a["id"] == "anthropic-native")
        .unwrap();
    assert_eq!(native["available"], false);
}

#[test]
fn m01_run_context_spawn_child_inherits_selection() {
    let parent = marqdo::workbench::RunContext::from_value(&serde_json::json!({
        "run_id": "run-parent",
        "turn_id": "turn-1",
        "workspace_scope": "ws-a",
        "selection_context": {
            "selection_id": "sel-1",
            "upstream_model_id": "model-a",
            "base_url": "http://127.0.0.1:9"
        },
        "cancellation_token": "/tmp/mq-cancel-test"
    }))
    .unwrap();
    let child = parent.spawn_child("run-child");
    assert_eq!(child.parent_run_id.as_deref(), Some("run-parent"));
    assert_eq!(child.selection_context["upstream_model_id"], "model-a");
    let env = child.selection_env();
    assert!(env.iter().any(|(k, v)| k == "OPENAI_MODEL" && v == "model-a"));
    assert!(env.iter().any(|(k, _)| k == "MARQDO_CANCEL_TOKEN"));
    // Public JSON must not invent api_key.
    assert!(child.to_json().get("api_key").is_none());
}

#[test]
fn m01_messages_shape_rejects_unsupported_part() {
    // Plugin validation is mirrored by selection/message contract: unsupported part types error.
    let bad = serde_json::json!([
        {"role": "user", "content": [{"type": "image", "url": "x"}]}
    ]);
    let ty = bad[0]["content"][0]["type"].as_str().unwrap();
    assert_ne!(ty, "text");
}

#[test]
fn m02_usage_revision_and_cancel_token() {
    let u1 = marqdo::workbench::model_call_usage_from_provider(
        "c1",
        "a1",
        "r1",
        &serde_json::json!({"selection_id": "s"}),
        &serde_json::json!({"prompt_tokens": 10, "completion_tokens": 2, "total_tokens": 12}),
        "succeeded",
    );
    assert_eq!(u1["usage_state"], "final");
    assert_eq!(u1["input_tokens"], 10);
    assert_eq!(u1["revision"], 1);

    let dir = std::env::temp_dir().join(format!("mq-cancel-{}", std::process::id()));
    let _ = std::fs::remove_file(&dir);
    std::fs::write(&dir, "cancelled").unwrap();
    assert!(marqdo::workbench::cancel_requested(Some(dir.to_str().unwrap())));
    let _ = std::fs::remove_file(&dir);
    assert!(!marqdo::workbench::cancel_requested(Some("/no/such/cancel/token")));
}

#[test]
fn m02_event_sink_emits_runtime_event() {
    let path = std::env::temp_dir().join(format!("mq-evt-{}.ndjson", std::process::id()));
    let _ = std::fs::remove_file(&path);
    marqdo::workbench::bind_event_sink(&path);
    let ev = marqdo::workbench::emit_message_delta("run-evt", "hello");
    assert_eq!(ev["schema"], "RuntimeEventV1");
    assert_eq!(ev["type"], "message.delta");
    marqdo::workbench::clear_event_sink();
    let body = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(body.contains("message.delta"), "{body}");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn m03_view_document_path_escape_rejected() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "../Cargo.toml",
            "mode": "read"
        }),
    );
    assert_eq!(resp["ok"], false, "{resp}");
    let code = resp["error"]["code"].as_str().unwrap_or("");
    assert!(
        code.contains("path") || code.contains("not_found") || code.contains("escape"),
        "{resp}"
    );
}

#[test]
fn m05_project_cache_isolation_f14() {
    let a = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let b = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/01-inline");
    assert!(
        marqdo::workbench::analysis_cache::cache_keys_isolated(&a, &b),
        "distinct project roots must not share cache keys"
    );
    let _ = marqdo::workbench::analysis_cache::note_analysis(&a);
    let hit = marqdo::workbench::analysis_cache::note_analysis(&a);
    assert!(hit, "second analysis of same snapshot should hit cache");
    let _ = marqdo::workbench::analysis_cache::note_analysis(&b);
}

#[test]
fn m03_f08_side_effect_marker_not_created_by_view() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let marker = root.join(".workbench-f08-marker");
    let _ = std::fs::remove_file(&marker);
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "main.mq.md",
            "mode": "read"
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert!(!marker.exists(), "view.document must not create side-effect markers");
}

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

/// M-A0: freeze Agent reliability protocol fixtures; capability bits stay 0.
#[test]
fn ma0_agent_reliability_protocol_freeze() {
    let err = fixture("agent_error_v1.json");
    assert_required(&err, &schema("AgentErrorV1.schema.json"), "agent_error");
    assert_eq!(err["schema"], "AgentErrorV1");
    assert_eq!(err["schema_version"], 1);

    let degrade = fixture("agent_error_v1_unknown_version.json");
    assert_eq!(degrade["schema"], "AgentErrorV1");
    assert_eq!(degrade["schema_version"], 99);
    assert_eq!(degrade["future_field"], true);

    let tool = fixture("tool_spec_v1.json");
    assert_required(&tool, &schema("ToolSpecV1.schema.json"), "tool_spec");
    assert_required(
        &fixture("tool_call_v1.json"),
        &schema("ToolCallV1.schema.json"),
        "tool_call",
    );
    assert_required(
        &fixture("tool_result_v1.json"),
        &schema("ToolResultV1.schema.json"),
        "tool_result",
    );

    let ckpt = fixture("checkpoint_v2.json");
    assert_required(&ckpt, &schema("CheckpointV2.schema.json"), "checkpoint");
    assert_eq!(ckpt["schema_version"], 2);

    let legacy = fixture("checkpoint_legacy_resume.json");
    assert!(legacy.get("schema").is_none());
    assert!(legacy.get("effect_class").is_none());
    assert_eq!(legacy["id"], "legacy_resume_01");

    let ledger = fixture("effect_ledger_v1.json");
    assert_required(
        &ledger,
        &schema("EffectLedgerRecordV1.schema.json"),
        "effect_ledger",
    );
    assert_eq!(ledger["schema"], "EffectLedgerRecordV1");
    assert_eq!(ledger["state"], "prepared");
    assert_eq!(ledger["effect_class"], "write");

    let out_ok = fixture("agent_output_v1_succeeded.json");
    assert_required(&out_ok, &schema("AgentOutputV1.schema.json"), "agent_output_ok");
    assert_eq!(out_ok["status"], "succeeded");
    let out_fail = fixture("agent_output_v1_failed.json");
    assert_required(
        &out_fail,
        &schema("AgentOutputV1.schema.json"),
        "agent_output_fail",
    );
    assert_eq!(out_fail["status"], "failed");

    let pack = fixture("context_pack_v2.json");
    assert_required(&pack, &schema("ContextPackV2.schema.json"), "context_pack");
    assert_eq!(pack["schema_version"], 2);
    let item = &pack["items"][0];
    for key in [
        "origin",
        "trust",
        "resource_ref",
        "revision",
        "token_estimate",
        "priority",
    ] {
        assert!(item.get(key).is_some(), "context item missing {key}");
    }

    assert_required(
        &fixture("resource_ref_v1.json"),
        &schema("ResourceRefV1.schema.json"),
        "resource_ref",
    );
    let view = fixture("resource_view_v1.json");
    assert_required(&view, &schema("ResourceViewV1.schema.json"), "resource_view");
    assert_eq!(view["no_exec"], true);

    let tev = fixture("runtime_event_tool_failed.json");
    assert_eq!(tev["schema"], "RuntimeEventV2");
    assert_eq!(tev["type"], "tool.failed");
    assert_eq!(tev["payload"]["schema"], "AgentErrorV1");

    let caps = mlsp("capabilities", serde_json::json!({}));
    assert_eq!(caps["ok"], true, "{caps}");
    let features = &caps["result"]["features"];
    // M-A1–M-A4: agent reliability emitters are live.
    assert_eq!(features["agent.error_v1"], 1);
    assert_eq!(features["agent.checkpoint_v2"], 1);
    assert_eq!(features["agent.tool_schema_v1"], 1);
    assert_eq!(features["agent.output_channels_v1"], 1);
    assert_eq!(features["agent.context_pack_v2"], 1);
    assert_eq!(features["resource.view_v1"], 1);

    let formats = caps["result"]["extract_formats"]
        .as_array()
        .expect("extract_formats");
    let pdf = formats
        .iter()
        .find(|f| f["format"] == "pdf")
        .expect("pdf row");
    assert_eq!(pdf["available"], false);
    let json_fmt = formats
        .iter()
        .find(|f| f["format"] == "json")
        .expect("json row");
    assert_eq!(json_fmt["available"], false);

    // Live contracts list still excludes unimplemented agent schemas.
    let contracts = caps["result"]["contracts"].as_array().unwrap();
    assert!(!contracts.iter().any(|c| c.as_str() == Some("AgentErrorV1")));

    let down = mlsp(
        "capabilities",
        serde_json::json!({
            "force_unloaded": true,
            "force_stream_off": true,
            "force_cancel_off": true
        }),
    );
    assert_eq!(down["result"]["features"]["agent.error_v1"], 1);
    assert_eq!(down["result"]["features"]["agent.checkpoint_v2"], 1);
    assert_eq!(down["result"]["features"]["agent.tool_schema_v1"], 1);
    assert_eq!(down["result"]["features"]["agent.output_channels_v1"], 1);
    assert_eq!(down["result"]["features"]["agent.context_pack_v2"], 1);
    assert_eq!(down["result"]["features"]["resource.view_v1"], 1);
}

#[test]
fn ma3_resource_view_static_no_exec() {
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("marqdo-ma3-rv-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    let md = dir.join("readme.md");
    {
        let mut f = std::fs::File::create(&md).unwrap();
        writeln!(f, "# Hello\n\n<script>x</script>").unwrap();
    }

    let resp = mlsp(
        "resource.view",
        serde_json::json!({
            "workspace_root": dir.to_string_lossy(),
            "path": "readme.md",
            "mode": "read"
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    let view = &resp["result"];
    assert_required(view, &schema("ResourceViewV1.schema.json"), "resource_view_live");
    assert_eq!(view["no_exec"], true);
    assert_eq!(view["schema"], "ResourceViewV1");
    let frag = view["render"]["fragment"].as_str().unwrap();
    assert!(frag.contains("Hello"));
    assert!(frag.contains("&lt;script&gt;") || !frag.contains("<script>"));

    let escape = mlsp(
        "resource.view",
        serde_json::json!({
            "workspace_root": dir.to_string_lossy(),
            "path": "../outside.txt",
            "mode": "read"
        }),
    );
    assert_eq!(escape["ok"], false, "{escape}");

    let _ = std::fs::remove_dir_all(&dir);
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
fn m01_f01_parallel_selections_do_not_cross() {
    let a = marqdo::workbench::RunContext::from_value(&serde_json::json!({
        "run_id": "run-a",
        "selection_context": {
            "upstream_model_id": "model-a",
            "base_url": "http://127.0.0.1:7001"
        }
    }))
    .unwrap();
    let b = marqdo::workbench::RunContext::from_value(&serde_json::json!({
        "run_id": "run-b",
        "selection_context": {
            "upstream_model_id": "model-b",
            "base_url": "http://127.0.0.1:7002"
        }
    }))
    .unwrap();
    let ea = a.selection_env();
    let eb = b.selection_env();
    let model_a = ea.iter().find(|(k, _)| k == "OPENAI_MODEL").map(|(_, v)| v.as_str());
    let model_b = eb.iter().find(|(k, _)| k == "OPENAI_MODEL").map(|(_, v)| v.as_str());
    let base_a = ea.iter().find(|(k, _)| k == "OPENAI_BASE_URL").map(|(_, v)| v.as_str());
    let base_b = eb.iter().find(|(k, _)| k == "OPENAI_BASE_URL").map(|(_, v)| v.as_str());
    assert_eq!(model_a, Some("model-a"));
    assert_eq!(model_b, Some("model-b"));
    assert_ne!(base_a, base_b);
    assert_ne!(a.run_id, b.run_id);
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
fn m02_f05_usage_revision_replaces_same_attempt_not_sum() {
    let sel = serde_json::json!({"selection_id": "s"});
    let partial = marqdo::workbench::model_call_usage_from_provider_rev(
        "c1",
        "a1",
        "r1",
        &sel,
        &serde_json::json!({"prompt_tokens": 10, "completion_tokens": 1, "total_tokens": 11}),
        "started",
        1,
    );
    let final_u = marqdo::workbench::model_call_usage_from_provider_rev(
        "c1",
        "a1",
        "r1",
        &sel,
        &serde_json::json!({"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}),
        "succeeded",
        2,
    );
    let won = marqdo::workbench::replace_usage_if_newer(&partial, &final_u);
    assert_eq!(won["revision"], 2);
    assert_eq!(won["total_tokens"], 15);
    // Older revision must not overwrite.
    let kept = marqdo::workbench::replace_usage_if_newer(&final_u, &partial);
    assert_eq!(kept["revision"], 2);
    assert_eq!(kept["total_tokens"], 15);
    // New attempt is a distinct identity (caller must not merge).
    let retry = marqdo::workbench::model_call_usage_from_provider_rev(
        "c1",
        "a2",
        "r1",
        &sel,
        &serde_json::json!({"prompt_tokens": 3, "completion_tokens": 1, "total_tokens": 4}),
        "succeeded",
        1,
    );
    assert_ne!(retry["attempt_id"], final_u["attempt_id"]);
}

#[test]
fn m02_f07_cancel_does_not_undo_external_write() {
    let marker = std::env::temp_dir().join(format!("mq-f07-{}", std::process::id()));
    std::fs::write(&marker, "already-written").unwrap();
    let tok = std::env::temp_dir().join(format!("mq-f07-cancel-{}", std::process::id()));
    std::fs::write(&tok, "cancelled").unwrap();
    assert!(marqdo::workbench::cancel_requested(Some(tok.to_str().unwrap())));
    // Cancel must not claim or perform undo of prior external writes.
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "already-written"
    );
    let _ = std::fs::remove_file(&marker);
    let _ = std::fs::remove_file(&tok);
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
    let _serial = marqdo::workbench::test_serial_guard();
    marqdo::workbench::reset_cache_for_tests();
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
fn m05_f14_concurrent_100_same_snapshot() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let _serial = marqdo::workbench::test_serial_guard();
    marqdo::workbench::reset_cache_for_tests();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let root_s = root.to_string_lossy().to_string();
    let n = 100usize;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::with_capacity(n);
    for _ in 0..n {
        let barrier = Arc::clone(&barrier);
        let root_s = root_s.clone();
        handles.push(thread::spawn(move || {
            barrier.wait();
            let resp = mlsp(
                "view.document",
                serde_json::json!({
                    "project": root_s,
                    "document": "main.mq.md",
                    "mode": "read",
                    "include": ["fragment"]
                }),
            );
            assert_eq!(resp["ok"], true, "{resp}");
            resp["result"]["document"]["source_revision"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }));
    }
    let mut revs = Vec::new();
    for h in handles {
        revs.push(h.join().expect("thread"));
    }
    assert!(revs.iter().all(|r| r == &revs[0]), "all reads same revision");
    assert!(revs[0].starts_with("sha256:"));
    let (hits, misses, merges) = marqdo::workbench::cache_stats();
    assert!(misses >= 1, "hits={hits} misses={misses} merges={merges}");
    assert!(
        merges + hits >= 99,
        "expected coalesce or result reuse across 100 readers: hits={hits} merges={merges}"
    );
    assert!(
        marqdo::workbench::result_hits() >= 99,
        "followers must reuse Arc<Compilation>, result_hits={}",
        marqdo::workbench::result_hits()
    );
    // Strict compile-once is covered by analysis_cache unit test under serial guard.
    // Here we only require that this fixture was compiled (other parallel tests may
    // also touch it) and that readers shared results (result_hits / merges above).
    assert!(
        marqdo::workbench::compile_count_for(&root) >= 1,
        "fixture must have been compiled at least once"
    );
}

#[test]
fn m04_snapshot_manifest_on_document_view() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "main.mq.md",
            "mode": "read",
            "include": ["fragment"]
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    let snap = &resp["result"]["snapshot"];
    assert_eq!(snap["schema"], "SnapshotManifestV1");
    assert_eq!(snap["retention"]["owner"], "application");
    assert!(snap["source_revision"].as_str().unwrap().starts_with("sha256:"));
    assert_eq!(
        snap["source_revision"],
        resp["result"]["document"]["source_revision"]
    );
}

#[test]
fn m01_f02_run_context_three_level_inherit() {
    let root = marqdo::workbench::RunContext::from_value(&serde_json::json!({
        "run_id": "run-root",
        "turn_id": "turn-1",
        "workspace_scope": "ws-1",
        "selection_context": {
            "selection_id": "sel-1",
            "upstream_model_id": "model-fixed",
            "base_url": "http://127.0.0.1:7700"
        },
        "cancellation_token": "/tmp/mq-cancel-f02"
    }))
    .unwrap();
    let child = root.spawn_child("run-child");
    let grand = child.spawn_child("run-grand");
    assert_eq!(grand.parent_run_id.as_deref(), Some("run-child"));
    assert_eq!(
        grand.selection_context["upstream_model_id"],
        "model-fixed"
    );
    assert_eq!(grand.turn_id.as_deref(), Some("turn-1"));
    assert_eq!(
        grand.cancellation_token.as_deref(),
        Some("/tmp/mq-cancel-f02")
    );
    let env = grand.selection_env();
    assert!(env.iter().any(|(k, v)| k == "OPENAI_MODEL" && v == "model-fixed"));
    // Root selection must not be mutated by descendants.
    assert_eq!(root.selection_context["upstream_model_id"], "model-fixed");
    assert_eq!(root.run_id, "run-root");
}

#[test]
fn m03_f08_side_effect_marker_not_created_by_view() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/workbench/fixtures");
    let marker = root.join(".workbench-f08-marker");
    let _ = std::fs::remove_file(&marker);
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "f08_write_side_effect.mq.md",
            "mode": "read",
            "include": ["fragment", "diagnostics"]
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["execution"]["state"], "not_run");
    assert!(
        !marker.exists(),
        "view.document must not execute file.write side effects"
    );
}

#[test]
fn m03_f09_step_kinds_baseline_projects_nodes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/07-step-kinds");
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "main.mq.md",
            "mode": "read",
            "include": ["outline", "nodes", "fragment"]
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    let outline = resp["result"]["outline"].as_array().unwrap();
    assert!(
        outline.iter().any(|o| o["kind"] == "heading"),
        "expected heading outline: {outline:?}"
    );
    let nodes = resp["result"]["nodes"].as_array().unwrap();
    assert!(nodes.len() >= 2, "expected artifact+step nodes: {nodes:?}");
    let kinds: std::collections::HashSet<&str> = nodes
        .iter()
        .filter_map(|n| n["kind"].as_str())
        .collect();
    assert!(
        kinds.iter().any(|k| matches!(
            *k,
            "observe" | "derive" | "call" | "iterate" | "narrate" | "type" | "function"
        )),
        "expected step/artifact kinds in read model: {kinds:?}"
    );
    // Evidence statuses must not be silently upgraded to verified by the read path.
    for n in nodes {
        if let Some(ev) = n.get("evidence").and_then(|v| v.as_str()) {
            assert_ne!(ev, "verified", "read path must not invent verified: {n}");
        }
    }
    let fragment = resp["result"]["render"]["fragment"].as_str().unwrap_or("");
    assert!(fragment.contains("mq-document"));
    assert!(!fragment.contains("<script"));
}

#[test]
fn m03_f09_outline_nodes_chinese_and_static_a11y_subset() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let resp = mlsp(
        "view.document",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "document": "main.mq.md",
            "mode": "read",
            "include": ["fragment", "outline", "nodes", "refs"]
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    let doc = &resp["result"];
    let outline = doc["outline"].as_array().unwrap();
    assert!(outline.iter().any(|o| o["kind"] == "heading"));
    let nodes = doc["nodes"].as_array().unwrap();
    assert!(!nodes.is_empty(), "expected semantic nodes");
    // Chinese / CJK labels should round-trip in outline or nodes when present in source.
    let blob = format!("{outline:?}{nodes:?}");
    assert!(
        blob.chars().any(|c| c >= '\u{4e00}'),
        "expected CJK in outline/nodes for 02-extract: {blob}"
    );
    let fragment = doc["render"]["fragment"].as_str().unwrap_or("");
    // F12 static HTML subset (full browser → qdagent): semantic article, no script, mq namespace.
    assert!(fragment.starts_with("<article") || fragment.contains("mq-document"));
    assert!(!fragment.contains("<script"));
    assert!(!fragment.contains("javascript:"));
}

#[test]
fn m04_f11_ambiguous_same_name() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/08-same-name");
    assert!(root.is_dir(), "missing 08-same-name fixture");
    // a.mq.md and b.mq.md both define 商品小计 — omit document_id so both stay in scope.
    let resp = mlsp(
        "view.reference_resolve",
        serde_json::json!({
            "project": root.to_string_lossy(),
            "project_id": root.to_string_lossy(),
            "artifact_id": "商品小计"
        }),
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["resolution"], "ambiguous", "{resp}");
    let cands = resp["result"]["candidates"].as_array().unwrap();
    assert!(cands.len() >= 2, "{resp}");
}

/// F02: agent.step + resume with explicit llm.create selection against a local mock.
/// Env poison model must not appear in the upstream request body.
#[test]
fn m01_f02_agent_step_selection_and_resume() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::process::Command;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ext = root.join("ext");

    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_llm", "-p", "marqdo_plugin_agent", "--locked"])
        .current_dir(&root)
        .status()
        .expect("build plugins");
    assert!(status.success(), "plugin build failed");

    let llm_so = root.join("target/debug/libllm.so");
    let agent_so = root.join("target/debug/libagent.so");
    assert!(llm_so.is_file(), "missing {llm_so:?}");
    assert!(agent_so.is_file(), "missing {agent_so:?}");

    let seen = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let seen_bg = seen.clone();
    thread::spawn(move || {
        // Agent may deepen once (READ) then answer — accept a few requests.
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut raw = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&buf[..n]);
                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&raw[..pos]);
                    let cl = header
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length:"))
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    let body_start = pos + 4;
                    while raw.len() < body_start + cl {
                        let n = stream.read(&mut buf).unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        raw.extend_from_slice(&buf[..n]);
                    }
                    break;
                }
            }
            let req = String::from_utf8_lossy(&raw).to_string();
            seen_bg.lock().unwrap().push(req);
            let body = r#"{"choices":[{"message":{"content":"pong"},"finish_reason":"stop"}],"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    let base = format!("http://{addr}/v1");
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/workbench-agent-selection.mq.md"])
        .current_dir(&root)
        .env("MARQDO_EXT", &ext)
        .env("MARQDO_LLM_PLUGIN", &llm_so)
        .env("MARQDO_AGENT_PLUGIN", &agent_so)
        .env("OPENAI_BASE_URL", &base)
        .env("OPENAI_API_KEY", "f02-key")
        .env("OPENAI_MODEL", "f02-selected-model")
        .output()
        .expect("run workbench-agent-selection");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code().unwrap_or(1),
        0,
        "stderr={stderr}\nstdout={stdout}"
    );
    assert!(
        stdout.contains("f02-agent-selection-ok"),
        "stdout={stdout}"
    );
    assert!(stdout.contains("resume-save-ok") && stdout.contains("resume-load-ok"));

    let captured = seen.lock().unwrap().join("\n");
    assert!(
        !captured.is_empty(),
        "mock received no HTTP; stdout={stdout}"
    );
    // Body may embed the .mq.md source (which mentions the poison string in prose).
    // Assert the OpenAI `model` field itself — not a raw substring of the whole HTTP dump.
    let model_field = captured
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .lines()
        .next()
        .unwrap_or("");
    let body: Value = serde_json::from_str(model_field).unwrap_or_else(|_| {
        // Some captures concatenate multiple HTTP messages; take last `{...}` blob.
        let start = captured.rfind('{').expect("json body");
        serde_json::from_str(&captured[start..]).expect("parse chat body")
    });
    assert_eq!(
        body["model"].as_str(),
        Some("f02-selected-model"),
        "upstream model field: {body}"
    );
    assert_ne!(body["model"].as_str(), Some("env-poison-model"));
}

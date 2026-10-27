//! MLSP `capabilities` — probe loaded plugins and declared features (M00 / F16).
//! Feature integers must reflect *implemented and testable* behavior only.

use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Host/tests may force stream/cancel off for honesty / F16 mismatch cases.
static FORCE_STREAM_OFF: AtomicBool = AtomicBool::new(false);
static FORCE_CANCEL_OFF: AtomicBool = AtomicBool::new(false);

pub fn set_event_stream_ready(ready: bool) {
    FORCE_STREAM_OFF.store(!ready, Ordering::Relaxed);
}

pub fn set_cancel_ready(ready: bool) {
    FORCE_CANCEL_OFF.store(!ready, Ordering::Relaxed);
}

/// Build the capabilities document. Does not call providers or execute project code.
pub fn probe_capabilities(params: &Value) -> Value {
    let runtime_version = env!("CARGO_PKG_VERSION").to_string();
    let mut adapters = vec![json!({
        "id": "openai-compatible",
        "available": true,
        "notes": "HTTP chat completions; messages[] preferred over single prompt"
    })];
    adapters.push(json!({
        "id": "ollama-compatible",
        "available": true,
        "notes": "OpenAI-compatible path; not a distinct native protocol"
    }));
    adapters.push(json!({
        "id": "anthropic-native",
        "available": false,
        "notes": "not verified in this release"
    }));
    adapters.push(json!({
        "id": "gemini-native",
        "available": false,
        "notes": "not verified in this release"
    }));

    let extract_formats = vec![
        json!({"format": "text", "available": true}),
        json!({"format": "markdown", "available": true}),
        json!({"format": "pdf", "available": false}),
        json!({"format": "docx", "available": false}),
        json!({"format": "ocr", "available": false}),
    ];

    let loaded_plugins = probe_loaded_plugins(params);

    let llm_loaded = loaded_plugins.iter().any(|p| {
        p.get("name").and_then(|n| n.as_str()) == Some("llm")
            && p.get("available").and_then(|a| a.as_bool()) == Some(true)
    });

    let stream_off = FORCE_STREAM_OFF.load(Ordering::Relaxed)
        || params
            .get("force_stream_off")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    let cancel_off = FORCE_CANCEL_OFF.load(Ordering::Relaxed)
        || params
            .get("force_cancel_off")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    let mut features = serde_json::Map::new();
    features.insert(
        "llm.explicit_selection".into(),
        json!(if llm_loaded { 1 } else { 0 }),
    );
    features.insert(
        "llm.call_usage".into(),
        json!(if llm_loaded { 1 } else { 0 }),
    );
    features.insert(
        "llm.messages".into(),
        json!(if llm_loaded { 1 } else { 0 }),
    );
    // M02 implemented: event_sink + web sse-live + MARQDO_CANCEL in llm plugin.
    features.insert(
        "runtime.event_stream".into(),
        json!(if stream_off { 0 } else { 1 }),
    );
    features.insert(
        "runtime.cancel".into(),
        json!(if cancel_off { 0 } else { 1 }),
    );
    // Isolated web runtime (doc/design/isolated-web-runtime-development-2026-10-26.md).
    // HTTP execute / sse-live defaults to the Supervisor worker pool (ADR 0009).
    features.insert("runtime.isolated_execution".into(), json!(1));
    // task_control_v2 / event_replay: host_query("runtime") + /_runtime/* system routes.
    features.insert("runtime.task_control_v2".into(), json!(1));
    features.insert("runtime.event_replay".into(), json!(1));
    // short/read/control HTTP invokes use isolated admission (no parent INVOKE_LOCK).
    features.insert("runtime.read_lane".into(), json!(1));
    #[cfg(feature = "view")]
    {
        features.insert("view.analysis_cache_v2".into(), json!(1));
        features.insert("view.document".into(), json!(1));
        features.insert("view.reference_resolve".into(), json!(1));
    }
    #[cfg(not(feature = "view"))]
    {
        features.insert("view.analysis_cache_v2".into(), json!(0));
        features.insert("view.document".into(), json!(0));
        features.insert("view.reference_resolve".into(), json!(0));
    }

    json!({
        "schema_version": 1,
        "runtime_version": runtime_version,
        "features": features,
        "adapters": adapters,
        "extract_formats": extract_formats,
        "loaded_plugins": loaded_plugins,
        "contracts": [
            "ModelSelectionV1",
            "ModelCallUsageV1",
            "RuntimeEventV1",
            "RuntimeEventV2",
            "InvokeRequestV2",
            "DocumentViewV1",
            "KnowledgeLocatorV1"
        ]
    })
}

fn probe_loaded_plugins(params: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    let candidates = [
        ("llm", "MARQDO_LLM_PLUGIN", plugin_name("llm")),
        ("agent", "MARQDO_AGENT_PLUGIN", plugin_name("agent")),
        ("web", "MARQDO_WEB_PLUGIN", plugin_name("web")),
    ];
    let ext_hint = params
        .get("ext_root")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("MARQDO_EXT").map(PathBuf::from));

    let force_unloaded = params
        .get("force_unloaded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    for (name, env_key, filename) in candidates {
        if force_unloaded {
            out.push(json!({
                "name": name,
                "available": false,
                "path": Value::Null,
                "capabilities": [],
            }));
            continue;
        }
        let mut path: Option<PathBuf> = std::env::var_os(env_key).map(PathBuf::from);
        if path.as_ref().map(|p| !p.is_file()).unwrap_or(true) {
            let debug = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join(&filename);
            if debug.is_file() {
                path = Some(debug);
            }
        }
        if path.as_ref().map(|p| !p.is_file()).unwrap_or(true) {
            if let Some(ext) = &ext_hint {
                let native = ext.join("native").join(&filename);
                if native.is_file() {
                    path = Some(native);
                }
            }
        }
        let available = path.as_ref().map(|p| p.is_file()).unwrap_or(false);
        let mut caps = Vec::new();
        if name == "llm" && available {
            caps.push("call_usage_v1");
            caps.push("messages_v1");
            caps.push("explicit_selection_v1");
            caps.push("cancel_v1");
        }
        if name == "agent" && available {
            caps.push("eng_bind_execute");
            caps.push("publish_gate");
        }
        out.push(json!({
            "name": name,
            "available": available,
            "path": path.map(|p| p.to_string_lossy().to_string()),
            "capabilities": caps,
        }));
    }
    out
}

fn plugin_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{stem}.dylib")
    } else {
        format!("lib{stem}.so")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_off_and_unloaded_honest() {
        let caps = probe_capabilities(&json!({
            "force_unloaded": true,
            "force_stream_off": true,
            "force_cancel_off": true,
        }));
        assert_eq!(caps["features"]["runtime.event_stream"], 0);
        assert_eq!(caps["features"]["runtime.cancel"], 0);
        assert_eq!(caps["features"]["llm.messages"], 0);
    }
}

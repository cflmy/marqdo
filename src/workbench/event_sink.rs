//! Request-local RuntimeEventV1 sink (M02). Avoids process-global event mix-up.

use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static SEQ: AtomicU64 = AtomicU64::new(0);

fn sink_path() -> &'static Mutex<Option<PathBuf>> {
    static SINK: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    SINK.get_or_init(|| Mutex::new(None))
}

/// Bind this process/request to an NDJSON event file (also used by web sse-live).
pub fn bind_event_sink(path: impl AsRef<Path>) {
    if let Ok(mut g) = sink_path().lock() {
        *g = Some(path.as_ref().to_path_buf());
    }
    crate::workbench::capabilities::set_event_stream_ready(true);
}

pub fn clear_event_sink() {
    if let Ok(mut g) = sink_path().lock() {
        *g = None;
    }
}

pub fn emit_event(run_id: &str, event_type: &str, payload: Value) -> Value {
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let event_id = format!("evt-{run_id}-{seq}");
    let ev = crate::workbench::usage::runtime_event(&event_id, seq, run_id, event_type, payload);
    if let Ok(g) = sink_path().lock() {
        if let Some(path) = g.as_ref() {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(f, "{}", ev);
            }
        }
    }
    // Also honor MARQDO_EVENT_FORWARD for subtask/web bridge.
    if let Ok(fwd) = std::env::var("MARQDO_EVENT_FORWARD") {
        if !fwd.is_empty() {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&fwd) {
                let _ = writeln!(f, "{}", ev);
            }
        }
    }
    ev
}

pub fn emit_message_delta(run_id: &str, text: &str) -> Value {
    emit_event(
        run_id,
        "message.delta",
        json!({"text": text, "format": "plain"}),
    )
}

//! Process-wide stream event bus for `marqdo view` SSE (S3).
//! Publishers: LLM SSE parse, optional `host_stream_publish`.
//! Subscribers: view `GET /api/events` / `POST /api/run`.
//!
//! File subtasks set `MARQDO_EVENT_FORWARD=<ndjson path>`; the child appends each
//! published event so the parent can fan them onto its own bus while `wait` blocks.

use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

use crate::host::json::value_to_json;
use crate::value::Value;

static BUS: OnceLock<EventBus> = OnceLock::new();

pub struct EventBus {
    subs: Mutex<Vec<Sender<String>>>,
}

impl EventBus {
    fn new() -> Self {
        Self {
            subs: Mutex::new(Vec::new()),
        }
    }

    pub fn global() -> &'static EventBus {
        BUS.get_or_init(EventBus::new)
    }

    /// Subscribe to JSON event lines (already serialized maps).
    pub fn subscribe(&self) -> Receiver<String> {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut g) = self.subs.lock() {
            g.push(tx);
        }
        rx
    }

    pub fn publish_json(&self, json_line: &str) {
        if let Ok(mut g) = self.subs.lock() {
            g.retain(|tx| tx.send(json_line.to_string()).is_ok());
        }
        // Child → parent bridge (no-op unless spawn set the env).
        forward_to_parent_file(json_line);
    }

    pub fn publish_value(&self, v: &Value) {
        if let Ok(j) = value_to_json(v) {
            self.publish_json(&j.to_string());
        }
    }
}

fn forward_to_parent_file(json_line: &str) {
    let Ok(path) = std::env::var("MARQDO_EVENT_FORWARD") else {
        return;
    };
    if path.is_empty() {
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{json_line}");
        let _ = f.flush();
    }
}

/// Publish a Marqdo map (or any value) onto the global bus.
pub fn publish(v: &Value) {
    EventBus::global().publish_value(v);
}

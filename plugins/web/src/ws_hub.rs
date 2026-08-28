//! Per-path WebSocket broadcast hubs (W6 remainder).

use std::collections::HashMap;
use std::sync::Mutex;

use tokio::sync::broadcast;

const CAP: usize = 256;

static HUBS: Mutex<Option<HashMap<String, broadcast::Sender<String>>>> = Mutex::new(None);

fn with_hubs<T>(f: impl FnOnce(&mut HashMap<String, broadcast::Sender<String>>) -> T) -> T {
    let mut g = HUBS.lock().unwrap_or_else(|e| e.into_inner());
    let m = g.get_or_insert_with(HashMap::new);
    f(m)
}

/// Subscribe to (or create) the broadcast channel for `path`.
pub fn subscribe(path: &str) -> broadcast::Receiver<String> {
    with_hubs(|m| {
        if let Some(tx) = m.get(path) {
            return tx.subscribe();
        }
        let (tx, rx) = broadcast::channel(CAP);
        m.insert(path.to_string(), tx);
        rx
    })
}

/// Publish a text frame to all subscribers on `path`.
pub fn publish(path: &str, text: String) {
    with_hubs(|m| {
        let tx = m.entry(path.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(CAP);
            tx
        });
        let _ = tx.send(text);
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WsMode {
    Echo,
    Broadcast,
    Drain,
}

impl WsMode {
    pub fn parse(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Bool(true) => WsMode::Echo,
            serde_json::Value::Bool(false) => WsMode::Drain,
            serde_json::Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
                "broadcast" | "广播" => WsMode::Broadcast,
                "drain" | "false" | "0" | "no" | "drain_only" => WsMode::Drain,
                _ => WsMode::Echo,
            },
            serde_json::Value::Object(m) => {
                if let Some(mode) = m.get("mode").or_else(|| m.get("模式")) {
                    return Self::parse(mode);
                }
                if let Some(echo) = m.get("echo").or_else(|| m.get("回显")) {
                    return Self::parse(echo);
                }
                WsMode::Echo
            }
            _ => WsMode::Echo,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            WsMode::Echo => "echo",
            WsMode::Broadcast => "broadcast",
            WsMode::Drain => "drain",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_fanout() {
        let mut a = subscribe("/t-fanout");
        let mut b = subscribe("/t-fanout");
        publish("/t-fanout", "hi".into());
        assert_eq!(a.try_recv().unwrap(), "hi");
        assert_eq!(b.try_recv().unwrap(), "hi");
    }
}

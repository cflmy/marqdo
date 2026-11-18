//! Length-prefixed JSON control frames + event channel types (M-R2 / M-R4).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL_VERSION: u32 = 2;
pub const CONTROL_FRAME_MAX_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeRequestV2 {
    pub schema: String,
    pub schema_version: u32,
    pub task_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub lease: LeaseRef,
    pub entry: EntryRef,
    #[serde(default)]
    pub args_ref: Option<String>,
    pub context: InvokeContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRef {
    pub id: String,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRef {
    pub module: String,
    pub function: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeContext {
    pub workspace_id: String,
    #[serde(default)]
    pub snapshot_id: Option<String>,
    #[serde(default)]
    pub selection_ref: Option<String>,
    #[serde(default)]
    pub capabilities_ref: Option<String>,
    pub deadline_at: String,
    /// Absolute site filesystem root (`--fs-root` for the worker).
    #[serde(default)]
    pub fs_root: Option<String>,
}

impl InvokeRequestV2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "InvokeRequestV2" {
            return Err(format!("protocol.schema: {}", self.schema));
        }
        if self.schema_version != PROTOCOL_VERSION {
            return Err(format!(
                "protocol.version: got {} want {PROTOCOL_VERSION}",
                self.schema_version
            ));
        }
        if self.task_id.is_empty() || self.run_id.is_empty() || self.attempt_id.is_empty() {
            return Err("protocol.ids_required".into());
        }
        if self.entry.module.is_empty() || self.entry.function.is_empty() {
            return Err("protocol.entry_required".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEventV2 {
    pub schema: String,
    pub schema_version: u32,
    pub event_id: String,
    pub seq: u64,
    pub task_id: String,
    pub run_id: String,
    pub attempt_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: Value,
}

impl RuntimeEventV2 {
    pub fn new(
        task_id: &str,
        run_id: &str,
        attempt_id: &str,
        seq: u64,
        event_type: &str,
        payload: Value,
    ) -> Self {
        Self {
            schema: "RuntimeEventV2".into(),
            schema_version: PROTOCOL_VERSION,
            event_id: format!("{task_id}:{seq}"),
            seq,
            task_id: task_id.into(),
            run_id: run_id.into(),
            attempt_id: attempt_id.into(),
            event_type: event_type.into(),
            payload,
        }
    }
}

/// Encode a control frame: u32 LE length + UTF-8 JSON (≤ CONTROL_FRAME_MAX_BYTES).
pub fn encode_frame(value: &Value) -> Result<Vec<u8>, String> {
    let body = serde_json::to_vec(value).map_err(|e| format!("protocol.encode: {e}"))?;
    if body.len() > CONTROL_FRAME_MAX_BYTES {
        return Err("protocol.frame_too_large".into());
    }
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decode one frame from the front of `buf`; returns (value, bytes_consumed).
pub fn decode_frame(buf: &[u8]) -> Result<Option<(Value, usize)>, String> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if len > CONTROL_FRAME_MAX_BYTES {
        return Err("protocol.frame_too_large".into());
    }
    if buf.len() < 4 + len {
        return Ok(None);
    }
    let value: Value = serde_json::from_slice(&buf[4..4 + len])
        .map_err(|e| format!("protocol.decode: {e}"))?;
    Ok(Some((value, 4 + len)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn frame_round_trip() {
        let v = json!({"hello": "世界"});
        let enc = encode_frame(&v).unwrap();
        let (got, n) = decode_frame(&enc).unwrap().unwrap();
        assert_eq!(n, enc.len());
        assert_eq!(got, v);
    }

    #[test]
    fn incomplete_frame_waits() {
        let v = json!({"a": 1});
        let enc = encode_frame(&v).unwrap();
        assert!(decode_frame(&enc[..3]).unwrap().is_none());
        assert!(decode_frame(&enc[..enc.len() - 1]).unwrap().is_none());
    }
}

//! Task index with idempotency, revision CAS, and event log (M-R3 / M-R4).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::runtime::protocol::RuntimeEventV2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Accepted,
    Queued,
    Running,
    Finalizing,
    Succeeded,
    Failed,
    CancelRequested,
    Cancelled,
    Interrupted,
    NeedsReconciliation,
}

impl TaskState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded
                | Self::Failed
                | Self::Cancelled
                | Self::Interrupted
                | Self::NeedsReconciliation
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub state: TaskState,
    pub revision: u64,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub lease_id: Option<String>,
    pub lease_epoch: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
    pub cancel_requested_at: Option<String>,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Default)]
struct Inner {
    tasks: HashMap<String, TaskRecord>,
    /// idempotency_key → task_id
    idem: HashMap<String, String>,
    /// task_id → events (monotonic seq)
    events: HashMap<String, Vec<RuntimeEventV2>>,
    next_epoch: u64,
}

pub struct TaskStore {
    inner: Arc<Mutex<Inner>>,
    persist_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub enum StoreError {
    Conflict(String),
    NotFound,
    AlreadyTerminal(TaskState),
    Io(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict(s) => write!(f, "conflict: {s}"),
            Self::NotFound => write!(f, "not_found"),
            Self::AlreadyTerminal(s) => write!(f, "already_terminal:{s:?}"),
            Self::Io(s) => write!(f, "io: {s}"),
        }
    }
}

impl TaskStore {
    pub fn memory() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            persist_dir: None,
        }
    }

    pub fn with_persist(dir: impl AsRef<Path>) -> Result<Self, StoreError> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir).map_err(|e| StoreError::Io(e.to_string()))?;
        let store = Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            persist_dir: Some(dir.clone()),
        };
        store.load()?;
        Ok(store)
    }

    fn now() -> String {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("{ms}")
    }

    pub fn payload_hash(payload: &Value) -> String {
        let bytes = serde_json::to_vec(payload).unwrap_or_default();
        format!("{:x}", Sha256::digest(&bytes))
    }

    /// Admit or return existing task for the same idempotency key.
    /// Same key + different payload hash → Conflict (409).
    pub fn admit(
        &self,
        task_id: impl Into<String>,
        run_id: impl Into<String>,
        workspace_id: impl Into<String>,
        session_id: Option<String>,
        idempotency_key: impl Into<String>,
        payload: &Value,
    ) -> Result<TaskRecord, StoreError> {
        let task_id = task_id.into();
        let idempotency_key = idempotency_key.into();
        let payload_hash = Self::payload_hash(payload);
        let mut g = self.inner.lock().unwrap();
        if let Some(existing_id) = g.idem.get(&idempotency_key).cloned() {
            let existing = g.tasks.get(&existing_id).cloned().ok_or(StoreError::NotFound)?;
            if existing.payload_hash != payload_hash {
                return Err(StoreError::Conflict(
                    "idempotency key reuse with different payload".into(),
                ));
            }
            return Ok(existing);
        }
        let now = Self::now();
        let rec = TaskRecord {
            task_id: task_id.clone(),
            run_id: run_id.into(),
            attempt_id: format!("attempt_{}", task_id),
            workspace_id: workspace_id.into(),
            session_id,
            state: TaskState::Accepted,
            revision: 1,
            idempotency_key: idempotency_key.clone(),
            payload_hash,
            lease_id: None,
            lease_epoch: None,
            created_at: now.clone(),
            updated_at: now,
            cancel_requested_at: None,
            result: None,
            error: None,
        };
        g.idem.insert(idempotency_key, task_id.clone());
        g.tasks.insert(task_id, rec.clone());
        g.events.insert(rec.task_id.clone(), Vec::new());
        drop(g);
        self.persist_task(&rec)?;
        Ok(rec)
    }

    pub fn get(&self, task_id: &str) -> Result<TaskRecord, StoreError> {
        self.inner
            .lock()
            .unwrap()
            .tasks
            .get(task_id)
            .cloned()
            .ok_or(StoreError::NotFound)
    }

    /// CAS state transition. `expected_revision` must match.
    pub fn cas_state(
        &self,
        task_id: &str,
        expected_revision: u64,
        next: TaskState,
        patch: impl FnOnce(&mut TaskRecord),
    ) -> Result<TaskRecord, StoreError> {
        let mut g = self.inner.lock().unwrap();
        let rec = g.tasks.get_mut(task_id).ok_or(StoreError::NotFound)?;
        if rec.revision != expected_revision {
            return Err(StoreError::Conflict(format!(
                "revision {} != {}",
                rec.revision, expected_revision
            )));
        }
        if rec.state.is_terminal() && next != rec.state {
            return Err(StoreError::AlreadyTerminal(rec.state));
        }
        rec.state = next;
        rec.revision += 1;
        rec.updated_at = Self::now();
        patch(rec);
        let out = rec.clone();
        drop(g);
        self.persist_task(&out)?;
        Ok(out)
    }

    pub fn request_cancel(&self, task_id: &str) -> Result<TaskRecord, StoreError> {
        let mut g = self.inner.lock().unwrap();
        let rec = g.tasks.get_mut(task_id).ok_or(StoreError::NotFound)?;
        if rec.state.is_terminal() {
            return Err(StoreError::AlreadyTerminal(rec.state));
        }
        let rev = rec.revision;
        rec.state = TaskState::CancelRequested;
        rec.revision = rev + 1;
        rec.cancel_requested_at = Some(Self::now());
        rec.updated_at = Self::now();
        let out = rec.clone();
        drop(g);
        self.persist_task(&out)?;
        Ok(out)
    }

    pub fn append_event(
        &self,
        task_id: &str,
        run_id: &str,
        attempt_id: &str,
        event_type: &str,
        payload: Value,
    ) -> Result<RuntimeEventV2, StoreError> {
        let mut g = self.inner.lock().unwrap();
        if !g.tasks.contains_key(task_id) {
            return Err(StoreError::NotFound);
        }
        let list = g.events.entry(task_id.to_string()).or_default();
        let seq = list.last().map(|e| e.seq + 1).unwrap_or(1);
        let ev = RuntimeEventV2::new(task_id, run_id, attempt_id, seq, event_type, payload);
        list.push(ev.clone());
        drop(g);
        self.persist_event(&ev)?;
        Ok(ev)
    }

    /// Events with seq > after_seq (Last-Event-ID resume).
    pub fn events_after(
        &self,
        task_id: &str,
        after_seq: u64,
    ) -> Result<Vec<RuntimeEventV2>, StoreError> {
        let g = self.inner.lock().unwrap();
        let list = g.events.get(task_id).ok_or(StoreError::NotFound)?;
        Ok(list.iter().filter(|e| e.seq > after_seq).cloned().collect())
    }

    pub fn next_lease_epoch(&self) -> u64 {
        let mut g = self.inner.lock().unwrap();
        g.next_epoch += 1;
        g.next_epoch
    }

    fn persist_task(&self, rec: &TaskRecord) -> Result<(), StoreError> {
        let Some(dir) = &self.persist_dir else {
            return Ok(());
        };
        let path = dir.join("tasks").join(format!("{}.json", rec.task_id));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| StoreError::Io(e.to_string()))?;
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_vec_pretty(rec).map_err(|e| StoreError::Io(e.to_string()))?;
        std::fs::write(&tmp, body).map_err(|e| StoreError::Io(e.to_string()))?;
        std::fs::rename(&tmp, &path).map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    fn persist_event(&self, ev: &RuntimeEventV2) -> Result<(), StoreError> {
        let Some(dir) = &self.persist_dir else {
            return Ok(());
        };
        let path = dir.join("events").join(format!("{}.ndjson", ev.task_id));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| StoreError::Io(e.to_string()))?;
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let line = serde_json::to_string(ev).map_err(|e| StoreError::Io(e.to_string()))?;
        writeln!(f, "{line}").map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    fn load(&self) -> Result<(), StoreError> {
        let Some(dir) = &self.persist_dir else {
            return Ok(());
        };
        let tasks_dir = dir.join("tasks");
        if !tasks_dir.is_dir() {
            return Ok(());
        }
        let mut g = self.inner.lock().unwrap();
        for ent in std::fs::read_dir(&tasks_dir).map_err(|e| StoreError::Io(e.to_string()))? {
            let ent = ent.map_err(|e| StoreError::Io(e.to_string()))?;
            if ent.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let body =
                std::fs::read_to_string(ent.path()).map_err(|e| StoreError::Io(e.to_string()))?;
            let rec: TaskRecord =
                serde_json::from_str(&body).map_err(|e| StoreError::Io(e.to_string()))?;
            g.idem
                .insert(rec.idempotency_key.clone(), rec.task_id.clone());
            g.events.entry(rec.task_id.clone()).or_default();
            g.tasks.insert(rec.task_id.clone(), rec);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn idempotent_admit_and_conflict() {
        let store = TaskStore::memory();
        let p = json!({"a": 1});
        let a = store
            .admit("t1", "r1", "ws", None, "key-1", &p)
            .unwrap();
        let b = store
            .admit("t2", "r2", "ws", None, "key-1", &p)
            .unwrap();
        assert_eq!(a.task_id, b.task_id);
        let err = store
            .admit("t3", "r3", "ws", None, "key-1", &json!({"a": 2}))
            .unwrap_err();
        assert!(matches!(err, StoreError::Conflict(_)));
    }

    #[test]
    fn cancel_cas_terminal() {
        let store = TaskStore::memory();
        let t = store
            .admit("t1", "r1", "ws", None, "k", &json!({}))
            .unwrap();
        store
            .cas_state(&t.task_id, t.revision, TaskState::Succeeded, |r| {
                r.result = Some(json!("ok"));
            })
            .unwrap();
        let err = store.request_cancel(&t.task_id).unwrap_err();
        assert!(matches!(err, StoreError::AlreadyTerminal(TaskState::Succeeded)));
    }

    #[test]
    fn event_seq_monotonic_and_resume() {
        let store = TaskStore::memory();
        let t = store
            .admit("t1", "r1", "ws", None, "k", &json!({}))
            .unwrap();
        let e1 = store
            .append_event(&t.task_id, &t.run_id, &t.attempt_id, "message.delta", json!({"n":1}))
            .unwrap();
        let e2 = store
            .append_event(&t.task_id, &t.run_id, &t.attempt_id, "message.delta", json!({"n":2}))
            .unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        let after = store.events_after(&t.task_id, 1).unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].seq, 2);
    }
}

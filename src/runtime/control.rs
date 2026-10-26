//! Task control plane: cancel without interpreter lock (M-R3).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::runtime::protocol::RuntimeEventV2;
use crate::runtime::scheduler::{AdmissionClass, ScheduleError, Scheduler};
use crate::runtime::task_store::{StoreError, TaskRecord, TaskState, TaskStore};

#[derive(Debug)]
pub enum ControlError {
    Store(StoreError),
    Schedule(ScheduleError),
    Forbidden,
}

impl From<StoreError> for ControlError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

impl From<ScheduleError> for ControlError {
    fn from(e: ScheduleError) -> Self {
        Self::Schedule(e)
    }
}

impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => write!(f, "{e}"),
            Self::Schedule(e) => write!(f, "{e}"),
            Self::Forbidden => write!(f, "forbidden"),
        }
    }
}

pub struct ControlService {
    pub store: Arc<TaskStore>,
    pub scheduler: Arc<Scheduler>,
    /// Per-lease cancel token directory (token files, not process-global env).
    cancel_dir: PathBuf,
}

impl ControlService {
    pub fn new(store: Arc<TaskStore>, scheduler: Arc<Scheduler>, cancel_dir: impl AsRef<Path>) -> Self {
        let cancel_dir = cancel_dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&cancel_dir);
        Self {
            store,
            scheduler,
            cancel_dir,
        }
    }

    pub fn admit(
        &self,
        task_id: &str,
        run_id: &str,
        workspace_id: &str,
        session_id: Option<String>,
        idempotency_key: &str,
        payload: &Value,
        class: AdmissionClass,
    ) -> Result<(TaskRecord, usize), ControlError> {
        let rec = self.store.admit(
            task_id,
            run_id,
            workspace_id,
            session_id,
            idempotency_key,
            payload,
        )?;
        let depth = self.scheduler.enqueue(&rec, class)?;
        let queued = self.store.cas_state(
            &rec.task_id,
            rec.revision,
            TaskState::Queued,
            |_| {},
        )?;
        Ok((queued, depth))
    }

    /// Cancel path never enters the Marqdo interpreter lock.
    pub fn cancel(&self, task_id: &str) -> Result<TaskRecord, ControlError> {
        let rec = self.store.request_cancel(task_id)?;
        let tok = self.cancel_token_path(task_id);
        let _ = std::fs::write(&tok, b"cancelled");
        let _ = self.store.append_event(
            task_id,
            &rec.run_id,
            &rec.attempt_id,
            "run.cancel_requested",
            serde_json::json!({"revision": rec.revision}),
        );
        Ok(rec)
    }

    pub fn cancel_token_path(&self, task_id: &str) -> PathBuf {
        self.cancel_dir.join(format!("{task_id}.tok"))
    }

    pub fn mark_cancelled(&self, task_id: &str, expected_revision: u64) -> Result<TaskRecord, ControlError> {
        Ok(self.store.cas_state(
            task_id,
            expected_revision,
            TaskState::Cancelled,
            |_| {},
        )?)
    }

    pub fn complete(
        &self,
        task_id: &str,
        expected_revision: u64,
        result: Value,
    ) -> Result<TaskRecord, ControlError> {
        match self.store.cas_state(task_id, expected_revision, TaskState::Succeeded, |r| {
            r.result = Some(result.clone());
        }) {
            Ok(rec) => Ok(rec),
            Err(StoreError::AlreadyTerminal(state)) => {
                // Late result after cancel: append evidence only.
                let rec = self.store.get(task_id)?;
                let _ = self.store.append_event(
                    task_id,
                    &rec.run_id,
                    &rec.attempt_id,
                    "run.late_result",
                    serde_json::json!({"ignored_terminal": format!("{state:?}"), "result": result}),
                );
                Err(ControlError::Store(StoreError::AlreadyTerminal(state)))
            }
            Err(e) => Err(ControlError::Store(e)),
        }
    }

    pub fn events_since(
        &self,
        task_id: &str,
        after_seq: u64,
    ) -> Result<Vec<RuntimeEventV2>, ControlError> {
        Ok(self.store.events_after(task_id, after_seq)?)
    }

    /// Collaboration grace + force-kill budget from the design (§6).
    pub fn cancel_deadlines() -> (Duration, Duration) {
        (Duration::from_secs(2), Duration::from_secs(3))
    }
}

/// Convenience wrapper used by HTTP/control adapters.
pub fn cancel_task(svc: &ControlService, task_id: &str) -> Result<TaskRecord, ControlError> {
    svc.cancel(task_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::scheduler::SchedulerConfig;
    use serde_json::json;

    #[test]
    fn cancel_writes_token_not_interpreter() {
        let dir = std::env::temp_dir().join(format!("mq-ctrl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Arc::new(TaskStore::memory());
        let sched = Arc::new(Scheduler::new(SchedulerConfig::default()));
        let ctl = ControlService::new(store, sched, &dir);
        let (t, _) = ctl
            .admit("t1", "r1", "ws", None, "k", &json!({}), AdmissionClass::Execute)
            .unwrap();
        let c = ctl.cancel(&t.task_id).unwrap();
        assert_eq!(c.state, TaskState::CancelRequested);
        let body = std::fs::read_to_string(ctl.cancel_token_path(&t.task_id)).unwrap();
        assert_eq!(body, "cancelled");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn late_complete_after_cancel_is_evidence_only() {
        let dir = std::env::temp_dir().join(format!("mq-ctrl2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Arc::new(TaskStore::memory());
        let sched = Arc::new(Scheduler::new(SchedulerConfig::default()));
        let ctl = ControlService::new(Arc::clone(&store), sched, &dir);
        let (t, _) = ctl
            .admit("t1", "r1", "ws", None, "k", &json!({}), AdmissionClass::Execute)
            .unwrap();
        let c = ctl.cancel(&t.task_id).unwrap();
        ctl.mark_cancelled(&t.task_id, c.revision).unwrap();
        let err = ctl.complete(&t.task_id, c.revision + 1, json!("late")).unwrap_err();
        assert!(matches!(err, ControlError::Store(StoreError::AlreadyTerminal(_))));
        let evs = store.events_after(&t.task_id, 0).unwrap();
        assert!(evs.iter().any(|e| e.event_type == "run.late_result"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

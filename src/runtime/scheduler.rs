//! Bounded admission classes and fair queueing (M-R3).

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::runtime::task_store::TaskRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionClass {
    Control,
    Read,
    Short,
    Execute,
    Commit,
    /// Undeclared endpoints stay on the legacy single-interpreter path.
    LegacySerial,
}

impl AdmissionClass {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "control" => Self::Control,
            "read" => Self::Read,
            "short" => Self::Short,
            "execute" => Self::Execute,
            "commit" => Self::Commit,
            _ => Self::LegacySerial,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub execute_slots: usize,
    pub short_slots: usize,
    pub read_slots: usize,
    pub global_queue_limit: usize,
    pub workspace_queue_limit: usize,
    pub session_pending_limit: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        // 4 vCPU / 8 GiB verification machine defaults from the design.
        Self {
            execute_slots: 2,
            short_slots: 1,
            read_slots: 1,
            global_queue_limit: 64,
            workspace_queue_limit: 16,
            session_pending_limit: 4,
        }
    }
}

#[derive(Debug)]
pub enum ScheduleError {
    QueueFull,
    WorkspaceFull,
    SessionFull,
    NoSlot,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull => write!(f, "queue_full"),
            Self::WorkspaceFull => write!(f, "workspace_full"),
            Self::SessionFull => write!(f, "session_full"),
            Self::NoSlot => write!(f, "no_slot"),
        }
    }
}

struct QueueState {
    global: VecDeque<String>,
    by_workspace: HashMap<String, VecDeque<String>>,
    by_session: HashMap<String, usize>,
    active_execute: usize,
    active_short: usize,
    active_read: usize,
    /// workspace rotation cursor
    workspace_order: VecDeque<String>,
}

pub struct Scheduler {
    cfg: SchedulerConfig,
    inner: Mutex<QueueState>,
}

impl Scheduler {
    pub fn new(cfg: SchedulerConfig) -> Self {
        Self {
            cfg,
            inner: Mutex::new(QueueState {
                global: VecDeque::new(),
                by_workspace: HashMap::new(),
                by_session: HashMap::new(),
                active_execute: 0,
                active_short: 0,
                active_read: 0,
                workspace_order: VecDeque::new(),
            }),
        }
    }

    pub fn enqueue(&self, task: &TaskRecord, class: AdmissionClass) -> Result<usize, ScheduleError> {
        if class == AdmissionClass::Control || class == AdmissionClass::LegacySerial {
            return Ok(0);
        }
        let mut g = self.inner.lock().unwrap();
        if g.global.len() >= self.cfg.global_queue_limit {
            return Err(ScheduleError::QueueFull);
        }
        let ws_len = g
            .by_workspace
            .get(&task.workspace_id)
            .map(|q| q.len())
            .unwrap_or(0);
        if ws_len >= self.cfg.workspace_queue_limit {
            return Err(ScheduleError::WorkspaceFull);
        }
        if let Some(sid) = &task.session_id {
            let n = g.by_session.entry(sid.clone()).or_insert(0);
            if *n >= self.cfg.session_pending_limit {
                return Err(ScheduleError::SessionFull);
            }
            *n += 1;
        }
        if !g.workspace_order.iter().any(|w| w == &task.workspace_id) {
            g.workspace_order.push_back(task.workspace_id.clone());
        }
        g.by_workspace
            .entry(task.workspace_id.clone())
            .or_default()
            .push_back(task.task_id.clone());
        g.global.push_back(task.task_id.clone());
        Ok(g.global.len())
    }

    pub fn try_acquire_slot(&self, class: AdmissionClass) -> Result<(), ScheduleError> {
        let mut g = self.inner.lock().unwrap();
        match class {
            AdmissionClass::Execute => {
                if g.active_execute >= self.cfg.execute_slots {
                    return Err(ScheduleError::NoSlot);
                }
                g.active_execute += 1;
            }
            AdmissionClass::Short | AdmissionClass::Commit => {
                if g.active_short >= self.cfg.short_slots {
                    return Err(ScheduleError::NoSlot);
                }
                g.active_short += 1;
            }
            AdmissionClass::Read => {
                if g.active_read >= self.cfg.read_slots {
                    return Err(ScheduleError::NoSlot);
                }
                g.active_read += 1;
            }
            AdmissionClass::Control | AdmissionClass::LegacySerial => {}
        }
        Ok(())
    }

    pub fn release_slot(&self, class: AdmissionClass) {
        let mut g = self.inner.lock().unwrap();
        match class {
            AdmissionClass::Execute => {
                g.active_execute = g.active_execute.saturating_sub(1);
            }
            AdmissionClass::Short | AdmissionClass::Commit => {
                g.active_short = g.active_short.saturating_sub(1);
            }
            AdmissionClass::Read => {
                g.active_read = g.active_read.saturating_sub(1);
            }
            AdmissionClass::Control | AdmissionClass::LegacySerial => {}
        }
    }

    /// Fair pop: rotate workspaces, then session FIFO within workspace.
    pub fn pop_next(&self) -> Option<String> {
        let mut g = self.inner.lock().unwrap();
        if g.workspace_order.is_empty() {
            return None;
        }
        for _ in 0..g.workspace_order.len() {
            let ws = g.workspace_order.pop_front()?;
            if let Some(q) = g.by_workspace.get_mut(&ws) {
                if let Some(tid) = q.pop_front() {
                    g.workspace_order.push_back(ws);
                    g.global.retain(|id| id != &tid);
                    return Some(tid);
                }
            }
        }
        None
    }

    pub fn queue_depth(&self) -> usize {
        self.inner.lock().unwrap().global.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::task_store::{TaskRecord, TaskState};

    fn task(id: &str, ws: &str, session: Option<&str>) -> TaskRecord {
        TaskRecord {
            task_id: id.into(),
            run_id: format!("r-{id}"),
            attempt_id: format!("a-{id}"),
            workspace_id: ws.into(),
            session_id: session.map(|s| s.into()),
            state: TaskState::Queued,
            revision: 1,
            idempotency_key: id.into(),
            payload_hash: "h".into(),
            lease_id: None,
            lease_epoch: None,
            created_at: "0".into(),
            updated_at: "0".into(),
            cancel_requested_at: None,
            result: None,
            error: None,
        }
    }

    #[test]
    fn workspace_rotation_fairness() {
        let sched = Scheduler::new(SchedulerConfig::default());
        sched
            .enqueue(&task("a1", "wsA", None), AdmissionClass::Execute)
            .unwrap();
        sched
            .enqueue(&task("a2", "wsA", None), AdmissionClass::Execute)
            .unwrap();
        sched
            .enqueue(&task("b1", "wsB", None), AdmissionClass::Execute)
            .unwrap();
        assert_eq!(sched.pop_next().as_deref(), Some("a1"));
        assert_eq!(sched.pop_next().as_deref(), Some("b1"));
        assert_eq!(sched.pop_next().as_deref(), Some("a2"));
    }
}

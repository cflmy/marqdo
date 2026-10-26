//! Worker lease identity and process lifecycle states (M-R2).

use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LeaseId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    Starting,
    Ready,
    Leased,
    Running,
    Draining,
    Unhealthy,
    Terminating,
    Exited,
}

#[derive(Debug, Clone)]
pub struct Lease {
    pub id: LeaseId,
    pub epoch: u64,
    pub task_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub deadline: SystemTime,
    pub state: WorkerState,
}

impl Lease {
    pub fn new(
        id: impl Into<String>,
        epoch: u64,
        task_id: impl Into<String>,
        run_id: impl Into<String>,
        attempt_id: impl Into<String>,
        ttl: Duration,
    ) -> Self {
        Self {
            id: LeaseId(id.into()),
            epoch,
            task_id: task_id.into(),
            run_id: run_id.into(),
            attempt_id: attempt_id.into(),
            deadline: SystemTime::now() + ttl,
            state: WorkerState::Leased,
        }
    }

    pub fn expired(&self) -> bool {
        SystemTime::now() > self.deadline
    }

    pub fn matches(&self, lease_id: &str, epoch: u64) -> bool {
        self.id.0 == lease_id && self.epoch == epoch
    }
}

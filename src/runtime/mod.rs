//! Isolated web runtime (M-R2–M-R4): supervisor, worker IPC, leases, task
//! admission/control, and persistent event log.
//!
//! Design: `doc/design/isolated-web-runtime-development-2026-10-26.md`.
//! Until capability probes report version ≥1, callers must treat this as opt-in.

pub mod control;
pub mod host_api;
pub mod lease;
pub mod protocol;
pub mod publish;
pub mod scheduler;
pub mod supervisor;
pub mod task_store;
pub mod worker;

pub use control::{cancel_task, ControlError, ControlService};
pub use lease::{Lease, LeaseId, WorkerState};
pub use protocol::{
    InvokeRequestV2, RuntimeEventV2, CONTROL_FRAME_MAX_BYTES, PROTOCOL_VERSION,
};
pub use publish::{PublishJournal, PublishOutcome, PublishPhase};
pub use scheduler::{AdmissionClass, Scheduler, SchedulerConfig};
pub use supervisor::{Supervisor, SupervisorConfig};
pub use task_store::{TaskRecord, TaskState, TaskStore};
pub use worker::{WorkerHandle, WorkerPool};

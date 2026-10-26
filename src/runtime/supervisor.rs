//! Supervisor: lease workers, enforce deadlines, reap orphans (M-R2).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::runtime::lease::{Lease, WorkerState};
use crate::runtime::protocol::InvokeRequestV2;
use crate::runtime::worker::{WorkerHandle, WorkerPool};

#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub execute_slots: usize,
    pub worker_ttl: Duration,
    pub cancel_grace: Duration,
    pub force_kill_after: Duration,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            execute_slots: 2,
            worker_ttl: Duration::from_secs(600),
            cancel_grace: Duration::from_secs(2),
            force_kill_after: Duration::from_secs(3),
        }
    }
}

struct ActiveWorker {
    handle: WorkerHandle,
    cancel_since: Option<Instant>,
}

pub struct Supervisor {
    cfg: SupervisorConfig,
    pool: Mutex<WorkerPool>,
    active: Mutex<HashMap<String, ActiveWorker>>,
    cancel_dir: PathBuf,
    event_dir: PathBuf,
}

impl Supervisor {
    pub fn new(
        cfg: SupervisorConfig,
        staging_root: impl AsRef<Path>,
        marqdo_bin: impl AsRef<Path>,
        cancel_dir: impl AsRef<Path>,
        event_dir: impl AsRef<Path>,
    ) -> Self {
        let cancel_dir = cancel_dir.as_ref().to_path_buf();
        let event_dir = event_dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&cancel_dir);
        let _ = std::fs::create_dir_all(&event_dir);
        Self {
            pool: Mutex::new(WorkerPool::new(
                staging_root,
                marqdo_bin,
                cfg.execute_slots,
            )),
            cfg,
            active: Mutex::new(HashMap::new()),
            cancel_dir,
            event_dir,
        }
    }

    pub fn warm(&self) -> Result<(), String> {
        self.pool.lock().unwrap().warm()
    }

    pub fn dispatch(&self, req: InvokeRequestV2) -> Result<String, String> {
        req.validate()?;
        {
            let active = self.active.lock().unwrap();
            if active.len() >= self.cfg.execute_slots {
                return Err("supervisor.no_slot".into());
            }
        }
        let mut pool = self.pool.lock().unwrap();
        let worker = pool.acquire()?;
        drop(pool);
        let epoch = Instant::now().elapsed().as_nanos() as u64;
        let lease = Lease::new(
            format!("lease_{}", req.task_id),
            epoch,
            &req.task_id,
            &req.run_id,
            &req.attempt_id,
            self.cfg.worker_ttl,
        );
        let cancel = self.cancel_dir.join(format!("{}.tok", req.task_id));
        if !cancel.exists() {
            std::fs::write(&cancel, b"").map_err(|e| e.to_string())?;
        }
        let events = self.event_dir.join(format!("{}.ndjson", req.task_id));
        std::fs::write(&events, b"").map_err(|e| e.to_string())?;
        // Durable emit path (not under staging — WorkerHandle::Drop removes staging).
        let emit = self.event_dir.join(format!("{}.result.json", req.task_id));
        let fs_root = req
            .context
            .fs_root
            .as_ref()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .ok_or_else(|| "supervisor.fs_root_required".to_string())?;
        let args_path = req.args_ref.as_ref().and_then(|r| {
            r.strip_prefix("file:")
                .map(PathBuf::from)
                .or_else(|| {
                    let p = PathBuf::from(r);
                    p.is_file().then_some(p)
                })
        });
        let handle = {
            if std::env::var_os("MARQDO_RUNTIME_DRY_RUN").is_some() {
                let mut w = worker;
                w.lease = Some(lease.clone());
                w.emit_result = Some(emit.clone());
                w.state = WorkerState::Running;
                w
            } else {
                self.pool.lock().unwrap().lease_and_start(
                    worker,
                    lease.clone(),
                    &req,
                    &cancel,
                    &events,
                    &fs_root,
                    args_path.as_deref(),
                    &emit,
                )?
            }
        };
        let lease_id = lease.id.0.clone();
        self.active.lock().unwrap().insert(
            lease_id.clone(),
            ActiveWorker {
                handle,
                cancel_since: None,
            },
        );
        Ok(lease_id)
    }

    pub fn event_path(&self, task_id: &str) -> PathBuf {
        self.event_dir.join(format!("{task_id}.ndjson"))
    }

    pub fn cancel_path(&self, task_id: &str) -> PathBuf {
        self.cancel_dir.join(format!("{task_id}.tok"))
    }

    pub fn emit_result_path(&self, lease_id: &str) -> Option<PathBuf> {
        self.active
            .lock()
            .unwrap()
            .get(lease_id)
            .and_then(|w| w.handle.emit_result.clone())
    }

    /// Non-blocking poll: if worker exited, remove from active and return exit code.
    pub fn try_reap(&self, lease_id: &str) -> Result<Option<i32>, String> {
        let mut active = self.active.lock().unwrap();
        let Some(w) = active.get_mut(lease_id) else {
            return Ok(None);
        };
        match w.handle.try_wait()? {
            Some(st) => {
                let code = st.code().unwrap_or(-1);
                active.remove(lease_id);
                let _ = self.pool.lock().unwrap().warm();
                Ok(Some(code))
            }
            None => Ok(None),
        }
    }

    pub fn signal_cancel(&self, lease_id: &str, task_id: &str) -> Result<(), String> {
        let tok = self.cancel_dir.join(format!("{task_id}.tok"));
        std::fs::write(&tok, b"cancelled").map_err(|e| e.to_string())?;
        let mut active = self.active.lock().unwrap();
        if let Some(w) = active.get_mut(lease_id) {
            w.cancel_since = Some(Instant::now());
            w.handle.state = WorkerState::Draining;
        }
        Ok(())
    }

    /// Reap expired / force-kill after cancel grace. Returns orphan count.
    pub fn janitor_tick(&self) -> usize {
        let mut orphans = 0usize;
        let mut active = self.active.lock().unwrap();
        let mut done = Vec::new();
        for (id, w) in active.iter_mut() {
            if let Some(lease) = &w.handle.lease {
                if lease.expired() {
                    w.handle.kill_process_group();
                    orphans += 1;
                    done.push(id.clone());
                    continue;
                }
            }
            if let Some(since) = w.cancel_since {
                if since.elapsed() > self.cfg.cancel_grace + self.cfg.force_kill_after {
                    w.handle.kill_process_group();
                    orphans += 1;
                    done.push(id.clone());
                }
            }
        }
        for id in done {
            active.remove(&id);
        }
        orphans
    }

    pub fn active_count(&self) -> usize {
        self.active.lock().unwrap().len()
    }

    pub fn release(&self, lease_id: &str) {
        let mut active = self.active.lock().unwrap();
        if let Some(mut w) = active.remove(lease_id) {
            w.handle.state = WorkerState::Draining;
            w.handle.kill_process_group();
        }
        // Async replenish hint.
        let _ = self.pool.lock().unwrap().warm();
    }
}

/// Shared supervisor handle for HTTP/control wiring.
pub type SharedSupervisor = Arc<Supervisor>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::protocol::{EntryRef, InvokeContext, LeaseRef};

    fn req(task: &str) -> InvokeRequestV2 {
        InvokeRequestV2 {
            schema: "InvokeRequestV2".into(),
            schema_version: 2,
            task_id: task.into(),
            run_id: format!("run_{task}"),
            attempt_id: format!("attempt_{task}"),
            lease: LeaseRef {
                id: format!("lease_{task}"),
                epoch: 1,
            },
            entry: EntryRef {
                module: "agent/qdagent.mq.md".into(),
                function: "run".into(),
            },
            args_ref: None,
            context: InvokeContext {
                workspace_id: "ws".into(),
                snapshot_id: None,
                selection_ref: None,
                capabilities_ref: None,
                deadline_at: "2099-01-01T00:00:00Z".into(),
                fs_root: Some("/tmp".into()),
            },
        }
    }

    #[test]
    fn dry_run_two_parallel_leases() {
        std::env::set_var("MARQDO_RUNTIME_DRY_RUN", "1");
        let root = std::env::temp_dir().join(format!("mq-sup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let bin = std::env::current_exe().unwrap();
        let _ = std::fs::create_dir_all(root.join("fs"));
        let mut ra = req("a");
        ra.context.fs_root = Some(root.join("fs").to_string_lossy().into());
        let mut rb = req("b");
        rb.context.fs_root = ra.context.fs_root.clone();
        let mut rc = req("c");
        rc.context.fs_root = ra.context.fs_root.clone();
        let sup = Supervisor::new(
            SupervisorConfig {
                execute_slots: 2,
                ..SupervisorConfig::default()
            },
            root.join("stage"),
            &bin,
            root.join("cancel"),
            root.join("events"),
        );
        sup.warm().unwrap();
        let a = sup.dispatch(ra).unwrap();
        let b = sup.dispatch(rb).unwrap();
        assert_ne!(a, b);
        assert_eq!(sup.active_count(), 2);
        assert!(sup.dispatch(rc).is_err());
        sup.release(&a);
        assert_eq!(sup.active_count(), 1);
        let _ = std::fs::remove_dir_all(&root);
        std::env::remove_var("MARQDO_RUNTIME_DRY_RUN");
    }
}

//! One-shot Marqdo worker process handles (M-R2).

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};

use crate::runtime::lease::{Lease, WorkerState};
use crate::runtime::protocol::{InvokeRequestV2, PROTOCOL_VERSION};

static WORKER_SEQ: AtomicU64 = AtomicU64::new(1);

pub struct WorkerHandle {
    pub id: String,
    pub state: WorkerState,
    pub lease: Option<Lease>,
    child: Option<Child>,
    staging: PathBuf,
    /// Where the child writes `--emit-result` JSON.
    pub emit_result: Option<PathBuf>,
}

impl WorkerHandle {
    pub fn staging(&self) -> &Path {
        &self.staging
    }

    pub fn take_child(&mut self) -> Option<Child> {
        self.child.take()
    }

    pub fn kill_process_group(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.state = WorkerState::Exited;
    }

    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, String> {
        let Some(child) = self.child.as_mut() else {
            return Ok(None);
        };
        child.try_wait().map_err(|e| format!("worker.wait: {e}"))
    }

    pub fn wait(&mut self) -> Result<std::process::ExitStatus, String> {
        let Some(mut child) = self.child.take() else {
            return Err("worker.no_child".into());
        };
        let st = child.wait().map_err(|e| format!("worker.wait: {e}"))?;
        self.state = WorkerState::Exited;
        Ok(st)
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.kill_process_group();
        let _ = std::fs::remove_dir_all(&self.staging);
    }
}

pub struct WorkerPool {
    staging_root: PathBuf,
    marqdo_bin: PathBuf,
    ready: Vec<WorkerHandle>,
    target_ready: usize,
}

impl WorkerPool {
    pub fn new(staging_root: impl AsRef<Path>, marqdo_bin: impl AsRef<Path>, target_ready: usize) -> Self {
        let staging_root = staging_root.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&staging_root);
        Self {
            staging_root,
            marqdo_bin: marqdo_bin.as_ref().to_path_buf(),
            ready: Vec::new(),
            target_ready,
        }
    }

    pub fn spawn_ready(&mut self) -> Result<WorkerHandle, String> {
        let seq = WORKER_SEQ.fetch_add(1, Ordering::Relaxed);
        let id = format!("worker_{seq}");
        let staging = self.staging_root.join(&id);
        std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
        if !self.marqdo_bin.exists() {
            return Err(format!("worker.binary_missing: {}", self.marqdo_bin.display()));
        }
        Ok(WorkerHandle {
            id,
            state: WorkerState::Ready,
            lease: None,
            child: None,
            staging,
            emit_result: None,
        })
    }

    pub fn warm(&mut self) -> Result<(), String> {
        while self.ready.len() < self.target_ready {
            let w = self.spawn_ready()?;
            self.ready.push(w);
        }
        Ok(())
    }

    pub fn acquire(&mut self) -> Result<WorkerHandle, String> {
        if self.ready.is_empty() {
            self.warm()?;
        }
        self.ready
            .pop()
            .ok_or_else(|| "worker.pool_empty".to_string())
    }

    pub fn replenish_async_hint(&self) -> usize {
        self.target_ready.saturating_sub(self.ready.len())
    }

    /// Lease a worker: spawn `marqdo run --call-lib-path …` with request-scoped env.
    /// `emit_result` must be outside staging so Drop does not erase the parent's read path.
    pub fn lease_and_start(
        &self,
        mut worker: WorkerHandle,
        lease: Lease,
        req: &InvokeRequestV2,
        cancel_token: &Path,
        event_sink: &Path,
        fs_root: &Path,
        args_json: Option<&Path>,
        emit_result: &Path,
    ) -> Result<WorkerHandle, String> {
        req.validate()?;
        let entry = PathBuf::from(&req.entry.module);
        if !entry.is_file() {
            return Err(format!(
                "worker.entry_missing: {} (need absolute site entry .mq.md)",
                entry.display()
            ));
        }
        if let Some(parent) = emit_result.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let emit = emit_result.to_path_buf();
        let mut cmd = Command::new(&self.marqdo_bin);
        cmd.arg("run")
            .arg(&entry)
            .arg("--call-lib-path")
            .arg(&req.entry.function)
            .arg("--fs-root")
            .arg(fs_root)
            .arg("--emit-result")
            .arg(&emit)
            // Worker cwd = site root so relative imports resolve; staging only for lease bookkeeping.
            .current_dir(fs_root)
            .env("MARQDO_RUN_ID", &req.run_id)
            .env("MARQDO_TASK_ID", &req.task_id)
            .env("MARQDO_ATTEMPT_ID", &req.attempt_id)
            .env("MARQDO_LEASE_ID", &lease.id.0)
            .env("MARQDO_LEASE_EPOCH", lease.epoch.to_string())
            .env("MARQDO_CANCEL_TOKEN", cancel_token)
            .env("MARQDO_EVENT_FORWARD", event_sink)
            .env("MARQDO_WORKSPACE_ID", &req.context.workspace_id)
            .env("MARQDO_PROTOCOL_VERSION", PROTOCOL_VERSION.to_string())
            .env("MARQDO_FS_ROOT", fs_root)
            .stdin(Stdio::null())
            // Avoid pipe-buffer deadlock: nobody drains worker stdout/stderr.
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(args) = args_json {
            cmd.arg("--args-json").arg(args);
        } else if let Some(r) = &req.args_ref {
            if let Some(path) = r.strip_prefix("file:") {
                cmd.arg("--args-json").arg(path);
            }
        }
        if let Some(sel) = &req.context.selection_ref {
            cmd.env("MARQDO_SELECTION_REF", sel);
        }
        if let Some(snap) = &req.context.snapshot_id {
            cmd.env("MARQDO_SNAPSHOT_ID", snap);
        }
        if let Some(cap) = &req.context.capabilities_ref {
            cmd.env("MARQDO_CAPABILITIES_REF", cap);
        }
        let child = cmd.spawn().map_err(|e| format!("worker.spawn: {e}"))?;
        worker.child = Some(child);
        worker.lease = Some(lease);
        worker.emit_result = Some(emit);
        worker.state = WorkerState::Running;
        Ok(worker)
    }
}

pub fn handshake_ok(report: &Value) -> Result<(), String> {
    let ver = report
        .get("protocol_version")
        .and_then(|v| v.as_u64())
        .ok_or("handshake.protocol_version")?;
    if ver != PROTOCOL_VERSION as u64 {
        return Err(format!("handshake.version_mismatch: {ver}"));
    }
    Ok(())
}

pub fn fake_handshake() -> Value {
    json!({
        "cli": true,
        "protocol_version": PROTOCOL_VERSION,
        "capabilities": ["runtime.isolated_execution"],
        "build": env!("CARGO_PKG_VERSION"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_version_gate() {
        assert!(handshake_ok(&fake_handshake()).is_ok());
        assert!(handshake_ok(&json!({"protocol_version": 1})).is_err());
    }

    #[test]
    fn pool_warm_requires_binary() {
        let dir = std::env::temp_dir().join(format!("mq-pool-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut pool = WorkerPool::new(&dir, "/no/such/marqdo-bin", 1);
        assert!(pool.warm().is_err());
        let self_bin = std::env::current_exe().unwrap();
        let mut pool = WorkerPool::new(&dir, &self_bin, 1);
        pool.warm().unwrap();
        assert_eq!(pool.ready.len(), 1);
        let w = pool.acquire().unwrap();
        assert_eq!(w.state, WorkerState::Ready);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn kill_clears_child() {
        let dir = std::env::temp_dir().join(format!("mq-wh-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut w = WorkerHandle {
            id: "w".into(),
            state: WorkerState::Running,
            lease: None,
            child: None,
            staging: dir.clone(),
            emit_result: None,
        };
        w.kill_process_group();
        assert_eq!(w.state, WorkerState::Exited);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

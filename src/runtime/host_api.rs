//! Process-global control plane via `host_query("runtime", …)` — no INVOKE_LOCK.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

use crate::runtime::control::ControlService;
use crate::runtime::protocol::{
    EntryRef, InvokeContext, InvokeRequestV2, LeaseRef, PROTOCOL_VERSION,
};
use crate::runtime::scheduler::{AdmissionClass, Scheduler, SchedulerConfig};
use crate::runtime::supervisor::{Supervisor, SupervisorConfig};
use crate::runtime::task_store::{TaskState, TaskStore};

struct RuntimeHost {
    control: ControlService,
    supervisor: Arc<Supervisor>,
    /// task_id → lease_id
    leases: Mutex<std::collections::HashMap<String, String>>,
    base: PathBuf,
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("MARQDO_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("marqdo-runtime"))
}

fn marqdo_bin() -> PathBuf {
    std::env::var_os("MARQDO_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_exe().unwrap_or_else(|_| PathBuf::from("marqdo")))
}

fn ensure_host() -> &'static RuntimeHost {
    static H: OnceLock<RuntimeHost> = OnceLock::new();
    static JANITOR: OnceLock<()> = OnceLock::new();
    let host = H.get_or_init(|| {
        let base = runtime_dir();
        let _ = std::fs::create_dir_all(&base);
        let store = match TaskStore::with_persist(base.join("store")) {
            Ok(s) => Arc::new(s),
            Err(_) => Arc::new(TaskStore::memory()),
        };
        let scheduler = Arc::new(Scheduler::new(SchedulerConfig::default()));
        let control = ControlService::new(
            Arc::clone(&store),
            Arc::clone(&scheduler),
            base.join("cancel"),
        );
        let supervisor = Arc::new(Supervisor::new(
            SupervisorConfig::default(),
            base.join("stage"),
            marqdo_bin(),
            base.join("cancel"),
            base.join("events"),
        ));
        let _ = supervisor.warm();
        RuntimeHost {
            control,
            supervisor,
            leases: Mutex::new(std::collections::HashMap::new()),
            base,
        }
    });
    JANITOR.get_or_init(|| {
        let sup = Arc::clone(&host.supervisor);
        let _ = thread::Builder::new()
            .name("mq-runtime-janitor".into())
            .spawn(move || loop {
                thread::sleep(Duration::from_millis(500));
                let _ = sup.janitor_tick();
            });
    });
    host
}

/// Dispatch `host_query("runtime", {op, ...})`. Never blocks on INVOKE_LOCK.
pub fn handle(args: &Value) -> Result<Value, String> {
    let op = args
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "runtime requires op".to_string())?;
    let host = ensure_host();
    let ctl = &host.control;
    match op {
        "ping" | "capabilities" => Ok(json!({
            "ok": true,
            "protocol_version": PROTOCOL_VERSION,
            "features": {
                "runtime.task_control_v2": 1,
                "runtime.event_replay": 1,
                "runtime.isolated_execution": 1,
            },
            "runtime_dir": host.base.to_string_lossy(),
        })),
        "admit" => {
            let task_id = req_str(args, "task_id")?;
            let run_id = args
                .get("run_id")
                .and_then(|v| v.as_str())
                .unwrap_or(task_id);
            let workspace_id = args
                .get("workspace_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let session_id = args
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let idem = args
                .get("idempotency_key")
                .and_then(|v| v.as_str())
                .unwrap_or(task_id);
            let class = AdmissionClass::parse(
                args.get("class").and_then(|v| v.as_str()).unwrap_or("execute"),
            );
            let payload = args.get("payload").cloned().unwrap_or(json!({}));
            let (rec, depth) = ctl
                .admit(
                    task_id,
                    run_id,
                    workspace_id,
                    session_id,
                    idem,
                    &payload,
                    class,
                )
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "ok": true,
                "task": task_json(&rec),
                "queue_depth": depth,
                "cancel_token": ctl.cancel_token_path(&rec.task_id).to_string_lossy(),
                "event_path": host.supervisor.event_path(&rec.task_id).to_string_lossy(),
            }))
        }
        "dispatch" => dispatch_op(host, args),
        "cancel" => {
            let task_id = req_str(args, "task_id")?;
            match ctl.cancel(task_id) {
                Ok(rec) => {
                    if let Some(lease) = host.leases.lock().unwrap().get(task_id) {
                        let _ = host.supervisor.signal_cancel(lease, task_id);
                    } else {
                        let _ = std::fs::write(ctl.cancel_token_path(task_id), b"cancelled");
                    }
                    Ok(json!({
                        "ok": true,
                        "task": task_json(&rec),
                        "cancel_token": ctl.cancel_token_path(task_id).to_string_lossy(),
                    }))
                }
                Err(crate::runtime::control::ControlError::Store(
                    crate::runtime::task_store::StoreError::NotFound,
                )) => Ok(json!({"ok": false, "error": "not_found", "status": 404})),
                Err(crate::runtime::control::ControlError::Store(
                    crate::runtime::task_store::StoreError::AlreadyTerminal(s),
                )) => Ok(json!({
                    "ok": true,
                    "already_terminal": true,
                    "state": format!("{s:?}").to_ascii_lowercase(),
                })),
                Err(e) => Err(e.to_string()),
            }
        }
        "get" => {
            let task_id = req_str(args, "task_id")?;
            match ctl.store.get(task_id) {
                Ok(rec) => {
                    let mut out = json!({"ok": true, "task": task_json(&rec)});
                    if let Some(lease) = host.leases.lock().unwrap().get(task_id).cloned() {
                        out["lease_id"] = json!(lease);
                        if let Some(p) = host.supervisor.emit_result_path(&lease) {
                            out["emit_result"] = json!(p.to_string_lossy());
                        }
                        if let Ok(Some(code)) = host.supervisor.try_reap(&lease) {
                            out["exit_code"] = json!(code);
                            host.control
                                .scheduler
                                .release_slot(AdmissionClass::Execute);
                            let _ = ctl.store.cas_state(
                                task_id,
                                rec.revision,
                                if code == 0 {
                                    TaskState::Succeeded
                                } else {
                                    TaskState::Failed
                                },
                                |_| {},
                            );
                        }
                    }
                    out["event_path"] =
                        json!(host.supervisor.event_path(task_id).to_string_lossy());
                    Ok(out)
                }
                Err(_) => Ok(json!({"ok": false, "error": "not_found", "status": 404})),
            }
        }
        "events" => {
            let task_id = req_str(args, "task_id")?;
            let after = args.get("after_seq").and_then(|v| v.as_u64()).unwrap_or(0);
            let path = host.supervisor.event_path(task_id);
            if path.is_file() {
                let lines = read_ndjson_lines(&path, after)?;
                return Ok(json!({
                    "ok": true,
                    "events": lines,
                    "after_seq": after,
                    "source": "worker_ndjson",
                    "event_path": path.to_string_lossy(),
                }));
            }
            match ctl.events_since(task_id, after) {
                Ok(evs) => Ok(json!({
                    "ok": true,
                    "events": evs,
                    "after_seq": after,
                    "source": "task_store",
                })),
                Err(_) => Ok(json!({"ok": false, "error": "not_found", "status": 404})),
            }
        }
        "append_event" => {
            let task_id = req_str(args, "task_id")?;
            let rec = ctl.store.get(task_id).map_err(|_| "not_found".to_string())?;
            let et = args
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("message.delta");
            let payload = args.get("payload").cloned().unwrap_or(json!({}));
            let ev = ctl
                .store
                .append_event(task_id, &rec.run_id, &rec.attempt_id, et, payload)
                .map_err(|e| e.to_string())?;
            Ok(json!({"ok": true, "event": ev}))
        }
        other => Err(format!("runtime.unknown_op:{other}")),
    }
}

fn dispatch_op(host: &RuntimeHost, args: &Value) -> Result<Value, String> {
    let task_id = req_str(args, "task_id")?;
    let rec = host
        .control
        .store
        .get(task_id)
        .map_err(|_| "not_found".to_string())?;
    let entry = args.get("entry").ok_or("dispatch.entry_required")?;
    let module = entry
        .get("module")
        .and_then(|v| v.as_str())
        .ok_or("dispatch.module_required")?;
    let function = entry
        .get("function")
        .and_then(|v| v.as_str())
        .ok_or("dispatch.function_required")?;
    let fs_root = args
        .get("fs_root")
        .and_then(|v| v.as_str())
        .or_else(|| {
            args.get("context")
                .and_then(|c| c.get("fs_root"))
                .and_then(|v| v.as_str())
        })
        .ok_or("dispatch.fs_root_required")?;
    let args_ref = args
        .get("args_ref")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            args.get("args_json")
                .and_then(|v| v.as_str())
                .map(|p| format!("file:{p}"))
        });

    let tok = host.control.cancel_token_path(task_id);
    if !tok.exists() {
        std::fs::write(&tok, b"").map_err(|e| e.to_string())?;
    }

    let epoch = host.control.store.next_lease_epoch();
    let req = InvokeRequestV2 {
        schema: "InvokeRequestV2".into(),
        schema_version: PROTOCOL_VERSION,
        task_id: task_id.into(),
        run_id: rec.run_id.clone(),
        attempt_id: rec.attempt_id.clone(),
        lease: LeaseRef {
            id: format!("lease_{task_id}"),
            epoch,
        },
        entry: EntryRef {
            module: module.into(),
            function: function.into(),
        },
        args_ref,
        context: InvokeContext {
            workspace_id: rec.workspace_id.clone(),
            snapshot_id: None,
            selection_ref: None,
            capabilities_ref: None,
            deadline_at: "2099-01-01T00:00:00Z".into(),
            fs_root: Some(fs_root.into()),
        },
    };

    if host
        .control
        .scheduler
        .try_acquire_slot(AdmissionClass::Execute)
        .is_err()
    {
        return Ok(json!({
            "ok": false,
            "error": "no_slot",
            "status": 429,
            "Retry-After": 2,
        }));
    }

    match host.supervisor.dispatch(req) {
        Ok(lease_id) => {
            host.leases
                .lock()
                .unwrap()
                .insert(task_id.to_string(), lease_id.clone());
            let _ = host.control.store.cas_state(
                task_id,
                rec.revision,
                TaskState::Running,
                |r| {
                    r.lease_id = Some(lease_id.clone());
                    r.lease_epoch = Some(epoch);
                },
            );
            Ok(json!({
                "ok": true,
                "lease_id": lease_id,
                "task_id": task_id,
                "cancel_token": tok.to_string_lossy(),
                "event_path": host.supervisor.event_path(task_id).to_string_lossy(),
                "emit_result": host.supervisor.emit_result_path(&lease_id)
                    .map(|p| p.to_string_lossy().to_string()),
            }))
        }
        Err(e) => {
            host.control
                .scheduler
                .release_slot(AdmissionClass::Execute);
            if e.contains("no_slot") {
                Ok(json!({
                    "ok": false,
                    "error": "no_slot",
                    "status": 429,
                    "Retry-After": 2,
                }))
            } else {
                Err(e)
            }
        }
    }
}

fn read_ndjson_lines(path: &Path, after_seq: u64) -> Result<Vec<Value>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let mut seq = 0u64;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        seq += 1;
        if seq <= after_seq {
            continue;
        }
        match serde_json::from_str::<Value>(line) {
            Ok(mut v) => {
                if v.get("seq").is_none() {
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert("seq".into(), json!(seq));
                    }
                }
                out.push(v);
            }
            Err(_) => out.push(json!({"seq": seq, "raw": line})),
        }
    }
    Ok(out)
}

fn req_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("runtime missing `{key}`"))
}

fn task_json(rec: &crate::runtime::task_store::TaskRecord) -> Value {
    json!({
        "task_id": rec.task_id,
        "run_id": rec.run_id,
        "attempt_id": rec.attempt_id,
        "workspace_id": rec.workspace_id,
        "session_id": rec.session_id,
        "state": state_str(rec.state),
        "revision": rec.revision,
        "idempotency_key": rec.idempotency_key,
        "created_at": rec.created_at,
        "updated_at": rec.updated_at,
        "cancel_requested_at": rec.cancel_requested_at,
        "result": rec.result,
        "error": rec.error,
        "lease_id": rec.lease_id,
    })
}

fn state_str(s: TaskState) -> &'static str {
    match s {
        TaskState::Accepted => "accepted",
        TaskState::Queued => "queued",
        TaskState::Running => "running",
        TaskState::Finalizing => "finalizing",
        TaskState::Succeeded => "succeeded",
        TaskState::Failed => "failed",
        TaskState::CancelRequested => "cancel_requested",
        TaskState::Cancelled => "cancelled",
        TaskState::Interrupted => "interrupted",
        TaskState::NeedsReconciliation => "needs_reconciliation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admit_cancel_without_host_context() {
        let id = format!("rt-{}", std::process::id());
        let adm = handle(&json!({
            "op": "admit",
            "task_id": id,
            "idempotency_key": id,
            "payload": {"n": 1}
        }))
        .unwrap();
        assert_eq!(adm["ok"], true);
        let can = handle(&json!({"op": "cancel", "task_id": id})).unwrap();
        assert_eq!(can["ok"], true);
        assert_eq!(can["task"]["state"], "cancel_requested");
    }
}

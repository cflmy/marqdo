//! Live-view debug session APIs (tree-walk only).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use crate::debug::{snapshot_json, DebugAction, DebugController, DebugSnapshot};
use crate::host::HostContext;
use crate::input_feed::{effective_stdin, split_stdin_text};
use crate::interp::Interpreter;
use crate::load::load_module;
use crate::RunOptions;

struct Session {
    ctrl: Arc<DebugController>,
    #[allow(dead_code)]
    path: PathBuf,
}

fn sessions() -> &'static Mutex<HashMap<String, Arc<Session>>> {
    static S: OnceLock<Mutex<HashMap<String, Arc<Session>>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("dbg-{t}")
}

fn parse_breakpoints(v: &serde_json::Value) -> HashSet<u32> {
    v.get("breakpoints")
        .and_then(|b| b.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u32))
                .collect()
        })
        .unwrap_or_default()
}

pub fn api_debug_start(root: &Path, body: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let rel = v.get("path").and_then(|p| p.as_str()).unwrap_or("");
    if rel.is_empty() {
        return serde_json::json!({ "ok": false, "error": "missing path" });
    }
    let abs = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !abs.is_file() {
        return serde_json::json!({ "ok": false, "error": format!("file not found: {rel}") });
    }

    let bps = parse_breakpoints(&v);
    let stdin_raw = v.get("stdin").and_then(|s| s.as_str()).unwrap_or("");
    let stdin_lines = split_stdin_text(stdin_raw);

    let ctrl = DebugController::new(bps.clone());
    if bps.is_empty() {
        // No breakpoints: pause on the first statement so Debug is always useful.
        ctrl.request_step();
    }
    ctrl.set_running();
    let id = new_id();
    let session = Arc::new(Session {
        ctrl: ctrl.clone(),
        path: abs.clone(),
    });
    sessions().lock().unwrap().insert(id.clone(), session);

    let opts = RunOptions {
        sleep_limit_ms: Some(0),
        fs_root: Some(root.to_path_buf()),
        ..RunOptions::default()
    };
    let path_for_thread = abs;
    let root_for_thread = root.to_path_buf();
    let ctrl_t = ctrl.clone();
    thread::spawn(move || {
        let source = match std::fs::read_to_string(&path_for_thread) {
            Ok(s) => s,
            Err(e) => {
                ctrl_t.set_done(false, String::new(), format!("read: {e}"));
                return;
            }
        };
        let stdin = effective_stdin(&source, &stdin_lines);
        let module = match load_module(&path_for_thread) {
            Ok(m) => m,
            Err(e) => {
                ctrl_t.set_done(false, String::new(), e.to_string());
                return;
            }
        };
        let mut host = HostContext::for_capture(Some(&path_for_thread), opts.host_caps());
        host.fs_root = Some(root_for_thread);
        if let Some(lim) = opts.sleep_limit_ms {
            host.sleep_limit_ms = Some(lim);
        }
        let mut interp = Interpreter::with_capture(Some(&path_for_thread), false)
            .with_stdin(stdin)
            .with_host(host)
            .with_debug(ctrl_t.clone());
        match interp.run_module(&module) {
            Ok(_) => {
                ctrl_t.set_done(true, interp.captured_stdout.clone(), String::new());
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("debug stopped") {
                    ctrl_t.set_done(true, interp.captured_stdout.clone(), "stopped".into());
                } else {
                    ctrl_t.set_done(false, interp.captured_stdout.clone(), msg);
                }
            }
        }
    });

    let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(30));
    wrap_ok(&id, &snap)
}

pub fn api_debug_action(body: &str, action: DebugAction) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let id = v.get("session").and_then(|s| s.as_str()).unwrap_or("");
    let sessions = sessions().lock().unwrap();
    let Some(sess) = sessions.get(id) else {
        return serde_json::json!({ "ok": false, "error": "unknown session" });
    };
    let ctrl = sess.ctrl.clone();
    drop(sessions);

    if v.get("breakpoints").map(|b| b.is_array()).unwrap_or(false) {
        ctrl.set_breakpoints(parse_breakpoints(&v));
    }

    match ctrl.snapshot() {
        DebugSnapshot::Done { .. } => return wrap_ok(id, &ctrl.snapshot()),
        DebugSnapshot::Paused(_) => {}
        other => {
            return serde_json::json!({
                "ok": false,
                "error": format!("session not paused (status={})", status_name(&other)),
                "session": id,
            });
        }
    }

    ctrl.send_action(action);
    let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(30));
    wrap_ok(id, &snap)
}

pub fn api_debug_stop(body: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let id = v.get("session").and_then(|s| s.as_str()).unwrap_or("");
    let mut map = sessions().lock().unwrap();
    if let Some(sess) = map.remove(id) {
        sess.ctrl.send_action(DebugAction::Stop);
        return serde_json::json!({ "ok": true, "status": "stopped", "session": id });
    }
    serde_json::json!({ "ok": false, "error": "unknown session" })
}

pub fn api_debug_set_breakpoints(body: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let id = v.get("session").and_then(|s| s.as_str()).unwrap_or("");
    let bps = parse_breakpoints(&v);
    let map = sessions().lock().unwrap();
    let Some(sess) = map.get(id) else {
        return serde_json::json!({ "ok": false, "error": "unknown session" });
    };
    sess.ctrl.set_breakpoints(bps.clone());
    let mut lines: Vec<u32> = bps.into_iter().collect();
    lines.sort_unstable();
    serde_json::json!({
        "ok": true,
        "session": id,
        "breakpoints": lines,
    })
}

fn wrap_ok(id: &str, snap: &DebugSnapshot) -> serde_json::Value {
    let mut out = snapshot_json(snap);
    if let Some(obj) = out.as_object_mut() {
        obj.insert("ok".into(), serde_json::Value::Bool(true));
        obj.insert("session".into(), serde_json::Value::String(id.to_string()));
    }
    out
}

fn status_name(s: &DebugSnapshot) -> &'static str {
    match s {
        DebugSnapshot::Idle => "idle",
        DebugSnapshot::Running => "running",
        DebugSnapshot::Paused(_) => "paused",
        DebugSnapshot::Done { .. } => "done",
    }
}

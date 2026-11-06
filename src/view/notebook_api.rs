//! Live-view notebook session APIs.

#[cfg(all(test, unix))]
mod interrupt_http_tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, MutexGuard, TryLockError};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::host::json::{json_to_value, value_to_json};
use crate::host::foreign_doc::DocumentSession;
use crate::value::Value;

struct ApiSession {
    closed: AtomicBool,
    document: OnceLock<Result<Mutex<DocumentSession>, String>>,
    control: crate::host::foreign_doc::DocumentControl,
}
impl ApiSession {
    fn mutex(&self) -> Result<&Mutex<DocumentSession>, String> {
        // `OnceLock::wait` needs Rust ≥1.84; MSRV is 1.81 — poll until init.
        loop {
            match self.document.get() {
                Some(slot) => return slot.as_ref().map_err(Clone::clone),
                None => std::thread::yield_now(),
            }
        }
    }
    fn lock(&self) -> Result<MutexGuard<'_, DocumentSession>, String> {
        let guard = self.mutex()?.lock().map_err(|_| "session lock poisoned".to_string())?;
        if self.closed.load(Ordering::Acquire) { return Err("session closed".into()); }
        Ok(guard)
    }
}
fn sessions() -> &'static Mutex<HashMap<String, Arc<ApiSession>>> {
    static S: OnceLock<Mutex<HashMap<String, Arc<ApiSession>>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn body_value(body: &str) -> serde_json::Value {
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

fn requested_path(root: &Path, v: &serde_json::Value) -> Result<(String, PathBuf), String> {
    let rel = v
        .get("path")
        .and_then(|p| p.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if rel.is_empty() {
        return Err("missing path".into());
    }
    let abs = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    Ok((rel, abs))
}

fn path_of(root: &Path, v: &serde_json::Value) -> Result<(String, PathBuf), String> {
    let (rel, abs) = requested_path(root, v)?;
    if !abs.is_file() {
        return Err(format!("file not found: {rel}"));
    }
    Ok((rel, abs))
}

fn get_or_open(root: &Path, abs: &Path) -> Result<Arc<ApiSession>, String> {
    let key = abs.display().to_string();
    let arc = {
        let mut map = sessions().lock().map_err(|_| "session lock poisoned".to_string())?;
        Arc::clone(map.entry(key.clone()).or_insert_with(|| Arc::new(ApiSession {
            closed: AtomicBool::new(false),
            control: Default::default(), document: OnceLock::new(),
        })))
    };
    // Kernel startup may block. It must never hold the registry lock needed by
    // interrupt requests for an unrelated already-running document.
    let initialized = arc.document.get_or_init(|| DocumentSession::open(abs, root).map(|mut session| {
        session.adopt_control(arc.control.clone());
        Mutex::new(session)
    }));
    if let Err(error) = initialized {
        let mut map = sessions().lock().map_err(|_| "session lock poisoned".to_string())?;
        if map.get(&key).is_some_and(|old| Arc::ptr_eq(old, &arc)) { map.remove(&key); }
        return Err(error.clone());
    }
    Ok(arc)
}

fn json_value(v: &Value) -> serde_json::Value {
    value_to_json(v).unwrap_or(serde_json::Value::Null)
}

fn ok_session(session: &DocumentSession, extra: serde_json::Value) -> serde_json::Value {
    let mut out = serde_json::json!({
        "ok": true,
        "session": json_value(&session.summary()),
    });
    if let (Some(dst), Some(src)) = (out.as_object_mut(), extra.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    out
}

fn err(error: String) -> serde_json::Value {
    serde_json::json!({ "ok": false, "error": error })
}

pub fn api_run_document(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let (_rel, abs) = match path_of(root, &v) {
        Ok(p) => p,
        Err(e) => return err(e),
    };
    let mut opts = crate::RunOptions::default();
    opts.fs_root = Some(root.to_path_buf());
    opts.sleep_limit_ms = Some(30_000);
    match crate::run_file_capture(&abs, &opts) {
        Ok(cap) => serde_json::json!({
            "ok": true,
            "stdout": cap.stdout,
            "value": json_value(&cap.value),
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": format!("{e:#}") }),
    }
}

pub fn api_open(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let (_rel, abs) = match path_of(root, &v) {
        Ok(p) => p,
        Err(e) => return err(e),
    };
    match get_or_open(root, &abs) {
        Ok(s) => match s.lock() {
            Ok(mut sess) => {
                if let Err(e) = sess.reload(root) {
                    return err(e);
                }
                ok_session(&sess, serde_json::json!({}))
            }
            Err(_) => err("session lock poisoned".into()),
        },
        Err(e) => err(e),
    }
}

pub fn api_reload(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let (_rel, abs) = match path_of(root, &v) {
        Ok(p) => p,
        Err(e) => return err(e),
    };
    match get_or_open(root, &abs) {
        Ok(s) => match s.lock() {
            Ok(mut sess) => match sess.reload(root) {
                Ok(()) => ok_session(&sess, serde_json::json!({})),
                Err(e) => err(e),
            },
            Err(_) => err("session lock poisoned".into()),
        },
        Err(e) => err(e),
    }
}

pub fn api_close(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let (_rel, abs) = match path_of(root, &v) {
        Ok(p) => p,
        Err(e) => return err(e),
    };
    let key = abs.display().to_string();
    let removed = sessions()
        .lock()
        .ok()
        .and_then(|mut m| m.remove(&key));
    if let Some(arc) = removed {
        if let Ok(mut s) = arc.lock() {
            arc.closed.store(true, Ordering::Release);
            s.close();
        }
    }
    serde_json::json!({ "ok": true })
}

fn with_session<F>(root: &Path, body: &str, action: &str, f: F) -> serde_json::Value
where
    F: FnOnce(&mut DocumentSession) -> Result<Value, String>,
{
    let v = body_value(body);
    let (_rel, abs) = match path_of(root, &v) {
        Ok(p) => p,
        Err(e) => return err(e),
    };
    let arc = match get_or_open(root, &abs) {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    let mut sess = match arc.lock() {
        Ok(s) => s,
        Err(_) => return err("session lock poisoned".into()),
    };
    match f(&mut sess) {
        Ok(value) => {
            let mut extra = serde_json::Map::new();
            extra.insert(action.into(), json_value(&value));
            ok_session(&sess, serde_json::Value::Object(extra))
        }
        Err(e) => err(e),
    }
}

pub fn api_persist_cell(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    let key = v.get("key").and_then(|x| x.as_str()).unwrap_or("");
    if cell.is_empty() {
        return err("missing cell".into());
    }
    with_session(root, body, "persisted", |s| s.persist_cell(cell, key))
}

pub fn api_interrupt(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    // A control request addresses an existing loaded session. It neither reads
    // the source again nor creates a process as a side effect of cancellation.
    let (_, abs) = match requested_path(root, &v) { Ok(p) => p, Err(e) => return err(e) };
    let session = match sessions().lock() {
        Ok(map) => match map.get(&abs.display().to_string()) { Some(s) => s.clone(), None => return err("no open session".into()) },
        Err(_) => return err("session lock poisoned".into()),
    };
    if session.closed.load(Ordering::Acquire) { return err("session closed".into()); }
    let mutex = match session.document.get() {
        Some(Ok(mutex)) => mutex, Some(Err(error)) => return err(error.clone()),
        None => return err("session is still opening; no evaluation is active".into()),
    };
    let result = match mutex.try_lock() {
        Ok(mut document) => if session.closed.load(Ordering::Acquire) { err("session closed".into()) } else { match document.interrupt() {
            Ok(value) => ok_session(&document, serde_json::json!({ "interrupted": json_value(&value), "in_flight": false })),
            Err(error) => err(error),
        } },
        Err(TryLockError::WouldBlock) => match session.control.interrupt() {
            Ok(value) => {
                let receipt = json_value(&value);
                let accepted = receipt["results"].as_array().is_some_and(|rows| rows.iter().all(|row| row["accepted"] == true));
                if accepted { serde_json::json!({ "ok": true, "interrupted": receipt, "in_flight": true }) }
                else { serde_json::json!({ "ok": false, "error": "interrupt delivery failed", "interrupted": receipt, "in_flight": true }) }
            },
            Err(error) => err(error),
        },
        Err(TryLockError::Poisoned(_)) => err("session lock poisoned".into()),
    };
    result
}

pub fn api_export_snapshot(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let compact = match v.get("compact") { None | Some(serde_json::Value::Bool(false)) => false,
        Some(serde_json::Value::Bool(true)) => true, _ => return err("compact must be boolean".into()) };
    with_session(root, body, "snapshot", |s| {
        let snapshot = s.export_snapshot()?;
        if compact { crate::host::foreign_doc::packing::pack(&snapshot) } else { Ok(snapshot) }
    })
}

pub fn api_import_snapshot(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let snapshot = v.get("snapshot").cloned().unwrap_or(serde_json::Value::Null);
    let snapshot = match json_to_value(&snapshot).and_then(|value| if crate::host::foreign_doc::packing::is_packed(&value) {
        crate::host::foreign_doc::packing::unpack(&value)
    } else { Ok(value) }) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    with_session(root, body, "restored", |s| s.import_snapshot(&snapshot))
}

pub fn api_replay_snapshot(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let snapshot = v.get("snapshot").cloned().unwrap_or(serde_json::Value::Null);
    let snapshot = match json_to_value(&snapshot).and_then(|value| if crate::host::foreign_doc::packing::is_packed(&value) {
        crate::host::foreign_doc::packing::unpack(&value)
    } else { Ok(value) }) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    with_session(root, body, "replay", |s| s.replay_snapshot(&snapshot))
}

pub fn api_replay_check(root: &Path, body: &str) -> serde_json::Value {
    // Decode before serde_json::Value can overwrite duplicate recording fields.
    let request = match crate::host::json::parse(&Value::Text(body.to_string())) {
        Ok(value) => value,
        Err(error) => return err(error),
    };
    let v = match value_to_json(&request) { Ok(v) => v, Err(e) => return err(e) };
    let (_, abs) = match path_of(root, &v) { Ok(path) => path, Err(e) => return err(e) };
    let Value::Map(fields) = request else { return err("request must be a map".into()) };
    let Some((_, snapshot)) = fields.iter().find(|(k,_)| k == "snapshot") else { return err("missing snapshot".into()) };
    let check = match DocumentSession::check_snapshot_isolated(&abs, root, snapshot) {
        Ok(check) => check, Err(error) => return err(error),
    };
    let existing = match sessions().lock() {
        Ok(map) => map.get(&abs.display().to_string()).cloned(),
        Err(_) => return err("session lock poisoned".into()),
    };
    let extra = serde_json::json!({"replay_check": json_value(&check)});
    if let Some(arc) = existing {
        match arc.lock() { Ok(session) => ok_session(&session, extra), Err(_) => err("session lock poisoned".into()) }
    } else {
        serde_json::json!({"ok": true, "replay_check": json_value(&check)})
    }
}

pub fn api_clear_cell(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    if cell.is_empty() {
        return err("missing cell".into());
    }
    with_session(root, body, "cleared", |s| s.clear_cell(cell))
}

pub fn api_clear(root: &Path, body: &str) -> serde_json::Value {
    with_session(root, body, "cleared", |s| Ok(s.clear_outputs()))
}

pub fn api_cell_run(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    let kernel = v.get("kernel").and_then(|x| x.as_str()).filter(|s| !s.is_empty());
    let stdin = v.get("stdin").and_then(|x| x.as_str());
    if cell.is_empty() {
        return err("missing cell".into());
    }
    with_session(root, body, "record", |s| s.run_cell_with_stdin(cell, kernel, stdin))
}

pub fn api_cell_step(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let stdin = v.get("stdin").and_then(|x| x.as_str());
    with_session(root, body, "record", |s| s.step_with_stdin(stdin))
}

pub fn api_run_all(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let restart = v.get("restart").and_then(|x| x.as_bool()).unwrap_or(false);
    let stdin = v.get("stdin").and_then(|x| x.as_str());
    let inputs: Option<HashMap<String, String>> = v.get("inputs").and_then(|x| x.as_object()).map(|m| {
        m.iter()
            .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
            .collect()
    });
    with_session(root, body, "summary", |s| s.run_all_with_inputs(restart, stdin, inputs.as_ref()))
}

pub fn api_kernel_restart(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    if kernel.is_empty() {
        return err("missing kernel".into());
    }
    with_session(root, body, "value", |s| s.restart_kernel(kernel))
}

pub fn api_plan_assign(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    if cell.is_empty() || kernel.is_empty() {
        return err("missing cell/kernel".into());
    }
    with_session(root, body, "value", |s| s.assign_kernel(cell, kernel))
}

pub fn api_plan_add(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    let lang = v.get("lang").and_then(|x| x.as_str()).unwrap_or("");
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    let source = v.get("source").and_then(|x| x.as_str()).unwrap_or("");
    if cell.is_empty() || lang.is_empty() || kernel.is_empty() {
        return err("missing cell/lang/kernel".into());
    }
    with_session(root, body, "value", |s| s.add_cell(cell, lang, source, kernel))
}

pub fn api_plan_remove(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    if cell.is_empty() {
        return err("missing cell".into());
    }
    with_session(root, body, "value", |s| s.remove_cell(cell))
}

pub fn api_plan_value(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
    if kernel.is_empty() || name.is_empty() {
        return err("missing kernel/name".into());
    }
    let value = match v.get("value") {
        Some(v) => match json_to_value(v) {
            Ok(v) => v,
            Err(e) => return err(e),
        },
        None => Value::None,
    };
    with_session(root, body, "value", |s| s.set_value(kernel, name, value))
}

pub fn api_assign(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    if cell.is_empty() || kernel.is_empty() {
        return err("missing cell/kernel".into());
    }
    with_session(root, body, "value", |s| s.assign_kernel(cell, kernel))
}

pub fn api_state_list(root: &Path, body: &str) -> serde_json::Value {
    with_session(root, body, "state", |s| s.state_list())
}

pub fn api_state_push(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
    if kernel.is_empty() || name.is_empty() {
        return err("missing kernel/name".into());
    }
    let value = match v.get("value") {
        Some(v) => match json_to_value(v) {
            Ok(v) => v,
            Err(e) => return err(e),
        },
        None => Value::None,
    };
    with_session(root, body, "value", |s| s.push_value(kernel, name, value))
}

pub fn api_state_pull(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
    if kernel.is_empty() || name.is_empty() {
        return err("missing kernel/name".into());
    }
    with_session(root, body, "value", |s| s.pull_value(kernel, name))
}

pub fn api_journal(root: &Path, body: &str) -> serde_json::Value {
    with_session(root, body, "journal", |s| Ok(s.journal_json()))
}

pub fn api_oracle_journal(root: &Path, body: &str) -> serde_json::Value {
    with_session(root, body, "oracle_journal", |s| Ok(s.oracle_journal_json()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_available() -> bool {
        std::process::Command::new("node")
            .args(["-e", "process.exit(0)"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn open_body() -> &'static str {
        r#"{"path":"shared.mq.md"}"#
    }

    #[test]
    fn api_shared_state_and_kernel_switch() {
        if !node_available() {
            return;
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/notebook");
        let open = api_open(&root, open_body());
        assert_eq!(open.get("ok").and_then(|v| v.as_bool()), Some(true), "{open:?}");
        let run = api_cell_run(
            &root,
            r#"{"path":"shared.mq.md","cell":"设置"}"#,
        );
        assert_eq!(run.get("ok").and_then(|v| v.as_bool()), Some(true), "{run:?}");
        let read = api_cell_run(
            &root,
            r#"{"path":"shared.mq.md","cell":"读取"}"#,
        );
        let stdout = read
            .get("record")
            .and_then(|r| r.get("stdout"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(stdout, "42\n", "{read:?}");
        let assign = api_assign(
            &root,
            r#"{"path":"shared.mq.md","cell":"读取","kernel":"绘图"}"#,
        );
        assert_eq!(assign.get("ok").and_then(|v| v.as_bool()), Some(true), "{assign:?}");
        let failed = api_cell_run(
            &root,
            r#"{"path":"shared.mq.md","cell":"读取"}"#,
        );
        assert_eq!(
            failed
                .get("record")
                .and_then(|r| r.get("ok"))
                .and_then(|v| v.as_bool()),
            Some(false),
            "{failed:?}"
        );
        let clear = api_clear(&root, r#"{"path":"shared.mq.md"}"#);
        assert_eq!(clear.get("ok").and_then(|v| v.as_bool()), Some(true), "{clear:?}");
        let cleared = clear.get("session").and_then(|s| s.get("cells")).and_then(|c| c.as_array()).cloned().unwrap_or_default();
        assert!(cleared.iter().all(|c| c.get("runs").and_then(|v| v.as_u64()) == Some(0)));
        let push = api_state_push(
            &root,
            r#"{"path":"shared.mq.md","kernel":"分析","name":"z","value":9}"#,
        );
        assert_eq!(push.get("ok").and_then(|v| v.as_bool()), Some(true), "{push:?}");
        let pull = api_state_pull(
            &root,
            r#"{"path":"shared.mq.md","kernel":"分析","name":"z"}"#,
        );
        assert_eq!(
            pull.get("value").and_then(|v| v.as_i64()),
            Some(9),
            "{pull:?}"
        );
        let _ = api_close(&root, open_body());
    }
}

#[cfg(test)]
mod replay_api_tests {
    use super::*;
    #[test]
    fn closed_api_session_cannot_be_reactivated_after_path_reopen() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let relative = format!("target/api-close-capability-{}.mq.md", std::process::id());
        let path = root.join(&relative);
        std::fs::write(&path, "---\nimport:\n  foreign: lib/foreign.mq.md\n---\n# main\n\n`会话` = > foreign.session\n\n`a` =\n```python\nn = 1\n```\n").unwrap();
        let old = get_or_open(&root, &path).unwrap();
        old.lock().unwrap().run_all(false).unwrap();
        let body = serde_json::json!({"path": relative}).to_string();
        assert_eq!(api_close(&root, &body)["ok"], true);
        let fresh = get_or_open(&root, &path).unwrap();
        assert!(!Arc::ptr_eq(&old, &fresh));
        assert!(old.lock().is_err());
        fresh.lock().unwrap().run_all(false).unwrap();
        assert_eq!(api_close(&root, &body)["ok"], true);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn strict_replay_endpoint_uses_isolation_and_preserves_warm_session() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let relative=format!("target/api-replay-isolated-{}.mq.md",std::process::id());let path=root.join(&relative);
        std::fs::write(&path,"---\nimport:\n  foreign: lib/foreign.mq.md\n---\n# main\n\n`会话` = > foreign.session\n\n`a` =\n```python\nstate = globals().get('state',0) + 1\nprint(state)\n```\n").unwrap();
        let arc=get_or_open(&root,&path).unwrap();
        let (snapshot,kernel)={let mut live=arc.lock().unwrap();live.run_all(false).unwrap();let snapshot=live.export_snapshot().unwrap();let kernel=live.cells[0].kernel.clone();live.set_value(&kernel,"state",Value::Int(99)).unwrap();(snapshot,kernel)};
        let before={let live=arc.lock().unwrap();live.summary()};
        let body=serde_json::json!({"path":relative,"snapshot":value_to_json(&snapshot).unwrap()}).to_string();
        let response=api_replay_check(&root,&body);
        assert_eq!(response["ok"],true,"{response}");assert_eq!(response["replay_check"]["ok"],true,"{response}");
        assert_eq!(arc.lock().unwrap().summary(),before,"verification mutated live state");
        let duplicate=body.replacen("\"snapshot\":", "\"snapshot\":null,\"snapshot\":",1);
        let bad=api_replay_check(&root,&duplicate);assert_eq!(bad["ok"],false);assert!(bad["error"].as_str().unwrap().contains("duplicate"));
        assert_eq!(arc.lock().unwrap().summary(),before);
        assert_eq!(arc.lock().unwrap().pull_value(&kernel,"state").unwrap(),Value::Int(99),"isolated verification reset the user's actual kernel");
        sessions().lock().unwrap().remove(&path.display().to_string());arc.lock().unwrap().close();std::fs::remove_file(path).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn malformed_replay_does_not_start_declared_seed_kernel() {
        use std::os::unix::fs::PermissionsExt;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR"));let pid=std::process::id();
        let relative=format!("target/api-replay-seeded-{pid}.mq.md");let path=root.join(&relative);
        let wrapper=root.join(format!("target/api-replay-wrapper-{pid}.py"));let marker=root.join(format!("target/api-replay-marker-{pid}"));
        let marker_literal=serde_json::to_string(&marker.display().to_string()).unwrap();
        let script=format!("#!/usr/bin/python3\nimport os,sys\nfrom pathlib import Path\nPath({marker_literal}).write_text('started')\nos.execv(sys.executable,[sys.executable]+sys.argv[1:])\n");
        std::fs::write(&wrapper,script).unwrap();std::fs::set_permissions(&wrapper,std::fs::Permissions::from_mode(0o755)).unwrap();
        let command=serde_json::to_string(&wrapper.display().to_string()).unwrap();
        let source=format!("---\nimport:\n  foreign: lib/foreign.mq.md\n---\n# main\n\n`kernels` =\n| 名称 | 语言 | 命令 |\n|---|---|---|\n| py | python | {command} |\n\n`units` =\n| 单元 | 内核 |\n|---|---|\n| a | py |\n\n`initial` =\n| 内核 | 名称 | 值 |\n|---|---|---|\n| py | n | 1 |\n\n`会话` = > foreign.session kernels=`kernels` cells=`units` values=`initial`\n\n`a` =\n```python\nprint(n)\n```\n");
        std::fs::write(&path,source).unwrap();let mut recording=DocumentSession::open(&path,&root).unwrap();
        recording.run_all(false).unwrap();let snapshot=recording.export_snapshot().unwrap();recording.close();assert!(marker.exists());std::fs::remove_file(&marker).unwrap();
        let valid=value_to_json(&snapshot).unwrap();let mut malformed=valid.clone();malformed["actions"].as_array_mut().unwrap().pop();
        let cold = api_interrupt(&root, &serde_json::json!({"path":relative}).to_string());
        assert_eq!(cold["ok"], false); assert!(!marker.exists(), "cold interrupt started a kernel");
        let body=serde_json::json!({"path":relative,"snapshot":malformed}).to_string();let response=api_replay_check(&root,&body);
        assert_eq!(response["ok"],false,"{response}");assert!(!marker.exists(),"malformed suffix started a seed kernel before preflight");
        assert!(!sessions().lock().unwrap().contains_key(&path.display().to_string()));
        let packed = crate::host::foreign_doc::packing::pack(&snapshot).unwrap();
        let mut corrupt = value_to_json(&packed).unwrap(); corrupt["sha256"] = "bad".into();
        let response = api_replay_check(&root, &serde_json::json!({"path":relative,"snapshot":corrupt}).to_string());
        assert_eq!(response["ok"], false); assert!(!marker.exists(), "corrupt DAG started a seed kernel");
        let mut controls = valid.clone();
        controls["actions"][0]["controls"] = serde_json::json!([{"after_us":-1,"results":[]}]);
        let response = api_replay_check(&root, &serde_json::json!({"path":relative,"snapshot":controls}).to_string());
        assert_eq!(response["ok"], false); assert!(!marker.exists(), "invalid control started a seed kernel");
        let response=api_replay_check(&root,&serde_json::json!({"path":relative,"snapshot":valid}).to_string());
        assert_eq!(response["replay_check"]["ok"],true,"{response}");assert!(marker.exists());
        assert!(!sessions().lock().unwrap().contains_key(&path.display().to_string()));
        std::fs::remove_file(path).unwrap();std::fs::remove_file(wrapper).unwrap();std::fs::remove_file(marker).unwrap();
    }

}

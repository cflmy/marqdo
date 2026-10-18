//! Live-view notebook session APIs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::host::json::{json_to_value, value_to_json};
use crate::host::foreign_doc::DocumentSession;
use crate::value::Value;

fn sessions() -> &'static Mutex<HashMap<String, Arc<Mutex<DocumentSession>>>> {
    static S: OnceLock<Mutex<HashMap<String, Arc<Mutex<DocumentSession>>>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn body_value(body: &str) -> serde_json::Value {
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

fn path_of(root: &Path, v: &serde_json::Value) -> Result<(String, PathBuf), String> {
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
    if !abs.is_file() {
        return Err(format!("file not found: {rel}"));
    }
    Ok((rel, abs))
}

fn get_or_open(root: &Path, abs: &Path) -> Result<Arc<Mutex<DocumentSession>>, String> {
    let key = abs.display().to_string();
    let mut map = sessions().lock().map_err(|_| "session lock poisoned".to_string())?;
    if let Some(s) = map.get(&key) {
        return Ok(Arc::clone(s));
    }
    let session = DocumentSession::open(abs, root)?;
    let arc = Arc::new(Mutex::new(session));
    map.insert(key, Arc::clone(&arc));
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

pub fn api_clear(root: &Path, body: &str) -> serde_json::Value {
    with_session(root, body, "cleared", |s| Ok(s.clear_outputs()))
}

pub fn api_cell_run(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let cell = v.get("cell").and_then(|x| x.as_str()).unwrap_or("");
    let kernel = v.get("kernel").and_then(|x| x.as_str()).filter(|s| !s.is_empty());
    if cell.is_empty() {
        return err("missing cell".into());
    }
    with_session(root, body, "record", |s| s.run_cell(cell, kernel))
}

pub fn api_cell_step(root: &Path, body: &str) -> serde_json::Value {
    with_session(root, body, "record", |s| s.step())
}

pub fn api_run_all(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let restart = v.get("restart").and_then(|x| x.as_bool()).unwrap_or(false);
    with_session(root, body, "summary", |s| s.run_all(restart))
}

pub fn api_kernel_restart(root: &Path, body: &str) -> serde_json::Value {
    let v = body_value(body);
    let kernel = v.get("kernel").and_then(|x| x.as_str()).unwrap_or("");
    if kernel.is_empty() {
        return err("missing kernel".into());
    }
    with_session(root, body, "value", |s| s.restart_kernel(kernel))
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

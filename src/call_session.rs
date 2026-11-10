//! Call-session journal for `marqdo call … toget` across process boundaries.
//!
//! Plugin handles do not survive a new OS process; replaying the journal rebuilds them.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::call::{self, CallEffects};
use crate::host::json::{json_to_value, value_to_json};
use crate::value::Value;
use crate::Backend;

const SESSION_DIR: &str = ".marqdo/call-sessions";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CallSession {
    pub session: String,
    #[serde(default)]
    pub steps: Vec<CallStep>,
    /// JSON-serializable bindings from `toget` (when the result was V_safe JSON).
    #[serde(default)]
    pub bindings: BTreeMap<String, serde_json::Value>,
    /// Names whose live value must be rebuilt by replaying `steps` (plugin handles / opaque).
    #[serde(default)]
    pub journal_bindings: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStep {
    pub file: String,
    pub function: String,
    pub args: BTreeMap<String, serde_json::Value>,
    pub effects: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub literal_args: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_root: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toget: Option<String>,
}

fn is_false(value: &bool) -> bool { !*value }

fn sessions_root() -> PathBuf {
    if let Ok(p) = std::env::var("MARQDO_CALL_SESSION_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(SESSION_DIR)
}

pub fn session_path(id: &str) -> PathBuf {
    let safe: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    sessions_root().join(format!("{safe}.json"))
}

pub fn load_session(id: &str) -> Result<CallSession> {
    let path = session_path(id);
    if !path.exists() {
        return Ok(CallSession {
            session: id.to_string(),
            ..Default::default()
        });
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read call session {}", path.display()))?;
    let mut s: CallSession = serde_json::from_str(&text)
        .with_context(|| format!("parse call session {}", path.display()))?;
    s.session = id.to_string();
    Ok(s)
}

pub fn save_session(session: &CallSession) -> Result<()> {
    let path = session_path(&session.session);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(session)?;
    fs::write(&path, text)?;
    Ok(())
}

/// Resolve `$name` / `@name` argument values from the session (replaying journal as needed).
pub fn resolve_args(
    session: &mut CallSession,
    args: &[(String, Value)],
    backend: Backend,
) -> Result<Vec<(String, Value)>> {
    let mut live: BTreeMap<String, Value> = BTreeMap::new();
    for (k, v) in &session.bindings {
        live.insert(k.clone(), json_to_value(v).map_err(|e| anyhow::anyhow!(e))?);
    }
    let needs_replay = args.iter().any(|(_, v)| {
        matches!(v, Value::Text(t) if t.starts_with('$') || t.starts_with('@')) && {
            let name = v_text_ref(v);
            session.journal_bindings.contains_key(name)
                || (!session.bindings.contains_key(name)
                    && session
                        .steps
                        .iter()
                        .any(|s| s.toget.as_deref() == Some(name)))
        }
    });
    if needs_replay
        || args.iter().any(|(_, v)| {
            matches!(v, Value::Text(t) if (t.starts_with('$') || t.starts_with('@'))
            && session.journal_bindings.contains_key(&t[1..]))
        })
    {
        live = replay_live(session, backend)?;
    }
    let mut out = Vec::with_capacity(args.len());
    for (name, value) in args {
        out.push((
            name.clone(),
            resolve_one(session, &live, value.clone(), backend)?,
        ));
    }
    Ok(out)
}

fn v_text_ref(v: &Value) -> &str {
    match v {
        Value::Text(t) if t.starts_with('$') || t.starts_with('@') => &t[1..],
        Value::Text(t) => t.as_str(),
        _ => "",
    }
}

fn resolve_one(
    session: &CallSession,
    live: &BTreeMap<String, Value>,
    value: Value,
    backend: Backend,
) -> Result<Value> {
    let Value::Text(t) = &value else {
        return Ok(value);
    };
    if !(t.starts_with('$') || t.starts_with('@')) {
        return Ok(value);
    }
    let key = &t[1..];
    if let Some(v) = live.get(key) {
        return Ok(v.clone());
    }
    if let Some(j) = session.bindings.get(key) {
        return json_to_value(j).map_err(|e| anyhow::anyhow!(e));
    }
    // Last resort: full replay
    let live = replay_live(session, backend)?;
    live.get(key).cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "call.session_unbound: `${key}` not in session {}",
            session.session
        )
    })
}

fn replay_live(session: &CallSession, backend: Backend) -> Result<BTreeMap<String, Value>> {
    let mut live: BTreeMap<String, Value> = BTreeMap::new();
    for (k, v) in &session.bindings {
        // Prefer journal rebuild for names also in journal_bindings
        if !session.journal_bindings.contains_key(k) {
            live.insert(k.clone(), json_to_value(v).map_err(|e| anyhow::anyhow!(e))?);
        }
    }
    for step in &session.steps {
        let mut args = Vec::new();
        for (k, jv) in &step.args {
            let mut val = json_to_value(jv).map_err(|e| anyhow::anyhow!(e))?;
            if let Value::Text(t) = &val {
                if !step.literal_args && (t.starts_with('$') || t.starts_with('@')) {
                    let key = &t[1..];
                    val = live
                        .get(key)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("call.session_replay: missing `${key}`"))?;
                }
            }
            args.push((k.clone(), val));
        }
        let effects = CallEffects::parse(&step.effects)?;
        let value = if effects.needs_snapshot() {
            let result = step.result.as_ref().ok_or_else(|| anyhow::anyhow!(
                "call.session_replay: effectful step has no durable result; refusing to repeat {}", step.effects))?;
            json_to_value(result).map_err(|e| anyhow::anyhow!(e))?
        } else {
            call::call_file_raw_with_root(
                Path::new(&step.file),
                &step.function,
                &args,
                effects,
                backend,
                step.fs_root.as_deref(),
            )?
        };
        if let Some(name) = &step.toget {
            live.insert(name.clone(), value);
        }
    }
    Ok(live)
}

pub fn record_step(
    session: &mut CallSession,
    file: &Path,
    function: &str,
    args: &[(String, Value)],
    effects: CallEffects,
    fs_root: Option<&Path>,
    toget: Option<&str>,
    result: &Value,
) -> Result<()> {
    let mut arg_map = BTreeMap::new();
    for (k, v) in args {
        // Keep $ refs as text so replay can resolve them.
        if let Value::Text(t) = v {
            if t.starts_with('$') || t.starts_with('@') {
                arg_map.insert(k.clone(), serde_json::Value::String(t.clone()));
                continue;
            }
        }
        match value_to_json(v) {
            Ok(j) => {
                arg_map.insert(k.clone(), j);
            }
            Err(_) => {
                bail!("call.session: argument `{k}` is not JSON-serializable for the journal");
            }
        }
    }
    let file_s = file
        .canonicalize()
        .unwrap_or_else(|_| file.to_path_buf())
        .display()
        .to_string();
    let step = CallStep {
        file: file_s,
        function: function.to_string(),
        args: arg_map,
        effects: effects.as_str(),
        literal_args: false,
        result: if is_json_friendly(result) {
            value_to_json(result).ok()
        } else {
            None
        },
        fs_root: fs_root.map(|p| p.canonicalize()).transpose()?,
        toget: toget.map(|s| s.to_string()),
    };
    session.steps.push(step);
    if let Some(name) = toget {
        match value_to_json(result) {
            Ok(j) if is_json_friendly(result) => {
                session.bindings.insert(name.to_string(), j);
                session.journal_bindings.remove(name);
            }
            _ => {
                session.bindings.remove(name);
                session
                    .journal_bindings
                    .insert(name.to_string(), session.steps.len() - 1);
            }
        }
    }
    Ok(())
}

fn is_json_friendly(v: &Value) -> bool {
    match v {
        Value::None | Value::Bool(_) | Value::Int(_) | Value::Num(_) | Value::Text(_) => true,
        Value::List(xs) => xs.iter().all(is_json_friendly),
        Value::Map(pairs) => {
            // Tagged quantum objects need journal replay across processes.
            if pairs.iter().any(|(k, _)| k == "_type") {
                return false;
            }
            pairs.iter().all(|(_, v)| is_json_friendly(v))
        }
        Value::Handle(_) | Value::Formula(_) | Value::Code(_) | Value::Secret(_) => false,
    }
}

/// Dump journal steps as a minimal Marqdo script for later `marqdo run`.
pub fn dump_replay_script(session: &CallSession) -> String {
    let mut out =
        String::from("# main\n\n# Replayed from call-session journal. Review before trusting.\n\n");
    for (i, step) in session.steps.iter().enumerate() {
        out.push_str(&format!(
            "> print text=\"step {i}: call {} {} effects={}\"\n",
            step.file, step.function, step.effects
        ));
    }
    if let Some(last) = session.steps.last().and_then(|s| s.toget.as_ref()) {
        out.push_str(&format!("\n> print text=\"last toget was {last}\"\n"));
    }
    out.push_str("*None*\n");
    out
}

pub fn default_session_id() -> String {
    std::env::var("MARQDO_CALL_SESSION").unwrap_or_else(|_| "default".into())
}

#[cfg(test)]
mod write_snapshot_tests {
    use super::*;
    #[test]
    fn replay_uses_durable_write_results_instead_of_repeating_io() {
        let dir = std::env::temp_dir().join(format!("mq-write-replay-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("log.txt"), "once").unwrap();
        for backend in [Backend::Tree, Backend::Bytecode] {
            let step = CallStep {
                literal_args: false,
                file: Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("lib/fs.mq.md")
                    .to_string_lossy()
                    .into(),
                function: "append_text".into(),
                args: BTreeMap::from([
                    ("path".into(), serde_json::json!("log.txt")),
                    ("text".into(), serde_json::json!("once")),
                ]),
                effects: "fs.write".into(),
                result: Some(serde_json::Value::Null),
                fs_root: Some(dir.clone()),
                toget: Some("written".into()),
            };
            let mut session = CallSession {
                steps: vec![step],
                ..Default::default()
            };
            assert_eq!(
                replay_live(&session, backend).unwrap()["written"],
                Value::None
            );
            assert_eq!(fs::read_to_string(dir.join("log.txt")).unwrap(), "once");
            session.steps[0].result = None;
            assert!(replay_live(&session, backend)
                .unwrap_err()
                .to_string()
                .contains("refusing to repeat"));
        }
        fs::remove_dir_all(dir).unwrap();
    }
}

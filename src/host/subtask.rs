//! OS subprocess subtasks (`marqdo run` children); parent exit kills all children.

use std::process::{Command, Stdio};

use crate::host::HostContext;
use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

fn child_id(_ctx: &HostContext, id: &Value) -> Result<u64, String> {
    match id {
        Value::Int(n) if *n > 0 => Ok(*n as u64),
        _ => Err("subtask id must be positive int".into()),
    }
}

pub fn kill_all(ctx: &mut HostContext) {
    for (_, mut child) in ctx.subtasks.drain() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

pub fn spawn(
    ctx: &mut HostContext,
    path: &Value,
    args: Option<&Value>,
) -> Result<Value, String> {
    if !ctx.caps.exec {
        return Err("subtask spawn disabled (exec capability off)".into());
    }
    let rel = as_text(path, "path")?;
    let file = if std::path::Path::new(rel).is_absolute() {
        std::path::PathBuf::from(rel)
    } else {
        ctx.cwd.join(rel)
    };
    let exe = std::env::current_exe().map_err(|e| format!("subtask: current_exe: {e}"))?;
    let mut cmd = Command::new(&exe);
    cmd.arg("run").arg(&file);
    cmd.current_dir(&ctx.cwd);
    cmd.stdin(Stdio::null());
    if let Some(Value::List(items)) = args {
        for item in items {
            cmd.arg(as_text(item, "arg")?);
        }
    }
    let child = cmd
        .spawn()
        .map_err(|e| format!("subtask spawn {}: {e}", file.display()))?;
    ctx.subtask_seq = ctx.subtask_seq.wrapping_add(1);
    let id = ctx.subtask_seq;
    ctx.subtasks.insert(id, child);
    Ok(Value::Int(id as i64))
}

pub fn poll(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = child_id(ctx, id)?;
    let child = ctx
        .subtasks
        .get_mut(&id)
        .ok_or_else(|| format!("subtask: unknown id {id}"))?;
    match child.try_wait().map_err(|e| format!("subtask poll: {e}"))? {
        None => Ok(Value::Map(vec![("status".into(), Value::Text("running".into()))])),
        Some(status) => {
            let code = status.code().unwrap_or(-1);
            let st = if status.success() {
                "done"
            } else {
                "failed"
            };
            Ok(Value::Map(vec![
                ("status".into(), Value::Text(st.into())),
                ("code".into(), Value::Int(code as i64)),
            ]))
        }
    }
}

pub fn join(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = child_id(ctx, id)?;
    let mut child = ctx
        .subtasks
        .remove(&id)
        .ok_or_else(|| format!("subtask: unknown id {id}"))?;
    let status = child
        .wait()
        .map_err(|e| format!("subtask join: {e}"))?;
    Ok(Value::Int(status.code().unwrap_or(-1) as i64))
}

pub fn kill(ctx: &mut HostContext, id: &Value) -> Result<Value, String> {
    let id = child_id(ctx, id)?;
    let mut child = ctx
        .subtasks
        .remove(&id)
        .ok_or_else(|| format!("subtask: unknown id {id}"))?;
    child.kill().map_err(|e| format!("subtask kill: {e}"))?;
    let _ = child.wait();
    Ok(Value::None)
}

pub fn wait_all(ctx: &mut HostContext) -> Result<Value, String> {
    let ids: Vec<u64> = ctx.subtasks.keys().copied().collect();
    let mut codes = Vec::new();
    for id in ids {
        let code = join(ctx, &Value::Int(id as i64))?;
        codes.push(code);
    }
    Ok(Value::List(codes))
}

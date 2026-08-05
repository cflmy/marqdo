//! System / process host primitives.

use std::process::Command;

use crate::host::HostContext;
use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

fn as_i64(v: &Value, label: &str) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        _ => Err(format!("{label} must be int")),
    }
}

pub fn env_get(name: &Value) -> Result<Value, String> {
    let key = as_text(name, "name")?;
    Ok(match std::env::var(key) {
        Ok(v) => Value::Text(v),
        Err(_) => Value::None,
    })
}

pub fn env_set(name: &Value, value: &Value) -> Result<Value, String> {
    let key = as_text(name, "name")?;
    let val = as_text(value, "value")?;
    std::env::set_var(key, val);
    Ok(Value::None)
}

pub fn args(ctx: &HostContext) -> Result<Value, String> {
    Ok(Value::List(
        ctx.argv.iter().map(|s| Value::Text(s.clone())).collect(),
    ))
}

pub fn cwd(ctx: &HostContext) -> Result<Value, String> {
    Ok(Value::Text(ctx.cwd.display().to_string()))
}

pub fn exit(ctx: &HostContext, code: &Value) -> Result<Value, String> {
    let c = as_i64(code, "code")?;
    if ctx.soft_side_effects {
        return Err(format!("exit {c} (soft mode does not terminate process)"));
    }
    std::process::exit(c as i32);
}

pub fn exec(ctx: &HostContext, cmd: &Value, args: Option<&Value>) -> Result<Value, String> {
    if !ctx.allow_exec() {
        return Err("exec denied by host policy".into());
    }
    let cmd = as_text(cmd, "cmd")?;
    let mut command = Command::new(cmd);
    command.current_dir(&ctx.cwd);
    if let Some(a) = args {
        match a {
            Value::List(xs) => {
                for x in xs {
                    command.arg(x.as_display());
                }
            }
            Value::Text(s) => {
                command.arg(s);
            }
            _ => return Err("exec args must be list or text".into()),
        }
    }
    let status = command
        .status()
        .map_err(|e| format!("exec `{cmd}`: {e}"))?;
    Ok(Value::Int(status.code().unwrap_or(1) as i64))
}

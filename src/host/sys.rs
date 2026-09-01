//! System / process host primitives.

#[cfg(feature = "exec-host")]
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

/// Load KEY=VALUE pairs from a `.env` file into the process environment.
/// Existing variables are **not** overridden. Default path: `.env` under host cwd.
pub fn dotenv_load(ctx: &HostContext, path: Option<&Value>) -> Result<Value, String> {
    let rel = match path {
        None | Some(Value::None) => ".env".to_string(),
        Some(v) => as_text(v, "path")?.to_string(),
    };
    let file = if std::path::Path::new(&rel).is_absolute() {
        std::path::PathBuf::from(&rel)
    } else {
        ctx.cwd.join(&rel)
    };
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Value::Int(0)),
        Err(e) => {
            return Err(format!("dotenv_load {}: {e}", file.display()));
        }
    };
    let mut loaded = 0i64;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line).trim();
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = k.trim();
        if key.is_empty() {
            continue;
        }
        if std::env::var_os(key).is_some() {
            continue;
        }
        let mut val = v.trim();
        if (val.starts_with('"') && val.ends_with('"'))
            || (val.starts_with('\'') && val.ends_with('\''))
        {
            val = &val[1..val.len() - 1];
        }
        std::env::set_var(key, val);
        loaded += 1;
    }
    Ok(Value::Int(loaded))
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
    #[cfg(target_arch = "wasm32")]
    {
        return Err(format!("exit {c} unavailable in browser wasm"));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::process::exit(c as i32);
    }
}

pub fn exec(ctx: &HostContext, cmd: &Value, args: Option<&Value>) -> Result<Value, String> {
    #[cfg(not(feature = "exec-host"))]
    {
        let _ = (ctx, cmd, args);
        return Err("exec unavailable in browser wasm".into());
    }
    #[cfg(feature = "exec-host")]
    {
    if !ctx.allow_exec() {
        return Err("exec denied by host policy".into());
    }
    let cmd = as_text(cmd, "cmd")?;
    let mut command = Command::new(cmd);
    command.current_dir(&ctx.cwd);
    match args {
        None | Some(Value::None) => {}
        Some(Value::List(xs)) => {
            for x in xs {
                command.arg(x.as_display());
            }
        }
        Some(Value::Text(s)) => {
            command.arg(s);
        }
        Some(_) => return Err("exec args must be list or text".into()),
    }
    let status = command
        .status()
        .map_err(|e| format!("exec `{cmd}`: {e}"))?;
    Ok(Value::Int(status.code().unwrap_or(1) as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::HostContext;
    use std::io::Write;

    #[test]
    fn dotenv_loads_without_override() {
        let dir = std::env::temp_dir().join(format!("marqdo-dotenv-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.env");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "# comment").unwrap();
            writeln!(f, "MARQDO_DOTENV_A=one").unwrap();
            writeln!(f, "export MARQDO_DOTENV_B=\"two\"").unwrap();
        }
        std::env::set_var("MARQDO_DOTENV_A", "keep");
        std::env::remove_var("MARQDO_DOTENV_B");
        let mut ctx = HostContext::default();
        ctx.cwd = dir.clone();
        let n = dotenv_load(&ctx, Some(&Value::Text("sample.env".into()))).unwrap();
        assert_eq!(n, Value::Int(1));
        assert_eq!(std::env::var("MARQDO_DOTENV_A").unwrap(), "keep");
        assert_eq!(std::env::var("MARQDO_DOTENV_B").unwrap(), "two");
        assert_eq!(
            dotenv_load(&ctx, Some(&Value::Text("missing.env".into()))).unwrap(),
            Value::Int(0)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

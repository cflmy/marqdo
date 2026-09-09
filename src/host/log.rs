//! Log level gate for Mid M6 (`lib/log`).

use crate::host::HostContext;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug = 10,
    Info = 20,
    Warn = 30,
    Error = 40,
}

impl Level {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "debug" | "调试" => Ok(Self::Debug),
            "info" | "信息" => Ok(Self::Info),
            "warn" | "warning" | "警告" => Ok(Self::Warn),
            "error" | "错误" => Ok(Self::Error),
            _ => Err(format!("log: unknown level {s:?}")),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

pub fn set_level(ctx: &mut HostContext, level: &Value) -> Result<Value, String> {
    let s = as_text(level, "level")?;
    ctx.log_min_level = Level::parse(s)? as u8;
    Ok(Value::None)
}

/// Return a ready-to-print line, or `None` if filtered out.
/// Optional `fields` map is appended as `key=value` pairs (stable map order).
pub fn line(
    ctx: &HostContext,
    level: &Value,
    text: &Value,
    fields: Option<&Value>,
) -> Result<Value, String> {
    let lvl = Level::parse(as_text(level, "level")?)?;
    let msg = as_text(text, "text")?;
    if (lvl as u8) < ctx.log_min_level {
        return Ok(Value::None);
    }
    let mut out = format!("{} {}", lvl.label(), msg);
    match fields {
        None | Some(Value::None) => {}
        Some(Value::Map(entries)) => {
            for (k, v) in entries {
                out.push(' ');
                out.push_str(k);
                out.push('=');
                out.push_str(&v.as_display());
            }
        }
        Some(_) => return Err("log: fields must be a map".into()),
    }
    Ok(Value::Text(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_debug() {
        let mut ctx = HostContext::default();
        assert_eq!(ctx.log_min_level, Level::Info as u8);
        let filtered = line(
            &ctx,
            &Value::Text("debug".into()),
            &Value::Text("x".into()),
            None,
        )
        .unwrap();
        assert_eq!(filtered, Value::None);
        set_level(&mut ctx, &Value::Text("debug".into())).unwrap();
        let shown = line(
            &ctx,
            &Value::Text("debug".into()),
            &Value::Text("x".into()),
            None,
        )
        .unwrap();
        assert_eq!(shown, Value::Text("DEBUG x".into()));
        let with = line(
            &ctx,
            &Value::Text("info".into()),
            &Value::Text("hi".into()),
            Some(&Value::Map(vec![
                ("user".into(), Value::Text("a".into())),
                ("n".into(), Value::Int(1)),
            ])),
        )
        .unwrap();
        assert_eq!(with, Value::Text("INFO hi user=a n=1".into()));
    }
}

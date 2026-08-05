//! Time host primitives.

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::host::HostContext;
use crate::value::Value;

pub fn now_unix() -> Result<Value, String> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("now_unix: {e}"))?
        .as_secs();
    Ok(Value::Int(secs as i64))
}

pub fn now_ms() -> Result<Value, String> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("now_ms: {e}"))?
        .as_millis();
    Ok(Value::Int(ms as i64))
}

fn as_i64(v: &Value, label: &str) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        _ => Err(format!("{label} must be int")),
    }
}

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

pub fn format_time(unix: &Value, pattern: &Value) -> Result<Value, String> {
    let secs = as_i64(unix, "unix")?;
    let pat = as_text(pattern, "pattern")?;
    let dt = DateTime::<Utc>::from_timestamp(secs, 0)
        .ok_or_else(|| format!("invalid unix timestamp: {secs}"))?;
    Ok(Value::Text(dt.format(pat).to_string()))
}

pub fn parse_time(text: &Value, pattern: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let pat = as_text(pattern, "pattern")?;
    let dt = NaiveDateTime::parse_from_str(t, pat).map_err(|e| format!("parse_time: {e}"))?;
    Ok(Value::Int(dt.and_utc().timestamp()))
}

pub fn sleep_ms(ctx: &HostContext, ms: &Value) -> Result<Value, String> {
    let n = as_i64(ms, "ms")?;
    if n < 0 {
        return Err("sleep_ms must be >= 0".into());
    }
    let n = n as u64;
    if let Some(lim) = ctx.sleep_limit_ms {
        if n > lim {
            return Err(format!("sleep_ms {n} exceeds limit {lim}"));
        }
    }
    if n > 0 {
        thread::sleep(Duration::from_millis(n));
    }
    Ok(Value::None)
}

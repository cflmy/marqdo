//! Calendar / timezone subset for Mid2 M7 (`lib/datetime`).
//!
//! Moments are plain maps: `{unix, zone, iso}`. Zone is a fixed offset (`+08:00`)
//! or alias (`UTC`); a few fixed-offset IANA names are accepted. Full tzdb is out of scope.

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::value::Value;

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

fn optional_i64(v: Option<&Value>, label: &str) -> Result<i64, String> {
    match v {
        None | Some(Value::None) => Ok(0),
        Some(x) => as_i64(x, label),
    }
}

fn format_offset(off: FixedOffset) -> String {
    let secs = off.local_minus_utc();
    let sign = if secs >= 0 { '+' } else { '-' };
    let abs = secs.abs();
    let h = abs / 3600;
    let m = (abs % 3600) / 60;
    format!("{sign}{h:02}:{m:02}")
}

fn parse_zone(s: &str) -> Result<FixedOffset, String> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("utc") || s.eq_ignore_ascii_case("gmt") || s == "Z" {
        return FixedOffset::east_opt(0).ok_or_else(|| "invalid UTC offset".into());
    }
    // Common fixed-offset IANA aliases (no DST).
    let aliased = match s {
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Taipei" | "Asia/Singapore" => Some(8 * 3600),
        "Asia/Tokyo" | "Asia/Seoul" => Some(9 * 3600),
        "Asia/Kolkata" | "Asia/Calcutta" => Some(5 * 3600 + 30 * 60),
        "Europe/Moscow" => Some(3 * 3600),
        "America/Phoenix" => Some(-7 * 3600),
        "Pacific/Honolulu" => Some(-10 * 3600),
        _ => None,
    };
    if let Some(secs) = aliased {
        return FixedOffset::east_opt(secs).ok_or_else(|| format!("invalid zone: {s}"));
    }
    let (sign, rest) = if let Some(r) = s.strip_prefix('+') {
        (1i32, r)
    } else if let Some(r) = s.strip_prefix('-') {
        (-1i32, r)
    } else {
        return Err(format!(
            "unknown zone {s:?}; use +HH:MM / -HH:MM / UTC or a fixed-offset alias (e.g. Asia/Shanghai)"
        ));
    };
    let (hh, mm) = if let Some((h, m)) = rest.split_once(':') {
        (
            h.parse::<i32>()
                .map_err(|_| format!("bad zone hour in {s}"))?,
            m.parse::<i32>()
                .map_err(|_| format!("bad zone minute in {s}"))?,
        )
    } else if rest.len() == 4 && rest.chars().all(|c| c.is_ascii_digit()) {
        (
            rest[..2]
                .parse::<i32>()
                .map_err(|_| format!("bad zone hour in {s}"))?,
            rest[2..]
                .parse::<i32>()
                .map_err(|_| format!("bad zone minute in {s}"))?,
        )
    } else if rest.len() == 2 && rest.chars().all(|c| c.is_ascii_digit()) {
        (
            rest.parse::<i32>()
                .map_err(|_| format!("bad zone hour in {s}"))?,
            0,
        )
    } else {
        return Err(format!("bad zone offset {s}"));
    };
    if !(0..=14).contains(&hh) || !(0..60).contains(&mm) {
        return Err(format!("zone offset out of range: {s}"));
    }
    let secs = sign * (hh * 3600 + mm * 60);
    FixedOffset::east_opt(secs).ok_or_else(|| format!("invalid zone offset: {s}"))
}

fn moment(unix: i64, zone: FixedOffset) -> Result<Value, String> {
    let dt = zone
        .timestamp_opt(unix, 0)
        .single()
        .ok_or_else(|| format!("invalid unix timestamp: {unix}"))?;
    Ok(Value::Map(vec![
        ("unix".into(), Value::Int(unix)),
        ("zone".into(), Value::Text(format_offset(zone))),
        (
            "iso".into(),
            Value::Text(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        ),
    ]))
}

fn zone_of_moment(m: &[(String, Value)]) -> Result<FixedOffset, String> {
    for (k, v) in m {
        if k == "zone" {
            return parse_zone(as_text(v, "zone")?);
        }
    }
    Ok(FixedOffset::east_opt(0).unwrap())
}

fn unix_of_value(v: &Value) -> Result<(i64, FixedOffset), String> {
    match v {
        Value::Int(n) => Ok((*n, FixedOffset::east_opt(0).unwrap())),
        Value::Map(entries) => {
            let mut unix = None;
            for (k, val) in entries {
                if k == "unix" {
                    unix = Some(as_i64(val, "unix")?);
                }
            }
            let unix = unix.ok_or_else(|| "datetime map needs unix".to_string())?;
            Ok((unix, zone_of_moment(entries)?))
        }
        Value::Text(s) => {
            // Treat bare ISO/RFC3339 text as a moment.
            let parsed = parse_text(s, None)?;
            unix_of_value(&parsed)
        }
        _ => Err("datetime needs int unix, moment map, or ISO text".into()),
    }
}

pub fn now() -> Result<Value, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return Err("datetime.now unavailable in browser wasm (no clock bridge yet)".into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let secs = Utc::now().timestamp();
        moment(secs, FixedOffset::east_opt(0).unwrap())
    }
}

pub fn from_unix(unix: &Value, zone: Option<&Value>) -> Result<Value, String> {
    let secs = as_i64(unix, "unix")?;
    let off = match zone {
        None | Some(Value::None) => FixedOffset::east_opt(0).unwrap(),
        Some(z) => parse_zone(as_text(z, "zone")?)?,
    };
    moment(secs, off)
}

pub fn to_unix(dt: &Value) -> Result<Value, String> {
    let (secs, _) = unix_of_value(dt)?;
    Ok(Value::Int(secs))
}

fn parse_text(text: &str, pattern: Option<&str>) -> Result<Value, String> {
    if let Some(pat) = pattern {
        if pat.eq_ignore_ascii_case("rfc3339") || pat.eq_ignore_ascii_case("iso") {
            return parse_rfc3339(text);
        }
        if let Ok(dt) = DateTime::parse_from_str(text, pat) {
            return moment(dt.timestamp(), *dt.offset());
        }
        let naive = NaiveDateTime::parse_from_str(text, pat)
            .or_else(|_| {
                NaiveDate::parse_from_str(text, pat).map(|d| d.and_hms_opt(0, 0, 0).unwrap())
            })
            .map_err(|e| format!("datetime.parse: {e}"))?;
        return moment(naive.and_utc().timestamp(), FixedOffset::east_opt(0).unwrap());
    }
    if let Ok(v) = parse_rfc3339(text) {
        return Ok(v);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S") {
        return moment(dt.and_utc().timestamp(), FixedOffset::east_opt(0).unwrap());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S") {
        return moment(dt.and_utc().timestamp(), FixedOffset::east_opt(0).unwrap());
    }
    if let Ok(d) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0).unwrap();
        return moment(dt.and_utc().timestamp(), FixedOffset::east_opt(0).unwrap());
    }
    Err(format!(
        "datetime.parse: unrecognized {text:?}; pass pattern= or use RFC3339 / YYYY-MM-DD"
    ))
}

fn parse_rfc3339(text: &str) -> Result<Value, String> {
    let dt = DateTime::parse_from_rfc3339(text).map_err(|e| format!("datetime.parse: {e}"))?;
    moment(dt.timestamp(), *dt.offset())
}

pub fn parse(text: &Value, pattern: Option<&Value>) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let pat = match pattern {
        None | Some(Value::None) => None,
        Some(p) => Some(as_text(p, "pattern")?),
    };
    parse_text(t, pat)
}

pub fn format(dt: &Value, style: Option<&Value>) -> Result<Value, String> {
    let (secs, zone) = unix_of_value(dt)?;
    let local = zone
        .timestamp_opt(secs, 0)
        .single()
        .ok_or_else(|| format!("invalid unix timestamp: {secs}"))?;
    let style = match style {
        None | Some(Value::None) => "rfc3339",
        Some(s) => as_text(s, "style")?,
    };
    let out = match style {
        "rfc3339" | "iso" => local.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "date" => local.format("%Y-%m-%d").to_string(),
        "time" => local.format("%H:%M:%S").to_string(),
        other => local.format(other).to_string(),
    };
    Ok(Value::Text(out))
}

pub fn add(
    dt: &Value,
    days: Option<&Value>,
    hours: Option<&Value>,
    minutes: Option<&Value>,
    seconds: Option<&Value>,
) -> Result<Value, String> {
    let (secs, zone) = unix_of_value(dt)?;
    let days = optional_i64(days, "days")?;
    let hours = optional_i64(hours, "hours")?;
    let minutes = optional_i64(minutes, "minutes")?;
    let seconds = optional_i64(seconds, "seconds")?;
    let delta = Duration::days(days)
        + Duration::hours(hours)
        + Duration::minutes(minutes)
        + Duration::seconds(seconds);
    let new_secs = secs
        .checked_add(delta.num_seconds())
        .ok_or_else(|| "datetime.add overflow".to_string())?;
    moment(new_secs, zone)
}

pub fn in_zone(dt: &Value, zone: &Value) -> Result<Value, String> {
    let (secs, _) = unix_of_value(dt)?;
    let off = parse_zone(as_text(zone, "zone")?)?;
    moment(secs, off)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_unix_utc_iso() {
        let m = from_unix(&Value::Int(0), None).unwrap();
        let Value::Map(entries) = m else { panic!() };
        let iso = entries.iter().find(|(k, _)| k == "iso").unwrap().1.clone();
        assert_eq!(iso, Value::Text("1970-01-01T00:00:00Z".into()));
    }

    #[test]
    fn shanghai_alias() {
        let m = from_unix(&Value::Int(0), Some(&Value::Text("Asia/Shanghai".into()))).unwrap();
        let Value::Map(entries) = m else { panic!() };
        let iso = entries.iter().find(|(k, _)| k == "iso").unwrap().1.clone();
        assert_eq!(iso, Value::Text("1970-01-01T08:00:00+08:00".into()));
    }

    #[test]
    fn parse_rfc3339_roundtrip() {
        let m = parse(
            &Value::Text("2020-01-02T03:04:05+08:00".into()),
            None,
        )
        .unwrap();
        let u = to_unix(&m).unwrap();
        assert_eq!(u, Value::Int(1_577_905_445));
    }

    #[test]
    fn add_day() {
        let m = from_unix(&Value::Int(0), None).unwrap();
        let n = add(&m, Some(&Value::Int(1)), None, None, None).unwrap();
        assert_eq!(to_unix(&n).unwrap(), Value::Int(86_400));
    }
}

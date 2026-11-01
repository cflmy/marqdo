//! JSON host primitives + Value↔JSON mapping.
//!
//! Integer gate matches Lean \(V_{\mathsf{safe}}\) (`doc/design/v-safe-json.md`):
//! JSON integers with `|n| > 2^53-1` refuse on encode and decode.

use crate::value::Value;

/// JavaScript `Number.MAX_SAFE_INTEGER` / Lean `ForeignJSON.maxSafe`.
pub const MAX_SAFE_INT: i64 = 9_007_199_254_740_991; // 2^53 - 1

pub fn in_safe_int(n: i64) -> bool {
    (-MAX_SAFE_INT..=MAX_SAFE_INT).contains(&n)
}

fn ood_int_err(n: impl std::fmt::Display) -> String {
    format!("json: integer outside V_safe (±(2^53-1)): {n}")
}

pub fn parse(text: &Value) -> Result<Value, String> {
    let s = match text {
        Value::Text(s) => s.as_str(),
        _ => return Err("json parse needs text".into()),
    };
    let v: serde_json::Value =
        serde_json::from_str(s).map_err(|e| format!("json parse: {e}"))?;
    json_to_value(&v)
}

pub fn stringify(value: &Value, indent: Option<&Value>) -> Result<Value, String> {
    let j = value_to_json(value)?;
    let pretty = match indent {
        None => false,
        Some(Value::Int(n)) => *n > 0,
        Some(Value::Bool(b)) => *b,
        Some(Value::None) => false,
        Some(_) => return Err("json stringify indent must be int or bool".into()),
    };
    let s = if pretty {
        serde_json::to_string_pretty(&j).map_err(|e| format!("json stringify: {e}"))?
    } else {
        serde_json::to_string(&j).map_err(|e| format!("json stringify: {e}"))?
    };
    Ok(Value::Text(s))
}

pub fn get(value: &Value, key: &Value) -> Result<Value, String> {
    let k = match key {
        Value::Text(s) => s.as_str(),
        _ => return Err("json get key must be text".into()),
    };
    match value {
        Value::Map(entries) => Ok(entries
            .iter()
            .find(|(kk, _)| kk == k)
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::None)),
        _ => Err("json get needs map".into()),
    }
}

pub fn keys(value: &Value) -> Result<Value, String> {
    match value {
        Value::Map(entries) => Ok(Value::List(
            entries.iter().map(|(k, _)| Value::Text(k.clone())).collect(),
        )),
        _ => Err("json keys needs map".into()),
    }
}

/// JSON-encode a text value including surrounding quotes (for building request bodies).
pub fn quote(text: &Value) -> Result<Value, String> {
    let s = match text {
        Value::Text(s) => s.as_str(),
        _ => return Err("json quote needs text".into()),
    };
    serde_json::to_string(s)
        .map(Value::Text)
        .map_err(|e| format!("json quote: {e}"))
}

pub(crate) fn json_to_value(v: &serde_json::Value) -> Result<Value, String> {
    Ok(match v {
        serde_json::Value::Null => Value::None,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if !in_safe_int(i) {
                    return Err(ood_int_err(i));
                }
                Value::Int(i)
            } else if let Some(u) = n.as_u64() {
                // Always > i64::MAX ⇒ outside V_safe; never coerce via f64/Text.
                return Err(ood_int_err(u));
            } else if let Some(f) = n.as_f64() {
                // Host extension: non-integer JSON numbers → Num (not in Lean SafeVal).
                Value::Num(f)
            } else {
                return Err(format!("json: unsupported number: {n}"));
            }
        }
        serde_json::Value::String(s) => Value::Text(s.clone()),
        serde_json::Value::Array(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for x in xs {
                out.push(json_to_value(x)?);
            }
            Value::List(out)
        }
        serde_json::Value::Object(map) => {
            let mut entries = Vec::with_capacity(map.len());
            for (k, v) in map {
                entries.push((k.clone(), json_to_value(v)?));
            }
            Value::Map(entries)
        }
    })
}

pub(crate) fn value_to_json(v: &Value) -> Result<serde_json::Value, String> {
    Ok(match v {
        Value::None => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => {
            if !in_safe_int(*n) {
                return Err(ood_int_err(*n));
            }
            serde_json::Value::Number((*n).into())
        }
        Value::Num(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::List(xs) => {
            let mut arr = Vec::with_capacity(xs.len());
            for x in xs {
                arr.push(value_to_json(x)?);
            }
            serde_json::Value::Array(arr)
        }
        Value::Map(entries) => {
            // Column-table maps (all fields lists): refuse unequal list lengths.
            let list_lens: Vec<usize> = entries
                .iter()
                .filter_map(|(_, v)| match v {
                    Value::List(xs) => Some(xs.len()),
                    _ => None,
                })
                .collect();
            if list_lens.len() == entries.len() && list_lens.len() >= 2 {
                let n0 = list_lens[0];
                if list_lens.iter().any(|&n| n != n0) {
                    return Err(
                        "json: table column arity mismatch (unequal list lengths)".into(),
                    );
                }
            }
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k.clone(), value_to_json(v)?);
            }
            serde_json::Value::Object(map)
        }
        Value::Formula(e) => match e {
            crate::formula::Expr::Matrix { rows, .. } => {
                let mut arr = Vec::with_capacity(rows.len());
                for row in rows {
                    let mut cells = Vec::with_capacity(row.len());
                    for &n in row {
                        cells.push(
                            serde_json::Number::from_f64(n)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null),
                        );
                    }
                    arr.push(serde_json::Value::Array(cells));
                }
                serde_json::Value::Array(arr)
            }
            other => {
                let s = crate::formula::simplify(other);
                if let crate::formula::Expr::Num(n) = s {
                    serde_json::Number::from_f64(n)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                } else {
                    serde_json::Value::String(e.as_display())
                }
            }
        },
        Value::Code(c) => serde_json::Value::String(c.source.clone()),
        Value::Secret(_) => serde_json::Value::String("<secret>".into()),
        Value::Handle(_) => {
            return Err("json: session handle cannot be serialized".into());
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_escapes_text() {
        let q = quote(&Value::Text("a\"b".into())).unwrap();
        assert_eq!(q, Value::Text("\"a\\\"b\"".into()));
    }

    #[test]
    fn roundtrip_max_safe() {
        for n in [MAX_SAFE_INT, -MAX_SAFE_INT, 0_i64, 1, -1] {
            let j = value_to_json(&Value::Int(n)).unwrap();
            let v = json_to_value(&j).unwrap();
            assert_eq!(v, Value::Int(n));
        }
    }

    #[test]
    fn refuse_ood_encode() {
        for n in [
            MAX_SAFE_INT + 1,
            -MAX_SAFE_INT - 1,
            1_i64 << 53,                 // 2^53
            1_i64 << 60,                 // 2^60
            (1_i64 << 60) + 1,           // 2^60+1
        ] {
            let err = value_to_json(&Value::Int(n)).unwrap_err();
            assert!(
                err.contains("outside V_safe"),
                "encode {n}: {err}"
            );
        }
    }

    #[test]
    fn refuse_ood_decode() {
        for n in [
            MAX_SAFE_INT + 1,
            -MAX_SAFE_INT - 1,
            1_i64 << 53,
            1_i64 << 60,
            (1_i64 << 60) + 1,
        ] {
            let raw = format!("{n}");
            let j: serde_json::Value = serde_json::from_str(&raw).unwrap();
            let err = json_to_value(&j).unwrap_err();
            assert!(
                err.contains("outside V_safe"),
                "decode {n}: {err}"
            );
        }
    }

    #[test]
    fn refuse_nested_ood() {
        let j: serde_json::Value =
            serde_json::from_str("{\"a\":[1152921504606846977]}").unwrap();
        let err = json_to_value(&j).unwrap_err();
        assert!(err.contains("outside V_safe"), "{err}");
    }

    #[test]
    fn decimal_string_ok() {
        let j = serde_json::Value::String("1152921504606846977".into());
        assert_eq!(
            json_to_value(&j).unwrap(),
            Value::Text("1152921504606846977".into())
        );
    }

    #[test]
    fn float_host_extension_ok() {
        let j: serde_json::Value = serde_json::from_str("1.5").unwrap();
        match json_to_value(&j).unwrap() {
            Value::Num(f) => assert!((f - 1.5).abs() < 1e-12),
            other => panic!("expected Num, got {other:?}"),
        }
    }

    #[test]
    fn refuse_u64_above_i64_max() {
        // 2^63 as JSON integer: as_i64 fails, as_u64 succeeds.
        let j: serde_json::Value = serde_json::from_str("9223372036854775808").unwrap();
        let err = json_to_value(&j).unwrap_err();
        assert!(err.contains("outside V_safe"), "{err}");
    }

    #[test]
    fn refuse_session_handle_encode() {
        let err = value_to_json(&Value::Handle("s1".into())).unwrap_err();
        assert!(
            err.contains("session handle cannot be serialized"),
            "{err}"
        );
    }

    #[test]
    fn refuse_unequal_column_table_encode() {
        let ragged = Value::Map(vec![
            ("a".into(), Value::List(vec![Value::Int(1), Value::Int(2)])),
            ("b".into(), Value::List(vec![Value::Int(3)])),
        ]);
        let err = value_to_json(&ragged).unwrap_err();
        assert!(err.contains("arity mismatch"), "{err}");
    }
}

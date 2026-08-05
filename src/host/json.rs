//! JSON host primitives + Value↔JSON mapping.

use crate::value::Value;

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

fn json_to_value(v: &serde_json::Value) -> Result<Value, String> {
    Ok(match v {
        serde_json::Value::Null => Value::None,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Text(n.to_string())
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

fn value_to_json(v: &Value) -> Result<serde_json::Value, String> {
    Ok(match v {
        Value::None => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::Value::Number((*n).into()),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::List(xs) => {
            let mut arr = Vec::with_capacity(xs.len());
            for x in xs {
                arr.push(value_to_json(x)?);
            }
            serde_json::Value::Array(arr)
        }
        Value::Map(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k.clone(), value_to_json(v)?);
            }
            serde_json::Value::Object(map)
        }
    })
}

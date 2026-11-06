//! Document-level observations, separate from process timing and execution logs.
use crate::value::Value;
use super::HostContext;

pub(crate) fn admit(value: &Value) -> Result<(), String> {
    match value {
        Value::None | Value::Bool(_) | Value::Text(_) => Ok(()),
        Value::Int(_) => super::json::value_to_json(value).map(|_| ()),
        Value::Num(_) => super::json::value_to_json(value).map(|_| ()),
        Value::List(xs) => { for x in xs { admit(x)?; } Ok(()) }
        Value::Map(xs) => {
            super::json::value_to_json(value)?;
            let mut keys = std::collections::HashSet::new();
            for (key, x) in xs {
                if !keys.insert(key) { return Err("oracle: duplicate record field".into()); }
                admit(x)?;
            }
            Ok(())
        }
        _ => Err("oracle: value outside V_safe".into()),
    }
}

pub(crate) fn append(ctx: &mut HostContext, channel: &str, value: &Value) -> Result<(), String> {
    admit(value)?;
    ctx.oracle_observations.push(Value::Map(vec![
        ("index".into(), Value::Int(ctx.oracle_observations.len() as i64)),
        ("channel".into(), Value::Text(channel.into())),
        ("value".into(), value.clone()),
    ]));
    Ok(())
}

/// Project a process result to its semantic observation. Success observes the
/// returned payload. Failure observes stable diagnostics, never elapsed time,
/// command paths or opaque handles. Both document and live runners use this.
pub(crate) fn eval_observation(record: &Value) -> Result<Value, String> {
    let Value::Map(fields) = record else { return Err("oracle: expected eval record".into()) };
    let get = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_,v)|v.clone()).unwrap_or(Value::None);
    let value = match get("ok") {
        Value::Bool(true) => get("result"),
        Value::Bool(false) => {
            let source = get("diagnostic_source");
            Value::Map(["ok", "code", "stdout", "stderr", "error", "timed_out", "alive"]
                .into_iter().map(|key| {
                    let value = get(key);
                    // Alpha-rename only this execution's allocated source path,
                    // only in diagnostics. Keep raw output and ordinary paths intact.
                    let value = match (key, value, &source) {
                        ("stderr" | "error", Value::Text(text), Value::Text(path)) if !path.is_empty() =>
                            Value::Text(text.replace(path, "<foreign-source>")),
                        (_, value, _) => value,
                    };
                    (key.to_string(), value)
                }).collect())
        },
        _ => return Err("oracle: eval record missing boolean ok".into()),
    };
    admit(&value)?;
    Ok(value)
}

fn validate(journal: &Value) -> Result<&[Value], String> {
    let Value::List(rows) = journal else { return Err("oracle journal must be a list".into()) };
    for (i, row) in rows.iter().enumerate() {
        let Value::Map(fields) = row else { return Err(format!("oracle step {i}: expected record")) };
        if fields.len() != 3 { return Err(format!("oracle step {i}: expected index/channel/value")); }
        let get = |k: &str| fields.iter().find(|(key,_)| key == k).map(|(_,v)|v);
        if get("index") != Some(&Value::Int(i as i64)) { return Err(format!("oracle step {i}: invalid index")); }
        if !matches!(get("channel"), Some(Value::Text(s)) if ["eval", "get", "decode"].contains(&s.as_str())) {
            return Err(format!("oracle step {i}: invalid channel"));
        }
        admit(get("value").ok_or_else(|| format!("oracle step {i}: missing value"))?)?;
    }
    Ok(rows)
}

/// Pure structural equality on admitted journals; reports the least differing index.
pub(crate) fn replay_check(expected: &Value, observed: &Value) -> Result<Value, String> {
    let a = validate(expected)?;
    let b = validate(observed)?;
    // Map field order is not part of record identity (matches JSON object semantics).
    let canonical = |v: &Value| super::json::value_to_json(v);
    let mut mismatch = None;
    for i in 0..a.len().max(b.len()) {
        if a.get(i).map(canonical).transpose()? != b.get(i).map(canonical).transpose()? {
            mismatch = Some(i);
            break;
        }
    }
    let index = mismatch.map(|i| Value::Int(i as i64)).unwrap_or(Value::None);
    Ok(Value::Map(vec![
        ("ok".into(), Value::Bool(mismatch.is_none())),
        ("index".into(), index),
        ("expected".into(), mismatch.and_then(|i| a.get(i)).cloned().unwrap_or(Value::None)),
        ("observed".into(), mismatch.and_then(|i| b.get(i)).cloned().unwrap_or(Value::None)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oracle_journal_structural_mismatch_and_length() {
        let mut ctx = HostContext::default();
        append(&mut ctx, "eval", &Value::Int(3)).unwrap();
        append(&mut ctx, "get", &Value::Int(2)).unwrap();
        let journal = Value::List(ctx.oracle_observations.clone());
        let result = replay_check(&journal, &journal).unwrap();
        assert_eq!(super::super::json::get(&result, &Value::Text("ok".into())).unwrap(), Value::Bool(true));
        let short = Value::List(ctx.oracle_observations[..1].to_vec());
        let result = replay_check(&journal, &short).unwrap();
        assert_eq!(super::super::json::get(&result, &Value::Text("index".into())).unwrap(), Value::Int(1));
        assert!(append(&mut ctx, "eval", &Value::List(vec![Value::Handle("x".into())])).is_err());
        let code = Value::Code(crate::value::CodeBlock { lang: "python".into(), source: "1".into() });
        assert!(admit(&Value::List(vec![code])).is_err());
        assert_eq!(ctx.oracle_observations.len(), 2);
    }
    #[test]
    fn diagnostic_projection_renames_only_allocated_source_identity() {
        let make = |path: &str, message: &str| Value::Map(vec![
            ("ok".into(), Value::Bool(false)),
            ("stderr".into(), Value::Text(format!("File {path}: {message}; input /data/original.csv"))),
            ("diagnostic_source".into(), Value::Text(path.into())),
        ]);
        let a = make("/tmp/allocated-a.py", "ValueError");
        let b = make("/tmp/allocated-b.py", "ValueError");
        assert_ne!(a, b, "raw diagnostics must survive");
        assert_eq!(eval_observation(&a).unwrap(), eval_observation(&b).unwrap());
        let c = make("/tmp/allocated-b.py", "TypeError");
        assert_ne!(eval_observation(&a).unwrap(), eval_observation(&c).unwrap());
        let d = make("/tmp/allocated-b.py", "ValueError /data/changed.csv");
        assert_ne!(eval_observation(&a).unwrap(), eval_observation(&d).unwrap());
    }

}

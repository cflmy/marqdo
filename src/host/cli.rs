//! CLI arg parsing for Mid M5 (`lib/cli`).

use crate::host::HostContext;
use crate::value::Value;

fn as_text_list(v: &Value) -> Result<Vec<String>, String> {
    match v {
        Value::List(xs) => {
            let mut out = Vec::new();
            for x in xs {
                match x {
                    Value::Text(s) => out.push(s.clone()),
                    _ => return Err("cli.parse: args must be list of text".into()),
                }
            }
            Ok(out)
        }
        Value::None => Ok(vec![]),
        _ => Err("cli.parse: args needs list".into()),
    }
}

/// Parse `--key value` / `--key=value` / `--flag` / positionals into a map.
/// Positionals are stored under key `_` as a list.
pub fn parse(ctx: &HostContext, args: Option<&Value>) -> Result<Value, String> {
    let argv = match args {
        None | Some(Value::None) => ctx.argv.clone(),
        Some(v) => as_text_list(v)?,
    };
    let mut map: Vec<(String, Value)> = Vec::new();
    let mut positionals: Vec<Value> = Vec::new();
    let mut i = 0usize;
    while i < argv.len() {
        let a = &argv[i];
        if let Some(rest) = a.strip_prefix("--") {
            if rest.is_empty() {
                // bare `--` — remaining are positionals
                i += 1;
                while i < argv.len() {
                    positionals.push(Value::Text(argv[i].clone()));
                    i += 1;
                }
                break;
            }
            if let Some((k, v)) = rest.split_once('=') {
                upsert(&mut map, k.to_string(), Value::Text(v.to_string()));
                i += 1;
                continue;
            }
            let key = rest.to_string();
            if i + 1 < argv.len() && !argv[i + 1].starts_with('-') {
                upsert(&mut map, key, Value::Text(argv[i + 1].clone()));
                i += 2;
            } else {
                upsert(&mut map, key, Value::Bool(true));
                i += 1;
            }
        } else if let Some(rest) = a.strip_prefix('-') {
            // short flags: -abc → a/b/c True; -k val not special-cased beyond single letter groups
            if rest.is_empty() {
                positionals.push(Value::Text(a.clone()));
                i += 1;
                continue;
            }
            if rest.len() == 1 && i + 1 < argv.len() && !argv[i + 1].starts_with('-') {
                upsert(&mut map, rest.to_string(), Value::Text(argv[i + 1].clone()));
                i += 2;
            } else {
                for ch in rest.chars() {
                    upsert(&mut map, ch.to_string(), Value::Bool(true));
                }
                i += 1;
            }
        } else {
            positionals.push(Value::Text(a.clone()));
            i += 1;
        }
    }
    if !positionals.is_empty() {
        upsert(&mut map, "_".into(), Value::List(positionals));
    }
    Ok(Value::Map(map))
}

fn upsert(map: &mut Vec<(String, Value)>, key: String, value: Value) {
    if let Some((_, slot)) = map.iter_mut().find(|(k, _)| *k == key) {
        *slot = value;
    } else {
        map.push((key, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flags() {
        let ctx = HostContext::default();
        let args = Value::List(vec![
            Value::Text("--input".into()),
            Value::Text("a.csv".into()),
            Value::Text("x".into()),
            Value::Text("--verbose".into()),
        ]);
        let m = parse(&ctx, Some(&args)).unwrap();
        match m {
            Value::Map(entries) => {
                assert!(entries
                    .iter()
                    .any(|(k, v)| k == "input" && *v == Value::Text("a.csv".into())));
                assert!(entries
                    .iter()
                    .any(|(k, v)| k == "verbose" && *v == Value::Bool(true)));
                assert!(entries
                    .iter()
                    .any(|(k, v)| k == "_" && matches!(v, Value::List(_))));
            }
            _ => panic!("map"),
        }
    }
}

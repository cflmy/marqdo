//! Field-driven collection transforms. No string evaluation or callback language.
use super::{as_text, collection::cmp_value};
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use std::collections::{HashMap, HashSet};

type Args = HashMap<String, Value>;
fn arg<'a>(b: &'a Args, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
fn text<'a>(b: &'a Args, k: &str) -> Result<&'a str, String> {
    as_text(arg(b, k)?, k)
}
fn list(v: &Value) -> Result<&[Value], String> {
    match v {
        Value::List(xs) => Ok(xs),
        _ => Err("expected list".into()),
    }
}
fn record(v: &Value) -> Result<&[(String, Value)], String> {
    match v {
        Value::Map(m) => Ok(m),
        _ => Err("expected record map".into()),
    }
}
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value, String> {
    record(v)?
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("missing field {key}"))
}
fn strings(v: &Value) -> Result<Vec<String>, String> {
    list(v)?
        .iter()
        .map(|v| as_text(v, "key").map(str::to_owned))
        .collect()
}
fn integer(v: &Value) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        _ => Err("expected integer".into()),
    }
}
fn number(v: &Value) -> Result<f64, String> {
    match v {
        Value::Int(n) => Ok(*n as f64),
        Value::Num(n) if n.is_finite() => Ok(*n),
        _ => Err("expected finite number".into()),
    }
}
fn row(fields: Vec<(&str, Value)>) -> Value {
    Value::Map(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
// Structural equality is type-sensitive and map-order-sensitive, like Value::eq.
// Fingerprints accelerate buckets; equality is still checked inside each bucket.
pub(super) fn fingerprint(v: &Value) -> String {
    match v {
        Value::Num(n) if *n == 0.0 => "Num(0)".into(),
        Value::List(xs) => format!("List{:?}", xs.iter().map(fingerprint).collect::<Vec<_>>()),
        Value::Map(xs) => format!(
            "Map{:?}",
            xs.iter()
                .map(|(k, v)| (k, fingerprint(v)))
                .collect::<Vec<_>>()
        ),
        Value::Formula(formula) => format!("Formula{}", formula.as_display()),
        _ => format!("{v:?}"),
    }
}

pub(super) fn call(_ctx: &mut HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    match id {
        HostFn::ListRange => {
            let start = integer(arg(b, "start")?)?;
            let end = integer(arg(b, "end")?)?;
            let step = integer(arg(b, "step")?)?;
            if step == 0 {
                return Err("range step cannot be zero".into());
            }
            let mut out = Vec::new();
            let mut n = start;
            while if step > 0 { n < end } else { n > end } {
                if out.len() >= 1_000_000 {
                    return Err("range exceeds 1000000 items".into());
                }
                out.push(Value::Int(n));
                match n.checked_add(step) {
                    Some(next) => n = next,
                    None => break,
                }
            }
            Ok(Value::List(out))
        }
        HostFn::ListEnumerate => {
            let start = integer(arg(b, "start")?)?;
            let out = list(arg(b, "list")?)?
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let index = start
                        .checked_add(i as i64)
                        .ok_or("enumerate index overflow")?;
                    Ok(row(vec![
                        ("index", Value::Int(index)),
                        ("value", v.clone()),
                    ]))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Value::List(out))
        }
        HostFn::ListPluck => {
            let key = text(b, "key")?;
            Ok(Value::List(
                list(arg(b, "list")?)?
                    .iter()
                    .map(|v| field(v, key).cloned())
                    .collect::<Result<_, _>>()?,
            ))
        }
        HostFn::ListProject => {
            let keys = strings(arg(b, "keys")?)?;
            if keys.iter().collect::<HashSet<_>>().len() != keys.len() {
                return Err("project keys must be unique".into());
            }
            let out = list(arg(b, "list")?)?
                .iter()
                .map(|v| {
                    keys.iter()
                        .map(|k| Ok((k.clone(), field(v, k)?.clone())))
                        .collect::<Result<Vec<_>, String>>()
                        .map(Value::Map)
                })
                .collect::<Result<_, _>>()?;
            Ok(Value::List(out))
        }
        HostFn::ListWhere => {
            let key = text(b, "key")?;
            let op = text(b, "op")?;
            let wanted = arg(b, "value")?;
            if !matches!(
                op,
                "eq" | "ne" | "gt" | "ge" | "lt" | "le" | "contains" | "in"
            ) {
                return Err(format!("unknown where operator {op}"));
            }
            let mut out = Vec::new();
            for v in list(arg(b, "list")?)? {
                let got = field(v, key)?;
                let yes = match op {
                    "eq" => got == wanted,
                    "ne" => got != wanted,
                    "gt" | "ge" | "lt" | "le" => {
                        let order = match (got, wanted) {
                            (Value::Text(a), Value::Text(b)) => a.cmp(b),
                            _ => number(got)?
                                .partial_cmp(&number(wanted)?)
                                .ok_or("unordered comparison")?,
                        };
                        match op {
                            "gt" => order.is_gt(),
                            "ge" => order.is_ge(),
                            "lt" => order.is_lt(),
                            _ => order.is_le(),
                        }
                    }
                    "contains" => match got {
                        Value::List(xs) => xs.contains(wanted),
                        Value::Text(s) => s.contains(as_text(wanted, "value")?),
                        _ => return Err("contains needs text or list".into()),
                    },
                    "in" => list(wanted)?.contains(got),
                    _ => return Err(format!("unknown where operator {op}")),
                };
                if yes {
                    out.push(v.clone())
                }
            }
            Ok(Value::List(out))
        }
        HostFn::ListGroupBy | HostFn::ListUniqueBy => {
            let key = text(b, "key")?;
            let mut groups: Vec<(Value, Vec<Value>)> = Vec::new();
            let mut index: HashMap<String, Vec<usize>> = HashMap::new();
            for v in list(arg(b, "list")?)? {
                let k = field(v, key)?;
                let fp = fingerprint(k);
                let bucket = index.entry(fp).or_default();
                if let Some(i) = bucket.iter().find(|i| groups[**i].0 == *k) {
                    groups[*i].1.push(v.clone());
                } else {
                    bucket.push(groups.len());
                    groups.push((k.clone(), vec![v.clone()]));
                }
            }
            Ok(Value::List(
                groups
                    .into_iter()
                    .map(|(key, rows)| {
                        if id == HostFn::ListUniqueBy {
                            rows.into_iter().next().unwrap()
                        } else {
                            row(vec![
                                ("key", key),
                                ("count", Value::Int(rows.len() as i64)),
                                ("rows", Value::List(rows)),
                            ])
                        }
                    })
                    .collect(),
            ))
        }
        HostFn::ListAggregate => {
            let xs = list(arg(b, "list")?)?;
            let op = text(b, "op")?;
            if op == "count" {
                return Ok(Value::Int(xs.len() as i64));
            }
            let key = arg(b, "key")?;
            let values = xs
                .iter()
                .map(|v| {
                    if matches!(key, Value::None) {
                        Ok(v)
                    } else {
                        field(v, as_text(key, "key")?)
                    }
                })
                .collect::<Result<Vec<_>, String>>()?;
            if op == "sum" && values.iter().all(|v| matches!(v, Value::Int(_))) {
                let sum = values.into_iter().try_fold(0i64, |sum, v| {
                    sum.checked_add(integer(v)?)
                        .ok_or_else(|| "sum integer overflow".to_string())
                })?;
                return Ok(Value::Int(sum));
            }
            let nums = values
                .into_iter()
                .map(number)
                .collect::<Result<Vec<_>, _>>()?;
            let n = match op {
                "sum" => nums.iter().sum(),
                "mean" if !nums.is_empty() => nums.iter().sum::<f64>() / nums.len() as f64,
                "min" if !nums.is_empty() => nums.iter().copied().fold(f64::INFINITY, f64::min),
                "max" if !nums.is_empty() => nums.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                _ => {
                    return Err(
                        "aggregate needs sum/count/min/max/mean; empty min/max/mean is invalid"
                            .into(),
                    )
                }
            };
            if !n.is_finite() {
                return Err("aggregate overflow".into());
            }
            Ok(Value::Num(n))
        }
        HostFn::ListSortKeys => {
            let keys = strings(arg(b, "keys")?)?;
            let descending = matches!(arg(b, "descending")?, Value::Bool(true));
            let mut xs = list(arg(b, "list")?)?.to_vec();
            for v in &xs {
                for k in &keys {
                    field(v, k)?;
                }
            }
            xs.sort_by(|a, b| {
                let mut order = std::cmp::Ordering::Equal;
                for k in &keys {
                    order = cmp_value(field(a, k).unwrap(), field(b, k).unwrap());
                    if !order.is_eq() {
                        break;
                    }
                }
                if descending {
                    order.reverse()
                } else {
                    order
                }
            });
            Ok(Value::List(xs))
        }
        HostFn::ListRename => {
            let names = record(arg(b, "fields")?)?;
            for (_, v) in names {
                as_text(v, "new field name")?;
            }
            let mut out = Vec::new();
            for v in list(arg(b, "list")?)? {
                let mut seen = HashSet::new();
                let mut fields = Vec::new();
                for (k, v) in record(v)? {
                    let name = names
                        .iter()
                        .find(|(old, _)| old == k)
                        .map(|(_, v)| as_text(v, "name").unwrap())
                        .unwrap_or(k);
                    if !seen.insert(name) {
                        return Err(format!("rename conflict: {name}"));
                    }
                    fields.push((name.into(), v.clone()));
                }
                out.push(Value::Map(fields));
            }
            Ok(Value::List(out))
        }
        HostFn::TableFromColumns => {
            let columns = record(arg(b, "columns")?)?;
            let n = columns
                .first()
                .map(|(_, v)| list(v).map(|xs| xs.len()))
                .transpose()?
                .unwrap_or(0);
            for (_, v) in columns {
                if list(v)?.len() != n {
                    return Err("column lengths differ".into());
                }
            }
            Ok(Value::List(
                (0..n)
                    .map(|i| {
                        Value::Map(
                            columns
                                .iter()
                                .map(|(k, v)| (k.clone(), list(v).unwrap()[i].clone()))
                                .collect(),
                        )
                    })
                    .collect(),
            ))
        }
        HostFn::TableToColumns => {
            let rows = list(arg(b, "rows")?)?;
            if rows.is_empty() {
                return Ok(Value::Map(vec![]));
            }
            let keys: Vec<_> = record(&rows[0])?.iter().map(|(k, _)| k.clone()).collect();
            for v in rows {
                let fields = record(v)?;
                if fields.len() != keys.len()
                    || keys.iter().any(|k| !fields.iter().any(|(f, _)| f == k))
                {
                    return Err("rows have different fields".into());
                }
            }
            Ok(Value::Map(
                keys.into_iter()
                    .map(|k| {
                        let values = rows.iter().map(|v| field(v, &k).unwrap().clone()).collect();
                        (k, Value::List(values))
                    })
                    .collect(),
            ))
        }
        HostFn::ListJoin => {
            let a = list(arg(b, "a")?)?;
            let b_rows = list(arg(b, "b")?)?;
            let key = text(b, "key")?;
            let right_key = text(b, "right_key")?;
            let mode = text(b, "mode")?;
            if !matches!(mode, "inner" | "left") {
                return Err("join mode must be inner or left".into());
            }
            let mut index: HashMap<String, Vec<&Value>> = HashMap::new();
            for v in b_rows {
                index
                    .entry(fingerprint(field(v, right_key)?))
                    .or_default()
                    .push(v);
            }
            let right_fields: Vec<_> = b_rows
                .first()
                .map(|v| record(v).map(|m| m.iter().map(|(k, _)| k.clone()).collect()))
                .transpose()?
                .unwrap_or_default();
            for right in b_rows {
                let fields = record(right)?;
                if fields.len() != right_fields.len()
                    || right_fields
                        .iter()
                        .any(|key| !fields.iter().any(|(k, _)| k == key))
                {
                    return Err("join right rows have different fields".into());
                }
            }
            let mut out = Vec::new();
            for left in a {
                let left_key = field(left, key)?;
                let hits = index.get(&fingerprint(left_key));
                let matches: Vec<Option<&Value>> = match hits {
                    Some(h) => {
                        let values: Vec<_> = h
                            .iter()
                            .filter(|v| field(v, right_key).is_ok_and(|key| key == left_key))
                            .map(|v| Some(*v))
                            .collect();
                        if values.is_empty() && mode == "left" {
                            vec![None]
                        } else {
                            values
                        }
                    }
                    None if mode == "left" => vec![None],
                    _ => vec![],
                };
                for hit in matches {
                    let mut fields = record(left)?.to_vec();
                    let source = hit.map(record).transpose()?;
                    for k in &right_fields {
                        if k == right_key && key == right_key {
                            continue;
                        }
                        let name = if fields.iter().any(|(f, _)| f == k) {
                            format!("right_{k}")
                        } else {
                            k.clone()
                        };
                        if fields.iter().any(|(f, _)| f == &name) {
                            return Err(format!("join field conflict: {name}"));
                        }
                        let value = source
                            .and_then(|m| m.iter().find(|(f, _)| f == k))
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::None);
                        fields.push((name, value));
                    }
                    if out.len() >= 1_000_000 {
                        return Err("join exceeds 1000000 rows".into());
                    }
                    out.push(Value::Map(fields));
                }
            }
            Ok(Value::List(out))
        }
        HostFn::TableCast => {
            let schema = record(arg(b, "schema")?)?;
            let mut out = Vec::new();
            for (i, v) in list(arg(b, "rows")?)?.iter().enumerate() {
                let mut fields = record(v)?.to_vec();
                for (k, ty) in schema {
                    let value = field(v, k)?;
                    let ty = as_text(ty, "type")?;
                    let cast = cast(value, ty).map_err(|e| format!("row {i} field {k}: {e}"))?;
                    fields.iter_mut().find(|(name, _)| name == k).unwrap().1 = cast;
                }
                out.push(Value::Map(fields));
            }
            Ok(Value::List(out))
        }
        HostFn::TableSummarize => {
            let groups = call(_ctx, HostFn::ListGroupBy, b)?;
            let field_name = text(b, "field")?;
            let op = text(b, "op")?;
            let mut out = Vec::new();
            for group in list(&groups)? {
                let rows = field(group, "rows")?;
                let mut args = HashMap::new();
                args.insert("list".into(), rows.clone());
                args.insert("key".into(), Value::Text(field_name.into()));
                args.insert("op".into(), Value::Text(op.into()));
                let value = call(_ctx, HostFn::ListAggregate, &args)?;
                out.push(row(vec![
                    ("key", field(group, "key")?.clone()),
                    ("count", field(group, "count")?.clone()),
                    ("value", value),
                ]));
            }
            Ok(Value::List(out))
        }
        HostFn::TableTryCast => {
            let answer =
                call(_ctx, HostFn::TableCast, b).map_err(|e| format!("validation_failed: {e}"));
            Ok(super::result::capture("table.cast", answer))
        }
        HostFn::TableValidate => {
            let rows = list(arg(b, "rows")?)?;
            let schema = record(arg(b, "schema")?)?;
            let mut errors = Vec::new();
            for (index, value) in rows.iter().enumerate() {
                let fields = record(value)?;
                for (key, ty) in schema {
                    let name = as_text(ty, "type")?;
                    let ty = crate::contract::Ty::parse(name);
                    let got = fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);
                    if !got.is_some_and(|v| ty.matches_value(v)) {
                        errors.push(row(vec![
                            ("row", Value::Int(index as i64)),
                            ("field", Value::Text(key.clone())),
                            ("expected", Value::Text(name.into())),
                            (
                                "actual",
                                Value::Text(
                                    got.map(crate::contract::type_of)
                                        .unwrap_or("missing")
                                        .into(),
                                ),
                            ),
                        ]));
                    }
                }
            }
            if errors.is_empty() {
                Ok(super::result::capture(
                    "table.validate",
                    Ok(arg(b, "rows")?.clone()),
                ))
            } else {
                let mut result = super::result::capture(
                    "table.validate",
                    Err("validation_failed: records do not match schema".into()),
                );
                if let Value::Map(fields) = &mut result {
                    if let Some((_, Value::Map(error))) =
                        fields.iter_mut().find(|(k, _)| k == "error")
                    {
                        error.push(("details".into(), Value::List(errors)));
                    }
                }
                Ok(result)
            }
        }
        HostFn::ListUnion | HostFn::ListIntersection | HostFn::ListDifference => {
            let left = list(arg(b, "a")?)?;
            let right = list(arg(b, "b")?)?;
            let mut index: HashMap<String, Vec<&Value>> = HashMap::new();
            for v in right {
                index.entry(fingerprint(v)).or_default().push(v)
            }
            let mut values = Vec::new();
            for v in left {
                let contains = index
                    .get(&fingerprint(v))
                    .is_some_and(|bucket| bucket.iter().any(|r| *r == v));
                if id == HostFn::ListUnion
                    || (id == HostFn::ListIntersection && contains)
                    || (id == HostFn::ListDifference && !contains)
                {
                    values.push(v.clone())
                }
            }
            if id == HostFn::ListUnion {
                values.extend_from_slice(right)
            }
            super::collection::list_unique(&Value::List(values))
        }
        _ => Err("not a collection transform".into()),
    }
}
fn cast(v: &Value, ty: &str) -> Result<Value, String> {
    match ty {
        "text" => Ok(Value::Text(v.as_display())),
        "int" => match v {
            Value::Int(_) => Ok(v.clone()),
            Value::Text(s) => s
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| "invalid integer".into()),
            _ => Err("int expects integer or text".into()),
        },
        "number" => {
            let n = match v {
                Value::Text(s) => s.parse::<f64>().map_err(|_| "invalid number")?,
                _ => number(v)?,
            };
            if !n.is_finite() {
                return Err("non-finite number".into());
            }
            Ok(Value::Num(n))
        }
        "bool" => match v {
            Value::Bool(_) => Ok(v.clone()),
            Value::Text(s) => match s.as_str() {
                "true" | "True" => Ok(Value::Bool(true)),
                "false" | "False" => Ok(Value::Bool(false)),
                _ => Err("invalid bool".into()),
            },
            _ => Err("bool expects bool or text".into()),
        },
        "any" => Ok(v.clone()),
        _ => Err(format!("unknown cast type {ty}")),
    }
}

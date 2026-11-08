//! List / map collection primitives for `lib/table` (and json compat aliases).

use crate::value::Value;

#[derive(Clone)]
enum PathSeg {
    Key(String),
    /// 1-based list index (aligned with footnote `[^1]`).
    Index1(usize),
}

fn path_segments(at: &Value) -> Result<Vec<PathSeg>, String> {
    match at {
        Value::Text(s) => Ok(vec![PathSeg::Key(s.clone())]),
        Value::Int(n) => {
            if *n < 1 {
                return Err("put path index must be >= 1 (1-based)".into());
            }
            Ok(vec![PathSeg::Index1(usize::try_from(*n).map_err(|_| "put path index exceeds target size")?)])
        }
        Value::List(xs) => {
            if xs.is_empty() {
                return Err("put path must not be empty".into());
            }
            let mut out = Vec::with_capacity(xs.len());
            for (i, seg) in xs.iter().enumerate() {
                match seg {
                    Value::Text(s) => out.push(PathSeg::Key(s.clone())),
                    Value::Int(n) => {
                        if *n < 1 {
                            return Err(format!(
                                "put path segment {i}: index must be >= 1 (1-based)"
                            ));
                        }
                        out.push(PathSeg::Index1(usize::try_from(*n).map_err(|_| "put path index exceeds target size")?));
                    }
                    _ => {
                        return Err(format!(
                            "put path segment {i}: need text key or int index"
                        ));
                    }
                }
            }
            Ok(out)
        }
        _ => Err("put at= needs text, int, or list path".into()),
    }
}

/// Deep put: `in` collection, `at` path (1-based ints), `value` leaf.
/// `in=None` + single text key → new one-entry map.
pub fn collection_put(input: &Value, at: &Value, value: &Value) -> Result<Value, String> {
    let path = path_segments(at)?;
    if matches!(input, Value::None) {
        if path.len() == 1 {
            if let PathSeg::Key(k) = &path[0] {
                return Ok(Value::Map(vec![(k.clone(), value.clone())]));
            }
        }
        return Err("put in=None only allowed with a single text key".into());
    }
    put_rec(input, &path, value)
}

fn put_rec(cur: &Value, path: &[PathSeg], value: &Value) -> Result<Value, String> {
    let Some((head, rest)) = path.split_first() else {
        return Ok(value.clone());
    };
    if rest.is_empty() {
        return put_leaf(cur, head, value);
    }
    match head {
        PathSeg::Key(k) => {
            let Value::Map(entries) = cur else {
                return Err(format!("put: expected map at key `{k}`"));
            };
            let child = entries
                .iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| format!("put: missing key `{k}` on path"))?;
            let new_child = put_rec(&child, rest, value)?;
            map_set(cur, &Value::Text(k.clone()), &new_child)
        }
        PathSeg::Index1(i1) => {
            let Value::List(xs) = cur else {
                return Err(format!("put: expected list at index {i1}"));
            };
            let i0 = i1 - 1;
            if i0 >= xs.len() {
                return Err(format!("put: list index {i1} out of range (len {})", xs.len()));
            }
            let new_child = put_rec(&xs[i0], rest, value)?;
            list_set_at(cur, i0 as i64, &new_child)
        }
    }
}

fn put_leaf(cur: &Value, head: &PathSeg, value: &Value) -> Result<Value, String> {
    match head {
        PathSeg::Key(k) => map_set(cur, &Value::Text(k.clone()), value),
        PathSeg::Index1(i1) => {
            let i0 = (*i1 as i64) - 1;
            list_set_at(cur, i0, value)
        }
    }
}

pub fn map_set(map: &Value, key: &Value, value: &Value) -> Result<Value, String> {
    let k = match key {
        Value::Text(s) => s.clone(),
        _ => return Err("map_set key must be text".into()),
    };
    match map {
        Value::Map(entries) => {
            let mut out = entries.clone();
            if let Some((_, slot)) = out.iter_mut().find(|(kk, _)| *kk == k) {
                *slot = value.clone();
            } else {
                out.push((k, value.clone()));
            }
            Ok(Value::Map(out))
        }
        Value::None => Ok(Value::Map(vec![(k, value.clone())])),
        _ => Err("map_set needs map".into()),
    }
}

pub fn map_get(map: &Value, key: &Value) -> Result<Value, String> {
    let k = match key {
        Value::Text(s) => s.as_str(),
        _ => return Err("map_get key must be text".into()),
    };
    match map {
        Value::Map(entries) => Ok(entries
            .iter()
            .find(|(kk, _)| kk == k)
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::None)),
        Value::None => Ok(Value::None),
        _ => Err("map_get needs map".into()),
    }
}

pub fn map_delete(map: &Value, key: &Value) -> Result<Value, String> {
    let k = match key {
        Value::Text(s) => s.as_str(),
        _ => return Err("map_delete key must be text".into()),
    };
    match map {
        Value::Map(entries) => {
            let out: Vec<_> = entries.iter().filter(|(kk, _)| kk != k).cloned().collect();
            Ok(Value::Map(out))
        }
        Value::None => Ok(Value::Map(vec![])),
        _ => Err("map_delete needs map".into()),
    }
}

pub fn map_has(map: &Value, key: &Value) -> Result<Value, String> {
    let k = match key {
        Value::Text(s) => s.as_str(),
        _ => return Err("map_has key must be text".into()),
    };
    match map {
        Value::Map(entries) => Ok(Value::Bool(entries.iter().any(|(kk, _)| kk == k))),
        Value::None => Ok(Value::Bool(false)),
        _ => Err("map_has needs map".into()),
    }
}

pub fn map_keys(map: &Value) -> Result<Value, String> {
    match map {
        Value::Map(entries) => Ok(Value::List(
            entries
                .iter()
                .map(|(k, _)| Value::Text(k.clone()))
                .collect(),
        )),
        Value::None => Ok(Value::List(vec![])),
        _ => Err("map_keys needs map".into()),
    }
}

pub fn map_values(map: &Value) -> Result<Value, String> {
    match map {
        Value::Map(entries) => Ok(Value::List(entries.iter().map(|(_, v)| v.clone()).collect())),
        Value::None => Ok(Value::List(vec![])),
        _ => Err("map_values needs map".into()),
    }
}

pub fn map_items(map: &Value) -> Result<Value, String> {
    match map {
        Value::Map(entries) => {
            let rows: Vec<Value> = entries
                .iter()
                .map(|(k, v)| {
                    Value::Map(vec![
                        ("key".into(), Value::Text(k.clone())),
                        ("value".into(), v.clone()),
                    ])
                })
                .collect();
            Ok(Value::List(rows))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("map_items needs map".into()),
    }
}

pub fn map_merge(left: &Value, right: &Value) -> Result<Value, String> {
    let mut out = match left {
        Value::Map(e) => e.clone(),
        Value::None => vec![],
        _ => return Err("map_merge left needs map".into()),
    };
    let right_entries = match right {
        Value::Map(e) => e,
        Value::None => return Ok(Value::Map(out)),
        _ => return Err("map_merge right needs map".into()),
    };
    for (k, v) in right_entries {
        if let Some((_, slot)) = out.iter_mut().find(|(kk, _)| kk == k) {
            *slot = v.clone();
        } else {
            out.push((k.clone(), v.clone()));
        }
    }
    Ok(Value::Map(out))
}

pub fn map_size(map: &Value) -> Result<Value, String> {
    match map {
        Value::Map(entries) => Ok(Value::Int(entries.len() as i64)),
        Value::None => Ok(Value::Int(0)),
        _ => Err("map_size needs map".into()),
    }
}

pub fn list_append(list: &Value, item: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => {
            let mut out = xs.clone();
            out.push(item.clone());
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![item.clone()])),
        _ => Err("list_append needs list".into()),
    }
}

pub fn list_prepend(list: &Value, item: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => {
            let mut out = vec![item.clone()];
            out.extend(xs.iter().cloned());
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![item.clone()])),
        _ => Err("list_prepend needs list".into()),
    }
}

pub fn list_concat(a: &Value, b: &Value) -> Result<Value, String> {
    let left = match a {
        Value::List(xs) => xs.clone(),
        Value::None => vec![],
        _ => return Err("list_concat left needs list".into()),
    };
    let right = match b {
        Value::List(xs) => xs,
        Value::None => return Ok(Value::List(left)),
        _ => return Err("list_concat right needs list".into()),
    };
    let mut out = left;
    out.extend(right.iter().cloned());
    Ok(Value::List(out))
}

pub fn list_insert(list: &Value, index: i64, item: &Value) -> Result<Value, String> {
    let mut xs = match list {
        Value::List(xs) => xs.clone(),
        Value::None => vec![],
        _ => return Err("list_insert needs list".into()),
    };
    if index < 0 || index > xs.len() as i64 {
        return Err(format!(
            "list_insert index {index} out of range (len {})",
            xs.len()
        ));
    }
    xs.insert(index as usize, item.clone());
    Ok(Value::List(xs))
}

pub fn list_set_at(list: &Value, index: i64, item: &Value) -> Result<Value, String> {
    let Value::List(xs) = list else {
        return Err("list_set_at needs list".into());
    };
    if index < 0 || index >= xs.len() as i64 {
        return Err(format!(
            "list_set_at index {index} out of range (len {})",
            xs.len()
        ));
    }
    let mut out = xs.clone();
    out[index as usize] = item.clone();
    Ok(Value::List(out))
}

pub fn list_remove_at(list: &Value, index: i64) -> Result<Value, String> {
    let Value::List(xs) = list else {
        return Err("list_remove_at needs list".into());
    };
    if index < 0 || index >= xs.len() as i64 {
        return Err(format!(
            "list_remove_at index {index} out of range (len {})",
            xs.len()
        ));
    }
    let mut out = xs.clone();
    out.remove(index as usize);
    Ok(Value::List(out))
}

pub fn list_slice(list: &Value, start: i64, end: Option<i64>) -> Result<Value, String> {
    let Value::List(xs) = list else {
        return Err("list_slice needs list".into());
    };
    let n = xs.len() as i64;
    let s = start.clamp(0, n) as usize;
    let e = end.unwrap_or(n).clamp(0, n) as usize;
    if e < s {
        return Ok(Value::List(vec![]));
    }
    Ok(Value::List(xs[s..e].to_vec()))
}

pub fn list_contains(list: &Value, item: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => Ok(Value::Bool(xs.iter().any(|x| x == item))),
        Value::None => Ok(Value::Bool(false)),
        _ => Err("list_contains needs list".into()),
    }
}

pub fn list_index_of(list: &Value, item: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => Ok(xs
            .iter()
            .position(|x| x == item)
            .map(|i| Value::Int(i as i64))
            .unwrap_or(Value::None)),
        Value::None => Ok(Value::None),
        _ => Err("list_index_of needs list".into()),
    }
}

pub fn list_reverse(list: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => {
            let mut out = xs.clone();
            out.reverse();
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("list_reverse needs list".into()),
    }
}

pub fn list_first(list: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => Ok(xs.first().cloned().unwrap_or(Value::None)),
        Value::None => Ok(Value::None),
        _ => Err("list_first needs list".into()),
    }
}

pub fn list_last(list: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => Ok(xs.last().cloned().unwrap_or(Value::None)),
        Value::None => Ok(Value::None),
        _ => Err("list_last needs list".into()),
    }
}

/// Total type order avoids cyclic comparisons between numeric and text keys.
pub(super) fn cmp_value(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    fn rank(v:&Value)->u8{match v{
        Value::Int(_)|Value::Num(_)=>0,Value::Text(_)=>1,Value::Bool(_)=>2,
        Value::None=>3,Value::List(_)=>4,Value::Map(_)=>5,Value::Formula(_)=>6,
        Value::Code(_)=>7,Value::Secret(_)=>8,Value::Handle(_)=>9,
    }}
    fn int_float(integer:i64,number:f64)->Ordering{
        if number.is_nan(){return Ordering::Less}
        if number >= -(i64::MIN as f64){return Ordering::Less}
        if number < i64::MIN as f64{return Ordering::Greater}
        let truncated=number as i64;
        match integer.cmp(&truncated){Ordering::Equal=>0.0f64.partial_cmp(&number.fract()).unwrap_or(Ordering::Equal),other=>other}
    }
    let order=rank(a).cmp(&rank(b));if !order.is_eq(){return order}
    match (a,b){
        (Value::Int(a),Value::Int(b))=>a.cmp(b),
        (Value::Num(a),Value::Num(b))=>match(a.is_nan(),b.is_nan()){
            (true,false)=>Ordering::Greater,(false,true)=>Ordering::Less,
            _=>a.partial_cmp(b).unwrap_or_else(||a.total_cmp(b)),
        },
        (Value::Int(a),Value::Num(b))=>int_float(*a,*b),
        (Value::Num(a),Value::Int(b))=>int_float(*b,*a).reverse(),
        (Value::Text(a),Value::Text(b))=>a.cmp(b),
        (Value::Bool(a),Value::Bool(b))=>a.cmp(b),
        _=>a.as_display().cmp(&b.as_display()),
    }
}

pub fn list_sort(list: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => {
            let mut out = xs.clone();
            out.sort_by(cmp_value);
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("list_sort needs list".into()),
    }
}

pub fn list_sort_by(list: &Value, key: &Value) -> Result<Value, String> {
    let key = match key {
        Value::Text(s) => s.as_str(),
        _ => return Err("sort_by key needs text".into()),
    };
    match list {
        Value::List(xs) => {
            let mut out = xs.clone();
            out.sort_by(|a, b| {
                let ka = match a {
                    Value::Map(m) => m
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|(_, v)| v)
                        .unwrap_or(&Value::None),
                    _ => &Value::None,
                };
                let kb = match b {
                    Value::Map(m) => m
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|(_, v)| v)
                        .unwrap_or(&Value::None),
                    _ => &Value::None,
                };
                cmp_value(ka, kb)
            });
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("list_sort_by needs list".into()),
    }
}

pub fn list_unique(list: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => {
            let mut out: Vec<Value> = Vec::new();
            let mut seen: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
            for x in xs {
                let bucket = seen.entry(super::collection_ops::fingerprint(x)).or_default();
                if !bucket.iter().any(|i| out[*i] == *x) {
                    bucket.push(out.len());
                    out.push(x.clone());
                }
            }
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("list_unique needs list".into()),
    }
}

pub fn list_chunk(list: &Value, size: &Value) -> Result<Value, String> {
    let n = match size {
        Value::Int(i) if *i > 0 => usize::try_from(*i).unwrap_or(usize::MAX),
        Value::Int(_) => return Err("chunk size must be positive".into()),
        _ => return Err("chunk size needs int".into()),
    };
    match list {
        Value::List(xs) => {
            let mut out = Vec::new();
            let mut i = 0;
            while i < xs.len() {
                let end = (i + n).min(xs.len());
                out.push(Value::List(xs[i..end].to_vec()));
                i = end;
            }
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("list_chunk needs list".into()),
    }
}

pub fn list_zip(a: &Value, b: &Value) -> Result<Value, String> {
    let (aa, bb) = match (a, b) {
        (Value::List(x), Value::List(y)) => (x, y),
        (Value::None, Value::List(y)) => (&Vec::new(), y),
        (Value::List(x), Value::None) => (x, &Vec::new()),
        (Value::None, Value::None) => return Ok(Value::List(vec![])),
        _ => return Err("list_zip needs two lists".into()),
    };
    let n = aa.len().min(bb.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(Value::List(vec![aa[i].clone(), bb[i].clone()]));
    }
    Ok(Value::List(out))
}

pub fn list_flatten(list: &Value) -> Result<Value, String> {
    match list {
        Value::List(xs) => {
            let mut out = Vec::new();
            for x in xs {
                match x {
                    Value::List(inner) => out.extend(inner.iter().cloned()),
                    other => out.push(other.clone()),
                }
            }
            Ok(Value::List(out))
        }
        Value::None => Ok(Value::List(vec![])),
        _ => Err("list_flatten needs list".into()),
    }
}

pub fn collection_clear(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(_) | Value::None => Ok(Value::List(vec![])),
        Value::Map(_) => Ok(Value::Map(vec![])),
        _ => Err("clear needs list or map".into()),
    }
}

fn require_i64(v: &Value, name: &str) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        _ => Err(format!("{name} must be int")),
    }
}

/// Bind helper used by dispatch for optional end=.
pub fn list_slice_bound(list: &Value, start: &Value, end: Option<&Value>) -> Result<Value, String> {
    let s = require_i64(start, "start")?;
    let e = match end {
        None | Some(Value::None) => None,
        Some(v) => Some(require_i64(v, "end")?),
    };
    list_slice(list, s, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_map_key_and_nested() {
        let m = Value::Map(vec![("a".into(), Value::Int(1))]);
        let m2 = collection_put(&m, &Value::Text("b".into()), &Value::Int(2)).unwrap();
        assert!(matches!(map_get(&m2, &Value::Text("b".into())).unwrap(), Value::Int(2)));

        let rows = Value::List(vec![Value::Map(vec![("qty".into(), Value::Int(1))])]);
        let path = Value::List(vec![Value::Int(1), Value::Text("qty".into())]);
        let rows2 = collection_put(&rows, &path, &Value::Int(9)).unwrap();
        let row = match &rows2 {
            Value::List(xs) => &xs[0],
            _ => panic!(),
        };
        assert!(matches!(
            map_get(row, &Value::Text("qty".into())).unwrap(),
            Value::Int(9)
        ));
    }

    #[test]
    fn put_none_single_key() {
        let m = collection_put(&Value::None, &Value::Text("k".into()), &Value::Text("v".into()))
            .unwrap();
        assert!(matches!(m, Value::Map(_)));
    }
}

#[cfg(test)]
mod task_scale_tests {
    use super::*;
    #[test]
    fn unique_preserves_signed_zero_nan_and_nested_equality(){
        let xs=Value::List(vec![Value::Num(-0.0),Value::Num(0.0),Value::Num(f64::NAN),Value::Num(f64::NAN),Value::Int(0),Value::List(vec![Value::Num(-0.0)]),Value::List(vec![Value::Num(0.0)])]);
        let Value::List(out)=list_unique(&xs).unwrap()else{panic!()};
        assert_eq!(out.len(),5);assert_eq!(out[0],Value::Num(-0.0));assert_eq!(out[3],Value::Int(0));
    }
    #[test]
    fn mixed_order_is_transitive_and_exact_at_integer_float_boundary(){
        let values=[Value::Int(2),Value::Int(10),Value::Text("11".into()),Value::Int(i64::MAX),Value::Num(i64::MAX as f64),Value::Num(f64::NAN),Value::Num(-0.0),Value::Bool(false),Value::None];
        for a in &values{for b in &values{for c in &values{
            if cmp_value(a,b).is_le()&&cmp_value(b,c).is_le(){assert!(cmp_value(a,c).is_le(),"{a:?} {b:?} {c:?}");}
        }}}
        assert!(cmp_value(&Value::Int(i64::MAX),&Value::Num(i64::MAX as f64)).is_lt());
    }
    #[test]
    fn bulk_unique_handles_one_hundred_thousand_values(){
        let xs=Value::List((0..100_000).map(|n|Value::Int(n%50_000)).collect());
        let Value::List(out)=list_unique(&xs).unwrap()else{panic!()};assert_eq!(out.len(),50_000);
    }
}

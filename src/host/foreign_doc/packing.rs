//! Lossless, typed, bottom-up sharing for complete history snapshots.
//! References always point backwards. Expansion costs and depth are checked
//! before building the expanded value, so a small DAG cannot expand unboundedly.
use crate::host::{json::value_to_json, oracle::admit};
use crate::value::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

const FORMAT: &str = "marqdo.history.dag";
const MAX_NODES: usize = 1_000_000;
const MAX_VALUES: usize = 8_000_000;
const MAX_BYTES: usize = 512 * 1024 * 1024;
const MAX_DEPTH: usize = 128;

fn text(s: &str) -> Value {
    Value::Text(s.into())
}
fn list(v: &Value) -> Result<&[Value], String> {
    if let Value::List(v) = v {
        Ok(v)
    } else {
        Err("snapshot DAG: expected list".into())
    }
}
fn string(v: &Value) -> Result<&str, String> {
    if let Value::Text(v) = v {
        Ok(v)
    } else {
        Err("snapshot DAG: expected text".into())
    }
}
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value, String> {
    if let Value::Map(fields) = v {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("snapshot DAG: missing {key}"))
    } else {
        Err("snapshot DAG: expected object".into())
    }
}
fn wire(v: &Value) -> Result<String, String> {
    serde_json::to_string(&value_to_json(v)?).map_err(|e| e.to_string())
}
fn checksum(root: &Value, nodes: &Value) -> Result<String, String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(wire(&Value::List(vec![root.clone(), nodes.clone()]))?.as_bytes())
    ))
}
pub fn is_packed(v: &Value) -> bool {
    field(v, "format").ok() == Some(&text(FORMAT))
}

pub fn pack(snapshot: &Value) -> Result<Value, String> {
    fn intern(
        v: &Value,
        nodes: &mut Vec<Value>,
        seen: &mut HashMap<String, usize>,
        depth: usize,
    ) -> Result<usize, String> {
        if depth > MAX_DEPTH {
            return Err("snapshot DAG: depth budget exceeded".into());
        }
        let row = match v {
            Value::None => vec![text("null")],
            Value::Bool(v) => vec![text("bool"), Value::Bool(*v)],
            Value::Int(v) => vec![text("int"), text(&v.to_string())],
            Value::Num(v) => vec![text("float"), text(&format!("{:016x}", v.to_bits()))],
            Value::Text(v) => vec![text("text"), text(v)],
            Value::List(values) => {
                let refs = values
                    .iter()
                    .map(|v| intern(v, nodes, seen, depth + 1).map(|i| Value::Int(i as i64)))
                    .collect::<Result<Vec<_>, _>>()?;
                vec![text("list"), Value::List(refs)]
            }
            Value::Map(fields) => {
                let refs = fields
                    .iter()
                    .map(|(k, v)| {
                        intern(v, nodes, seen, depth + 1)
                            .map(|i| Value::List(vec![text(k), Value::Int(i as i64)]))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                vec![text("map"), Value::List(refs)]
            }
            _ => return Err("snapshot DAG: value outside V_safe".into()),
        };
        let row = Value::List(row);
        let key = wire(&row)?;
        if let Some(index) = seen.get(&key) {
            return Ok(*index);
        }
        if nodes.len() >= MAX_NODES {
            return Err("snapshot DAG: node budget exceeded".into());
        }
        let index = nodes.len();
        nodes.push(row);
        seen.insert(key, index);
        Ok(index)
    }
    let mut nodes = Vec::new();
    let root = intern(snapshot, &mut nodes, &mut HashMap::new(), 0)?;
    admit(snapshot)?;
    let nodes = Value::List(nodes);
    let root = Value::Int(root as i64);
    let hash = checksum(&root, &nodes)?;
    let envelope = Value::Map(vec![
        ("format".into(), text(FORMAT)),
        ("version".into(), Value::Int(1)),
        ("root".into(), root),
        ("nodes".into(), nodes),
        ("sha256".into(), text(&hash)),
    ]);
    // Apply exactly the decoder's resource limits to locally produced envelopes.
    validate(&envelope)?;
    Ok(envelope)
}

enum Node {
    Atom(Value),
    List(Vec<usize>),
    Map(Vec<(String, usize)>),
}
impl Node {
    fn refs(&self) -> Vec<usize> {
        match self {
            Self::Atom(_) => Vec::new(),
            Self::List(v) => v.clone(),
            Self::Map(v) => v.iter().map(|(_, i)| *i).collect(),
        }
    }
}
fn index(v: &Value, before: usize) -> Result<usize, String> {
    match v {
        Value::Int(i) if *i >= 0 && (*i as u64) < before as u64 => Ok(*i as usize),
        _ => Err("snapshot DAG: forward/cyclic/out-of-range reference".into()),
    }
}
fn validate(envelope: &Value) -> Result<(Vec<Node>, usize), String> {
    let Value::Map(fields) = envelope else {
        return Err("snapshot DAG: expected envelope".into());
    };
    let names: HashSet<_> = fields.iter().map(|(k, _)| k.as_str()).collect();
    if fields.len() != 5
        || names != HashSet::from(["format", "version", "root", "nodes", "sha256"])
        || string(field(envelope, "format")?)? != FORMAT
        || field(envelope, "version")? != &Value::Int(1)
    {
        return Err("snapshot DAG: invalid envelope schema".into());
    }
    let rows = list(field(envelope, "nodes")?)?;
    if rows.is_empty() || rows.len() > MAX_NODES {
        return Err("snapshot DAG: node budget exceeded".into());
    }
    let root = index(field(envelope, "root")?, rows.len())?;
    if root + 1 != rows.len() {
        return Err("snapshot DAG: root must be last".into());
    }
    let expected = string(field(envelope, "sha256")?)?;
    if expected != checksum(field(envelope, "root")?, field(envelope, "nodes")?)? {
        return Err("snapshot DAG: checksum differs".into());
    }
    let mut nodes = Vec::new();
    let mut costs: Vec<(usize, usize, usize)> = Vec::new();
    let mut unique = HashSet::new();
    for (i, raw) in rows.iter().enumerate() {
        if !unique.insert(wire(raw)?) {
            return Err("snapshot DAG: duplicate dictionary node".into());
        }
        let row = list(raw)?;
        let tag = string(row.first().ok_or("snapshot DAG: empty node")?)?;
        let node = match (tag, row) {
            ("null", [_]) => Node::Atom(Value::None),
            ("bool", [_, Value::Bool(v)]) => Node::Atom(Value::Bool(*v)),
            ("text", [_, Value::Text(v)]) => Node::Atom(text(v)),
            ("int", [_, Value::Text(v)]) => {
                let n = v
                    .parse::<i64>()
                    .map_err(|_| "snapshot DAG: invalid integer")?;
                if n.to_string() != *v {
                    return Err("snapshot DAG: noncanonical integer".into());
                }
                Node::Atom(Value::Int(n))
            }
            ("float", [_, Value::Text(v)]) if v.len() == 16 => {
                let bits =
                    u64::from_str_radix(v, 16).map_err(|_| "snapshot DAG: invalid float bits")?;
                if format!("{bits:016x}") != *v {
                    return Err("snapshot DAG: noncanonical float bits".into());
                }
                Node::Atom(Value::Num(f64::from_bits(bits)))
            }
            ("list", [_, refs]) => Node::List(
                list(refs)?
                    .iter()
                    .map(|v| index(v, i))
                    .collect::<Result<_, _>>()?,
            ),
            ("map", [_, refs]) => {
                let mut keys = HashSet::new();
                let mut fields = Vec::new();
                for pair in list(refs)? {
                    let [key, value] = list(pair)? else {
                        return Err("snapshot DAG: invalid map pair".into());
                    };
                    let key = string(key)?;
                    if !keys.insert(key) {
                        return Err("snapshot DAG: duplicate map key".into());
                    }
                    fields.push((key.into(), index(value, i)?));
                }
                Node::Map(fields)
            }
            _ => return Err("snapshot DAG: invalid node tag/payload".into()),
        };
        let mut values = 1usize;
        let mut bytes = match &node {
            Node::Atom(v) => {
                admit(v)?;
                match v {
                    Value::Text(s) => s
                        .len()
                        .checked_mul(6)
                        .and_then(|n| n.checked_add(2))
                        .ok_or("snapshot DAG: size overflow")?,
                    _ => 32,
                }
            }
            Node::Map(v) => v
                .iter()
                .try_fold(2usize, |n, (k, _)| {
                    n.checked_add(k.len().checked_mul(6)?.checked_add(4)?)
                })
                .ok_or("snapshot DAG: size overflow")?,
            Node::List(_) => 2,
        };
        let mut depth = 1;
        for reference in node.refs() {
            let (count, size, height) = costs[reference];
            values = values
                .checked_add(count)
                .ok_or("snapshot DAG: expansion overflow")?;
            bytes = bytes
                .checked_add(size)
                .and_then(|n| n.checked_add(1))
                .ok_or("snapshot DAG: expansion overflow")?;
            depth = depth.max(height + 1);
        }
        if values > MAX_VALUES || bytes > MAX_BYTES || depth > MAX_DEPTH {
            return Err("snapshot DAG: expansion budget exceeded".into());
        }
        costs.push((values, bytes, depth));
        nodes.push(node);
    }
    let mut reachable = HashSet::new();
    let mut todo = vec![root];
    while let Some(i) = todo.pop() {
        if reachable.insert(i) {
            todo.extend(nodes[i].refs());
        }
    }
    if reachable.len() != nodes.len() {
        return Err("snapshot DAG: unreachable dictionary node".into());
    }
    Ok((nodes, root))
}

pub fn unpack(envelope: &Value) -> Result<Value, String> {
    fn expand(nodes: &[Node], i: usize) -> Value {
        match &nodes[i] {
            Node::Atom(v) => v.clone(),
            Node::List(refs) => Value::List(refs.iter().map(|i| expand(nodes, *i)).collect()),
            Node::Map(fields) => Value::Map(
                fields
                    .iter()
                    .map(|(k, i)| (k.clone(), expand(nodes, *i)))
                    .collect(),
            ),
        }
    }
    let (nodes, root) = validate(envelope)?;
    let value = expand(&nodes, root);
    admit(&value)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn envelope(rows: Vec<Value>) -> Value {
        let root = Value::Int(rows.len() as i64 - 1);
        let nodes = Value::List(rows);
        let hash = checksum(&root, &nodes).unwrap();
        Value::Map(vec![
            ("format".into(), text(FORMAT)),
            ("version".into(), Value::Int(1)),
            ("root".into(), root),
            ("nodes".into(), nodes),
            ("sha256".into(), text(&hash)),
        ])
    }
    fn row(tag: &str, value: Value) -> Value {
        Value::List(vec![text(tag), value])
    }
    #[test]
    fn typed_roundtrip_preserves_order_float_bits_and_repeated_values() {
        let shared = Value::Map(vec![
            (
                "z".into(),
                Value::List(vec![
                    Value::None,
                    Value::Bool(true),
                    Value::Int(1),
                    Value::Num(-0.0),
                    Value::Num(1.25),
                ]),
            ),
            ("汉字".into(), text("🧪")),
        ]);
        let input = Value::List(vec![shared; 128]);
        let packed = pack(&input).unwrap();
        let decoded_wire =
            crate::host::json::parse(&crate::host::json::stringify(&packed, None).unwrap())
                .unwrap();
        let expanded = unpack(&decoded_wire).unwrap();
        assert_eq!(input, expanded);
        assert_eq!(wire(&input).unwrap(), wire(&expanded).unwrap());
        let Value::List(ref xs) = expanded else {
            panic!()
        };
        let Value::Map(ref fields) = xs[0] else {
            panic!()
        };
        assert_eq!(fields[0].0, "z");
        let Value::List(ref values) = fields[0].1 else {
            panic!()
        };
        let Value::Num(number) = values[3] else {
            panic!()
        };
        assert_eq!(number.to_bits(), (-0.0f64).to_bits());
        assert!(list(field(&packed, "nodes").unwrap()).unwrap().len() < 20);
        assert!(wire(&packed).unwrap().len() < wire(&input).unwrap().len());
    }
    #[test]
    fn refuses_cycles_amplification_unreachable_duplicates_and_unsafe_atoms() {
        let bad = [
            envelope(vec![row("list", Value::List(vec![Value::Int(0)]))]),
            envelope(vec![row("text", text("unused")), row("text", text("root"))]),
            envelope(vec![row("text", text("same")), row("text", text("same"))]),
            envelope(vec![row("int", text("9007199254740992"))]),
            envelope(vec![row("float", text("7ff8000000000000"))]),
            envelope(vec![row("int", text("00"))]),
        ];
        for value in bad {
            assert!(unpack(&value).is_err());
        }
        let mut rows = vec![row("text", text("x"))];
        for n in 0..40 {
            rows.push(row("list", Value::List(vec![Value::Int(n), Value::Int(n)])));
        }
        let error = unpack(&envelope(rows)).unwrap_err();
        assert!(error.contains("budget"), "{error}");
        let mut hash = pack(&Value::Int(1)).unwrap();
        if let Value::Map(ref mut fields) = hash {
            fields.iter_mut().find(|(k, _)| k == "sha256").unwrap().1 = text("bad");
        }
        assert!(unpack(&hash).unwrap_err().contains("checksum"));
    }
    #[test]
    fn sharing_does_not_admit_ragged_columns_or_duplicate_keys() {
        let ragged = Value::Map(vec![
            ("a".into(), Value::List(vec![Value::Int(1)])),
            ("b".into(), Value::List(Vec::new())),
        ]);
        assert!(pack(&ragged).is_err());
        let duplicate = Value::Map(vec![
            ("a".into(), Value::Int(1)),
            ("a".into(), Value::Int(2)),
        ]);
        assert!(pack(&duplicate).is_err());
        let value = envelope(vec![
            row("int", text("1")),
            row("list", Value::List(vec![Value::Int(0)])),
            row("list", Value::List(Vec::new())),
            row(
                "map",
                Value::List(vec![
                    Value::List(vec![text("a"), Value::Int(1)]),
                    Value::List(vec![text("b"), Value::Int(2)]),
                ]),
            ),
        ]);
        assert!(unpack(&value).is_err());
    }
}

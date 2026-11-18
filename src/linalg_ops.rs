//! Infix / runtime helpers for `ext/linalg` matrix values.
//!
//! Keeps MatExpr in the plugin; core only dispatches `+` / `-` / `*` when both
//! sides look like matrices (map `_type` or numeric `Formula::Matrix`).

use std::collections::HashMap;

use crate::ast::BinaryOp;
use crate::formula::Expr as FormulaExpr;
use crate::host::plugin;
use crate::host::HostContext;
use crate::value::Value;

fn map_type(v: &Value) -> Option<&str> {
    match v {
        Value::Map(entries) => entries
            .iter()
            .find(|(k, _)| k == "_type")
            .and_then(|(_, t)| match t {
                Value::Text(s) => Some(s.as_str()),
                _ => None,
            }),
        _ => None,
    }
}

/// Author-facing matrix object types (EN / ZH) plus legacy plugin tags.
pub fn is_matrix_value(v: &Value) -> bool {
    if matches!(v, Value::Formula(FormulaExpr::Matrix { .. })) {
        return true;
    }
    matches!(
        map_type(v),
        Some("matrix" | "矩阵" | "linalg_expr" | "linalg_dense")
    )
}

fn prefer_result_type(left: &Value, right: &Value) -> &'static str {
    match map_type(left) {
        Some("矩阵") => "矩阵",
        Some("matrix") | Some("linalg_expr") | Some("linalg_dense") => "matrix",
        _ => match map_type(right) {
            Some("矩阵") => "矩阵",
            _ => "matrix",
        },
    }
}

/// Stamp `_type` for method chaining (EN `matrix` / ZH `矩阵`).
pub fn retag_matrix_like(type_name: &str, value: Value) -> Value {
    if !is_matrix_value(&value) {
        return value;
    }
    tag_type(type_name, value)
}

fn tag_type(type_name: &str, value: Value) -> Value {
    match value {
        Value::Map(mut entries) => {
            entries.retain(|(k, _)| k != "_type");
            entries.insert(0, ("_type".into(), Value::Text(type_name.into())));
            Value::Map(entries)
        }
        other => Value::Map(vec![
            ("_type".into(), Value::Text(type_name.into())),
            ("value".into(), other),
        ]),
    }
}

fn plugin_name(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("linalg_add"),
        BinaryOp::Sub => Some("linalg_sub"),
        BinaryOp::Mul => Some("linalg_mul"),
        _ => None,
    }
}

/// If both sides are matrices, call the linalg plugin; otherwise `Ok(None)`.
pub fn try_matrix_binary(
    host: &mut HostContext,
    op: BinaryOp,
    left: &Value,
    right: &Value,
) -> Result<Option<Value>, String> {
    let Some(fname) = plugin_name(op) else {
        return Ok(None);
    };
    if !is_matrix_value(left) || !is_matrix_value(right) {
        return Ok(None);
    }
    if host.plugins.get(fname).is_none() {
        let op_s = match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            _ => "?",
        };
        return Err(format!(
            "matrix `{op_s}` needs the linalg plugin loaded (import ext/linalg / `marqdo ext add linalg`)"
        ));
    }
    let mut bound = HashMap::new();
    bound.insert("a".into(), left.clone());
    bound.insert("b".into(), right.clone());
    // ZH param aliases used by some L1 wrappers — plugin accepts both.
    bound.insert("左".into(), left.clone());
    bound.insert("右".into(), right.clone());
    let out = plugin::call_registered(host, fname, &bound)?;
    Ok(Some(tag_type(prefer_result_type(left, right), out)))
}

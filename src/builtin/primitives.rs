//! Scalar operator semantics shared by source evaluation and VM.
use crate::{ast::BinaryOp, value::Value};
pub fn eval_binary(op: BinaryOp, l: &Value, r: &Value) -> Result<Value, String> {
    match op {
        BinaryOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => {
                crate::builtin::checked_int_binary(op, *a, *b).map(Value::Int)
            }
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (Value::Int(a), Value::Num(b)) => Ok(Value::Num(*a as f64 + b)),
            (Value::Num(a), Value::Int(b)) => Ok(Value::Num(a + *b as f64)),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{a}{b}"))),
            (Value::Text(a), Value::Secret(b)) => Ok(Value::Text(format!("{a}{b}"))),
            (Value::Secret(a), Value::Text(b)) => Ok(Value::Text(format!("{a}{b}"))),
            (Value::Secret(a), Value::Secret(b)) => Ok(Value::Text(format!("{a}{b}"))),
            (Value::Text(a), b) => Ok(Value::Text(format!("{a}{}", b.as_display()))),
            (a, Value::Text(b)) => Ok(Value::Text(format!("{}{b}", a.as_display()))),
            _ => Err("`+` needs ints, nums, or text".into()),
        },
        BinaryOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => {
                crate::builtin::checked_int_binary(op, *a, *b).map(Value::Int)
            }
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),
            (Value::Int(a), Value::Num(b)) => Ok(Value::Num(*a as f64 - b)),
            (Value::Num(a), Value::Int(b)) => Ok(Value::Num(a - *b as f64)),
            _ => Err("`-` needs ints or nums".into()),
        },
        BinaryOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => {
                crate::builtin::checked_int_binary(op, *a, *b).map(Value::Int)
            }
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
            (Value::Int(a), Value::Num(b)) => Ok(Value::Num(*a as f64 * b)),
            (Value::Num(a), Value::Int(b)) => Ok(Value::Num(a * *b as f64)),
            _ => Err("`*` needs ints or nums".into()),
        },
        BinaryOp::Div => match (l, r) {
            (Value::Int(_), Value::Int(0)) => Err("division by zero".into()),
            (Value::Int(a), Value::Int(b)) => {
                crate::builtin::checked_int_binary(op, *a, *b).map(Value::Int)
            }
            (Value::Num(_), Value::Num(b)) if *b == 0.0 => Err("division by zero".into()),
            (Value::Int(_), Value::Num(b)) if *b == 0.0 => Err("division by zero".into()),
            (Value::Num(_), Value::Int(0)) => Err("division by zero".into()),
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a / b)),
            (Value::Int(a), Value::Num(b)) => Ok(Value::Num(*a as f64 / b)),
            (Value::Num(a), Value::Int(b)) => Ok(Value::Num(a / *b as f64)),
            _ => Err("`/` needs ints or nums".into()),
        },
        BinaryOp::Eq => Ok(Value::Bool(l == r)),
        BinaryOp::Ne => Ok(Value::Bool(l != r)),
        BinaryOp::Lt => cmp_ord(l, r, |o| o.is_lt()),
        BinaryOp::Le => cmp_ord(l, r, |o| o.is_le()),
        BinaryOp::Gt => cmp_ord(l, r, |o| o.is_gt()),
        BinaryOp::Ge => cmp_ord(l, r, |o| o.is_ge()),
        BinaryOp::And | BinaryOp::Or => unreachable!("handled above"),
    }
}

fn cmp_ord(l: &Value, r: &Value, f: impl Fn(std::cmp::Ordering) -> bool) -> Result<Value, String> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        (Value::Num(a), Value::Num(b)) => Ok(Value::Bool(f(a
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal)))),
        (Value::Int(a), Value::Num(b)) => Ok(Value::Bool(f((*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal)))),
        (Value::Num(a), Value::Int(b)) => Ok(Value::Bool(f(a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal)))),
        (Value::Text(a), Value::Text(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        _ => Err("comparison needs two ints/nums or two texts".into()),
    }
}

/// Scalar unary semantics shared by source and target execution.
pub fn eval_unary(op: crate::ast::UnaryOp, value: &Value) -> Result<Value, String> {
    match op {
        crate::ast::UnaryOp::Not => Ok(Value::Bool(!value.truthy())),
        crate::ast::UnaryOp::Neg => match value {
            Value::Int(n) => crate::builtin::checked_int_neg(*n).map(Value::Int),
            Value::Num(n) => Ok(Value::Num(-n)),
            _ => Err("unary `-` needs int or num".into()),
        },
    }
}

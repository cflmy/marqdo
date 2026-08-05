//! Runtime values (Phase I).

use std::fmt;

use crate::formula::Expr as FormulaExpr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    /// Floating-point number (`type` → `num`).
    Num(f64),
    Text(String),
    /// Single-column table rows (v0); multi-column later.
    List(Vec<Value>),
    /// JSON object / string-keyed map (insertion order).
    Map(Vec<(String, Value)>),
    /// Parsed symbolic expression (`type` → `formula`).
    Formula(FormulaExpr),
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::None => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Num(n) => *n != 0.0 && !n.is_nan(),
            Value::Text(s) => !s.is_empty(),
            Value::List(xs) => !xs.is_empty(),
            Value::Map(xs) => !xs.is_empty(),
            Value::Formula(_) => true,
        }
    }

    pub fn as_display(&self) -> String {
        match self {
            Value::None => "None".into(),
            Value::Bool(true) => "True".into(),
            Value::Bool(false) => "False".into(),
            Value::Int(n) => n.to_string(),
            Value::Num(n) => crate::formula::format_num(*n),
            Value::Text(s) => s.clone(),
            Value::List(xs) => {
                let parts: Vec<String> = xs.iter().map(|v| v.as_display()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Map(_) => "<map>".into(),
            Value::Formula(e) => e.as_display(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_display())
    }
}

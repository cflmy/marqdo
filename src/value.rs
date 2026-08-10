//! Runtime values (Phase I).

use std::fmt;

use crate::formula::Expr as FormulaExpr;

/// Bound Markdown code fence (`type` → `code`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub lang: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    /// Floating-point number (`type` → `num`).
    Num(f64),
    Text(String),
    /// Vertical (1-column) table rows, or nested list values in a column map.
    List(Vec<Value>),
    /// String-keyed map (insertion order): JSON objects / horizontal tables.
    Map(Vec<(String, Value)>),
    /// Parsed symbolic expression (`type` → `formula`).
    Formula(FormulaExpr),
    /// External language source from ```lang fence (`type` → `code`).
    Code(CodeBlock),
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
            Value::Code(c) => !c.source.is_empty(),
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
            Value::Map(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.as_display()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
            Value::Formula(e) => e.as_display(),
            Value::Code(c) => format!("```{}\n{}\n```", c.lang, c.source),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_display())
    }
}

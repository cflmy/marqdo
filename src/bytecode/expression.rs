//! Source-directed expression emission obligations. Call staging and selection
//! are checked separately in call.rs; dispatch semantics remain a proof boundary.
use super::{FnChunk, Op};
use crate::{
    ast::{BinaryOp, Expr, IndexKey, InterpPart, Literal, UnaryOp},
    value::Value,
};

#[derive(Debug, Clone)]
pub enum ExpressionRule {
    Literal(Value),
    Read(String),
    Unary(UnaryOp),
    Binary(BinaryOp),
    Interpolation(Vec<InterpPart>),
    List(usize),
    Map(Vec<String>),
    Index {
        label: IndexKey,
        base_variable: Option<String>,
    },
    Occluded(bool),
    CallBoundary(Vec<String>),
    CallFault(String),
}
impl ExpressionRule {
    pub fn from_source(expr: &Expr) -> Self {
        match expr {
            Expr::Literal(value) => Self::Literal(match value {
                Literal::None => Value::None,
                Literal::Bool(v) => Value::Bool(*v),
                Literal::Int(v) => Value::Int(*v),
                Literal::Num(v) => Value::Num(*v),
                Literal::Text(v) => Value::Text(v.clone()),
            }),
            Expr::Var(name) => Self::Read(name.clone()),
            Expr::Unary { op, .. } => Self::Unary(*op),
            Expr::Binary { op, .. } => Self::Binary(*op),
            Expr::Interp(parts) => Self::Interpolation(parts.clone()),
            Expr::List(items) => Self::List(items.len()),
            Expr::Map(items) => Self::Map(items.iter().map(|(k, _)| k.clone()).collect()),
            Expr::Index { base, label } => Self::Index {
                label: label.clone(),
                base_variable: if let Expr::Var(name) = base.as_ref() {
                    Some(name.clone())
                } else {
                    None
                },
            },
            Expr::Formula(value) => Self::Literal(Value::Formula(value.clone())),
            Expr::Code(value) => Self::Literal(Value::Code(value.clone())),
            Expr::Occluded { payload, .. } => Self::Occluded(payload.is_some()),
            Expr::Call(call) if call.for_evaluation().is_err() => {
                Self::CallFault(call.for_evaluation().unwrap_err())
            }
            Expr::Call(call) => Self::CallBoundary(
                call.receiver
                    .iter()
                    .cloned()
                    .chain(call.path.iter().filter_map(|path| path.first().cloned()))
                    .collect(),
            ),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ExpressionEmission {
    pub rule: ExpressionRule,
    pub start: usize,
    pub end: usize,
    pub children: Vec<usize>,
}

pub(super) fn formula_equal(a: &crate::formula::Expr, b: &crate::formula::Expr) -> bool {
    use crate::formula::Expr::*;
    match (a, b) {
        (Num(x), Num(y)) => x.to_bits() == y.to_bits(),
        (Var(x), Var(y)) => x == y,
        (UnaryNeg(x), UnaryNeg(y)) => formula_equal(x, y),
        (
            Bin {
                op: a,
                left: x,
                right: y,
            },
            Bin {
                op: b,
                left: u,
                right: v,
            },
        ) => a == b && formula_equal(x, u) && formula_equal(y, v),
        (Call { name: a, args: x }, Call { name: b, args: y }) => {
            a == b && x.len() == y.len() && x.iter().zip(y).all(|(a, b)| formula_equal(a, b))
        }
        (Matrix { source: a, rows: x }, Matrix { source: b, rows: y }) => {
            a == b
                && x.len() == y.len()
                && x.iter().zip(y).all(|(a, b)| {
                    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
                })
        }
        _ => false,
    }
}
fn value_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x.to_bits() == y.to_bits(),
        (Value::Formula(x), Value::Formula(y)) => formula_equal(x, y),
        _ => a == b,
    }
}
pub(super) struct Cursor<'a> {
    pub(super) fun: &'a FnChunk,
    pub(super) position: usize,
    pub(super) end: usize,
}
impl Cursor<'_> {
    pub(super) fn op(&mut self, expected: Op) -> Result<(), String> {
        if self.position >= self.end || self.fun.code.get(self.position) != Some(&expected) {
            return Err(format!(
                "source expression opcode differs at {}: expected {expected:?}",
                self.position
            ));
        }
        self.position += 1;
        Ok(())
    }
    pub(super) fn literal(&mut self, value: &Value) -> Result<(), String> {
        match value {
            Value::None => self.op(Op::None_),
            Value::Bool(true) => self.op(Op::True),
            Value::Bool(false) => self.op(Op::False),
            _ => {
                let Some(Op::Constant(index)) = self.fun.code.get(self.position) else {
                    return Err("source literal is not a constant instruction".into());
                };
                if !self
                    .fun
                    .constants
                    .get(*index)
                    .is_some_and(|actual| value_equal(actual, value))
                {
                    return Err("source literal value differs".into());
                }
                self.op(Op::Constant(*index))
            }
        }
    }
    fn read(&mut self, name: &str) -> Result<(), String> {
        let slot = self
            .fun
            .locals
            .iter()
            .position(|local| local == name)
            .ok_or("source variable has no slot")?;
        self.op(Op::GetLocal(slot))
    }
    pub(super) fn child(&mut self, id: usize) -> Result<(), String> {
        let child = &self.fun.expressions[id];
        if child.start != self.position {
            return Err("source operand order/range differs".into());
        }
        self.position = child.end;
        Ok(())
    }
}
pub fn verify(fun: &FnChunk) -> Result<usize, String> {
    let mut boundaries = 0;
    for (id, node) in fun.expressions.iter().enumerate() {
        if node.start >= node.end || node.end > fun.code.len() {
            return Err("invalid source expression interval".into());
        }
        for child in &node.children {
            if *child <= id || *child >= fun.expressions.len() {
                return Err("invalid source expression child".into());
            }
            let c = &fun.expressions[*child];
            if c.start < node.start || c.end > node.end {
                return Err("expression child outside parent".into());
            }
        }
        let mut c = Cursor {
            fun,
            position: node.start,
            end: node.end,
        };
        let expected_children = match &node.rule {
            ExpressionRule::Literal(v) => {
                c.literal(v)?;
                0
            }
            ExpressionRule::Read(name) => {
                c.read(name)?;
                0
            }
            ExpressionRule::Unary(op) => {
                if node.children.len() != 1 {
                    return Err("unary source arity".into());
                }
                c.child(node.children[0])?;
                c.op(match op {
                    UnaryOp::Not => Op::Not,
                    UnaryOp::Neg => Op::Negate,
                })?;
                1
            }
            ExpressionRule::Binary(op) => {
                if node.children.len() != 2 {
                    return Err("binary source arity".into());
                }
                let left = &fun.expressions[node.children[0]];
                let right = &fun.expressions[node.children[1]];
                c.child(node.children[0])?;
                match op {
                    BinaryOp::And => {
                        c.op(Op::JumpIfFalse(right.end + 5))?;
                        c.child(node.children[1])?;
                        for op in [
                            Op::JumpIfFalse(right.end + 3),
                            Op::True,
                            Op::Jump(node.end),
                            Op::False,
                            Op::Jump(node.end),
                            Op::False,
                        ] {
                            c.op(op)?;
                        }
                    }
                    BinaryOp::Or => {
                        c.op(Op::JumpIfFalse(right.start))?;
                        c.op(Op::True)?;
                        c.op(Op::Jump(node.end))?;
                        c.child(node.children[1])?;
                        for op in [
                            Op::JumpIfFalse(right.end + 3),
                            Op::True,
                            Op::Jump(node.end),
                            Op::False,
                        ] {
                            c.op(op)?;
                        }
                    }
                    _ => {
                        let _ = left;
                        c.child(node.children[1])?;
                        c.op(match op {
                            BinaryOp::Add => Op::Add,
                            BinaryOp::Sub => Op::Sub,
                            BinaryOp::Mul => Op::Mul,
                            BinaryOp::Div => Op::Div,
                            BinaryOp::Eq => Op::Equal,
                            BinaryOp::Ne => Op::NotEqual,
                            BinaryOp::Lt => Op::Less,
                            BinaryOp::Le => Op::Lessequal,
                            BinaryOp::Gt => Op::Greater,
                            BinaryOp::Ge => Op::GreaterEqual,
                            _ => unreachable!(),
                        })?;
                    }
                }
                2
            }
            ExpressionRule::List(count) => {
                for child in &node.children {
                    c.child(*child)?;
                }
                c.op(Op::BuildList(*count))?;
                *count
            }
            ExpressionRule::Map(keys) => {
                if node.children.len() != keys.len() {
                    return Err("map source arity".into());
                }
                for (key, child) in keys.iter().zip(&node.children) {
                    c.literal(&Value::Text(key.clone()))?;
                    c.child(*child)?;
                }
                c.op(Op::BuildMap(keys.len()))?;
                keys.len()
            }
            ExpressionRule::Occluded(payload) => {
                if *payload {
                    if node.children.len() != 1 {
                        return Err("occluded source arity".into());
                    }
                    c.child(node.children[0])?;
                    1
                } else {
                    c.literal(&Value::None)?;
                    0
                }
            }
            ExpressionRule::Index {
                label,
                base_variable,
            } => {
                if node.children.len() != 1 {
                    return Err("index source arity".into());
                }
                c.child(node.children[0])?;
                match label {
                    IndexKey::Lit(key) => c.literal(&Value::Text(key.clone()))?,
                    IndexKey::Var(name) => {
                        c.read(name)?;
                        c.op(Op::Str)?;
                    }
                }
                if let Some(name) = base_variable
                    .as_ref()
                    .filter(|name| fun.var_contracts.contains_key(*name))
                {
                    let slot = fun
                        .locals
                        .iter()
                        .position(|local| local == name)
                        .ok_or("index source slot")?;
                    c.op(Op::ContractFootnoteGet(slot))?;
                } else {
                    c.op(Op::FootnoteGet)?;
                }
                1
            }
            ExpressionRule::Interpolation(parts) => {
                if parts.is_empty() {
                    c.literal(&Value::Text(String::new()))?;
                }
                for (i, part) in parts.iter().enumerate() {
                    match part {
                        InterpPart::Lit(value) => c.literal(&Value::Text(value.clone()))?,
                        InterpPart::Var(name) => c.read(name)?,
                        InterpPart::Index { base, labels } => {
                            c.read(base)?;
                            for label in labels {
                                c.literal(&Value::Text(label.clone()))?;
                                c.op(Op::FootnoteGet)?;
                            }
                        }
                    }
                    c.op(Op::Str)?;
                    if i > 0 {
                        c.op(Op::Add)?;
                    }
                }
                0
            }
            ExpressionRule::CallFault(message) => {
                let Some(Op::RejectArgumentExpansion(index)) = fun.code.get(c.position) else {
                    return Err("missing argument preparation fault".into());
                };
                if fun.constants.get(*index) != Some(&Value::Text(message.clone())) {
                    return Err("argument preparation message differs".into());
                }
                c.op(Op::RejectArgumentExpansion(*index))?;
                0
            }
            ExpressionRule::CallBoundary(_) => {
                boundaries += 1;
                continue;
            }
        };
        if node.children.len() != expected_children || c.position != node.end {
            return Err("source expression coverage differs".into());
        }
    }
    Ok(boundaries)
}

/// Local evidence preserves machine values, not the narrower foreign copy domain.
pub(super) fn interpolation_evidence(part: &InterpPart) -> serde_json::Value {
    use serde_json::json;
    match part {
        InterpPart::Lit(value) => json!({"kind":"text","value":value}),
        InterpPart::Var(name) => json!({"kind":"read","name":name}),
        InterpPart::Index { base, labels } => json!({"kind":"index","base":base,"labels":labels}),
    }
}
pub(super) fn index_evidence(key: &IndexKey) -> serde_json::Value {
    use serde_json::json;
    match key {
        IndexKey::Lit(value) => json!({"kind":"literal","value":value}),
        IndexKey::Var(name) => json!({"kind":"read","name":name}),
    }
}
pub(super) fn formula_evidence(value: &crate::formula::Expr) -> serde_json::Value {
    use crate::formula::Expr::*;
    use serde_json::json;
    match value {
        Num(n) => json!({"kind":"number","bits":n.to_bits()}),
        Var(name) => json!({"kind":"variable","name":name}),
        UnaryNeg(value) => json!({"kind":"negative","value":formula_evidence(value)}),
        Bin { op, left, right } => {
            json!({"kind":"binary","op":format!("{op:?}"),"left":formula_evidence(left),"right":formula_evidence(right)})
        }
        Call { name, args } => {
            json!({"kind":"call","name":name,"args":args.iter().map(formula_evidence).collect::<Vec<_>>()})
        }
        Matrix { source, rows } => {
            json!({"kind":"matrix","source":source,"rows":rows.iter().map(|row|row.iter().map(|n|n.to_bits()).collect::<Vec<_>>()).collect::<Vec<_>>()})
        }
    }
}
pub fn value_evidence(value: &Value) -> serde_json::Value {
    use serde_json::json;
    match value {
        Value::None => json!({"tag":"none"}),
        Value::Bool(v) => json!({"tag":"bool","value":v}),
        Value::Int(v) => json!({"tag":"int","value":v}),
        Value::Num(v) => json!({"tag":"num","bits":v.to_bits()}),
        Value::Text(v) => json!({"tag":"text","value":v}),
        Value::Code(v) => json!({"tag":"code","lang":v.lang,"source":v.source}),
        Value::Formula(v) => json!({"tag":"formula","value":formula_evidence(v)}),
        _ => json!({"tag":"nonliteral","value":format!("{value:?}")}),
    }
}
impl ExpressionRule {
    pub fn evidence(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Self::Literal(v) => json!({"kind":"literal","value":value_evidence(v)}),
            Self::Read(name) => json!({"kind":"read","name":name}),
            Self::Unary(op) => json!({"kind":"unary","op":format!("{op:?}")}),
            Self::Binary(op) => json!({"kind":"binary","op":format!("{op:?}")}),
            Self::CallBoundary(names) => json!({"kind":"call_boundary","names":names}),
            Self::CallFault(message) => json!({"kind":"call_expansion_fault","message":message}),
            Self::Interpolation(parts) => {
                json!({"kind":"interpolation_boundary","parts":parts.iter().map(interpolation_evidence).collect::<Vec<_>>()})
            }
            Self::List(n) => json!({"kind":"list","arity":n}),
            Self::Map(keys) => json!({"kind":"map","keys":keys}),
            Self::Index {
                label,
                base_variable,
            } => {
                json!({"kind":"index_boundary","label":index_evidence(label),"base_variable":base_variable})
            }
            Self::Occluded(payload) => json!({"kind":"occluded","payload":payload}),
        }
    }
}

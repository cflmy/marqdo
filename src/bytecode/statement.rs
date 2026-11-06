//! Source-directed statement and control-flow templates. These checks validate
//! emitted templates; they do not prove the parser or source-witness exporter.
use super::expression::Cursor;
use super::{FnChunk, Op};
use crate::ast::Stmt;

#[derive(Debug, Clone)]
pub enum StatementRule {
    Assign(String),
    Return,
    Call,
    Expression,
    Branch(Vec<(bool, usize)>),
    While(usize),
    ForEach {
        item: String,
        collection: String,
        body: usize,
    },
}
impl StatementRule {
    pub fn from_source(stmt: &Stmt) -> Self {
        match stmt {
            Stmt::Assign { name, .. } => Self::Assign(name.clone()),
            Stmt::Return { .. } => Self::Return,
            Stmt::Call { .. } => Self::Call,
            Stmt::Expr { .. } => Self::Expression,
            Stmt::Branch { arms, .. } => Self::Branch(
                arms.iter()
                    .map(|a| (a.condition.is_some(), a.body.len()))
                    .collect(),
            ),
            Stmt::While { body, .. } => Self::While(body.len()),
            Stmt::ForEach {
                item,
                collection,
                body,
                ..
            } => Self::ForEach {
                item: item.clone(),
                collection: collection.clone(),
                body: body.len(),
            },
        }
    }
}
#[derive(Debug, Clone)]
pub struct StatementEmission {
    pub rule: StatementRule,
    pub start: usize,
    pub end: usize,
    pub statements: Vec<usize>,
    pub expressions: Vec<usize>,
    pub auxiliary: Vec<usize>,
}
fn statement(c: &mut Cursor<'_>, id: usize) -> Result<(), String> {
    let child = c.fun.statements.get(id).ok_or("missing statement child")?;
    if child.start != c.position {
        return Err("source statement order/range differs".into());
    }
    c.position = child.end;
    Ok(())
}
pub fn verify(fun: &FnChunk) -> Result<(), String> {
    if fun.defaults.len() != fun.params.len() || fun.default_expressions.len() != fun.params.len() {
        return Err("default expression inventory".into());
    }
    let mut prefix = 0;
    for (parameter, default) in fun.default_expressions.iter().enumerate() {
        if fun.defaults[parameter] != default.is_some() {
            return Err("default source presence differs".into());
        }
        if let Some(id) = default {
            let expr = fun
                .expressions
                .get(*id)
                .ok_or("default source expression missing")?;
            if expr.start != prefix + 1
                || fun.code.get(prefix) != Some(&Op::JumpIfProvided(parameter, expr.end + 2))
                || fun.code.get(expr.end..expr.end + 2)
                    != Some([Op::SetLocal(parameter), Op::Pop].as_slice())
            {
                return Err("source default prologue differs".into());
            }
            prefix = expr.end + 2;
        }
    }
    if fun.code.get(prefix) != Some(&Op::FillMissingParameters) {
        return Err("missing parameter-fill boundary".into());
    }
    prefix += 1;
    if fun.contract.is_some() {
        if fun.code.get(prefix) != Some(&Op::CheckArguments) {
            return Err("missing input contract boundary".into());
        }
        prefix += 1;
    }
    if fun.receiver_aliases {
        if fun.code.get(prefix)!=Some(&Op::RestoreReceiver){return Err("missing receiver restoration".into());}prefix+=1;
        if fun.contract.is_some(){if fun.code.get(prefix)!=Some(&Op::CheckFrameArguments){return Err("missing frame contract boundary".into());}prefix+=1;}
    }
    if prefix != fun.body_start {
        return Err("function prologue/body boundary differs".into());
    }
    let mut protected = fun
        .params
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();
    protected.extend(["self".to_string(), "自".to_string()]);
    for expression in &fun.expressions {
        use super::ExpressionRule;
        match &expression.rule {
            ExpressionRule::Read(name) => {
                protected.insert(name.clone());
            }
            ExpressionRule::CallBoundary(names) => protected.extend(names.iter().cloned()),
            ExpressionRule::Interpolation(parts) => {
                for part in parts {
                    match part {
                        crate::ast::InterpPart::Var(name)
                        | crate::ast::InterpPart::Index { base: name, .. } => {
                            protected.insert(name.clone());
                        }
                        _ => {}
                    }
                }
            }
            ExpressionRule::Index {
                label,
                base_variable,
            } => {
                if let Some(name) = base_variable {
                    protected.insert(name.clone());
                }
                if let crate::ast::IndexKey::Var(name) = label {
                    protected.insert(name.clone());
                }
            }
            _ => {}
        }
    }
    for statement in &fun.statements {
        match &statement.rule {
            StatementRule::Assign(name) => {
                protected.insert(name.clone());
            }
            StatementRule::ForEach {
                item, collection, ..
            } => {
                protected.insert(item.clone());
                protected.insert(collection.clone());
            }
            _ => {}
        }
    }
    let mut auxiliary_slots = std::collections::HashSet::new();
    let mut parent_counts = vec![0usize; fun.statements.len()];
    for (id, node) in fun.statements.iter().enumerate() {
        if node.start > node.end || node.end > fun.code.len() {
            return Err("invalid statement interval".into());
        }
        for &child in &node.statements {
            if child <= id || child >= fun.statements.len() {
                return Err("invalid statement child".into());
            }
            parent_counts[child] += 1;
            let child = &fun.statements[child];
            if child.start < node.start || child.end > node.end {
                return Err("statement child outside parent".into());
            }
        }
        for &expr in &node.expressions {
            let expr = fun
                .expressions
                .get(expr)
                .ok_or("invalid statement expression")?;
            if expr.start < node.start || expr.end > node.end {
                return Err("expression outside statement".into());
            }
        }
        let mut c = Cursor {
            fun,
            position: node.start,
            end: node.end,
        };
        match &node.rule {
            StatementRule::Assign(_)
            | StatementRule::Return
            | StatementRule::Call
            | StatementRule::Expression => {
                if !node.statements.is_empty() || node.expressions.len() != 1 {
                    return Err("statement source arity".into());
                }
                c.child(node.expressions[0])?;
                match &node.rule {
                    StatementRule::Assign(name) => {
                        let slot = fun
                            .locals
                            .iter()
                            .position(|n| n == name)
                            .ok_or("assignment slot")?;
                        if fun.var_contracts.contains_key(name) {
                            c.op(Op::CheckBinding(slot))?;
                        }
                        c.op(Op::SetLocal(slot))?;
                        c.op(Op::Pop)?;
                    }
                    StatementRule::Return => {
                        if fun.contract.is_some() {
                            c.op(Op::CheckReturn)?;
                        }
                        c.op(Op::Return)?;
                    }
                    StatementRule::Call | StatementRule::Expression => c.op(Op::Pop)?,
                    _ => unreachable!(),
                }
            }
            StatementRule::While(count) => {
                if node.expressions.len() != 1 || node.statements.len() != *count {
                    return Err("while source arity".into());
                }
                c.op(Op::ResetLoop(id))?;
                let head = c.position;
                c.child(node.expressions[0])?;
                c.op(Op::JumpIfFalse(node.end))?;
                c.op(Op::CheckLoop(id))?;
                for &child in &node.statements {
                    statement(&mut c, child)?;
                }
                c.op(Op::Jump(head))?;
            }
            StatementRule::Branch(arms) => {
                if node.expressions.len() != arms.iter().filter(|(condition, _)| *condition).count()
                    || node.statements.len() != arms.iter().map(|(_, n)| n).sum::<usize>()
                {
                    return Err("branch source arity".into());
                }
                let (mut expr, mut stmt) = (0, 0);
                for (index, (condition, count)) in arms.iter().enumerate() {
                    let guard = if *condition {
                        c.child(node.expressions[expr])?;
                        expr += 1;
                        let pc = c.position;
                        c.position += 1;
                        Some(pc)
                    } else {
                        None
                    };
                    for _ in 0..*count {
                        statement(&mut c, node.statements[stmt])?;
                        stmt += 1;
                    }
                    if index + 1 < arms.len() {
                        c.op(Op::Jump(node.end))?;
                    }
                    if let Some(guard) = guard {
                        if fun.code.get(guard) != Some(&Op::JumpIfFalse(c.position)) {
                            return Err("source branch false target differs".into());
                        }
                    }
                }
            }
            StatementRule::ForEach {
                item,
                collection,
                body,
            } => {
                if node.expressions.len() != 1
                    || node.statements.len() != *body
                    || node.auxiliary.len() != 2
                {
                    return Err("foreach source arity".into());
                }
                let (coll, index) = (node.auxiliary[0], node.auxiliary[1]);
                if coll == index || coll >= fun.locals.len() || index >= fun.locals.len() {
                    return Err("foreach auxiliary slots".into());
                }
                for slot in [coll, index] {
                    if protected.contains(&fun.locals[slot]) || !auxiliary_slots.insert(slot) {
                        return Err("loop temporary aliases source or another loop".into());
                    }
                }
                // The initial expression must read the authored collection.
                if !matches!(&fun.expressions[node.expressions[0]].rule,super::ExpressionRule::Read(name) if name==collection)
                {
                    return Err("foreach source collection differs".into());
                }
                c.child(node.expressions[0])?;
                c.op(Op::IterItems)?;
                c.op(Op::SetLocal(coll))?;
                c.op(Op::Pop)?;
                c.literal(&crate::value::Value::Int(0))?;
                c.op(Op::SetLocal(index))?;
                c.op(Op::Pop)?;
                let head = c.position;
                for op in [
                    Op::GetLocal(index),
                    Op::GetLocal(coll),
                    Op::Len,
                    Op::Less,
                    Op::JumpIfFalse(node.end),
                    Op::GetLocal(coll),
                    Op::GetLocal(index),
                    Op::GetIndex,
                ] {
                    c.op(op)?;
                }
                let item = fun
                    .locals
                    .iter()
                    .position(|n| n == item)
                    .ok_or("foreach item slot")?;
                c.op(Op::SetLocal(item))?;
                c.op(Op::Pop)?;
                for &child in &node.statements {
                    statement(&mut c, child)?;
                }
                c.op(Op::GetLocal(index))?;
                c.literal(&crate::value::Value::Int(1))?;
                for op in [Op::Add, Op::SetLocal(index), Op::Pop, Op::Jump(head)] {
                    c.op(op)?;
                }
            }
        }
        if c.position != node.end {
            return Err("statement instruction coverage differs".into());
        }
    }
    let mut cursor = fun.body_start;
    for &root in &fun.statement_roots {
        let node = fun.statements.get(root).ok_or("statement root missing")?;
        if parent_counts[root] != 0 || node.start != cursor {
            return Err("source body root order differs".into());
        }
        parent_counts[root] += 1;
        cursor = node.end;
    }
    if parent_counts.iter().any(|n| *n != 1) {
        return Err("statement forest coverage differs".into());
    }
    if fun.code.get(cursor..) != Some([Op::None_, Op::Return].as_slice()) {
        return Err("implicit function return differs".into());
    }
    Ok(())
}

impl StatementRule {
    pub fn evidence(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Self::Assign(name) => json!({"kind":"assign","name":name}),
            Self::Return => json!({"kind":"return"}),
            Self::Call => json!({"kind":"call"}),
            Self::Expression => json!({"kind":"expression"}),
            Self::Branch(arms) => json!({"kind":"branch","arms":arms}),
            Self::While(body) => json!({"kind":"while","body":body}),
            Self::ForEach {
                item,
                collection,
                body,
            } => json!({"kind":"foreach","item":item,"collection":collection,"body":body}),
        }
    }
}

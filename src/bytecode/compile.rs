//! AST → bytecode Program.

use anyhow::{bail, Result};
use std::collections::HashMap;

use crate::ast::{
    Arg, BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Stmt, UnaryOp,
};
use crate::bytecode::{FnChunk, Op, Program};
use crate::value::Value;

struct FlatFun {
    name: String,
    parent: Option<usize>,
    params: Vec<String>,
    body: Vec<Stmt>,
    children: Vec<usize>,
}

pub fn compile_module(module: &Module) -> Result<Program> {
    let mut flat = Vec::new();
    for fun in &module.functions {
        collect_fun(fun, None, &mut flat);
    }
    if flat.is_empty() {
        bail!("bytecode: no functions to compile");
    }

    let entry = flat
        .iter()
        .position(|f| f.name == "main" && f.parent.is_none())
        .or_else(|| {
            flat.iter()
                .position(|f| f.parent.is_none() && f.params.is_empty())
        })
        .ok_or_else(|| anyhow::anyhow!("bytecode: no entry function"))?;

    let mut functions = Vec::new();
    for i in 0..flat.len() {
        functions.push(compile_function(i, &flat)?);
    }

    Ok(Program { functions, entry })
}

fn collect_fun(fun: &Function, parent: Option<usize>, flat: &mut Vec<FlatFun>) -> usize {
    let id = flat.len();
    flat.push(FlatFun {
        name: fun.name.clone(),
        parent,
        params: fun.params.clone(),
        body: fun.body.clone(),
        children: Vec::new(),
    });
    let mut child_ids = Vec::new();
    for child in &fun.children {
        child_ids.push(collect_fun(child, Some(id), flat));
    }
    flat[id].children = child_ids;
    id
}

struct FnCompiler<'a> {
    flat: &'a [FlatFun],
    fn_id: usize,
    chunk: FnChunk,
    locals: HashMap<String, u8>,
}

fn compile_function(fn_id: usize, flat: &[FlatFun]) -> Result<FnChunk> {
    let fun = &flat[fn_id];
    let mut locals = HashMap::new();
    for (i, p) in fun.params.iter().enumerate() {
        locals.insert(p.clone(), i as u8);
    }
    let mut c = FnCompiler {
        flat,
        fn_id,
        chunk: FnChunk {
            name: fun.name.clone(),
            params: fun.params.clone(),
            code: Vec::new(),
            constants: Vec::new(),
            locals: fun.params.clone(),
        },
        locals,
    };
    for stmt in &fun.body {
        c.compile_stmt(stmt)?;
    }
    c.emit(Op::None_);
    c.emit(Op::Return);
    let mut names = vec![String::new(); c.locals.len()];
    for (name, slot) in &c.locals {
        names[*slot as usize] = name.clone();
    }
    c.chunk.locals = names;
    Ok(c.chunk)
}

impl<'a> FnCompiler<'a> {
    fn emit(&mut self, op: Op) {
        self.chunk.code.push(op);
    }

    fn here(&self) -> u16 {
        self.chunk.code.len() as u16
    }

    fn patch(&mut self, at: u16, op: Op) {
        self.chunk.code[at as usize] = op;
    }

    fn add_const(&mut self, v: Value) -> u16 {
        if let Some(i) = self.chunk.constants.iter().position(|c| c == &v) {
            return i as u16;
        }
        let i = self.chunk.constants.len() as u16;
        self.chunk.constants.push(v);
        i
    }

    fn local_slot(&mut self, name: &str) -> u8 {
        if let Some(&s) = self.locals.get(name) {
            return s;
        }
        let s = self.locals.len() as u8;
        self.locals.insert(name.to_string(), s);
        s
    }

    fn resolve_call(&self, name: &str) -> Result<usize> {
        let fun = &self.flat[self.fn_id];
        for &cid in &fun.children {
            if self.flat[cid].name == name {
                return Ok(cid);
            }
        }
        let mut parent = fun.parent;
        while let Some(pid) = parent {
            for &cid in &self.flat[pid].children {
                if self.flat[cid].name == name {
                    return Ok(cid);
                }
            }
            parent = self.flat[pid].parent;
        }
        for (i, f) in self.flat.iter().enumerate() {
            if f.parent.is_none() && f.name == name {
                return Ok(i);
            }
        }
        bail!("bytecode: unknown function `{name}`")
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Assign { name, value, .. } => {
                self.compile_expr(value)?;
                let slot = self.local_slot(name);
                self.emit(Op::SetLocal(slot));
                self.emit(Op::Pop);
            }
            Stmt::Return { value, .. } => {
                self.compile_expr(value)?;
                self.emit(Op::Return);
            }
            Stmt::Call { call, .. } => self.compile_call(call, true)?,
            Stmt::Branch { arms, .. } => {
                let mut end_jumps = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let is_last = i + 1 == arms.len();
                    let mut else_jump = None;
                    if let Some(cond) = &arm.condition {
                        self.compile_expr(cond)?;
                        let jf = self.here();
                        self.emit(Op::JumpIfFalse(0));
                        else_jump = Some(jf);
                    }
                    for st in &arm.body {
                        self.compile_stmt(st)?;
                    }
                    if !is_last {
                        let j = self.here();
                        self.emit(Op::Jump(0));
                        end_jumps.push(j);
                    }
                    if let Some(jf) = else_jump {
                        self.patch(jf, Op::JumpIfFalse(self.here()));
                    }
                }
                let end = self.here();
                for j in end_jumps {
                    self.patch(j, Op::Jump(end));
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                let loop_start = self.here();
                self.compile_expr(condition)?;
                let exit_j = self.here();
                self.emit(Op::JumpIfFalse(0));
                for st in body {
                    self.compile_stmt(st)?;
                }
                self.emit(Op::Jump(loop_start));
                self.patch(exit_j, Op::JumpIfFalse(self.here()));
            }
            Stmt::ForEach {
                item,
                collection,
                body,
                ..
            } => {
                self.compile_expr(&Expr::Var(collection.clone()))?;
                let coll_slot = self.local_slot(&format!("__coll_{collection}"));
                self.emit(Op::SetLocal(coll_slot));
                self.emit(Op::Pop);
                let i_slot = self.local_slot(&format!("__i_{collection}"));
                let zero = self.add_const(Value::Int(0));
                self.emit(Op::Constant(zero));
                self.emit(Op::SetLocal(i_slot));
                self.emit(Op::Pop);
                let item_slot = self.local_slot(item);

                let loop_start = self.here();
                self.emit(Op::GetLocal(i_slot));
                self.emit(Op::GetLocal(coll_slot));
                self.emit(Op::Len);
                self.emit(Op::Less);
                let exit_j = self.here();
                self.emit(Op::JumpIfFalse(0));

                self.emit(Op::GetLocal(coll_slot));
                self.emit(Op::GetLocal(i_slot));
                self.emit(Op::GetIndex);
                self.emit(Op::SetLocal(item_slot));
                self.emit(Op::Pop);

                for st in body {
                    self.compile_stmt(st)?;
                }

                self.emit(Op::GetLocal(i_slot));
                let one = self.add_const(Value::Int(1));
                self.emit(Op::Constant(one));
                self.emit(Op::Add);
                self.emit(Op::SetLocal(i_slot));
                self.emit(Op::Pop);
                self.emit(Op::Jump(loop_start));
                self.patch(exit_j, Op::JumpIfFalse(self.here()));
            }
        }
        Ok(())
    }

    fn compile_call(&mut self, call: &CallExpr, as_stmt: bool) -> Result<()> {
        match call.callee.as_str() {
            "print" => {
                let text_expr = call
                    .args
                    .iter()
                    .find_map(|a| match a {
                        Arg::Named { name, value } if name == "text" => Some(value),
                        _ => None,
                    })
                    .or_else(|| {
                        call.args.iter().find_map(|a| match a {
                            Arg::Positional(e) => Some(e),
                            _ => None,
                        })
                    })
                    .ok_or_else(|| anyhow::anyhow!("bytecode: print needs text"))?;
                self.compile_expr(text_expr)?;
                self.emit(Op::Print);
                if !as_stmt {
                    self.emit(Op::None_);
                }
                return Ok(());
            }
            "input" => bail!("bytecode: input not supported"),
            _ => {}
        }

        let fid = self.resolve_call(&call.callee)?;
        let params = self.flat[fid].params.clone();
        let mut named: HashMap<String, &Expr> = HashMap::new();
        let mut positionals = Vec::new();
        for a in &call.args {
            match a {
                Arg::Positional(e) => positionals.push(e),
                Arg::Named { name, value } => {
                    named.insert(name.clone(), value);
                }
            }
        }
        let mut pos_i = 0;
        for p in &params {
            if let Some(e) = named.get(p) {
                self.compile_expr(e)?;
            } else if pos_i < positionals.len() {
                self.compile_expr(positionals[pos_i])?;
                pos_i += 1;
            } else {
                bail!("bytecode: missing argument for parameter `{p}`");
            }
        }
        if pos_i < positionals.len() {
            bail!("bytecode: too many positional arguments");
        }
        self.emit(Op::Call(fid as u16, params.len() as u8));
        if as_stmt {
            self.emit(Op::Pop);
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::None => self.emit(Op::None_),
                Literal::Bool(true) => self.emit(Op::True),
                Literal::Bool(false) => self.emit(Op::False),
                Literal::Int(n) => {
                    let i = self.add_const(Value::Int(*n));
                    self.emit(Op::Constant(i));
                }
                Literal::Text(t) => {
                    let i = self.add_const(Value::Text(t.clone()));
                    self.emit(Op::Constant(i));
                }
            },
            Expr::Var(name) => {
                let slot = self
                    .locals
                    .get(name)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("bytecode: undefined variable `{name}`"))?;
                self.emit(Op::GetLocal(slot));
            }
            Expr::Interp(parts) => {
                if parts.is_empty() {
                    let i = self.add_const(Value::Text(String::new()));
                    self.emit(Op::Constant(i));
                    return Ok(());
                }
                for (idx, part) in parts.iter().enumerate() {
                    match part {
                        InterpPart::Lit(t) => {
                            let i = self.add_const(Value::Text(t.clone()));
                            self.emit(Op::Constant(i));
                        }
                        InterpPart::Var(n) => {
                            let slot = self.locals.get(n).copied().ok_or_else(|| {
                                anyhow::anyhow!("bytecode: undefined variable `{n}`")
                            })?;
                            self.emit(Op::GetLocal(slot));
                        }
                    }
                    if idx > 0 {
                        self.emit(Op::Add);
                    }
                }
            }
            Expr::Unary { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Not => self.emit(Op::Not),
                    UnaryOp::Neg => self.emit(Op::Negate),
                }
            }
            Expr::Binary {
                op: BinaryOp::And,
                left,
                right,
            } => {
                // JumpIfFalse always pops the tested value.
                self.compile_expr(left)?;
                let j_false = self.here();
                self.emit(Op::JumpIfFalse(0));
                self.compile_expr(right)?;
                let j_false2 = self.here();
                self.emit(Op::JumpIfFalse(0));
                self.emit(Op::True);
                let j_end = self.here();
                self.emit(Op::Jump(0));
                let f2 = self.here();
                self.patch(j_false2, Op::JumpIfFalse(f2));
                self.emit(Op::False);
                let j_end2 = self.here();
                self.emit(Op::Jump(0));
                let f1 = self.here();
                self.patch(j_false, Op::JumpIfFalse(f1));
                self.emit(Op::False);
                let end = self.here();
                self.patch(j_end, Op::Jump(end));
                self.patch(j_end2, Op::Jump(end));
            }
            Expr::Binary {
                op: BinaryOp::Or,
                left,
                right,
            } => {
                self.compile_expr(left)?;
                let j_else = self.here();
                self.emit(Op::JumpIfFalse(0));
                self.emit(Op::True);
                let j_end = self.here();
                self.emit(Op::Jump(0));
                let else_ip = self.here();
                self.patch(j_else, Op::JumpIfFalse(else_ip));
                self.compile_expr(right)?;
                let j_f = self.here();
                self.emit(Op::JumpIfFalse(0));
                self.emit(Op::True);
                let j_end2 = self.here();
                self.emit(Op::Jump(0));
                let f = self.here();
                self.patch(j_f, Op::JumpIfFalse(f));
                self.emit(Op::False);
                let end = self.here();
                self.patch(j_end, Op::Jump(end));
                self.patch(j_end2, Op::Jump(end));
            }
            Expr::Binary { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.emit(match op {
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
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                });
            }
            Expr::Call(call) => self.compile_call(call, false)?,
            Expr::List(items) => {
                for it in items {
                    self.compile_expr(it)?;
                }
                self.emit(Op::BuildList(items.len() as u16));
            }
        }
        Ok(())
    }
}

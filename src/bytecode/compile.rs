//! AST → bytecode Program.

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::ast::{
    Arg, BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Param, Stmt, UnaryOp,
};
use crate::bytecode::{FnChunk, Op, Program};
use crate::diagnostics::{bail_at, Span};
use crate::value::Value;

struct FlatFun {
    name: String,
    level: u8,
    parent: Option<usize>,
    params: Vec<Param>,
    body: Vec<Stmt>,
    children: Vec<usize>,
    span: Span,
    /// Full path segments (`lib.member` / `lib.Type.method`) → flat index.
    path_index: HashMap<Vec<String>, usize>,
    /// Short name from frontmatter `use` → path segments.
    uses: HashMap<String, Vec<String>>,
}

pub fn compile_module(path: Option<&Path>, module: &Module) -> Result<Program> {
    let mut flat = Vec::new();
    let _exports = collect_module_tree(module, &mut flat);
    if flat.is_empty() {
        bail!("bytecode: no functions to compile");
    }

    let entry = flat
        .iter()
        .position(|f| f.name == "main" && f.parent.is_none())
        .or_else(|| {
            let tops: Vec<usize> = flat
                .iter()
                .enumerate()
                .filter(|(_, f)| f.parent.is_none() && f.level == 1 && f.params.is_empty())
                .map(|(i, _)| i)
                .collect();
            if tops.len() == 1 {
                Some(tops[0])
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("bytecode: no `# main` object"))?;

    let mut objects = Vec::new();
    let mut methods = Vec::new();
    for (i, f) in flat.iter().enumerate() {
        if f.parent.is_none() && f.level == 1 {
            objects.push((f.name.clone(), i));
            for &cid in &f.children {
                methods.push((i, flat[cid].name.clone(), cid));
            }
        }
    }

    let path_buf = path.map(|p| p.to_path_buf());
    let mut functions = Vec::new();
    for i in 0..flat.len() {
        functions.push(compile_function(i, &flat, path_buf.as_deref())?);
    }

    let parents: Vec<Option<usize>> = flat.iter().map(|f| f.parent).collect();
    let children: Vec<Vec<usize>> = flat.iter().map(|f| f.children.clone()).collect();

    Ok(Program {
        functions,
        entry,
        objects,
        methods,
        parents,
        children,
    })
}

/// Collect `module` and its imports into `flat`.
/// Returns relative export paths within this module (e.g. `["parse"]`, `["agent","step"]`).
fn collect_module_tree(module: &Module, flat: &mut Vec<FlatFun>) -> HashMap<Vec<String>, usize> {
    let mut path_index = HashMap::new();
    for (bind, dep) in &module.import_modules {
        let dep_exports = collect_module_tree(dep, flat);
        for (rel, idx) in dep_exports {
            let mut full = Vec::with_capacity(rel.len() + 1);
            full.push(bind.clone());
            full.extend(rel);
            path_index.insert(full, idx);
        }
    }

    let uses: HashMap<String, Vec<String>> = module
        .uses
        .iter()
        .map(|u| (u.bind.clone(), u.path.clone()))
        .collect();

    let mut exports = HashMap::new();
    for fun in &module.functions {
        let id = collect_fun(fun, None, flat, &path_index, &uses);
        export_tree(&mut exports, &[], id, flat);
    }
    exports
}

fn export_tree(
    out: &mut HashMap<Vec<String>, usize>,
    prefix: &[String],
    id: usize,
    flat: &[FlatFun],
) {
    let mut p = prefix.to_vec();
    p.push(flat[id].name.clone());
    out.insert(p.clone(), id);
    for &cid in &flat[id].children {
        export_tree(out, &p, cid, flat);
    }
}

fn collect_fun(
    fun: &Function,
    parent: Option<usize>,
    flat: &mut Vec<FlatFun>,
    path_index: &HashMap<Vec<String>, usize>,
    uses: &HashMap<String, Vec<String>>,
) -> usize {
    let id = flat.len();
    flat.push(FlatFun {
        name: fun.name.clone(),
        level: fun.level,
        parent,
        params: fun.params.clone(),
        body: fun.body.clone(),
        children: Vec::new(),
        span: fun.span,
        path_index: path_index.clone(),
        uses: uses.clone(),
    });
    let mut child_ids = Vec::new();
    for child in &fun.children {
        child_ids.push(collect_fun(child, Some(id), flat, path_index, uses));
    }
    flat[id].children = child_ids;
    id
}

struct FnCompiler<'a> {
    path: Option<&'a Path>,
    flat: &'a [FlatFun],
    fn_id: usize,
    chunk: FnChunk,
    locals: HashMap<String, u8>,
    stmt_span: Span,
}

fn compile_function(fn_id: usize, flat: &[FlatFun], path: Option<&Path>) -> Result<FnChunk> {
    let fun = &flat[fn_id];
    let mut locals = HashMap::new();
    for (i, p) in fun.params.iter().enumerate() {
        locals.insert(p.name.clone(), i as u8);
    }
    let is_method = fun
        .parent
        .map(|pid| flat[pid].level == 1 && flat[pid].parent.is_none())
        .unwrap_or(false);
    if is_method {
        for name in ["自", "self"] {
            if !locals.contains_key(name) {
                let s = locals.len() as u8;
                locals.insert(name.to_string(), s);
            }
        }
    }
    let mut c = FnCompiler {
        path,
        flat,
        fn_id,
        chunk: FnChunk {
            name: fun.name.clone(),
            params: fun.params.iter().map(|p| p.name.clone()).collect(),
            code: Vec::new(),
            spans: Vec::new(),
            constants: Vec::new(),
            locals: fun.params.iter().map(|p| p.name.clone()).collect(),
        },
        locals,
        stmt_span: fun.span,
    };
    for stmt in &fun.body {
        c.compile_stmt(stmt)?;
    }
    c.stmt_span = fun.span;
    c.emit(Op::None_);
    c.emit(Op::Return);
    let mut names = vec![String::new(); c.locals.len()];
    for (name, slot) in &c.locals {
        names[*slot as usize] = name.clone();
    }
    c.chunk.locals = names;
    debug_assert_eq!(c.chunk.code.len(), c.chunk.spans.len());
    Ok(c.chunk)
}

impl<'a> FnCompiler<'a> {
    fn err(&self, message: impl Into<String>) -> anyhow::Error {
        bail_at(self.path, self.stmt_span, message)
    }

    fn emit(&mut self, op: Op) {
        self.chunk.code.push(op);
        self.chunk.spans.push(self.stmt_span);
    }

    fn named_or_first<'b>(&self, call: &'b CallExpr, name: &str) -> Option<&'b Expr> {
        call.args
            .iter()
            .find_map(|a| match a {
                Arg::Named { name: n, value } if n == name => Some(value),
                _ => None,
            })
            .or_else(|| {
                call.args.iter().find_map(|a| match a {
                    Arg::Positional(e) => Some(e),
                    _ => None,
                })
            })
    }

    fn named_or_pos<'b>(&self, call: &'b CallExpr, name: &str, pos: usize) -> Option<&'b Expr> {
        if let Some(e) = call.args.iter().find_map(|a| match a {
            Arg::Named { name: n, value } if n == name => Some(value),
            _ => None,
        }) {
            return Some(e);
        }
        let mut i = 0usize;
        for a in &call.args {
            if let Arg::Positional(e) = a {
                if i == pos {
                    return Some(e);
                }
                i += 1;
            }
        }
        None
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
        Err(self.err(format!("unknown function `{name}`")))
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Assign { name, value, span, .. } => {
                self.stmt_span = *span;
                self.compile_expr(value)?;
                let slot = self.local_slot(name);
                self.emit(Op::SetLocal(slot));
                self.emit(Op::Pop);
            }
            Stmt::Return { value, span } => {
                self.stmt_span = *span;
                self.compile_expr(value)?;
                self.emit(Op::Return);
            }
            Stmt::Call { call, span } => {
                self.stmt_span = *span;
                self.compile_call(call, true)?;
            }
            Stmt::Branch { arms, span } => {
                self.stmt_span = *span;
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
                    self.stmt_span = *span;
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
                condition,
                body,
                span,
            } => {
                self.stmt_span = *span;
                let loop_start = self.here();
                self.compile_expr(condition)?;
                let exit_j = self.here();
                self.emit(Op::JumpIfFalse(0));
                for st in body {
                    self.compile_stmt(st)?;
                }
                self.stmt_span = *span;
                self.emit(Op::Jump(loop_start));
                self.patch(exit_j, Op::JumpIfFalse(self.here()));
            }
            Stmt::ForEach {
                item,
                collection,
                body,
                span,
            } => {
                self.stmt_span = *span;
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

                self.stmt_span = *span;
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
        let mut call = call.clone();
        if let Some(path) = &call.path {
            return self.compile_path_call(path, &call, as_stmt);
        }
        if let Some(path) = self.flat[self.fn_id].uses.get(&call.callee).cloned() {
            return self.compile_path_call(&path, &call, as_stmt);
        }
        call.callee =
            crate::aliases::normalize_call_callee_and_args(&call.callee, &mut call.args);

        if let Some(recv) = &call.receiver {
            return self.compile_method_call(recv, &call, as_stmt);
        }

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
                    .ok_or_else(|| self.err("print requires text (named or positional)"))?;
                self.compile_expr(text_expr)?;
                self.emit(Op::Print);
                if !as_stmt {
                    self.emit(Op::None_);
                }
                return Ok(());
            }
            "input" => {
                let prompt_expr = call
                    .args
                    .iter()
                    .find_map(|a| match a {
                        Arg::Named { name, value } if name == "prompt" => Some(value),
                        _ => None,
                    })
                    .or_else(|| {
                        call.args.iter().find_map(|a| match a {
                            Arg::Positional(e) => Some(e),
                            _ => None,
                        })
                    });
                if let Some(e) = prompt_expr {
                    self.compile_expr(e)?;
                } else {
                    let i = self.add_const(Value::Text(String::new()));
                    self.emit(Op::Constant(i));
                }
                self.emit(Op::Input);
                if as_stmt {
                    self.emit(Op::Pop);
                }
                return Ok(());
            }
            "len" | "str" | "int" | "type" | "trim" => {
                let name = call.callee.as_str();
                let value_expr = self
                    .named_or_first(&call, "value")
                    .ok_or_else(|| self.err(format!("{name} requires value")))?;
                self.compile_expr(value_expr)?;
                self.emit(match name {
                    "len" => Op::Len,
                    "str" => Op::Str,
                    "int" => Op::Int,
                    "type" => Op::TypeOf,
                    "trim" => Op::Trim,
                    _ => unreachable!(),
                });
                if as_stmt {
                    self.emit(Op::Pop);
                }
                return Ok(());
            }
            "split" | "join" => {
                let name = call.callee.as_str();
                let value_expr = self
                    .named_or_pos(&call, "value", 0)
                    .ok_or_else(|| self.err(format!("{name} requires value")))?;
                let sep_expr = self
                    .named_or_pos(&call, "sep", 1)
                    .ok_or_else(|| self.err(format!("{name} requires sep")))?;
                self.compile_expr(value_expr)?;
                self.compile_expr(sep_expr)?;
                self.emit(match name {
                    "split" => Op::Split,
                    "join" => Op::Join,
                    _ => unreachable!(),
                });
                if as_stmt {
                    self.emit(Op::Pop);
                }
                return Ok(());
            }
            "at" => {
                let value_expr = self
                    .named_or_pos(&call, "value", 0)
                    .ok_or_else(|| self.err("at requires value"))?;
                let index_expr = self
                    .named_or_pos(&call, "index", 1)
                    .ok_or_else(|| self.err("at requires index"))?;
                self.compile_expr(value_expr)?;
                self.compile_expr(index_expr)?;
                self.emit(Op::GetIndex);
                if as_stmt {
                    self.emit(Op::Pop);
                }
                return Ok(());
            }
            "call_fn" => {
                let name_expr = self
                    .named_or_first(&call, "name")
                    .ok_or_else(|| self.err("call_fn requires name"))?;
                self.compile_expr(name_expr)?;
                self.emit(Op::CallFn);
                if as_stmt {
                    self.emit(Op::Pop);
                }
                return Ok(());
            }
            other if crate::host::HostFn::from_name(other).is_some() => {
                let hf = crate::host::HostFn::from_name(other).unwrap();
                let params = hf.all_params();
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
                let mut pos_i = 0usize;
                let required_n = hf.required_params().len();
                for (i, p) in params.iter().enumerate() {
                    if let Some(e) = named.get(*p) {
                        self.compile_expr(e)?;
                    } else if pos_i < positionals.len() {
                        self.compile_expr(positionals[pos_i])?;
                        pos_i += 1;
                    } else if i < required_n {
                        return Err(self.err(format!("{} requires `{p}`", hf.name())));
                    } else {
                        self.emit(Op::None_);
                    }
                }
                if pos_i < positionals.len() {
                    return Err(self.err(format!(
                        "too many positional arguments ({} extra)",
                        positionals.len() - pos_i
                    )));
                }
                self.emit(Op::HostCall(hf.as_u16(), params.len() as u8));
                if as_stmt {
                    self.emit(Op::Pop);
                }
                return Ok(());
            }
            _ => {}
        }

        match self.resolve_call(&call.callee) {
            Ok(fid) => self.compile_user_call(fid, &call, as_stmt),
            Err(_) => self.compile_plugin_call(&call, as_stmt),
        }
    }

    fn compile_path_call(&mut self, path: &[String], call: &CallExpr, as_stmt: bool) -> Result<()> {
        let display = path.join(".");
        if path.len() < 2 {
            return Err(self.err(format!(
                "library path `{display}`: need at least `lib.member`"
            )));
        }
        let fid = *self.flat[self.fn_id]
            .path_index
            .get(path)
            .ok_or_else(|| self.err(format!("unknown `{display}`")))?;
        if let Some(pid) = self.flat[fid].parent {
            let parent = &self.flat[pid];
            if parent.level == 1 && parent.parent.is_none() && !(self.flat[fid].level == 1) {
                return Err(self.err(format!(
                    "`{display}` is an instance method; call it as `var`.{}",
                    self.flat[fid].name
                )));
            }
        }
        self.compile_user_call(fid, call, as_stmt)
    }

    fn compile_user_call(&mut self, fid: usize, call: &CallExpr, as_stmt: bool) -> Result<()> {
        let params = &self.flat[fid].params;
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
        for p in params {
            if let Some(e) = named.get(&p.name) {
                self.compile_expr(e)?;
            } else if pos_i < positionals.len() {
                self.compile_expr(positionals[pos_i])?;
                pos_i += 1;
            } else if let Some(def) = &p.default {
                self.compile_expr(def)?;
            } else {
                return Err(self.err(format!("missing argument for parameter `{}`", p.name)));
            }
        }
        if pos_i < positionals.len() {
            return Err(self.err(format!(
                "too many positional arguments ({} extra)",
                positionals.len() - pos_i
            )));
        }
        self.emit(Op::Call(fid as u16, params.len() as u8));
        if self.flat[fid].parent.is_none() && self.flat[fid].level == 1 {
            let ti = self.add_const(Value::Text(self.flat[fid].name.clone()));
            self.emit(Op::TagInstance(ti));
        }
        if as_stmt {
            self.emit(Op::Pop);
        }
        Ok(())
    }

    /// Unresolved callee → runtime plugin registry (or unknown-fn error).
    /// Named args: stack pairs (name, value)×N, argc = 2N, name const prefixed with `@`.
    /// Positional: stack values, argc = N, name const as-is.
    fn compile_plugin_call(&mut self, call: &CallExpr, as_stmt: bool) -> Result<()> {
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
        if !named.is_empty() && !positionals.is_empty() {
            return Err(self.err(
                "plugin/unknown calls cannot mix positional and named args in bytecode",
            ));
        }
        if !named.is_empty() {
            let mut pairs: Vec<_> = named.into_iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            for (k, e) in &pairs {
                let ki = self.add_const(Value::Text(k.clone()));
                self.emit(Op::Constant(ki));
                self.compile_expr(e)?;
            }
            let ni = self.add_const(Value::Text(format!("@{}", call.callee)));
            self.emit(Op::PluginCall(ni, (pairs.len() * 2) as u8));
        } else {
            for e in &positionals {
                self.compile_expr(e)?;
            }
            let ni = self.add_const(Value::Text(call.callee.clone()));
            self.emit(Op::PluginCall(ni, positionals.len() as u8));
        }
        if as_stmt {
            self.emit(Op::Pop);
        }
        Ok(())
    }

    fn compile_method_call(&mut self, recv: &str, call: &CallExpr, as_stmt: bool) -> Result<()> {
        // Resolve method among all object children by name (unique enough for compile);
        // runtime still checks `_type`.
        let mut method_fid = None;
        for f in self.flat {
            if f.parent.is_none() && f.level == 1 {
                for &cid in &f.children {
                    if self.flat[cid].name == call.callee {
                        method_fid = Some(cid);
                        break;
                    }
                }
            }
            if method_fid.is_some() {
                break;
            }
        }
        let fid = method_fid.ok_or_else(|| {
            self.err(format!("unknown method `{}`", call.callee))
        })?;
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
            if let Some(e) = named.get(&p.name) {
                self.compile_expr(e)?;
            } else if pos_i < positionals.len() {
                self.compile_expr(positionals[pos_i])?;
                pos_i += 1;
            } else if let Some(def) = &p.default {
                self.compile_expr(def)?;
            } else {
                return Err(self.err(format!("missing argument for parameter `{}`", p.name)));
            }
        }
        if pos_i < positionals.len() {
            return Err(self.err(format!(
                "too many positional arguments ({} extra)",
                positionals.len() - pos_i
            )));
        }
        let slot = self
            .locals
            .get(recv)
            .copied()
            .ok_or_else(|| self.err(format!("undefined variable `{recv}`")))?;
        self.emit(Op::GetLocal(slot));
        let name_i = self.add_const(Value::Text(call.callee.clone()));
        self.emit(Op::MethodCall(name_i, params.len() as u8));
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
                    .ok_or_else(|| self.err(format!("undefined variable `{name}`")))?;
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
                            let slot = self
                                .locals
                                .get(n)
                                .copied()
                                .ok_or_else(|| self.err(format!("undefined variable `{n}`")))?;
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
            Expr::Map(pairs) => {
                for (k, e) in pairs {
                    let ki = self.add_const(Value::Text(k.clone()));
                    self.emit(Op::Constant(ki));
                    self.compile_expr(e)?;
                }
                self.emit(Op::BuildMap(pairs.len() as u16));
            }
            Expr::Formula(e) => {
                let i = self.add_const(Value::Formula(e.clone()));
                self.emit(Op::Constant(i));
            }
            Expr::Code(c) => {
                let i = self.add_const(Value::Code(c.clone()));
                self.emit(Op::Constant(i));
            }
        }
        Ok(())
    }
}

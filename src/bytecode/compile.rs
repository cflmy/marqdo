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

struct FlatFun<'m> {
    owner: &'m Module,
    definition: &'m Function,
    source: crate::diagnostics::SourceIdentity,
    module_path: Vec<String>,
    name: String,
    level: u8,
    /// Receiver-free children of an object are associated functions, not
    /// instance methods. This flag is computed from executable AST only.
    uses_receiver: bool,
    parent: Option<usize>,
    /// Inheritance base type name (`# Child = > Parent`), objects only.
    base: Option<String>,
    params: Vec<Param>,
    contract: Option<crate::contract::Contract>,
    var_contracts: HashMap<String, crate::contract::Contract>,
    body: Vec<Stmt>,
    children: Vec<usize>,
    span: Span,
    /// Full path segments (`lib.member` / `lib.Type.method`) → flat index.
    path_index: HashMap<Vec<String>, usize>,
    /// Short name from frontmatter `use` → path segments.
    uses: HashMap<String, Vec<String>>,
    /// Module collection scope (G02): bare-name resolve must not cross imports.
    scope: u32,
}

pub fn compile_module(path: Option<&Path>, module: &Module) -> Result<Program> {
    compile_with(path, module, true)
}

/// Compile every function even when the file has no single program entry.
/// `Program::entry` is then only a placeholder; callers must use [`Vm::call_function`].
pub fn compile_for_call(path: Option<&Path>, module: &Module) -> Result<Program> {
    compile_with(path, module, false)
}

fn compile_with(path: Option<&Path>, module: &Module, require_entry: bool) -> Result<Program> {
    crate::ast::validate_parameters(module)?;
    let mut flat = Vec::new();
    let mut next_scope = 1u32;
    let _exports = collect_module_tree(module, &mut flat, 0, &mut next_scope, &[]);
    for fun in &mut flat {
        if fun.scope == 0 && fun.source.path.is_none() {
            fun.source.path = path.map(Path::to_path_buf);
        }
    }
    if flat.is_empty() {
        bail!("bytecode: no functions to compile");
    }

    let chosen = flat
        .iter()
        .position(|f| f.scope == 0 && f.name == "main" && f.parent.is_none())
        .or_else(|| {
            let tops: Vec<usize> = flat
                .iter()
                .enumerate()
                .filter(|(_, f)| f.scope == 0 && f.parent.is_none() && f.level == 1 && f.params.is_empty())
                .map(|(i, _)| i)
                .collect();
            if tops.len() == 1 {
                Some(tops[0])
            } else {
                None
            }
        });
    let entry = match chosen {
        Some(index) => index,
        None if !require_entry => 0,
        None => bail!("bytecode: no `# main` object"),
    };
    // Imports are compiled first, so a missing entry must take paths from the file's own functions.
    let path_owner = if chosen.is_some() {
        entry
    } else {
        flat.len() - 1
    };

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
    let mut object_bases = Vec::with_capacity(objects.len());
    for (_, flat_i) in &objects {
        let owner = &flat[*flat_i];
        let base = if let Some(base) = &owner.base {
            let path = owner.uses.get(base).cloned().unwrap_or_else(|| vec![base.clone()]);
            let (name, prefix) = path.split_last().expect("base path");
            let mut wanted = owner.module_path.clone();
            wanted.extend_from_slice(prefix);
            let local = objects.iter().find(|(n,id)| n == name && flat[*id].module_path == wanted).map(|(_,id)| *id);
            if local.is_some() || !prefix.is_empty() { local } else {
                let candidates = objects.iter().filter(|(n,id)| n == name && flat[*id].module_path.starts_with(&owner.module_path)).map(|(_,id)| *id).collect::<Vec<_>>();
                if candidates.len() == 1 { Some(candidates[0]) } else { None }
            }
        } else { None };
        object_bases.push(base);
    }

    let mut functions = Vec::new();
    for i in 0..flat.len() {
        let meta = if i == entry {
            Some(module.metadata.as_slice())
        } else {
            None
        };
        let compiled =
            compile_function(i, &flat, flat[i].source.path.as_deref(), meta).map_err(|error| {
                if let Some(diagnostic) = crate::diagnostics::Diagnostic::find(&error).cloned() {
                    anyhow::Error::new(diagnostic.with_source(&flat[i].source))
                } else {
                    error
                }
            })?;
        functions.push(compiled);
    }

    let parents: Vec<Option<usize>> = flat.iter().map(|f| f.parent).collect();
    let children: Vec<Vec<usize>> = flat.iter().map(|f| f.children.clone()).collect();
    let path_index = flat[path_owner].path_index.clone();

    let program=Program {
        entry_metadata: module.metadata.clone(),
        functions,
        entry,
        objects,
        object_bases,
        methods,
        parents,
        children,
        path_index,
    };
    program.verify().map_err(|error|anyhow::anyhow!(error))?;
    Ok(program)
}

/// Collect `module` and its imports into `flat`.
/// Returns relative export paths within this module (e.g. `["parse"]`, `["agent","step"]`).
///
/// Each module gets a distinct `scope` id. Bare-name `resolve_call` only searches
/// within the caller's scope so imported same-named functions cannot be silently
/// selected (BoundCall / G02). Cross-module calls must use `bind.member` paths.
fn collect_module_tree<'m>(
    module: &'m Module,
    flat: &mut Vec<FlatFun<'m>>,
    scope: u32,
    next_scope: &mut u32,
    module_path: &[String],
) -> HashMap<Vec<String>, usize> {
    let mut path_index = HashMap::new();
    let mut imports = module.import_modules.iter().collect::<Vec<_>>();
    imports.sort_by_key(|(name,_)| *name);
    for (bind, dep) in imports {
        let child_scope = *next_scope;
        *next_scope += 1;
        let mut child_path=module_path.to_vec(); child_path.push(bind.clone());
        let dep_exports = collect_module_tree(dep, flat, child_scope, next_scope, &child_path);
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

    let start = flat.len();
    let mut exports = HashMap::new();
    for fun in &module.functions {
        let id = collect_fun(fun, module, None, flat, &path_index, &uses, scope, &module.source, module_path);
        export_tree(&mut exports, &[], id, flat);
    }
    // Same-module BoundCall paths (`Type.member`) must be visible to siblings.
    for (rel, idx) in &exports {
        path_index.insert(rel.clone(), *idx);
    }
    for f in &mut flat[start..] {
        if f.scope == scope {
            for (rel, idx) in &exports {
                f.path_index.insert(rel.clone(), *idx);
            }
        }
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
    out.entry(p.clone()).or_insert(id);
    for &cid in &flat[id].children {
        export_tree(out, &p, cid, flat);
    }
}

fn collect_fun<'m>(
    fun: &'m Function,
    owner: &'m Module,
    parent: Option<usize>,
    flat: &mut Vec<FlatFun<'m>>,
    path_index: &HashMap<Vec<String>, usize>,
    uses: &HashMap<String, Vec<String>>,
    scope: u32,
    source: &crate::diagnostics::SourceIdentity,
    module_path: &[String],
) -> usize {
    let id = flat.len();
    flat.push(FlatFun {
        owner, definition: fun,
        source: source.clone(),
        module_path: module_path.to_vec(),
        name: fun.name.clone(),
        level: fun.level,
        uses_receiver: crate::ast::function_uses_receiver(fun),
        parent,
        base: fun.base.clone(),
        params: fun.params.clone(),
        contract: fun.contract.clone(),
        var_contracts: fun.var_contracts.clone(),
        body: fun.body.clone(),
        children: Vec::new(),
        span: fun.span,
        path_index: path_index.clone(),
        uses: uses.clone(),
        scope,
    });
    let mut child_ids = Vec::new();
    for child in &fun.children {
        child_ids.push(collect_fun(
            child,
            owner,
            Some(id),
            flat,
            path_index,
            uses,
            scope,
            source,
            module_path,
        ));
    }
    flat[id].children = child_ids;
    id
}

#[derive(Hash, PartialEq, Eq)]
enum ConstantKey {
    None,
    Bool(bool),
    Int(i64),
    Num(u64),
    Text(String),
}

struct FnCompiler<'a> {
    path: Option<&'a Path>,
    flat: &'a [FlatFun<'a>],
    fn_id: usize,
    chunk: FnChunk,
    locals: HashMap<String, usize>,
    stmt_span: Span,
    call_depth: usize,
    constant_indices: HashMap<ConstantKey, usize>,
    argument_scratch: HashMap<(usize,usize),(String,usize)>,
    expression_stack: Vec<usize>,
    statement_stack: Vec<usize>,
    call_emission_stack: Vec<usize>,
}

/// Reserve source names before synthesizing scratch names. This covers future
/// statements as well as defaults, so even an authored backtick name containing
/// the internal prefix cannot alias a temporary.
fn reserve_function_names(fun:&FlatFun,locals:&mut HashMap<String,usize>) {
    fn name(name:&str,locals:&mut HashMap<String,usize>) {
        if !locals.contains_key(name) { let slot=locals.len();locals.insert(name.into(),slot); }
    }
    fn call(call:&CallExpr,locals:&mut HashMap<String,usize>) {
        if let Some(receiver)=&call.receiver { name(receiver,locals); }
        if let Some(path)=&call.path { if let Some(first)=path.first() { name(first,locals); } }
        for arg in &call.args { match arg { Arg::Named{value,..}|Arg::Positional(value)=>expr(value,locals) } }
    }
    fn expr(value:&Expr,locals:&mut HashMap<String,usize>) {
        match value {
            Expr::Var(value)=>name(value,locals),
            Expr::Interp(parts)=>for part in parts { match part {
                InterpPart::Lit(_)=>{},
                InterpPart::Var(value)|InterpPart::Index{base:value,..}=>name(value,locals),
            } },
            Expr::Unary{expr:value,..}=>expr(value,locals),
            Expr::Binary{left,right,..}=>{expr(left,locals);expr(right,locals);},
            Expr::Call(value)=>call(value,locals),
            Expr::List(values)=>for value in values {expr(value,locals);},
            Expr::Map(values)=>for (_,value) in values {expr(value,locals);},
            Expr::Index{base,label}=>{expr(base,locals);if let crate::ast::IndexKey::Var(value)=label {name(value,locals);}},
            Expr::Occluded{payload,..}=>if let Some(value)=payload {expr(value,locals);},
            Expr::Literal(_)|Expr::Formula(_)|Expr::Code(_)=>{},
        }
    }
    fn block(statements:&[Stmt],locals:&mut HashMap<String,usize>) {
        for statement in statements { match statement {
            Stmt::Assign{name:key,value,..}=>{name(key,locals);expr(value,locals);},
            Stmt::Return{value,..}|Stmt::Expr{value,..}=>expr(value,locals),
            Stmt::Call{call:value,..}=>call(value,locals),
            Stmt::Branch{arms,..}=>for arm in arms {if let Some(condition)=&arm.condition {expr(condition,locals);}block(&arm.body,locals);},
            Stmt::While{condition,body,..}=>{expr(condition,locals);block(body,locals);},
            Stmt::ForEach{item,collection,body,..}=>{name(item,locals);name(collection,locals);block(body,locals);},
        } }
    }
    for param in &fun.params {if let Some(default)=&param.default {expr(default,locals);}}
    block(&fun.body,locals);
}

fn compile_function(
    fn_id: usize,
    flat: &[FlatFun],
    path: Option<&Path>,
    metadata: Option<&[(String, Value)]>,
) -> Result<FnChunk> {
    let fun = &flat[fn_id];
    let mut locals = HashMap::new();
    for (i, p) in fun.params.iter().enumerate() {
        locals.insert(p.name.clone(), i as usize);
    }
    let is_method = fun.uses_receiver;
    if is_method {
        for name in ["自", "self"] {
            if !locals.contains_key(name) {
                let s = locals.len() as usize;
                locals.insert(name.to_string(), s);
            }
        }
    }
    // Phase 2: reserve locals for entry metadata keys before body compile.
    if let Some(meta) = metadata {
        for (k, _) in meta {
            if !locals.contains_key(k) {
                let s = locals.len() as usize;
                locals.insert(k.clone(), s);
            }
        }
    }
    let mut c = FnCompiler {
        path,
        flat,
        fn_id,
        chunk: FnChunk {
            source_ast: crate::bytecode::SourceFunction::from_declaration(fun.owner, fun.definition),
            module_scope: fun.scope,
            module_path: fun.module_path.clone(),
            source: fun.source.clone(),
            declaration_span: fun.span,
            name: fun.name.clone(),
            params: fun.params.iter().map(|p| p.name.clone()).collect(),
            contract: fun.contract.clone(),
            var_contracts: fun.var_contracts.clone(),
            receiver_aliases: is_method,
            defaults: fun.params.iter().map(|p| p.default.is_some()).collect(),
            default_expressions:vec![None;fun.params.len()],
            call_layouts: Vec::new(),
            calls: Vec::new(), source_local_count:0,
            expressions: Vec::new(),
            statements: Vec::new(),statement_roots: Vec::new(),body_start:0,
            code: Vec::new(),
            spans: Vec::new(),
            constants: Vec::new(),
            locals: fun.params.iter().map(|p| p.name.clone()).collect(),
        },
        locals,
        stmt_span: fun.span,
        call_depth: 0,
        constant_indices: HashMap::new(),
        argument_scratch: HashMap::new(),
        expression_stack: Vec::new(),
        statement_stack: Vec::new(),
        call_emission_stack: Vec::new(),
    };
    reserve_function_names(fun,&mut c.locals);
    c.chunk.source_local_count=c.locals.len();
    // Defaults execute in the declaration's lexical scope and callee frame.
    // None is a valid supplied value, so omission has a separate presence bit.
    for (index, param) in fun.params.iter().enumerate() {
        if let Some(default) = &param.default {
            c.stmt_span=param.span;
            let jump = c.here();
            c.emit(Op::JumpIfProvided(index as usize, 0));
            c.chunk.default_expressions[index]=Some(c.chunk.expressions.len());
            c.compile_expr(default)?;
            c.emit(Op::SetLocal(index as usize));
            c.emit(Op::Pop);
            c.patch(jump, Op::JumpIfProvided(index as usize, c.here()));
        }
    }
    c.stmt_span=fun.span;
    // Metadata is an input to Program::run, not code baked into a function.
    // Fill only after defaults, so a later omitted default stays unbound while
    // an earlier default evaluates.
    c.emit(Op::FillMissingParameters);
    if fun.contract.is_some() { c.emit(Op::CheckArguments); }
    if is_method {c.emit(Op::RestoreReceiver);if fun.contract.is_some(){c.emit(Op::CheckFrameArguments);}}
    c.chunk.body_start=c.here();
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

    fn here(&self) -> usize {
        self.chunk.code.len() as usize
    }

    fn patch(&mut self, at: usize, op: Op) {
        self.chunk.code[at as usize] = op;
    }

    fn add_const(&mut self, v: Value) -> usize {
        // Index widths follow Vec rather than truncating at 8/16 bits. Scalar
        // interning is linear overall; float keys preserve their exact bits.
        let key = match &v {
            Value::None => Some(ConstantKey::None),
            Value::Bool(value) => Some(ConstantKey::Bool(*value)),
            Value::Int(value) => Some(ConstantKey::Int(*value)),
            Value::Num(value) => Some(ConstantKey::Num(value.to_bits())),
            Value::Text(value) => Some(ConstantKey::Text(value.clone())),
            _ => None,
        };
        if let Some(index) = key.as_ref().and_then(|key| self.constant_indices.get(key)) {
            return *index;
        }
        let i = self.chunk.constants.len();
        if let Some(key) = key { self.constant_indices.insert(key, i); }
        self.chunk.constants.push(v);
        i
    }

    fn fresh_slot(&mut self,label:&str)->usize {
        let mut name=format!("\0{label}.{}",self.locals.len());
        while self.locals.contains_key(&name){name.push('\0');}
        let slot=self.locals.len();self.locals.insert(name,slot);slot
    }

    fn local_slot(&mut self, name: &str) -> usize {
        if let Some(&s) = self.locals.get(name) {
            return s;
        }
        let s = self.locals.len() as usize;
        self.locals.insert(name.to_string(), s);
        s
    }

    fn compile_stmt(&mut self,stmt:&Stmt)->Result<()> {
        let id=self.chunk.statements.len();let start=self.here();
        self.chunk.statements.push(crate::bytecode::StatementEmission{
            rule:crate::bytecode::StatementRule::from_source(stmt),start,end:start,
            statements:Vec::new(),expressions:Vec::new(),auxiliary:Vec::new(),
        });
        if let Some(parent)=self.statement_stack.last(){self.chunk.statements[*parent].statements.push(id);}else{self.chunk.statement_roots.push(id);}
        self.statement_stack.push(id);let result=self.compile_stmt_inner(stmt);self.statement_stack.pop();
        self.chunk.statements[id].end=self.here();result
    }
    fn compile_stmt_inner(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Assign {
                name, value, span, ..
            } => {
                self.stmt_span = *span;
                self.compile_expr(value)?;
                let slot = self.local_slot(name);
                if self.chunk.var_contracts.contains_key(name) { self.emit(Op::CheckBinding(slot)); }
                self.emit(Op::SetLocal(slot));
                self.emit(Op::Pop);
            }
            Stmt::Return { value, span } => {
                self.stmt_span = *span;
                self.compile_expr(value)?;
                if self.chunk.contract.is_some() { self.emit(Op::CheckReturn); }
                self.emit(Op::Return);
            }
            Stmt::Call { call, span } => {
                self.stmt_span = *span;
                self.compile_expr(&Expr::Call(call.clone()))?;
                self.emit(Op::Pop);
            }
            Stmt::Expr { value, span } => {
                self.stmt_span = *span;
                self.compile_expr(value)?;
                self.emit(Op::Pop);
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
                let loop_id=*self.statement_stack.last().unwrap();
                self.emit(Op::ResetLoop(loop_id));
                let loop_start = self.here();
                self.compile_expr(condition)?;
                let exit_j = self.here();
                self.emit(Op::JumpIfFalse(0));
                self.emit(Op::CheckLoop(loop_id));
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
                label: _,
                body,
                span,
            } => {
                self.stmt_span = *span;
                self.compile_expr(&Expr::Var(collection.clone()))?;
                self.emit(Op::IterItems);
                let coll_slot = self.fresh_slot("loop.collection");
                self.emit(Op::SetLocal(coll_slot));
                self.emit(Op::Pop);
                let i_slot = self.fresh_slot("loop.index");
                let statement=*self.statement_stack.last().unwrap();
                self.chunk.statements[statement].auxiliary=vec![coll_slot,i_slot];
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
        let id=self.chunk.calls.len();
        self.chunk.calls.push(crate::bytecode::CallEmission{
            expression:*self.expression_stack.last().ok_or_else(||self.err("call has no source expression"))?,
            source:call.clone(),stages:Vec::new(),dispatch:None,as_statement:as_stmt,
        });
        self.call_emission_stack.push(id);
        self.call_depth += 1;
        let result = self.compile_call_inner(call, as_stmt);
        self.call_depth -= 1;
        self.call_emission_stack.pop();
        result
    }

    fn record_dispatch(&mut self, dispatch:crate::bytecode::CallDispatch){
        let id=*self.call_emission_stack.last().expect("active call emission");
        self.chunk.calls[id].dispatch=Some(dispatch);
    }

    fn compile_call_inner(&mut self, call: &CallExpr, as_stmt: bool) -> Result<()> {
        // Mirror `sema::bind::resolve_for_exec` (G02 BoundCall):
        // static → Op::Call(fid); dynamic → MethodCall / CallFn / PluginCall.
        let mut call = call.for_evaluation().map_err(|m| self.err(m))?;
        let emission=*self.call_emission_stack.last().expect("active call emission");
        self.chunk.calls[emission].source=call.clone();
        // All actuals run once, left to right, including unused extra named
        // arguments. Subsequent parameter permutation only reads temporaries.
        for (index, arg) in call.args.iter_mut().enumerate() {
            let value = match arg {
                Arg::Positional(value) => value,
                Arg::Named { value, .. } => value
            };
            let expression=self.chunk.expressions.len();
            self.compile_expr(value)?;
            let key=(self.call_depth,index);
            let (name,slot)=if let Some(existing)=self.argument_scratch.get(&key) { existing.clone() } else {
                let mut name=format!("\0call.arg.{}.{}", self.call_depth, index);
                while self.locals.contains_key(&name) { name.push('\0'); }
                let slot = self.locals.len() as usize;
                self.locals.insert(name.clone(), slot);
                self.argument_scratch.insert(key,(name.clone(),slot));
                (name,slot)
            };
            self.emit(Op::SetLocal(slot));
            self.emit(Op::Pop);
            self.chunk.calls[emission].stages.push(crate::bytecode::CallStage{expression,slot});
            *value = Expr::Var(name);
        }
        if let Some(recv) = &call.receiver {
            return self.compile_method_call(recv, &call, as_stmt);
        }
        use crate::sema::{ExecTarget, DynamicReason};
        let caller=&self.flat[self.fn_id];
        let target=crate::sema::resolve_for_exec(caller.owner,caller.definition,&call);
        let user=match &target {
            ExecTarget::Local(fun)=>Some((caller.owner,*fun)),
            ExecTarget::Import{lib,fun,..}=>Some((*lib,*fun)),
            _=>None,
        };
        let fid=user.map(|(owner,definition)|self.flat.iter().position(|f|std::ptr::eq(f.owner,owner)&&std::ptr::eq(f.definition,definition))
            .ok_or_else(||self.err("resolved declaration missing from flat program"))).transpose()?;
        if let Some(path)=call.path.as_ref().filter(|p|p.len()==2) {
            let fallback=if let Some(target)=fid { crate::bytecode::PathFallback::User{target,tag_instance:self.flat[target].parent.is_none()&&self.flat[target].level==1} }
                else { crate::bytecode::PathFallback::Error{message:match target {
                    ExecTarget::Unresolved{message,..}=>message,
                    _=>format!("unresolved dynamic call `{}`",crate::sema::call_display(&call)),
                }} };
            self.record_dispatch(crate::bytecode::CallDispatch::Path{receiver:path[0].clone(),check_receiver:!caller.owner.import_modules.contains_key(&path[0]),fallback});
            let layout=self.argument_layout(&call)?;
            self.emit(Op::PathCallArgs(emission,layout));
            if as_stmt{self.emit(Op::Pop);}return Ok(());
        }
        if let Some(fid)=fid{return self.compile_user_call(fid,&call,as_stmt);}
        match target {
          ExecTarget::Builtin(_) | ExecTarget::Dynamic{reason:DynamicReason::CallFn,..}=>{
            self.record_dispatch(crate::bytecode::CallDispatch::Builtin);
            let layout=self.argument_layout(&call)?;let name=self.add_const(Value::Text(call.callee.clone()));
            self.emit(Op::BuiltinCallArgs(name,layout));if as_stmt{self.emit(Op::Pop);}return Ok(());
          },
          ExecTarget::Host(hf)=>{
            self.record_dispatch(crate::bytecode::CallDispatch::Host{function:hf.as_u16()});
            let layout=self.argument_layout(&call)?;self.emit(Op::HostCallArgs(hf.as_u16(),layout));
            if as_stmt{self.emit(Op::Pop);}return Ok(());
          },
          ExecTarget::Dynamic{reason:DynamicReason::Plugin,..}=>self.compile_plugin_call(&call,as_stmt),
          ExecTarget::Unresolved{message,..}=>{
            self.record_dispatch(crate::bytecode::CallDispatch::ResolutionFault{message:message.clone()});
            let index=self.add_const(Value::Text(message));self.emit(Op::RejectCallResolution(index));Ok(())
          },
          _=>Err(self.err("unsupported resolved call")),
        }
    }

    fn argument_layout(&mut self, call: &CallExpr) -> Result<usize> {
        let mut layout = Vec::new();
        for arg in &call.args {
            let (name, expr) = match arg {
                Arg::Positional(expr) => (None, expr),
                Arg::Named { name, value } => (Some(name.clone()), value),
            };
            self.compile_expr(expr)?;
            layout.push(name);
        }
        let index = self.chunk.call_layouts.len() as usize;
        self.chunk.call_layouts.push(layout);
        Ok(index)
    }

    fn compile_user_call(&mut self, fid: usize, call: &CallExpr, as_stmt: bool) -> Result<()> {
        self.record_dispatch(crate::bytecode::CallDispatch::User{target:fid,tag_instance:self.flat[fid].parent.is_none() && self.flat[fid].level==1});
        let layout = self.argument_layout(call)?;
        self.emit(Op::CallArgs(fid as usize, layout));
        if self.flat[fid].parent.is_none() && self.flat[fid].level == 1 {
            self.emit(Op::TagInstance(fid));
        }
        if as_stmt { self.emit(Op::Pop); }
        Ok(())
    }

    /// Unresolved callee → runtime plugin registry (or unknown-fn error).
    /// Preserve authored labels until the runtime registry supplies the signature.
    fn compile_plugin_call(&mut self, call: &CallExpr, as_stmt: bool) -> Result<()> {
        self.record_dispatch(crate::bytecode::CallDispatch::Plugin);
        let layout=self.argument_layout(call)?;
        let name=self.add_const(Value::Text(call.callee.clone()));
        self.emit(Op::PluginCallArgs(name,layout));
        if as_stmt{self.emit(Op::Pop);}Ok(())
    }

    fn compile_method_call(&mut self, recv: &str, call: &CallExpr, as_stmt: bool) -> Result<()> {
        self.record_dispatch(crate::bytecode::CallDispatch::Method{receiver:recv.into()});
        // A method name alone does not determine a signature. Its owner is
        // selected from the actual receiver before runtime argument binding.
        let layout = self.argument_layout(call)?;
        let slot = self.locals.get(recv).copied()
            .ok_or_else(|| self.err(format!("undefined variable `{recv}`")))?;
        self.emit(Op::GetLocal(slot));
        let name = self.add_const(Value::Text(call.callee.clone()));
        self.emit(Op::MethodCallArgs(name, layout));
        if as_stmt { self.emit(Op::Pop); }
        Ok(())
    }

    fn compile_expr(&mut self, expr:&Expr)->Result<()> {
        let id=self.chunk.expressions.len();
        let start=self.here();
        self.chunk.expressions.push(crate::bytecode::ExpressionEmission{
            rule:crate::bytecode::ExpressionRule::from_source(expr),start,end:start,children:Vec::new(),
        });
        if let Some(parent)=self.expression_stack.last(){self.chunk.expressions[*parent].children.push(id);}else if let Some(stmt)=self.statement_stack.last(){self.chunk.statements[*stmt].expressions.push(id);}
        self.expression_stack.push(id);
        let result=self.compile_expr_inner(expr);
        self.expression_stack.pop();
        self.chunk.expressions[id].end=self.here();
        result
    }

    fn compile_expr_inner(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            // `<TAB>` M2: occlusion is transparent for execution — only the
            // payload runs; the label is diagnosed/rendered, never evaluated.
            Expr::Occluded { payload, .. } => match payload {
                Some(inner) => self.compile_expr(inner)?,
                None => self.emit(Op::None_),
            },
            Expr::Literal(lit) => match lit {
                Literal::None => self.emit(Op::None_),
                Literal::Bool(true) => self.emit(Op::True),
                Literal::Bool(false) => self.emit(Op::False),
                Literal::Int(n) => {
                    let i = self.add_const(Value::Int(*n));
                    self.emit(Op::Constant(i));
                }
                Literal::Num(n) => {
                    let i = self.add_const(Value::Num(*n));
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
                        InterpPart::Index { base, labels } => {
                            let slot =
                                self.locals.get(base).copied().ok_or_else(|| {
                                    self.err(format!("undefined variable `{base}`"))
                                })?;
                            self.emit(Op::GetLocal(slot));
                            for label in labels {
                                let li = self.add_const(Value::Text(label.clone()));
                                self.emit(Op::Constant(li));
                                self.emit(Op::FootnoteGet);
                            }
                        }
                    }
                    self.emit(Op::Str);
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
            Expr::Call(call) => {
                if let Err(message)=call.for_evaluation(){
                    let index=self.add_const(Value::Text(message));
                    self.emit(Op::RejectArgumentExpansion(index));
                }else{self.compile_call(call, false)?;}
            },
            Expr::List(items) => {
                for it in items {
                    self.compile_expr(it)?;
                }
                self.emit(Op::BuildList(items.len() as usize));
            }
            Expr::Map(pairs) => {
                for (k, e) in pairs {
                    let ki = self.add_const(Value::Text(k.clone()));
                    self.emit(Op::Constant(ki));
                    self.compile_expr(e)?;
                }
                self.emit(Op::BuildMap(pairs.len() as usize));
            }
            Expr::Index { base, label } => {
                self.compile_expr(base)?;
                match label {
                    crate::ast::IndexKey::Lit(s) => {
                        let li = self.add_const(Value::Text(s.clone()));
                        self.emit(Op::Constant(li));
                    }
                    crate::ast::IndexKey::Var(name) => {
                        let slot = self.locals.get(name).copied().ok_or_else(|| {
                            self.err(format!("undefined variable `{name}` in footnote key"))
                        })?;
                        self.emit(Op::GetLocal(slot));
                        self.emit(Op::Str);
                    }
                }
                if let Expr::Var(name)=base.as_ref() {
                    if self.chunk.var_contracts.contains_key(name) {
                        self.emit(Op::ContractFootnoteGet(self.locals[name]));
                    } else { self.emit(Op::FootnoteGet); }
                } else { self.emit(Op::FootnoteGet); }
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

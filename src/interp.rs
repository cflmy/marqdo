//! Tree-walk interpreter (Phase I).

use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{
    Arg, BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Param, Stmt, UnaryOp,
};
use crate::builtin::{
    builtin_at, builtin_footnote_get, builtin_int, builtin_iter_items, builtin_join,
    builtin_len, builtin_split, builtin_str, builtin_trim,
    builtin_type,
};
use crate::debug::{emit_trace, DebugController};
use crate::diagnostics::{bail_at, Span};
use crate::host::{call_host, HostContext, HostFn};
use crate::input_feed::InputFeed;
use crate::value::Value;
use std::sync::Arc;

pub struct Interpreter {
    pub path: Option<PathBuf>,
    pub trace: bool,
    capture: bool,
    pub captured_stdout: String,
    current_span: Span,
    input: InputFeed,
    host: HostContext,
    debug: Option<Arc<DebugController>>,
}

struct Env {
    vars: HashMap<String, Value>,
}

impl Env {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    fn set(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

enum EvArg {
    Positional(Value),
    Named(String, Value),
}

impl Interpreter {
    pub fn new(path: Option<&Path>, trace: bool) -> Self {
        Self {
            path: path.map(|p| p.to_path_buf()),
            trace,
            capture: false,
            captured_stdout: String::new(),
            current_span: Span::new(1, 1),
            input: InputFeed::new(false, Vec::new()),
            host: HostContext::for_run(path, Default::default(), Vec::new()),
            debug: None,
        }
    }

    pub fn with_capture(path: Option<&Path>, trace: bool) -> Self {
        Self {
            path: path.map(|p| p.to_path_buf()),
            trace,
            capture: true,
            captured_stdout: String::new(),
            current_span: Span::new(1, 1),
            input: InputFeed::new(true, Vec::new()),
            host: HostContext::for_capture(path, Default::default()),
            debug: None,
        }
    }

    pub fn with_stdin(mut self, lines: Vec<String>) -> Self {
        self.input = InputFeed::new(self.capture, lines);
        self
    }

    pub fn with_host(mut self, host: HostContext) -> Self {
        self.host = host;
        self
    }

    pub fn with_debug(mut self, debug: Arc<DebugController>) -> Self {
        self.debug = Some(debug);
        self
    }

    pub fn take_plots(&mut self) -> Vec<crate::host::PlotArtifact> {
        std::mem::take(&mut self.host.plots)
    }

    fn err(&self, message: impl Into<String>) -> anyhow::Error {
        bail_at(self.path.as_deref(), self.current_span, message)
    }

    fn emit_line(&mut self, text: &str) {
        if self.capture {
            self.captured_stdout.push_str(text);
            self.captured_stdout.push('\n');
        } else {
            println!("{text}");
        }
    }

    fn emit_prompt(&mut self, prompt: &str) {
        if prompt.is_empty() {
            return;
        }
        if self.capture {
            self.captured_stdout.push_str(prompt);
        } else {
            print!("{prompt}");
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }

    /// Run a function by name anywhere in the module (for `lib/subtask` workers).
    pub fn invoke_function(
        &mut self,
        module: &Module,
        name: &str,
        args: &[(String, Value)],
    ) -> Result<Value> {
        let fun = find_function_anywhere(module, name)
            .ok_or_else(|| self.err(format!("unknown function `{name}`")))?;
        self.run_function(module, fun, Env::new(), args)
    }

    pub fn run_module(&mut self, module: &Module) -> Result<Value> {
        if let Some(main) = find_top(module, "main") {
            return self.run_function(module, main, Env::new(), &[]);
        }
        // Document-as-entry: sole top-level `#` object with no params (e.g. `# Hello World`).
        let entries: Vec<&Function> = module
            .functions
            .iter()
            .filter(|f| f.is_object() && f.params.is_empty())
            .collect();
        if entries.len() == 1 {
            return self.run_function(module, entries[0], Env::new(), &[]);
        }
        bail!("no `# main` object to run");
    }

    fn run_function(
        &mut self,
        module: &Module,
        fun: &Function,
        mut env: Env,
        args: &[(String, Value)],
    ) -> Result<Value> {
        if self.trace {
            emit_trace(
                self.path.as_deref(),
                Some(fun.span),
                "enter_fn",
                &[("fn", fun.name.as_str())],
            );
        }
        for (k, v) in args {
            env.set(k.clone(), v.clone());
        }
        for p in &fun.params {
            if env.get(&p.name).is_none() {
                env.set(p.name.clone(), Value::None);
            }
        }

        let ret = Value::None;
        self.host.push_call_frame(&fun.name);
        let body_result = (|| -> Result<Value> {
            for stmt in &fun.body {
                if let Some(v) = self.exec_stmt(module, fun, &mut env, stmt)? {
                    return Ok(v);
                }
            }
            Ok(ret)
        })();
        self.host.pop_call_frame();
        let ret = body_result?;
        if self.trace {
            let display = ret.as_display();
            emit_trace(
                self.path.as_deref(),
                Some(fun.span),
                "leave_fn",
                &[("fn", fun.name.as_str()), ("value", display.as_str())],
            );
        }
        Ok(ret)
    }

    fn exec_stmt(
        &mut self,
        module: &Module,
        fun: &Function,
        env: &mut Env,
        stmt: &Stmt,
    ) -> Result<Option<Value>> {
        let span = stmt_span(stmt);
        self.current_span = span;
        self.host.current_line = span.line;
        if let Some(dbg) = &self.debug {
            dbg.on_stmt(span.line, &fun.name, &env.vars, &self.captured_stdout)
                .map_err(|m| self.err(m))?;
        }
        match stmt {
            Stmt::Assign { name, value, span, .. } => {
                self.current_span = *span;
                let v = self.eval_expr(module, fun, env, value)?;
                if self.trace {
                    let display = v.as_display();
                    emit_trace(
                        self.path.as_deref(),
                        Some(*span),
                        "stmt",
                        &[
                            ("kind", "assign"),
                            ("name", name.as_str()),
                            ("value", display.as_str()),
                        ],
                    );
                }
                env.set(name.clone(), v);
                Ok(None)
            }
            Stmt::Return { value, span } => {
                self.current_span = *span;
                let v = self.eval_expr(module, fun, env, value)?;
                if self.trace {
                    let display = v.as_display();
                    emit_trace(
                        self.path.as_deref(),
                        Some(*span),
                        "stmt",
                        &[("kind", "return"), ("value", display.as_str())],
                    );
                }
                Ok(Some(v))
            }
            Stmt::Call { call, span } => {
                self.current_span = *span;
                if self.trace {
                    emit_trace(
                        self.path.as_deref(),
                        Some(*span),
                        "stmt",
                        &[("kind", "call"), ("callee", call.callee.as_str())],
                    );
                }
                let _ = self.eval_call(module, fun, env, call)?;
                Ok(None)
            }
            Stmt::Branch { arms, span } => {
                self.current_span = *span;
                if self.trace {
                    emit_trace(
                        self.path.as_deref(),
                        Some(*span),
                        "stmt",
                        &[("kind", "branch")],
                    );
                }
                for arm in arms {
                    let take = match &arm.condition {
                        None => true,
                        Some(cond) => self.eval_expr(module, fun, env, cond)?.truthy(),
                    };
                    if take {
                        for s in &arm.body {
                            if let Some(v) = self.exec_stmt(module, fun, env, s)? {
                                return Ok(Some(v));
                            }
                        }
                        break;
                    }
                }
                Ok(None)
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                self.current_span = *span;
                if self.trace {
                    emit_trace(
                        self.path.as_deref(),
                        Some(*span),
                        "stmt",
                        &[("kind", "while")],
                    );
                }
                let mut guard = 0u32;
                while self.eval_expr(module, fun, env, condition)?.truthy() {
                    guard += 1;
                    if guard > 1_000_000 {
                        return Err(self.err("while loop exceeded iteration limit"));
                    }
                    for s in body {
                        if let Some(v) = self.exec_stmt(module, fun, env, s)? {
                            return Ok(Some(v));
                        }
                    }
                }
                Ok(None)
            }
            Stmt::ForEach {
                item,
                collection,
                body,
                span,
            } => {
                self.current_span = *span;
                if self.trace {
                    emit_trace(
                        self.path.as_deref(),
                        Some(*span),
                        "stmt",
                        &[
                            ("kind", "foreach"),
                            ("item", item.as_str()),
                            ("collection", collection.as_str()),
                        ],
                    );
                }
                let coll = env.get(collection).cloned().ok_or_else(|| {
                    self.err(format!("undefined collection `{collection}`"))
                })?;
                let items = match builtin_iter_items(&coll) {
                    Ok(Value::List(xs)) => xs,
                    Ok(_) => unreachable!(),
                    Err(m) => return Err(self.err(m)),
                };
                for val in items {
                    env.set(item.clone(), val);
                    for s in body {
                        if let Some(v) = self.exec_stmt(module, fun, env, s)? {
                            return Ok(Some(v));
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    fn eval_expr(
        &mut self,
        module: &Module,
        fun: &Function,
        env: &mut Env,
        expr: &Expr,
    ) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Ok(lit_to_value(lit)),
            Expr::Var(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| self.err(format!("undefined variable `{name}`"))),
            Expr::Interp(parts) => {
                let mut s = String::new();
                for part in parts {
                    match part {
                        InterpPart::Lit(t) => s.push_str(t),
                        InterpPart::Var(n) => {
                            let v = env
                                .get(n)
                                .ok_or_else(|| self.err(format!("undefined variable `{n}`")))?;
                            s.push_str(&v.as_display());
                        }
                    }
                }
                Ok(Value::Text(s))
            }
            Expr::Unary { op, expr } => {
                let v = self.eval_expr(module, fun, env, expr)?;
                match op {
                    UnaryOp::Not => Ok(Value::Bool(!v.truthy())),
                    UnaryOp::Neg => match v {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        _ => Err(self.err("unary `-` needs int")),
                    },
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(module, fun, env, left)?;
                match op {
                    BinaryOp::And => {
                        if !l.truthy() {
                            return Ok(Value::Bool(false));
                        }
                        let r = self.eval_expr(module, fun, env, right)?;
                        return Ok(Value::Bool(r.truthy()));
                    }
                    BinaryOp::Or => {
                        if l.truthy() {
                            return Ok(Value::Bool(true));
                        }
                        let r = self.eval_expr(module, fun, env, right)?;
                        return Ok(Value::Bool(r.truthy()));
                    }
                    _ => {}
                }
                let r = self.eval_expr(module, fun, env, right)?;
                eval_binary(*op, &l, &r).map_err(|e| self.err(e.to_string()))
            }
            Expr::Call(call) => self.eval_call(module, fun, env, call),
            Expr::List(items) => {
                let mut out = Vec::new();
                for it in items {
                    out.push(self.eval_expr(module, fun, env, it)?);
                }
                Ok(Value::List(out))
            }
            Expr::Map(pairs) => {
                let mut out = Vec::new();
                for (k, e) in pairs {
                    out.push((k.clone(), self.eval_expr(module, fun, env, e)?));
                }
                Ok(Value::Map(out))
            }
            Expr::Index { base, label } => {
                let v = self.eval_expr(module, fun, env, base)?;
                builtin_footnote_get(&v, label).map_err(|m| self.err(m))
            }
            Expr::Formula(e) => Ok(Value::Formula(e.clone())),
            Expr::Code(c) => Ok(Value::Code(c.clone())),
        }
    }

    fn eval_call(
        &mut self,
        module: &Module,
        fun: &Function,
        env: &mut Env,
        call: &CallExpr,
    ) -> Result<Value> {
        let mut call = call.clone();
        if call.path.is_none() {
            let callee = crate::aliases::normalize_call_callee_and_args(
                &call.callee,
                &mut call.args,
            );
            call.callee = callee;
        }

        let mut ev_args = Vec::new();
        for arg in &call.args {
            match arg {
                Arg::Positional(e) => {
                    ev_args.push(EvArg::Positional(self.eval_expr(module, fun, env, e)?));
                }
                Arg::Named { name, value } => {
                    ev_args.push(EvArg::Named(
                        name.clone(),
                        self.eval_expr(module, fun, env, value)?,
                    ));
                }
            }
        }

        if let Some(recv_name) = &call.receiver {
            return self.eval_method_call(module, fun, env, recv_name, &call.callee, &ev_args);
        }

        if let Some(path) = &call.path {
            return self.eval_path_call(module, path, &ev_args);
        }

        if let Some(u) = module.uses.iter().find(|u| u.bind == call.callee) {
            return self.eval_path_call(module, &u.path, &ev_args);
        }

        match call.callee.as_str() {
            "print" => {
                let bound = bind_args(&["text".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let text = bound
                    .get("text")
                    .map(|v| v.as_display())
                    .ok_or_else(|| self.err("print requires text (named or positional)"))?;
                self.emit_line(&text);
                return Ok(Value::None);
            }
            "input" => {
                let bound = bind_args(&["prompt".into()], &ev_args, true)
                    .map_err(|m| self.err(m))?;
                let prompt = bound
                    .get("prompt")
                    .map(|v| v.as_display())
                    .unwrap_or_default();
                self.emit_prompt(&prompt);
                let line = self.input.read_line().map_err(|e| {
                    let msg = e.to_string();
                    if msg.contains("input needs a line") {
                        self.err(msg)
                    } else {
                        e
                    }
                })?;
                return Ok(Value::Text(line));
            }
            "len" => {
                let bound = bind_args(&["value".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let v = bound
                    .get("value")
                    .ok_or_else(|| self.err("len requires value"))?;
                let n = builtin_len(v).map_err(|m| self.err(m))?;
                return Ok(Value::Int(n));
            }
            "str" => {
                let bound = bind_args(&["value".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let v = bound
                    .get("value")
                    .ok_or_else(|| self.err("str requires value"))?;
                return Ok(builtin_str(v));
            }
            "int" => {
                let bound = bind_args(&["value".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let v = bound
                    .get("value")
                    .ok_or_else(|| self.err("int requires value"))?;
                let n = builtin_int(v).map_err(|m| self.err(m))?;
                return Ok(Value::Int(n));
            }
            "type" => {
                let bound = bind_args(&["value".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let v = bound
                    .get("value")
                    .ok_or_else(|| self.err("type requires value"))?;
                return Ok(builtin_type(v));
            }
            "trim" => {
                let bound = bind_args(&["value".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let v = bound
                    .get("value")
                    .ok_or_else(|| self.err("trim requires value"))?;
                return builtin_trim(v).map_err(|m| self.err(m));
            }
            "split" => {
                let bound = bind_args(&["value".into(), "sep".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let value = bound
                    .get("value")
                    .ok_or_else(|| self.err("split requires value"))?;
                let sep = bound
                    .get("sep")
                    .ok_or_else(|| self.err("split requires sep"))?;
                return builtin_split(value, sep).map_err(|m| self.err(m));
            }
            "join" => {
                let bound = bind_args(&["value".into(), "sep".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let value = bound
                    .get("value")
                    .ok_or_else(|| self.err("join requires value"))?;
                let sep = bound
                    .get("sep")
                    .ok_or_else(|| self.err("join requires sep"))?;
                return builtin_join(value, sep).map_err(|m| self.err(m));
            }
            "at" => {
                let bound = bind_args(&["value".into(), "index".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let value = bound
                    .get("value")
                    .ok_or_else(|| self.err("at requires value"))?;
                let index = bound
                    .get("index")
                    .ok_or_else(|| self.err("at requires index"))?;
                return builtin_at(value, index).map_err(|m| self.err(m));
            }
            "call_fn" => {
                let bound = bind_args(&["name".into()], &ev_args, false)
                    .map_err(|m| self.err(m))?;
                let name = match bound.get("name") {
                    Some(Value::Text(s)) => s.as_str(),
                    _ => return Err(self.err("call_fn requires text name")),
                };
                let target = lookup_function(module, fun, name).ok_or_else(|| {
                    self.err(format!("call_fn: unknown function `{name}`"))
                })?;
                if !target.params.is_empty() {
                    return Err(self.err(format!(
                        "call_fn: `{name}` must have no parameters"
                    )));
                }
                return self.run_function(module, target, Env::new(), &[]);
            }
            other if HostFn::from_name(other).is_some() => {
                let hf = HostFn::from_name(other).unwrap();
                let mut params: Vec<String> =
                    hf.required_params().iter().map(|s| (*s).to_string()).collect();
                for p in hf.optional_params() {
                    params.push((*p).to_string());
                }
                let bound = bind_args(&params, &ev_args, true).map_err(|m| self.err(m))?;
                for req in hf.required_params() {
                    if !bound.contains_key(*req) {
                        return Err(self.err(format!("{} requires `{req}`", hf.name())));
                    }
                }
                return call_host(&mut self.host, hf, &bound).map_err(|m| self.err(m));
            }
            _ => {}
        }

        if let Some(reg) = self.host.plugins.get(&call.callee).cloned() {
            let bound = bind_args(&reg.params, &ev_args, false).map_err(|m| self.err(m))?;
            return crate::host::plugin::call_registered(&self.host, &call.callee, &bound)
                .map_err(|m| self.err(m));
        }

        let target = lookup_function(module, fun, &call.callee)
            .ok_or_else(|| self.err(format!("unknown function `{}`", call.callee)))?;

        let bound = bind_function_args(self, module, fun, env, &target.params, &ev_args, false)
            .map_err(|m| self.err(m))?;
        let mut call_env = Env::new();
        for (k, v) in bound {
            call_env.set(k, v);
        }

        self.host.push_call_site_line(self.host.current_line);
        let result = self.run_function(module, target, call_env, &[]);
        self.host.pop_call_site_line();
        let result = result?;
        if target.is_object() {
            Ok(tag_instance(&target.name, result))
        } else {
            Ok(result)
        }
    }

    /// Resolve bare path `lib.member` / `lib.Type.member` (definition-tree addressing).
    fn eval_path_call(
        &mut self,
        module: &Module,
        path: &[String],
        ev_args: &[EvArg],
    ) -> Result<Value> {
        let display = path.join(".");
        if path.len() < 2 {
            return Err(self.err(format!("library path `{display}`: need at least `lib.member`")));
        }
        let lib_name = &path[0];
        let lib = module.import_modules.get(lib_name).ok_or_else(|| {
            self.err(format!("unknown library `{lib_name}` (import it in frontmatter)"))
        })?;

        let mut parent_is_object = false;
        let mut node = lib
            .functions
            .iter()
            .find(|f| f.name == path[1])
            .ok_or_else(|| self.err(format!("unknown `{display}`")))?;
        for seg in &path[2..] {
            parent_is_object = node.is_object();
            node = node
                .children
                .iter()
                .find(|c| c.name == *seg)
                .ok_or_else(|| self.err(format!("unknown `{display}`")))?;
        }

        if parent_is_object && !node.is_object() {
            return Err(self.err(format!(
                "`{display}` is an instance method; call it as `var`.{}",
                node.name
            )));
        }

        let bound = {
            let mut dummy_env = Env::new();
            bind_function_args(
                self,
                lib,
                node,
                &mut dummy_env,
                &node.params,
                ev_args,
                false,
            )
            .map_err(|m| self.err(m))?
        };
        let mut call_env = Env::new();
        for (k, v) in bound {
            call_env.set(k, v);
        }

        self.host.push_call_site_line(self.host.current_line);
        let result = self.run_function(lib, node, call_env, &[]);
        self.host.pop_call_site_line();
        let result = result?;
        if node.is_object() {
            Ok(tag_instance(&node.name, result))
        } else {
            Ok(result)
        }
    }

    fn eval_method_call(
        &mut self,
        module: &Module,
        fun: &Function,
        env: &mut Env,
        recv_name: &str,
        method: &str,
        ev_args: &[EvArg],
    ) -> Result<Value> {
        let recv = env
            .get(recv_name)
            .cloned()
            .ok_or_else(|| self.err(format!("undefined variable `{recv_name}`")))?;
        let type_name = instance_type_name(&recv).map_err(|m| self.err(m))?;
        let (owner, obj) = find_object_type(module, &type_name)
            .ok_or_else(|| self.err(format!("unknown object type `{type_name}`")))?;
        let target = obj
            .children
            .iter()
            .find(|c| c.name == method)
            .ok_or_else(|| self.err(format!("unknown method `{type_name}.{method}`")))?;
        let bound = bind_function_args(self, module, fun, env, &target.params, ev_args, false)
            .map_err(|m| self.err(m))?;
        let mut call_env = Env::new();
        for (k, v) in bound {
            call_env.set(k, v);
        }
        call_env.set("自".into(), recv.clone());
        call_env.set("self".into(), recv);
        self.host.push_call_site_line(self.host.current_line);
        let result = self.run_function(owner, target, call_env, &[]);
        self.host.pop_call_site_line();
        result
    }
}

fn instance_type_name(v: &Value) -> std::result::Result<String, String> {
    match v {
        Value::Map(entries) => entries
            .iter()
            .find(|(k, _)| k == "_type")
            .and_then(|(_, v)| match v {
                Value::Text(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| "method receiver needs a map with `_type`".into()),
        _ => Err("method receiver must be an object map".into()),
    }
}

fn tag_instance(type_name: &str, value: Value) -> Value {
    match value {
        Value::Map(mut entries) => {
            if !entries.iter().any(|(k, _)| k == "_type") {
                entries.insert(0, ("_type".into(), Value::Text(type_name.into())));
            }
            Value::Map(entries)
        }
        Value::None => Value::Map(vec![("_type".into(), Value::Text(type_name.into()))]),
        other => Value::Map(vec![
            ("_type".into(), Value::Text(type_name.into())),
            ("value".into(), other),
        ]),
    }
}

/// Bind evaluated args onto parameter names. Err is a plain message (caller adds span/path).
fn bind_function_args(
    interp: &mut Interpreter,
    module: &Module,
    caller: &Function,
    caller_env: &mut Env,
    params: &[Param],
    args: &[EvArg],
    allow_missing: bool,
) -> std::result::Result<HashMap<String, Value>, String> {
    let mut named: HashMap<String, Value> = HashMap::new();
    let mut positionals = Vec::new();
    for a in args {
        match a {
            EvArg::Positional(v) => positionals.push(v.clone()),
            EvArg::Named(k, v) => {
                if named.contains_key(k) {
                    return Err(format!("duplicate named argument `{k}`"));
                }
                named.insert(k.clone(), v.clone());
            }
        }
    }

    let mut out: HashMap<String, Value> = HashMap::new();
    let mut used_named = HashSet::new();
    let mut pos_i = 0usize;

    for p in params {
        if let Some(v) = named.get(&p.name) {
            out.insert(p.name.clone(), v.clone());
            used_named.insert(p.name.clone());
        } else if pos_i < positionals.len() {
            out.insert(p.name.clone(), positionals[pos_i].clone());
            pos_i += 1;
        } else if let Some(def) = &p.default {
            let v = interp
                .eval_expr(module, caller, caller_env, def)
                .map_err(|e| e.to_string())?;
            out.insert(p.name.clone(), v);
        } else if !allow_missing {
            return Err(format!("missing argument for parameter `{}`", p.name));
        }
    }

    if pos_i < positionals.len() {
        return Err(format!(
            "too many positional arguments ({} extra)",
            positionals.len() - pos_i
        ));
    }

    for (k, v) in named {
        if !used_named.contains(&k) && !out.contains_key(&k) {
            out.insert(k, v);
        }
    }

    Ok(out)
}

/// Bind evaluated args onto parameter names (host / plugin functions).
fn bind_args(
    params: &[String],
    args: &[EvArg],
    allow_missing: bool,
) -> std::result::Result<HashMap<String, Value>, String> {
    let mut named: HashMap<String, Value> = HashMap::new();
    let mut positionals = Vec::new();
    for a in args {
        match a {
            EvArg::Positional(v) => positionals.push(v.clone()),
            EvArg::Named(k, v) => {
                if named.contains_key(k) {
                    return Err(format!("duplicate named argument `{k}`"));
                }
                named.insert(k.clone(), v.clone());
            }
        }
    }

    let mut out: HashMap<String, Value> = HashMap::new();
    let mut used_named = HashSet::new();
    let mut pos_i = 0usize;

    for p in params {
        if let Some(v) = named.get(p) {
            out.insert(p.clone(), v.clone());
            used_named.insert(p.clone());
        } else if pos_i < positionals.len() {
            out.insert(p.clone(), positionals[pos_i].clone());
            pos_i += 1;
        } else if !allow_missing {
            return Err(format!("missing argument for parameter `{p}`"));
        }
    }

    if pos_i < positionals.len() {
        return Err(format!(
            "too many positional arguments ({} extra)",
            positionals.len() - pos_i
        ));
    }

    for (k, v) in named {
        if !used_named.contains(&k) && !out.contains_key(&k) {
            out.insert(k, v);
        }
    }

    Ok(out)
}

fn find_top<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module.functions.iter().find(|f| f.name == name)
}

/// Resolve `#` object type by name in the entry module or any imported library.
fn find_object_type<'a>(module: &'a Module, name: &str) -> Option<(&'a Module, &'a Function)> {
    module
        .functions
        .iter()
        .find(|f| f.name == name && f.is_object())
        .map(|f| (module, f))
        .or_else(|| {
            module.import_modules.values().find_map(|lib| {
                lib.functions
                    .iter()
                    .find(|f| f.name == name && f.is_object())
                    .map(|f| (lib, f))
            })
        })
}

fn find_function_anywhere<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    for f in &module.functions {
        if f.name == name {
            return Some(f);
        }
        if let Some(child) = find_in_tree(f, name) {
            return Some(child);
        }
    }
    None
}

fn lookup_function<'a>(
    module: &'a Module,
    current: &'a Function,
    name: &str,
) -> Option<&'a Function> {
    if let Some(f) = find_in_tree(current, name) {
        return Some(f);
    }
    module.functions.iter().find(|f| f.name == name)
}

fn find_in_tree<'a>(fun: &'a Function, name: &str) -> Option<&'a Function> {
    for child in &fun.children {
        if child.name == name {
            return Some(child);
        }
        if let Some(f) = find_in_tree(child, name) {
            return Some(f);
        }
    }
    None
}

fn lit_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::None => Value::None,
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Int(n) => Value::Int(*n),
        Literal::Text(s) => Value::Text(s.clone()),
    }
}

fn eval_binary(op: BinaryOp, l: &Value, r: &Value) -> Result<Value> {
    match op {
        BinaryOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{a}{b}"))),
            (Value::Text(a), b) => Ok(Value::Text(format!("{a}{}", b.as_display()))),
            (a, Value::Text(b)) => Ok(Value::Text(format!("{}{b}", a.as_display()))),
            _ => bail!("`+` needs ints or text"),
        },
        BinaryOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            _ => bail!("`-` needs ints"),
        },
        BinaryOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            _ => bail!("`*` needs ints"),
        },
        BinaryOp::Div => match (l, r) {
            (Value::Int(_), Value::Int(0)) => bail!("division by zero"),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            _ => bail!("`/` needs ints"),
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

fn cmp_ord(l: &Value, r: &Value, f: impl Fn(std::cmp::Ordering) -> bool) -> Result<Value> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        (Value::Num(a), Value::Num(b)) => Ok(Value::Bool(f(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)))),
        (Value::Int(a), Value::Num(b)) => {
            Ok(Value::Bool(f((*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))))
        }
        (Value::Num(a), Value::Int(b)) => {
            Ok(Value::Bool(f(a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal))))
        }
        (Value::Text(a), Value::Text(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        _ => bail!("comparison needs two ints/nums or two texts"),
    }
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Assign { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Branch { span, .. }
        | Stmt::While { span, .. }
        | Stmt::ForEach { span, .. } => *span,
    }
}

//! Tree-walk interpreter (Phase I).

use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{
    Arg, BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Stmt, UnaryOp,
};
use crate::builtin::{builtin_int, builtin_len, builtin_str};
use crate::debug::emit_trace;
use crate::diagnostics::{bail_at, Span};
use crate::input_feed::InputFeed;
use crate::value::Value;

pub struct Interpreter {
    pub path: Option<PathBuf>,
    pub trace: bool,
    capture: bool,
    pub captured_stdout: String,
    current_span: Span,
    input: InputFeed,
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
        }
    }

    pub fn with_stdin(mut self, lines: Vec<String>) -> Self {
        self.input = InputFeed::new(self.capture, lines);
        self
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

    pub fn run_module(&mut self, module: &Module) -> Result<Value> {
        if let Some(main) = find_top(module, "main") {
            return self.run_function(module, main, Env::new(), &[]);
        }
        let tops: Vec<&Function> = module.functions.iter().filter(|f| f.level == 1).collect();
        if tops.is_empty() {
            bail!("no `# main` and no level-1 function to run");
        }
        let mut last = Value::None;
        for fun in tops {
            if fun.params.is_empty() {
                last = self.run_function(module, fun, Env::new(), &[])?;
            }
        }
        Ok(last)
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
            if env.get(p).is_none() {
                env.set(p.clone(), Value::None);
            }
        }

        let mut ret = Value::None;
        for stmt in &fun.body {
            if let Some(v) = self.exec_stmt(module, fun, &mut env, stmt)? {
                ret = v;
                break;
            }
        }
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
        match stmt {
            Stmt::Assign { name, value, span } => {
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
                let items = match coll {
                    Value::List(xs) => xs,
                    other => {
                        return Err(self.err(format!("foreach needs a list, got {other:?}")));
                    }
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
        }
    }

    fn eval_call(
        &mut self,
        module: &Module,
        fun: &Function,
        env: &mut Env,
        call: &CallExpr,
    ) -> Result<Value> {
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
                    if e.to_string().contains("input is not available") {
                        self.err("input is not available under capture / view")
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
            _ => {}
        }

        let target = lookup_function(module, fun, &call.callee)
            .ok_or_else(|| self.err(format!("unknown function `{}`", call.callee)))?;

        let bound = bind_args(&target.params, &ev_args, false).map_err(|m| self.err(m))?;
        let mut call_env = Env::new();
        for (k, v) in bound {
            call_env.set(k, v);
        }

        self.run_function(module, target, call_env, &[])
    }
}

/// Bind evaluated args onto parameter names. Err is a plain message (caller adds span/path).
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
        (Value::Text(a), Value::Text(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        _ => bail!("comparison needs two ints or two texts"),
    }
}

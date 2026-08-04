//! Tree-walk interpreter (Phase I).

use anyhow::{bail, Result};
use std::collections::HashMap;

use crate::ast::{
    BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Stmt, UnaryOp,
};
use crate::value::Value;

pub struct Interpreter {
    pub trace: bool,
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

impl Interpreter {
    pub fn new(trace: bool) -> Self {
        Self { trace }
    }

    pub fn run_module(&mut self, module: &Module) -> Result<Value> {
        if let Some(main) = find_top(module, "main") {
            return self.run_function(module, main, Env::new(), &[]);
        }
        // No main: run each level-1 function with empty args (hello-style demos).
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
            eprintln!("enter fun {:?}", fun.name);
        }
        for (k, v) in args {
            env.set(k.clone(), v.clone());
        }
        // Bind missing params to None
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
            eprintln!("leave fun {:?} => {ret}", fun.name);
        }
        Ok(ret)
    }

    /// Returns Some if function should return.
    fn exec_stmt(
        &mut self,
        module: &Module,
        fun: &Function,
        env: &mut Env,
        stmt: &Stmt,
    ) -> Result<Option<Value>> {
        match stmt {
            Stmt::Assign { name, value, span } => {
                let v = self.eval_expr(module, fun, env, value)?;
                if self.trace {
                    eprintln!("  assign {name} = {v:?} @{span}");
                }
                env.set(name.clone(), v);
                Ok(None)
            }
            Stmt::Return { value, span } => {
                let v = self.eval_expr(module, fun, env, value)?;
                if self.trace {
                    eprintln!("  return {v:?} @{span}");
                }
                Ok(Some(v))
            }
            Stmt::Call { call, span } => {
                if self.trace {
                    eprintln!("  call-stmt {} @{span}", call.callee);
                }
                let _ = self.eval_call(module, fun, env, call)?;
                Ok(None)
            }
            Stmt::Branch { arms, span } => {
                if self.trace {
                    eprintln!("  branch @{span}");
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
                if self.trace {
                    eprintln!("  while @{span}");
                }
                // Guard against infinite loops in buggy programs.
                let mut guard = 0u32;
                while self.eval_expr(module, fun, env, condition)?.truthy() {
                    guard += 1;
                    if guard > 1_000_000 {
                        bail!("{span}: while loop exceeded iteration limit");
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
                if self.trace {
                    eprintln!("  foreach {item} in {collection} @{span}");
                }
                let coll = env
                    .get(collection)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("{span}: undefined collection `{collection}`"))?;
                let items = match coll {
                    Value::List(xs) => xs,
                    other => bail!("{span}: foreach needs a list, got {other:?}"),
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
                .ok_or_else(|| anyhow::anyhow!("undefined variable `{name}`")),
            Expr::Interp(parts) => {
                let mut s = String::new();
                for part in parts {
                    match part {
                        InterpPart::Lit(t) => s.push_str(t),
                        InterpPart::Var(n) => {
                            let v = env
                                .get(n)
                                .ok_or_else(|| anyhow::anyhow!("undefined variable `{n}`"))?;
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
                        _ => bail!("unary `-` needs int"),
                    },
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(module, fun, env, left)?;
                // Short-circuit and/or
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
                eval_binary(*op, &l, &r)
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
        let mut arg_vals = Vec::new();
        for (k, e) in &call.args {
            arg_vals.push((k.clone(), self.eval_expr(module, fun, env, e)?));
        }

        match call.callee.as_str() {
            "print" => {
                let text = arg_vals
                    .iter()
                    .find(|(k, _)| k == "text")
                    .map(|(_, v)| v.as_display())
                    .ok_or_else(|| anyhow::anyhow!("print requires text=..."))?;
                println!("{text}");
                return Ok(Value::None);
            }
            "input" => {
                let prompt = arg_vals
                    .iter()
                    .find(|(k, _)| k == "prompt")
                    .map(|(_, v)| v.as_display())
                    .unwrap_or_default();
                if !prompt.is_empty() {
                    print!("{prompt}");
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                }
                let mut line = String::new();
                std::io::stdin().read_line(&mut line)?;
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                return Ok(Value::Text(line));
            }
            _ => {}
        }

        let target = lookup_function(module, fun, &call.callee).ok_or_else(|| {
            anyhow::anyhow!("unknown function `{}`", call.callee)
        })?;

        // Bind by param names: prefer named args matching params; also allow arg keys as env names.
        let mut call_env = Env::new();
        for p in &target.params {
            if let Some((_, v)) = arg_vals.iter().find(|(k, _)| k == p) {
                call_env.set(p.clone(), v.clone());
            }
        }
        for (k, v) in &arg_vals {
            if call_env.get(k).is_none() {
                call_env.set(k.clone(), v.clone());
            }
        }

        self.run_function(module, target, call_env, &[])
    }
}

fn find_top<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module.functions.iter().find(|f| f.name == name)
}

fn lookup_function<'a>(
    module: &'a Module,
    current: &'a Function,
    name: &str,
) -> Option<&'a Function> {
    // Nested children of current (hoisted).
    if let Some(f) = find_in_tree(current, name) {
        return Some(f);
    }
    // Top-level / imported.
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
            _ => bail!("`+` needs ints or text, got {l:?} and {r:?}"),
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
        BinaryOp::Eq => Ok(Value::Bool(values_eq(l, r))),
        BinaryOp::Ne => Ok(Value::Bool(!values_eq(l, r))),
        BinaryOp::Lt => cmp_ord(l, r, |o| o.is_lt()),
        BinaryOp::Le => cmp_ord(l, r, |o| o.is_le()),
        BinaryOp::Gt => cmp_ord(l, r, |o| o.is_gt()),
        BinaryOp::Ge => cmp_ord(l, r, |o| o.is_ge()),
        BinaryOp::And | BinaryOp::Or => unreachable!("handled above"),
    }
}

fn values_eq(l: &Value, r: &Value) -> bool {
    l == r
}

fn cmp_ord(l: &Value, r: &Value, f: impl Fn(std::cmp::Ordering) -> bool) -> Result<Value> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        (Value::Text(a), Value::Text(b)) => Ok(Value::Bool(f(a.cmp(b)))),
        _ => bail!("comparison needs two ints or two texts"),
    }
}

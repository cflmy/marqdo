//! Stack VM for Marqdo bytecode.

use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::builtin::{
    builtin_at, builtin_int, builtin_join, builtin_len, builtin_split, builtin_str, builtin_trim,
    builtin_type,
};
use crate::host::{call_host, HostContext, HostFn};
use crate::bytecode::{Op, Program};
use crate::debug::emit_trace;
use crate::diagnostics::{bail_at, Span};
use crate::input_feed::InputFeed;
use crate::value::Value;
use std::collections::HashMap;

pub struct Vm {
    path: Option<PathBuf>,
    capture: bool,
    pub captured_stdout: String,
    input: InputFeed,
    trace: bool,
    host: HostContext,
}

struct Frame {
    fn_idx: usize,
    ip: usize,
    slots: Vec<Value>,
}

impl Vm {
    pub fn new(path: Option<&Path>) -> Self {
        Self {
            path: path.map(|p| p.to_path_buf()),
            capture: false,
            captured_stdout: String::new(),
            input: InputFeed::new(false, Vec::new()),
            trace: false,
            host: HostContext::for_run(path, Default::default(), Vec::new()),
        }
    }

    pub fn with_capture(path: Option<&Path>) -> Self {
        Self {
            path: path.map(|p| p.to_path_buf()),
            capture: true,
            captured_stdout: String::new(),
            input: InputFeed::new(true, Vec::new()),
            trace: false,
            host: HostContext::for_capture(path, Default::default()),
        }
    }

    pub fn with_stdin(mut self, lines: Vec<String>) -> Self {
        self.input = InputFeed::new(self.capture, lines);
        self
    }

    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = trace;
        self
    }

    pub fn with_host(mut self, host: HostContext) -> Self {
        self.host = host;
        self
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
            let _ = std::io::stdout().flush();
        }
    }

    fn err_at(&self, span: Span, message: impl Into<String>) -> anyhow::Error {
        bail_at(self.path.as_deref(), span, message)
    }

    pub fn run(&mut self, program: &Program) -> Result<Value> {
        let mut stack: Vec<Value> = Vec::new();
        let entry = program.entry;
        let arity = program.functions[entry].params.len();
        if arity != 0 {
            let span = program.functions[entry]
                .spans
                .first()
                .copied()
                .unwrap_or(Span::new(1, 1));
            return Err(self.err_at(span, "bytecode: entry function needs arguments"));
        }
        let mut frames = vec![Frame {
            fn_idx: entry,
            ip: 0,
            slots: vec![Value::None; program.functions[entry].locals.len().max(1)],
        }];

        loop {
            let frame = frames.last_mut().unwrap();
            let fun = &program.functions[frame.fn_idx];
            if frame.ip >= fun.code.len() {
                let span = fun.spans.last().copied().unwrap_or(Span::new(1, 1));
                return Err(self.err_at(span, format!("bytecode: ip out of range in {}", fun.name)));
            }
            let ip = frame.ip;
            let op = fun.code[ip];
            let span = fun.spans.get(ip).copied().unwrap_or(Span::new(1, 1));
            frame.ip += 1;

            if self.trace {
                match op {
                    Op::Print | Op::Input | Op::Call(_, _) | Op::Return => {
                        let kind = match op {
                            Op::Print => "print",
                            Op::Input => "input",
                            Op::Call(_, _) => "call",
                            Op::Return => "return",
                            _ => unreachable!(),
                        };
                        let ip_s = ip.to_string();
                        emit_trace(
                            self.path.as_deref(),
                            Some(span),
                            "op",
                            &[
                                ("kind", kind),
                                ("fn", fun.name.as_str()),
                                ("ip", ip_s.as_str()),
                            ],
                        );
                    }
                    _ => {}
                }
            }

            match op {
                Op::Constant(i) => stack.push(fun.constants[i as usize].clone()),
                Op::True => stack.push(Value::Bool(true)),
                Op::False => stack.push(Value::Bool(false)),
                Op::None_ => stack.push(Value::None),
                Op::Pop => {
                    stack.pop();
                }
                Op::GetLocal(i) => {
                    let v = frames.last().unwrap().slots[i as usize].clone();
                    stack.push(v);
                }
                Op::SetLocal(i) => {
                    let v = stack.last().cloned().unwrap_or(Value::None);
                    let slot = i as usize;
                    let fr = frames.last_mut().unwrap();
                    if fr.slots.len() <= slot {
                        fr.slots.resize(slot + 1, Value::None);
                    }
                    fr.slots[slot] = v;
                }
                Op::Add => {
                    let b = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let a = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(add(a, b).map_err(|m| self.err_at(span, m))?);
                }
                Op::Sub => bin_int(&mut stack, |a, b| a - b).map_err(|m| self.err_at(span, m))?,
                Op::Mul => bin_int(&mut stack, |a, b| a * b).map_err(|m| self.err_at(span, m))?,
                Op::Div => {
                    let b = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let a = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    match (a, b) {
                        (Value::Int(_), Value::Int(0)) => {
                            return Err(self.err_at(span, "division by zero"));
                        }
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(x / y)),
                        _ => return Err(self.err_at(span, "`/` needs ints")),
                    }
                }
                Op::Negate => match pop(&mut stack).map_err(|m| self.err_at(span, m))? {
                    Value::Int(n) => stack.push(Value::Int(-n)),
                    _ => return Err(self.err_at(span, "unary `-` needs int")),
                },
                Op::Not => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(Value::Bool(!v.truthy()));
                }
                Op::Equal => {
                    let b = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let a = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(Value::Bool(a == b));
                }
                Op::NotEqual => {
                    let b = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let a = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(Value::Bool(a != b));
                }
                Op::Greater => cmp(&mut stack, |o| o.is_gt()).map_err(|m| self.err_at(span, m))?,
                Op::GreaterEqual => {
                    cmp(&mut stack, |o| o.is_ge()).map_err(|m| self.err_at(span, m))?
                }
                Op::Less => cmp(&mut stack, |o| o.is_lt()).map_err(|m| self.err_at(span, m))?,
                Op::Lessequal => cmp(&mut stack, |o| o.is_le()).map_err(|m| self.err_at(span, m))?,
                Op::Jump(t) => {
                    frames.last_mut().unwrap().ip = t as usize;
                }
                Op::JumpIfFalse(t) => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    if !v.truthy() {
                        frames.last_mut().unwrap().ip = t as usize;
                    }
                }
                Op::Print => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    self.emit_line(&v.as_display());
                }
                Op::Input => {
                    let prompt = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let text = prompt.as_display();
                    self.emit_prompt(&text);
                    let line = self.input.read_line().map_err(|e| {
                        let msg = e.to_string();
                        if msg.contains("input needs a line") {
                            self.err_at(span, msg)
                        } else {
                            e
                        }
                    })?;
                    stack.push(Value::Text(line));
                }
                Op::BuildList(n) => {
                    let n = n as usize;
                    let mut items = Vec::with_capacity(n);
                    for _ in 0..n {
                        items.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?);
                    }
                    items.reverse();
                    stack.push(Value::List(items));
                }
                Op::Len => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let n = builtin_len(&v).map_err(|m| self.err_at(span, m))?;
                    stack.push(Value::Int(n));
                }
                Op::Str => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_str(&v));
                }
                Op::Int => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let n = builtin_int(&v).map_err(|m| self.err_at(span, m))?;
                    stack.push(Value::Int(n));
                }
                Op::TypeOf => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_type(&v));
                }
                Op::Trim => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_trim(&v).map_err(|m| self.err_at(span, m))?);
                }
                Op::Split => {
                    let sep = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let value = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_split(&value, &sep).map_err(|m| self.err_at(span, m))?);
                }
                Op::Join => {
                    let sep = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let value = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_join(&value, &sep).map_err(|m| self.err_at(span, m))?);
                }
                Op::GetIndex => {
                    let idx = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let list = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_at(&list, &idx).map_err(|m| self.err_at(span, m))?);
                }
                Op::HostCall(id, argc) => {
                    let hf = HostFn::from_u16(id)
                        .ok_or_else(|| self.err_at(span, format!("unknown host id {id}")))?;
                    let argc = argc as usize;
                    let params = hf.all_params();
                    if params.len() != argc {
                        return Err(self.err_at(span, "host call argc mismatch"));
                    }
                    let mut vals = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        vals.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?);
                    }
                    vals.reverse();
                    let mut bound = HashMap::new();
                    for (p, v) in params.iter().zip(vals.into_iter()) {
                        bound.insert((*p).to_string(), v);
                    }
                    for req in hf.required_params() {
                        if !bound.contains_key(*req) {
                            return Err(self.err_at(
                                span,
                                format!("{} requires `{req}`", hf.name()),
                            ));
                        }
                    }
                    let result =
                        call_host(&self.host, hf, &bound).map_err(|m| self.err_at(span, m))?;
                    stack.push(result);
                }
                Op::Call(fid, argc) => {
                    let fid = fid as usize;
                    let argc = argc as usize;
                    let callee = &program.functions[fid];
                    if callee.params.len() != argc {
                        return Err(self.err_at(span, "call argument count mismatch"));
                    }
                    let mut slots = vec![Value::None; callee.locals.len().max(argc).max(1)];
                    for i in (0..argc).rev() {
                        slots[i] = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    }
                    frames.push(Frame {
                        fn_idx: fid,
                        ip: 0,
                        slots,
                    });
                }
                Op::Return => {
                    let ret = pop(&mut stack).unwrap_or(Value::None);
                    frames.pop();
                    if frames.is_empty() {
                        return Ok(ret);
                    }
                    stack.push(ret);
                }
            }
        }
    }
}

fn pop(stack: &mut Vec<Value>) -> Result<Value, String> {
    stack
        .pop()
        .ok_or_else(|| "bytecode: stack underflow".to_string())
}

fn add(a: Value, b: Value) -> Result<Value, String> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (Value::Text(x), Value::Text(y)) => Ok(Value::Text(format!("{x}{y}"))),
        (Value::Text(x), y) => Ok(Value::Text(format!("{x}{}", y.as_display()))),
        (x, Value::Text(y)) => Ok(Value::Text(format!("{}{y}", x.as_display()))),
        (Value::Int(x), y) => Ok(Value::Text(format!("{}{}", x, y.as_display()))),
        (x, Value::Int(y)) => Ok(Value::Text(format!("{}{y}", x.as_display()))),
        _ => Err("`+` needs ints or text".into()),
    }
}

fn bin_int(stack: &mut Vec<Value>, f: impl Fn(i64, i64) -> i64) -> Result<(), String> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => {
            stack.push(Value::Int(f(x, y)));
            Ok(())
        }
        _ => Err("binary op needs ints".into()),
    }
}

fn cmp(stack: &mut Vec<Value>, pred: impl Fn(std::cmp::Ordering) -> bool) -> Result<(), String> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    let ord = match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        _ => return Err("comparison needs two ints or two texts".into()),
    };
    stack.push(Value::Bool(pred(ord)));
    Ok(())
}

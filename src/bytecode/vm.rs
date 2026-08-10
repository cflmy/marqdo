//! Stack VM for Marqdo bytecode.

use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::builtin::{
    builtin_at, builtin_footnote_get, builtin_int, builtin_iter_items, builtin_join,
    builtin_len, builtin_split, builtin_str, builtin_trim,
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

    pub fn take_plots(&mut self) -> Vec<crate::host::PlotArtifact> {
        std::mem::take(&mut self.host.plots)
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
        self.host
            .push_call_frame(&program.functions[entry].name.clone());

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
            self.host.current_line = span.line;

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
                Op::BuildMap(n) => {
                    let n = n as usize;
                    let mut pairs = Vec::with_capacity(n);
                    for _ in 0..n {
                        let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                        let k = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                        let ks = match k {
                            Value::Text(s) => s,
                            _ => {
                                return Err(self.err_at(span, "map key must be text"));
                            }
                        };
                        pairs.push((ks, v));
                    }
                    pairs.reverse();
                    stack.push(Value::Map(pairs));
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
                Op::IterItems => {
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(builtin_iter_items(&v).map_err(|m| self.err_at(span, m))?);
                }
                Op::FootnoteGet => {
                    let label = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let base = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let label = match label {
                        Value::Text(s) => s,
                        _ => {
                            return Err(self.err_at(span, "footnote label must be text"));
                        }
                    };
                    stack.push(
                        builtin_footnote_get(&base, &label).map_err(|m| self.err_at(span, m))?,
                    );
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
                        call_host(&mut self.host, hf, &bound).map_err(|m| self.err_at(span, m))?;
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
                    let fname = program.functions[fid].name.clone();
                    frames.push(Frame {
                        fn_idx: fid,
                        ip: 0,
                        slots,
                    });
                    self.host.push_call_frame(&fname);
                    self.host.push_call_site_line(span.line);
                }
                Op::TagInstance(idx) => {
                    let type_name = match fun.constants.get(idx as usize) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => {
                            return Err(self.err_at(span, "TagInstance needs type name constant"));
                        }
                    };
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(tag_instance_value(&type_name, v));
                }
                Op::MethodCall(name_idx, argc) => {
                    let argc = argc as usize;
                    let method_name = match fun.constants.get(name_idx as usize) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => {
                            return Err(self.err_at(span, "MethodCall needs method name constant"));
                        }
                    };
                    let recv = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let mut vals = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        vals.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?);
                    }
                    vals.reverse();
                    let type_name = instance_type_name_bc(&recv).map_err(|m| self.err_at(span, m))?;
                    let start_oi = program
                        .objects
                        .iter()
                        .position(|(n, _)| n == &type_name)
                        .ok_or_else(|| {
                            self.err_at(span, format!("unknown object type `{type_name}`"))
                        })?;
                    let method_fid = resolve_method_fid(program, start_oi, &method_name).ok_or_else(
                        || {
                            let mut chain = Vec::new();
                            let mut oi = Some(start_oi);
                            let mut seen = std::collections::HashSet::new();
                            while let Some(i) = oi {
                                if !seen.insert(i) {
                                    break;
                                }
                                chain.push(program.objects[i].0.as_str());
                                oi = program.object_bases.get(i).copied().flatten().and_then(
                                    |base_flat| {
                                        program.objects.iter().position(|(_, f)| *f == base_flat)
                                    },
                                );
                            }
                            self.err_at(
                                span,
                                format!(
                                    "unknown method `{type_name}.{method_name}` (searched {})",
                                    chain.join(" → ")
                                ),
                            )
                        },
                    )?;
                    let callee = &program.functions[method_fid];
                    if callee.params.len() != argc {
                        return Err(self.err_at(span, "method argument count mismatch"));
                    }
                    let mut slots = vec![Value::None; callee.locals.len().max(1)];
                    for (i, v) in vals.into_iter().enumerate() {
                        slots[i] = v;
                    }
                    for (i, name) in callee.locals.iter().enumerate() {
                        if name == "自" || name == "self" {
                            slots[i] = recv.clone();
                        }
                    }
                    let fname = callee.name.clone();
                    frames.push(Frame {
                        fn_idx: method_fid,
                        ip: 0,
                        slots,
                    });
                    self.host.push_call_frame(&fname);
                    self.host.push_call_site_line(span.line);
                }
                Op::PluginCall(name_idx, argc) => {
                    let argc = argc as usize;
                    let raw_name = match fun.constants.get(name_idx as usize) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => {
                            return Err(self.err_at(span, "PluginCall needs name constant"));
                        }
                    };
                    let mut bound = HashMap::new();
                    if let Some(name) = raw_name.strip_prefix('@') {
                        // named pairs: (name, value) × N, argc = 2N
                        if argc % 2 != 0 {
                            return Err(self.err_at(span, "PluginCall named argc must be even"));
                        }
                        let pairs = argc / 2;
                        for _ in 0..pairs {
                            let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                            let k = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                            let ks = match k {
                                Value::Text(s) => s,
                                _ => {
                                    return Err(self.err_at(span, "plugin arg name must be text"));
                                }
                            };
                            bound.insert(ks, v);
                        }
                        let result = crate::host::plugin::call_registered(&self.host, name, &bound)
                            .map_err(|m| self.err_at(span, m))?;
                        stack.push(result);
                    } else {
                        let mut vals = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            vals.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?);
                        }
                        vals.reverse();
                        let reg = self
                            .host
                            .plugins
                            .get(&raw_name)
                            .cloned()
                            .ok_or_else(|| {
                                self.err_at(span, format!("unknown function `{raw_name}`"))
                            })?;
                        if reg.params.len() != vals.len() {
                            return Err(self.err_at(
                                span,
                                format!(
                                    "plugin `{raw_name}` expects {} args, got {}",
                                    reg.params.len(),
                                    vals.len()
                                ),
                            ));
                        }
                        for (p, v) in reg.params.iter().zip(vals.into_iter()) {
                            bound.insert(p.clone(), v);
                        }
                        let result =
                            crate::host::plugin::call_registered(&self.host, &raw_name, &bound)
                                .map_err(|m| self.err_at(span, m))?;
                        stack.push(result);
                    }
                }
                Op::CallFn => {
                    let name_v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let name = match name_v {
                        Value::Text(s) => s,
                        _ => return Err(self.err_at(span, "call_fn requires text name")),
                    };
                    let current = frames.last().unwrap().fn_idx;
                    let fid = resolve_fn_dynamic(program, current, &name)
                        .map_err(|m| self.err_at(span, m))?;
                    let callee = &program.functions[fid];
                    if !callee.params.is_empty() {
                        return Err(self.err_at(
                            span,
                            format!("call_fn: `{name}` must have no parameters"),
                        ));
                    }
                    let slots = vec![Value::None; callee.locals.len().max(1)];
                    let fname = callee.name.clone();
                    frames.push(Frame {
                        fn_idx: fid,
                        ip: 0,
                        slots,
                    });
                    self.host.push_call_frame(&fname);
                    self.host.push_call_site_line(span.line);
                }
                Op::Return => {
                    let ret = pop(&mut stack).unwrap_or(Value::None);
                    frames.pop();
                    self.host.pop_call_frame();
                    self.host.pop_call_site_line();
                    if frames.is_empty() {
                        return Ok(ret);
                    }
                    stack.push(ret);
                }
            }
        }
    }
}

fn resolve_method_fid(program: &Program, start_oi: usize, method_name: &str) -> Option<usize> {
    let mut oi = Some(start_oi);
    let mut seen = std::collections::HashSet::new();
    while let Some(i) = oi {
        if !seen.insert(i) {
            break;
        }
        let obj_flat = program.objects[i].1;
        if let Some((_, _, fid)) = program
            .methods
            .iter()
            .find(|(o, m, _)| *o == obj_flat && m == method_name)
        {
            return Some(*fid);
        }
        oi = program.object_bases.get(i).copied().flatten().and_then(|base_flat| {
            program.objects.iter().position(|(_, f)| *f == base_flat)
        });
    }
    None
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
        (Value::Num(x), Value::Num(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(x), Value::Num(y)) => {
            (*x as f64).partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::Num(x), Value::Int(y)) => {
            x.partial_cmp(&(*y as f64)).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        _ => return Err("comparison needs two ints/nums or two texts".into()),
    };
    stack.push(Value::Bool(pred(ord)));
    Ok(())
}

fn instance_type_name_bc(v: &Value) -> Result<String, String> {
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

fn tag_instance_value(type_name: &str, value: Value) -> Value {
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

fn resolve_fn_dynamic(program: &Program, current: usize, name: &str) -> Result<usize, String> {
    for &cid in &program.children[current] {
        if program.functions[cid].name == name {
            return Ok(cid);
        }
    }
    let mut parent = program.parents[current];
    while let Some(pid) = parent {
        for &cid in &program.children[pid] {
            if program.functions[cid].name == name {
                return Ok(cid);
            }
        }
        parent = program.parents[pid];
    }
    for (i, f) in program.functions.iter().enumerate() {
        if program.parents[i].is_none() && f.name == name {
            return Ok(i);
        }
    }
    Err(format!("call_fn: unknown function `{name}`"))
}

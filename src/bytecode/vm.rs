//! Stack VM for Marqdo bytecode.

use anyhow::{bail, Result};

use crate::bytecode::{Op, Program};
use crate::value::Value;

pub struct Vm {
    capture: bool,
    pub captured_stdout: String,
}

struct Frame {
    fn_idx: usize,
    ip: usize,
    slots: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            capture: false,
            captured_stdout: String::new(),
        }
    }

    pub fn with_capture() -> Self {
        Self {
            capture: true,
            captured_stdout: String::new(),
        }
    }

    fn emit_line(&mut self, text: &str) {
        if self.capture {
            self.captured_stdout.push_str(text);
            self.captured_stdout.push('\n');
        } else {
            println!("{text}");
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<Value> {
        let mut stack: Vec<Value> = Vec::new();
        let entry = program.entry;
        let arity = program.functions[entry].params.len();
        if arity != 0 {
            bail!("bytecode: entry function needs arguments");
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
                bail!("bytecode: ip out of range in {}", fun.name);
            }
            let op = fun.code[frame.ip];
            frame.ip += 1;

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
                    let b = pop(&mut stack)?;
                    let a = pop(&mut stack)?;
                    stack.push(add(a, b)?);
                }
                Op::Sub => bin_int(&mut stack, |a, b| a - b)?,
                Op::Mul => bin_int(&mut stack, |a, b| a * b)?,
                Op::Div => {
                    let b = pop(&mut stack)?;
                    let a = pop(&mut stack)?;
                    match (a, b) {
                        (Value::Int(_), Value::Int(0)) => bail!("division by zero"),
                        (Value::Int(x), Value::Int(y)) => stack.push(Value::Int(x / y)),
                        _ => bail!("`/` needs ints"),
                    }
                }
                Op::Negate => match pop(&mut stack)? {
                    Value::Int(n) => stack.push(Value::Int(-n)),
                    _ => bail!("unary `-` needs int"),
                },
                Op::Not => {
                    let v = pop(&mut stack)?;
                    stack.push(Value::Bool(!v.truthy()));
                }
                Op::Equal => {
                    let b = pop(&mut stack)?;
                    let a = pop(&mut stack)?;
                    stack.push(Value::Bool(a == b));
                }
                Op::NotEqual => {
                    let b = pop(&mut stack)?;
                    let a = pop(&mut stack)?;
                    stack.push(Value::Bool(a != b));
                }
                Op::Greater => cmp(&mut stack, |o| o.is_gt())?,
                Op::GreaterEqual => cmp(&mut stack, |o| o.is_ge())?,
                Op::Less => cmp(&mut stack, |o| o.is_lt())?,
                Op::Lessequal => cmp(&mut stack, |o| o.is_le())?,
                Op::Jump(t) => {
                    frames.last_mut().unwrap().ip = t as usize;
                }
                Op::JumpIfFalse(t) => {
                    let v = pop(&mut stack)?;
                    if !v.truthy() {
                        frames.last_mut().unwrap().ip = t as usize;
                    }
                }
                Op::Print => {
                    let v = pop(&mut stack)?;
                    self.emit_line(&v.as_display());
                }
                Op::BuildList(n) => {
                    let n = n as usize;
                    let mut items = Vec::with_capacity(n);
                    for _ in 0..n {
                        items.push(pop(&mut stack)?);
                    }
                    items.reverse();
                    stack.push(Value::List(items));
                }
                Op::Len => match pop(&mut stack)? {
                    Value::List(xs) => stack.push(Value::Int(xs.len() as i64)),
                    _ => bail!("len needs list"),
                },
                Op::GetIndex => {
                    let idx = pop(&mut stack)?;
                    let list = pop(&mut stack)?;
                    match (list, idx) {
                        (Value::List(xs), Value::Int(i)) if i >= 0 && (i as usize) < xs.len() => {
                            stack.push(xs[i as usize].clone());
                        }
                        (Value::List(_), Value::Int(_)) => stack.push(Value::None),
                        _ => bail!("GetIndex needs list and int"),
                    }
                }
                Op::Call(fid, argc) => {
                    let fid = fid as usize;
                    let argc = argc as usize;
                    let callee = &program.functions[fid];
                    if callee.params.len() != argc {
                        bail!("bytecode: call argc mismatch");
                    }
                    let mut slots = vec![Value::None; callee.locals.len().max(argc).max(1)];
                    for i in (0..argc).rev() {
                        slots[i] = pop(&mut stack)?;
                    }
                    // args were pushed in param order; we popped reverse so slots[0] is last param — fix:
                    // stack had p0, p1, p2 (top). pop → p2, p1, p0. So assign slots[argc-1-i] or pop into reverse.
                    // Actually we did: for i in (0..argc).rev() { slots[i] = pop() } 
                    // first pop → slots[argc-1] = top = last param. Good if push order was p0..plast.
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

fn pop(stack: &mut Vec<Value>) -> Result<Value> {
    stack
        .pop()
        .ok_or_else(|| anyhow::anyhow!("bytecode: stack underflow"))
}

fn add(a: Value, b: Value) -> Result<Value> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (Value::Text(x), Value::Text(y)) => Ok(Value::Text(format!("{x}{y}"))),
        (Value::Text(x), y) => Ok(Value::Text(format!("{x}{}", y.as_display()))),
        (x, Value::Text(y)) => Ok(Value::Text(format!("{}{y}", x.as_display()))),
        // Int + display for interp GetLocal Int + Lit text
        (Value::Int(x), y) => Ok(Value::Text(format!("{}{}", x, y.as_display()))),
        (x, Value::Int(y)) => Ok(Value::Text(format!("{}{y}", x.as_display()))),
        _ => bail!("`+` needs ints or text"),
    }
}

fn bin_int(stack: &mut Vec<Value>, f: impl Fn(i64, i64) -> i64) -> Result<()> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => {
            stack.push(Value::Int(f(x, y)));
            Ok(())
        }
        _ => bail!("binary op needs ints"),
    }
}

fn cmp(stack: &mut Vec<Value>, pred: impl Fn(std::cmp::Ordering) -> bool) -> Result<()> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    let ord = match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        _ => bail!("comparison needs two ints or two texts"),
    };
    stack.push(Value::Bool(pred(ord)));
    Ok(())
}

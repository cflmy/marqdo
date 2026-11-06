//! Stack VM for Marqdo bytecode.

use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::builtin::{
    builtin_at, builtin_footnote_get, builtin_int, builtin_iter_items, builtin_join, builtin_len,
    builtin_split, builtin_str, builtin_trim, builtin_type,
};
use crate::bytecode::{FnChunk, Op, Program};
use crate::debug::emit_trace;
use crate::diagnostics::{Diagnostic, DiagnosticFrame, SourceIdentity, Span};
use crate::host::{call_host, HostContext, HostFn};
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
    current_source: SourceIdentity,
    current_scope: Option<u32>,
    diagnostic_frames: Vec<DiagnosticFrame>,
}

struct Frame {
    fn_idx: usize,
    ip: usize,
    slots: Vec<Option<Value>>,
    provided: Vec<bool>,
    loop_counts: HashMap<usize,u32>,
    receiver: Option<Value>,
    defaults_active: bool,
    /// When set, retag matrix-like return to this `_type` (method chaining).
    retag_matrix: Option<(String, Option<Vec<String>>)>,
    has_call_site: bool,
    retag_instance: Option<usize>,
}

impl Frame {
    fn read(&self, fun: &FnChunk, index: usize) -> Option<Value> {
        let name=&fun.locals[index];
        let overlay=self.defaults_active && self.receiver.is_some() && (name=="self" || name=="自") &&
            (index>=fun.params.len() || self.provided.get(index).copied().unwrap_or(false) || self.slots[index].is_none());
        if overlay {self.receiver.clone()} else {self.slots[index].clone()}
    }
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
            current_source: SourceIdentity {
                path: path.map(Path::to_path_buf),
                sha256: None,
            },
            current_scope: None,
            diagnostic_frames: Vec::new(),
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
            current_source: SourceIdentity {
                path: path.map(Path::to_path_buf),
                sha256: None,
            },
            current_scope: None,
            diagnostic_frames: Vec::new(),
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

    #[cfg(all(test, feature = "exec-host"))]
    pub(crate) fn foreign_source_evidence(
        &mut self,
    ) -> (
        Vec<Value>,
        Vec<Value>,
        Vec<crate::host::foreign_session::ProtocolStep>,
    ) {
        crate::source_refinement_tests::boundary_evidence(&mut self.host)
    }

    #[cfg(test)]
    pub(crate) fn diagnostic_scope_state(
        &self,
    ) -> (SourceIdentity, usize, Vec<String>, Vec<u32>, Option<u32>) {
        (
            self.current_source.clone(),
            self.diagnostic_frames.len(),
            self.host.call_stack.clone(),
            self.host.call_site_lines.clone(),
            self.current_scope,
        )
    }

    #[cfg(all(test, feature = "exec-host"))]
    pub(crate) fn causal_replay_evidence(&mut self) -> serde_json::Value {
        crate::causal_replay_tests::evidence(&mut self.host)
    }

    #[cfg(all(test, feature = "exec-host"))]
    pub(crate) fn application_evidence(
        &self,
    ) -> (
        Vec<Value>,
        Vec<crate::host::foreign_session::ProtocolStep>,
        usize,
    ) {
        (
            self.host.oracle_observations.clone(),
            self.host.foreign_protocol_steps.clone(),
            self.host.foreign_sessions.len(),
        )
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
        Diagnostic::new(self.current_source.path.as_deref(), span, message)
            .with_source(&self.current_source)
            .with_frames(&self.diagnostic_frames)
            .into()
    }

    fn push_source_frame(&mut self, callee: &FnChunk, call_span: Option<Span>) {
        let mut source = callee.source.clone();
        if source.path.is_none() {
            if self.diagnostic_frames.is_empty() {
                source.path = self.path.clone();
            } else if self.current_scope == Some(callee.module_scope) {
                source.path = self.current_source.path.clone();
            }
        }
        self.diagnostic_frames.push(DiagnosticFrame {
            function: callee.name.clone(),
            declaration: source.at(callee.declaration_span),
            call_site: call_span.map(|span| self.current_source.at(span)),
        });
    }

    pub fn run(&mut self, program: &Program) -> Result<Value> {
        // Entry execution, like Interpreter::run_function, seeds unbound locals
        // with None. Inferred prose parameters do not require a program caller.
        let mut seen=std::collections::HashSet::new();
        let metadata=program.entry_metadata.iter().filter(|(name,_)|seen.insert(name.clone())).cloned().collect::<Vec<_>>();
        self.execute(program, program.entry, &metadata)
    }

    /// Run `fn_idx` with named arguments already bound to local slots.
    /// Does not fall back to [`Program::entry`].
    pub fn call_function(
        &mut self,
        program: &Program,
        fn_idx: usize,
        args: &[(String, Value)],
    ) -> Result<Value> {
        if fn_idx >= program.functions.len() {
            return Err(self.err_at(
                Span::new(1, 1),
                format!("bytecode: function index {fn_idx} is out of range"),
            ));
        }
        self.execute(program, fn_idx, args)
    }

    fn execute(
        &mut self,
        program: &Program,
        fn_idx: usize,
        args: &[(String, Value)],
    ) -> Result<Value> {
        program.verify().map_err(|error|self.err_at(Span::new(1,1),error))?;
        let previous_source = self.current_source.clone();
        let previous_scope = self.current_scope;
        let previous_depth = self.diagnostic_frames.len();
        let previous_calls = self.host.call_stack.clone();
        let previous_sites = self.host.call_site_lines.clone();
        let previous_line = self.host.current_line;
        // Scope numbers belong to one compiled program; never carry them into
        // a new external invocation, even when it preserves an outer prefix.
        self.current_scope = None;
        let result = self.execute_inner(program, fn_idx, args);
        self.diagnostic_frames.truncate(previous_depth);
        self.current_source = previous_source;
        self.current_scope = previous_scope;
        self.host.call_stack = previous_calls;
        self.host.call_site_lines = previous_sites;
        self.host.current_line = previous_line;
        result
    }

    fn execute_inner(
        &mut self,
        program: &Program,
        fn_idx: usize,
        args: &[(String, Value)],
    ) -> Result<Value> {
        let mut stack: Vec<Value> = Vec::new();
        let mut slots = vec![None; program.functions[fn_idx].locals.len().max(1)];
        for (name, value) in args {
            if let Some(i) = program.functions[fn_idx]
                .locals
                .iter()
                .position(|local| local == name)
            {
                if i < slots.len() {
                    slots[i] = Some(value.clone());
                }
            }
        }
        let mut frames = vec![Frame {
            fn_idx,
            loop_counts: HashMap::new(),
            receiver: None, defaults_active: true,
            ip: 0,
            provided: initial_presence(&slots,program.functions[fn_idx].params.len()),
            slots,
            retag_matrix: None,
            has_call_site: false,
            retag_instance: None,
        }];
        self.push_source_frame(&program.functions[fn_idx], None);
        self.host
            .push_call_frame(&program.functions[fn_idx].name.clone());

        let mut source_owner = None;
        loop {
            let frame = frames.last_mut().unwrap();
            let fun = &program.functions[frame.fn_idx];
            if source_owner != Some(frame.fn_idx) {
                self.current_source = self
                    .diagnostic_frames
                    .last()
                    .expect("active source frame")
                    .declaration
                    .source
                    .clone();
                self.current_scope = Some(fun.module_scope);
                source_owner = Some(frame.fn_idx);
            }
            if frame.ip >= fun.code.len() {
                let span = fun.spans.last().copied().unwrap_or(Span::new(1, 1));
                return Err(self.err_at(span, format!("bytecode: ip out of range in {}", fun.name)));
            }
            let ip = frame.ip;
            let mut op = fun.code[ip];
            let span = fun.spans.get(ip).copied().unwrap_or(Span::new(1, 1));
            frame.ip += 1;
            self.host.current_line = span.line;

            let mut print_returns_none=false;
            let mut method_name_override=None;
            let mut constructor_tag=None;
            if let Op::PathCallArgs(owner,layout)=op {
                let call=&fun.calls[owner];
                let Some(super::CallDispatch::Path{receiver,check_receiver,fallback})=&call.dispatch else{unreachable!("verified path dispatch")};
                let value=if *check_receiver {fun.locals.iter().position(|n|n==receiver).and_then(|slot|frame.read(fun,slot))}else{None};
                if let Some(value)=value {
                    stack.push(value);method_name_override=Some(call.source.callee.clone());op=Op::MethodCallArgs(0,layout);
                } else {match fallback {
                    super::PathFallback::User{target,tag_instance}=>{op=Op::CallArgs(*target,layout);if *tag_instance{constructor_tag=Some(*target);}},
                    super::PathFallback::Error{message}=>return Err(self.err_at(span,message)),
                }}
            }
            if let Op::BuiltinCallArgs(name,layout)=op {
                let name=fun.constants[name].as_display();let names=&fun.call_layouts[layout];
                let values=take_actuals(&mut stack,names.len()).map_err(|m|self.err_at(span,m))?;
                let (params,allow_missing)=crate::arguments::builtin_signature(&name).ok_or_else(||self.err_at(span,"unknown builtin signature"))?;
                let parameters=params.iter().map(|p|p.to_string()).collect::<Vec<_>>();
                let bound=crate::arguments::bind(&parameters,names,&values,allow_missing).map_err(|m|self.err_at(span,m))?;
                for param in params {
                    stack.push(bound.get(*param).cloned().unwrap_or_else(||Value::Text(String::new())));
                }
                op=match name.as_str(){"print"=>{print_returns_none=true;Op::Print},"input"=>Op::Input,"len"=>Op::Len,"str"=>Op::Str,"int"=>Op::Int,"type"=>Op::TypeOf,"trim"=>Op::Trim,"split"=>Op::Split,"join"=>Op::Join,"at"=>Op::GetIndex,"call_fn"=>Op::CallFn,_=>unreachable!()};
            }

            if self.trace {
                match op {
                    Op::Print | Op::Input | Op::Call(_, _) | Op::CallArgs(_, _) | Op::Return => {
                        let kind = match op {
                            Op::Print => "print",
                            Op::Input => "input",
                            Op::Call(_, _) | Op::CallArgs(_, _) => "call",
                            Op::Return => "return",
                            _ => unreachable!(),
                        };
                        let ip_s = ip.to_string();
                        emit_trace(
                            self.current_source.path.as_deref(),
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
                Op::CheckArguments => {
                    if let Some(contract)=&fun.contract {
                        let bound=fun.locals.iter().zip(&frame.slots).filter_map(|(name,value)|
                            value.clone().map(|value|(name.clone(),value))).collect();
                        // Internal call errors identify the authored call site, like Tree.
                        let site=self.diagnostic_frames.last().and_then(|f|f.call_site.as_ref());
                        let call_site=site.cloned().unwrap_or_else(||self.current_source.at(span));
                        if let Some(diag)=crate::contract::call_arg_diag_in(contract,&fun.name,&bound,&call_site,&fun.source) {
                            let depth=self.diagnostic_frames.len()-usize::from(site.is_some());
                            return Err(diag.with_frames(&self.diagnostic_frames[..depth]).into());
                        }
                    }
                }
                Op::CheckReturn => {
                    if let Some(contract)=&fun.contract {
                        let value=stack.last().cloned().unwrap_or(Value::None);
                        if let Some(diag)=crate::contract::return_diag(contract,&fun.name,&value,self.current_source.path.as_deref(),span) {
                            return Err(diag.into());
                        }
                    }
                }
                Op::CheckBinding(slot) => {
                    let name=&fun.locals[slot];
                    if let Some(contract)=fun.var_contracts.get(name) {
                        let value=stack.last().cloned().unwrap_or(Value::None);
                        if let Some(diag)=crate::contract::fields_diag(contract,name,&value,self.current_source.path.as_deref(),span) {
                            return Err(diag.into());
                        }
                    }
                }
                Op::RestoreReceiver => {
                    if let Some(receiver)=&frame.receiver {
                        for (index,name) in fun.locals.iter().enumerate() {
                            if name=="self" || name=="自" {frame.slots[index]=Some(receiver.clone());}
                        }
                    }
                    frame.defaults_active=false;
                }
                Op::CheckFrameArguments => {
                    if let Some(contract)=&fun.contract {
                        let bound=fun.locals.iter().zip(&frame.slots).filter_map(|(name,value)|value.clone().map(|value|(name.clone(),value))).collect();
                        if let Some(diag)=crate::contract::call_arg_diag(contract,&fun.name,&bound,self.current_source.path.as_deref(),fun.declaration_span){return Err(diag.with_source(&self.current_source).with_frames(&self.diagnostic_frames).into());}
                    }
                }
                Op::ResetLoop(id) => {frame.loop_counts.insert(id,0);}
                Op::CheckLoop(id) => {
                    let counter=frame.loop_counts.get_mut(&id).ok_or_else(||self.err_at(span,"uninitialized loop counter"))?;
                    if *counter>=1_000_000 {return Err(self.err_at(span,"while loop exceeded iteration limit"));}
                    *counter+=1;
                }
                Op::FillMissingParameters => {
                    for index in 0..fun.params.len() {
                        if frame.slots[index].is_none() { frame.slots[index]=Some(Value::None); }
                    }
                }
                Op::Constant(i) => stack.push(fun.constants[i as usize].clone()),
                Op::True => stack.push(Value::Bool(true)),
                Op::False => stack.push(Value::Bool(false)),
                Op::None_ => stack.push(Value::None),
                Op::Pop => {
                    stack.pop();
                }
                Op::GetLocal(i) => {
                    let current=frames.last().unwrap();
                    let index=i as usize;let name=&fun.locals[index];
                    let value=current.read(fun,index);
                    let v=value.ok_or_else(||self.err_at(span,format!("undefined variable `{name}`")))?;
                    stack.push(v);
                }
                Op::SetLocal(i) => {
                    let v = stack.last().cloned().unwrap_or(Value::None);
                    let slot = i as usize;
                    let fr = frames.last_mut().unwrap();
                    if fr.slots.len() <= slot {
                        fr.slots.resize(slot + 1, None);
                    }
                    fr.slots[slot] = Some(v);
                }
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Equal | Op::NotEqual |
                Op::Greater | Op::GreaterEqual | Op::Less | Op::Lessequal => {
                    use crate::ast::BinaryOp;
                    let operation=match op {
                        Op::Add=>BinaryOp::Add,Op::Sub=>BinaryOp::Sub,Op::Mul=>BinaryOp::Mul,Op::Div=>BinaryOp::Div,
                        Op::Equal=>BinaryOp::Eq,Op::NotEqual=>BinaryOp::Ne,Op::Greater=>BinaryOp::Gt,
                        Op::GreaterEqual=>BinaryOp::Ge,Op::Less=>BinaryOp::Lt,Op::Lessequal=>BinaryOp::Le,
                        _=>unreachable!(),
                    };
                    let right=pop(&mut stack).map_err(|m|self.err_at(span,m))?;
                    let left=pop(&mut stack).map_err(|m|self.err_at(span,m))?;
                    let value=if let Some(value)=crate::linalg_ops::try_matrix_binary(&mut self.host,operation,&left,&right)
                        .map_err(|m|self.err_at(span,m))? {value} else {
                        crate::builtin::eval_binary(operation,&left,&right).map_err(|m|self.err_at(span,m))?
                    };
                    stack.push(value);
                }
                Op::Negate | Op::Not => {
                    let value=pop(&mut stack).map_err(|m|self.err_at(span,m))?;
                    let operation=if matches!(op,Op::Negate){crate::ast::UnaryOp::Neg}else{crate::ast::UnaryOp::Not};
                    stack.push(crate::builtin::eval_unary(operation,&value).map_err(|m|self.err_at(span,m))?);
                }
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
                    let line = if self.capture {
                        self.emit_prompt(&text);
                        self.input.read_line()
                    } else if crate::input_feed::stdin_is_tty() {
                        self.input
                            .read_line_with_prompt(&text)
                            .map(|(line, _)| line)
                    } else {
                        self.emit_prompt(&text);
                        self.input.read_line()
                    }
                    .map_err(|e| {
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
                Op::FootnoteGet | Op::ContractFootnoteGet(_) => {
                    let label = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let base = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let label = match label {
                        Value::Text(s) => s,
                        _ => {
                            return Err(self.err_at(span, "footnote label must be text"));
                        }
                    };
                    let contract=if let Op::ContractFootnoteGet(slot)=op {
                        let name=&fun.locals[slot];fun.var_contracts.get(name).map(|c|(name,c))
                    } else {None};
                    if let Some((name,c))=contract {
                        if let Some(diag)=crate::contract::index_diag(c,name,&label,&Value::None,self.current_source.path.as_deref(),span) {return Err(diag.into());}
                    }
                    let value=builtin_footnote_get(&base,&label).map_err(|m|self.err_at(span,m))?;
                    if let Some((name,c))=contract {
                        if let Some(diag)=crate::contract::index_diag(c,name,&label,&value,self.current_source.path.as_deref(),span) {return Err(diag.into());}
                    }
                    stack.push(value);
                }
                Op::HostCallArgs(id, layout) => {
                    let hf=HostFn::from_u16(id).ok_or_else(||self.err_at(span,"unknown host id"))?;
                    let names=&fun.call_layouts[layout];let values=take_actuals(&mut stack,names.len()).map_err(|m|self.err_at(span,m))?;
                    let params=hf.all_params().iter().map(|p|p.to_string()).collect::<Vec<_>>();
                    let bound=crate::arguments::bind(&params,names,&values,true).map_err(|m|self.err_at(span,m))?;
                    for req in hf.required_params(){if !bound.contains_key(*req){return Err(self.err_at(span,format!("{} requires `{req}`",hf.name())));}}
                    stack.push(call_host(&mut self.host,hf,&bound).map_err(|m|self.err_at(span,m))?);
                }
                Op::BuiltinCallArgs(_,_)=>unreachable!("builtin dispatch was prepared above"),
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
                            return Err(
                                self.err_at(span, format!("{} requires `{req}`", hf.name()))
                            );
                        }
                    }
                    let result =
                        call_host(&mut self.host, hf, &bound).map_err(|m| self.err_at(span, m))?;
                    stack.push(result);
                }
                Op::RejectDuplicateArgument(index) => {
                    let name=fun.constants[index].as_display();
                    return Err(self.err_at(span,format!("duplicate named argument `{name}`")));
                }
                Op::RejectArgumentExpansion(index) => {
                    return Err(self.err_at(span,fun.constants[index].as_display()));
                }
                Op::RejectCallResolution(index)=>return Err(self.err_at(span,fun.constants[index].as_display())),
                Op::PathCallArgs(_,_)=>unreachable!("path dispatch was selected above"),
                Op::JumpIfProvided(index, target) => {
                    if frame.provided.get(index as usize).copied().unwrap_or(false) {
                        frame.ip = target as usize;
                    }
                }
                Op::Call(_, _) | Op::CallArgs(_, _) => {
                    let (fid, argc) = match op {
                        Op::Call(fid, argc) => (fid, argc as usize),
                        Op::CallArgs(fid, layout) => (fid, layout),
                        _ => unreachable!(),
                    };
                    let fid = fid as usize;
                    let callee = &program.functions[fid];
                    let layout = if matches!(op, Op::CallArgs(_, _)) {
                        fun.call_layouts[argc as usize].clone()
                    } else { vec![None; argc as usize] };
                    let mut values = Vec::new();
                    for _ in &layout { values.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?); }
                    values.reverse();
                    let (slots, provided) = bind_call_values(callee, &layout, values).map_err(|m| self.err_at(span, m))?;
                    let fname = program.functions[fid].name.clone();
                    frames.push(Frame {
                        fn_idx: fid,
                        loop_counts: HashMap::new(),
            receiver: None, defaults_active: true,
                        ip: 0,
                        provided,
                        slots,
                        retag_matrix: None,
                        has_call_site: true,
                        retag_instance: constructor_tag,
                    });
                    self.push_source_frame(callee, Some(span));
                    self.host.push_call_frame(&fname);
                    self.host.push_call_site_line(span.line);
                }
                Op::TagInstance(idx) => {
                    let class = program.functions.get(idx).ok_or_else(|| self.err_at(span,"TagInstance needs an object function"))?;
                    let v = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    stack.push(crate::value::tag_instance(&class.name, &class.module_path, v));
                }
                Op::MethodCall(_, _) | Op::MethodCallArgs(_, _) => {
                    let (name_idx, argc) = match op {
                        Op::MethodCall(name, argc) => (name, argc as usize),
                        Op::MethodCallArgs(name, layout) => (name, layout),
                        _ => unreachable!(),
                    };
                    let layout = if matches!(op, Op::MethodCallArgs(_, _)) {
                        fun.call_layouts[argc as usize].clone()
                    } else { vec![None; argc as usize] };
                    let argc = layout.len();
                    let method_name = if let Some(name)=method_name_override {name} else {match fun.constants.get(name_idx as usize) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => {
                            return Err(self.err_at(span, "MethodCall needs method name constant"));
                        }
                    }};
                    let recv = pop(&mut stack).map_err(|m| self.err_at(span, m))?;
                    let mut vals = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        vals.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?);
                    }
                    vals.reverse();
                    let type_name =
                        instance_type_name_bc(&recv).map_err(|m| self.err_at(span, m))?;
                    let namespace=crate::value::instance_module(&recv).map_err(|m| self.err_at(span,m))?;
                    let start_oi = if let Some(namespace)=&namespace {
                        program.objects.iter().position(|(name,id)| name==&type_name && &program.functions[*id].module_path==namespace)
                    } else {
                        let local=program.objects.iter().position(|(name,id)| name==&type_name && program.functions[*id].module_scope==fun.module_scope);
                        if local.is_some() { local } else {
                            let candidates=program.objects.iter().enumerate().filter(|(_, (name,_))| name==&type_name).map(|(index,_)|index).collect::<Vec<_>>();
                            if candidates.len()>1 { return Err(self.err_at(span,format!("ambiguous object type `{type_name}`: a module tag is required"))); }
                            candidates.first().copied()
                        }
                    }
                        .ok_or_else(|| {
                            self.err_at(span, format!("unknown object type `{type_name}`"))
                        })?;
                    let method_fid = resolve_method_fid(program, start_oi, &method_name)
                        .ok_or_else(|| {
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
                        })?;
                    let callee = &program.functions[method_fid];
                    let (slots, provided) = bind_call_values(callee, &layout, vals).map_err(|m| self.err_at(span, m))?;
                    let retag_matrix = if crate::linalg_ops::is_matrix_value(&recv) {
                        Some((type_name.clone(), namespace))
                    } else {
                        None
                    };
                    let fname = callee.name.clone();
                    frames.push(Frame {
                        fn_idx: method_fid,
                        loop_counts: HashMap::new(),
                        receiver: Some(recv.clone()), defaults_active: true,
                        ip: 0,
                        provided,
                        slots,
                        retag_matrix,
                        has_call_site: true,
                        retag_instance: None,
                    });
                    self.push_source_frame(callee, Some(span));
                    self.host.push_call_frame(&fname);
                    self.host.push_call_site_line(span.line);
                }
                Op::PluginCallArgs(name_idx,layout) => {
                    let name=fun.constants[name_idx].as_display();let names=&fun.call_layouts[layout];
                    let values=take_actuals(&mut stack,names.len()).map_err(|m|self.err_at(span,m))?;
                    let reg=self.host.plugins.get(&name).cloned().ok_or_else(||self.err_at(span,format!("unknown function `{name}`")))?;
                    let bound=crate::arguments::bind(&reg.params,names,&values,false).map_err(|m|self.err_at(span,m))?;
                    stack.push(crate::host::plugin::call_registered(&mut self.host,&name,&bound).map_err(|m|self.err_at(span,m))?);
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
                        let result =
                            crate::host::plugin::call_registered(&mut self.host, name, &bound)
                                .map_err(|m| self.err_at(span, m))?;
                        stack.push(result);
                    } else {
                        let mut vals = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            vals.push(pop(&mut stack).map_err(|m| self.err_at(span, m))?);
                        }
                        vals.reverse();
                        let reg = self.host.plugins.get(&raw_name).cloned().ok_or_else(|| {
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
                            crate::host::plugin::call_registered(&mut self.host, &raw_name, &bound)
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
                        return Err(
                            self.err_at(span, format!("call_fn: `{name}` must have no parameters"))
                        );
                    }
                    let slots = vec![None; callee.locals.len().max(1)];
                    let fname = callee.name.clone();
                    frames.push(Frame {
                        fn_idx: fid,
                        loop_counts: HashMap::new(),
            receiver: None, defaults_active: true,
                        ip: 0,
                        provided: Vec::new(),
                        slots,
                        retag_matrix: None,
                        has_call_site: true,
                        retag_instance: None,
                    });
                    self.push_source_frame(callee, Some(span));
                    self.host.push_call_frame(&fname);
                    self.host.push_call_site_line(span.line);
                }
                Op::Return => {
                    let ret = pop(&mut stack).unwrap_or(Value::None);
                    let frame = frames.pop();
                    let constructor=frame.as_ref().and_then(|f|f.retag_instance);
                    self.diagnostic_frames.pop();
                    self.host.pop_call_frame();
                    if frame.as_ref().is_some_and(|f| f.has_call_site) {
                        self.host.pop_call_site_line();
                    }
                    let ret = match frame.and_then(|f| f.retag_matrix) {
                        Some((ty,namespace)) => {
                            let result=crate::linalg_ops::retag_matrix_like(&ty,ret);
                            if let Some(namespace)=namespace { crate::value::tag_instance(&ty,&namespace,result) } else { result }
                        },
                        None => ret,
                    };
                    let ret=if let Some(id)=constructor {let class=&program.functions[id];crate::value::tag_instance(&class.name,&class.module_path,ret)}else{ret};
                    if frames.is_empty() {
                        return Ok(ret);
                    }
                    stack.push(ret);
                }
            }
            if print_returns_none{stack.push(Value::None);}
        }
    }
}

fn take_actuals(stack:&mut Vec<Value>,count:usize)->Result<Vec<Value>,String>{
    if stack.len()<count{return Err("bytecode argument stack underflow".into());}
    Ok(stack.split_off(stack.len()-count))
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
        oi = program
            .object_bases
            .get(i)
            .copied()
            .flatten()
            .and_then(|base_flat| program.objects.iter().position(|(_, f)| *f == base_flat));
    }
    None
}

fn pop(stack: &mut Vec<Value>) -> Result<Value, String> {
    stack
        .pop()
        .ok_or_else(|| "bytecode: stack underflow".to_string())
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

fn resolve_fn_dynamic(program: &Program, current: usize, name: &str) -> Result<usize, String> {
    super::names::lookup(program,current,name).ok_or_else(||format!("call_fn: unknown function `{name}`"))
}

/// Bind already evaluated actuals; no expression or foreign effect runs here.
fn bind_call_values(callee: &FnChunk, layout: &[Option<String>], values: Vec<Value>)
    -> std::result::Result<(Vec<Option<Value>>, Vec<bool>), String> {
    if layout.len()!=values.len(){return Err("argument layout/value count mismatch".into());}
    let plan=crate::arguments::plan_with_omissions(&callee.params,layout,&callee.defaults)?;
    let mut slots = vec![None; callee.locals.len().max(1)];
    for (name,actual) in plan.assignments {
        if let Some(index)=callee.locals.iter().position(|local|local==&name){slots[index]=Some(values[actual].clone());}
    }
    let provided=initial_presence(&slots,callee.params.len());
    Ok((slots, provided))
}

/// Presence is derived once, before defaults, rather than independently rebound.
fn initial_presence(slots:&[Option<Value>],parameters:usize)->Vec<bool> {
    slots.iter().take(parameters).map(Option::is_some).collect()
}

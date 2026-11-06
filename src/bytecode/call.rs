//! Check actual staging, scratch separation and post-evaluation call selection.
//! Resolution and dispatch implementations remain semantic interfaces.
use super::{expression::Cursor, ExpressionRule, FnChunk, Op, Program};
use crate::{
    ast::{Arg, CallExpr},
    value::Value,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CallStage {
    pub expression: usize,
    pub slot: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathFallback {
    User { target: usize, tag_instance: bool },
    Error { message: String },
}
impl PathFallback {
    fn evidence(&self) -> serde_json::Value {
        match self {
            Self::User { target, tag_instance } => serde_json::json!({"kind":"user","target":target,"tag_instance":tag_instance}),
            Self::Error { message } => serde_json::json!({"kind":"error","message":message}),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallDispatch {
    User { target: usize, tag_instance: bool },
    Method { receiver: String },
    Builtin,
    Host { function: u16 },
    Plugin,
    RejectDuplicate { name: String },
    Path { receiver: String, check_receiver: bool, fallback: PathFallback },
    ResolutionFault { message: String },
}
#[derive(Debug, Clone)]
pub struct CallEmission {
    pub expression: usize,
    /// Expanded and normalized source, captured before argument replacement.
    pub source: CallExpr,
    pub stages: Vec<CallStage>,
    pub dispatch: Option<CallDispatch>,
    pub as_statement: bool,
}
impl CallEmission {
    pub fn binding_evidence(&self,program:&Program)->serde_json::Value {
        let signature=match self.dispatch.as_ref() {
            Some(CallDispatch::User{target,..})|Some(CallDispatch::Path{fallback:PathFallback::User{target,..},..})=>{
                program.functions.get(*target).map(|callee|(callee.params.clone(),callee.defaults.clone()))
            },
            Some(CallDispatch::Builtin)=>crate::arguments::builtin_signature(&self.source.callee).map(|(params,optional)|
                (params.iter().map(|p|p.to_string()).collect::<Vec<_>>(),vec![optional;params.len()])),
            Some(CallDispatch::Host{function})=>crate::host::HostFn::from_u16(*function).map(|h|{
                let params=h.all_params();let len=params.len();(params.into_iter().map(str::to_string).collect(),vec![true;len])
            }),
            _=>None,
        };
        match signature {
            Some((params,omissions))=>crate::arguments::plan_evidence(&params,&names(self),&omissions),
            None=>serde_json::Value::Null,
        }
    }
    pub fn evidence(&self) -> serde_json::Value {
        use serde_json::json;
        let dispatch = match &self.dispatch {
            Some(CallDispatch::User {
                target,
                tag_instance,
            }) => json!({"kind":"user","target":target,"tag_instance":tag_instance}),
            Some(CallDispatch::Method { receiver }) => json!({"kind":"method","receiver":receiver}),
            Some(CallDispatch::Builtin) => json!({"kind":"builtin"}),
            Some(CallDispatch::Host { function }) => {
                let host = crate::host::HostFn::from_u16(*function);
                json!({"kind":"host","function":function,"params":host.map(|h|h.all_params()),"required":host.map(|h|h.required_params().len())})
            }
            Some(CallDispatch::Plugin) => json!({"kind":"plugin"}),
            Some(CallDispatch::RejectDuplicate { name }) => json!({"kind":"duplicate","name":name}),
            Some(CallDispatch::Path { receiver, check_receiver, fallback }) => json!({"kind":"path","receiver":receiver,"check_receiver":check_receiver,"fallback":fallback.evidence()}),
            Some(CallDispatch::ResolutionFault { message }) => json!({"kind":"resolution_fault","message":message}),
            None => serde_json::Value::Null,
        };
        json!({"expression":self.expression,"callee":self.source.callee,"receiver":self.source.receiver,"path":self.source.path,
            "arguments":self.source.args.iter().map(|a|match a {Arg::Positional(_)=>None,Arg::Named{name,..}=>Some(name)}).collect::<Vec<_>>(),
            "source_rules":self.source.args.iter().map(|a|match a {Arg::Positional(e)|Arg::Named{value:e,..}=>ExpressionRule::from_source(e).evidence()}).collect::<Vec<_>>(),
            "stages":self.stages.iter().map(|s|json!({"expression":s.expression,"slot":s.slot})).collect::<Vec<_>>(),
            "dispatch":dispatch,"as_statement":self.as_statement})
    }
}
fn names(call: &CallEmission) -> Vec<Option<String>> {
    call.source
        .args
        .iter()
        .map(|a| match a {
            Arg::Positional(_) => None,
            Arg::Named { name, .. } => Some(name.clone()),
        })
        .collect()
}
fn read(c: &mut Cursor<'_>, slot: usize) -> Result<(), String> {
    c.op(Op::GetLocal(slot))
}
fn read_child(
    c: &mut Cursor<'_>,
    ids: &mut std::slice::Iter<'_, usize>,
    slot: usize,
) -> Result<(), String> {
    let id = *ids.next().ok_or("missing staged argument reload witness")?;
    let expr = c
        .fun
        .expressions
        .get(id)
        .ok_or("missing reload expression")?;
    if expr.start != c.position
        || expr.end != c.position + 1
        || !matches!(&expr.rule,ExpressionRule::Read(name) if c.fun.locals.get(slot)==Some(name))
        || !expr.children.is_empty()
    {
        return Err("staged argument reload source differs".into());
    }
    read(c, slot)
}
fn layout(c: &mut Cursor<'_>, arguments: &[Option<String>]) -> Result<usize, String> {
    let id = match c.fun.code.get(c.position) {
        Some(Op::CallArgs(_, id) | Op::MethodCallArgs(_, id)) => *id,
        _ => return Err("missing argument-layout call".into()),
    };
    if c.fun.call_layouts.get(id).map(Vec::as_slice) != Some(arguments) {
        return Err("call argument layout differs from source".into());
    }
    Ok(id)
}
pub fn verify(program: &Program, fun: &FnChunk) -> Result<(), String> {
    let caller=program.functions.iter().position(|f|std::ptr::eq(f,fun)).ok_or("missing call owner")?;
    if fun.source_local_count < fun.params.len() || fun.source_local_count > fun.locals.len() {
        return Err("invalid source/scratch local boundary".into());
    }
    let mut owners = HashSet::new();
    let loops = fun
        .statements
        .iter()
        .flat_map(|s| s.auxiliary.iter().copied())
        .collect::<HashSet<_>>();
    let mut active: Vec<(&super::ExpressionEmission, HashSet<usize>)> = Vec::new();
    for call in &fun.calls {
        let node = fun
            .expressions
            .get(call.expression)
            .ok_or("missing call source expression")?;
        if !matches!(node.rule, ExpressionRule::CallBoundary(_))
            || !owners.insert(call.expression)
            || call.as_statement
        {
            return Err("call source owner differs".into());
        }
        let arguments = names(call);
        if call.stages.len() != arguments.len() || node.children.len() < arguments.len() {
            return Err("call authored-actual inventory".into());
        }
        let slots = call.stages.iter().map(|s| s.slot).collect::<HashSet<_>>();
        if slots.len() != arguments.len()
            || slots
                .iter()
                .any(|s| *s < fun.source_local_count || *s >= fun.locals.len() || loops.contains(s))
        {
            return Err("call scratch aliases source/loop/another actual".into());
        }
        while active
            .last()
            .is_some_and(|(parent, _)| parent.end <= node.start)
        {
            active.pop();
        }
        if active.iter().any(|(parent, other)| {
            node.start < parent.start || node.end > parent.end || !slots.is_disjoint(other)
        }) {
            return Err("nested call scratch overlap or interval order".into());
        }
        let mut c = Cursor {
            fun,
            position: node.start,
            end: node.end,
        };
        for (i, stage) in call.stages.iter().enumerate() {
            if node.children[i] != stage.expression {
                return Err("authored actual order differs".into());
            }
            let source = match &call.source.args[i] {
                Arg::Positional(e) | Arg::Named { value: e, .. } => e,
            };
            if ExpressionRule::from_source(source).evidence()
                != fun.expressions[stage.expression].rule.evidence()
            {
                return Err("authored argument source rule differs".into());
            }
            c.child(stage.expression)?;
            c.op(Op::SetLocal(stage.slot))?;
            c.op(Op::Pop)?;
        }
        let mut reloads = node.children[arguments.len()..].iter();
        let dispatch=call.dispatch.as_ref().ok_or("missing call dispatch obligation")?;
        use crate::sema::{DynamicReason, resolution::Target};
        let target=super::names::resolve(program,caller,&call.source);
        let user=match &target { Target::Local(id)|Target::Import{fun:id,..}=>Some(*id),_=>None };
        let user_dispatch=|id:usize|CallDispatch::User{target:id,tag_instance:program.parents[id].is_none()&&program.functions[id].source_ast.level==1};
        let expected=if let Some(receiver)=&call.source.receiver {
            CallDispatch::Method{receiver:receiver.clone()}
        } else if let Some(path)=call.source.path.as_ref().filter(|p|p.len()==2) {
            let fallback=match user.map(user_dispatch) {
                Some(CallDispatch::User{target,tag_instance})=>PathFallback::User{target,tag_instance},
                _=>PathFallback::Error{message:match target {Target::Unresolved{message,..}=>message,_=>format!("unresolved dynamic call `{}`",crate::sema::call_display(&call.source))}},
            };
            CallDispatch::Path{receiver:path[0].clone(),check_receiver:!fun.source_ast.imports.contains(&path[0]),fallback}
        } else if let Some(id)=user {user_dispatch(id)} else {match target {
            Target::Builtin(_)|Target::Dynamic{reason:DynamicReason::CallFn,..}=>CallDispatch::Builtin,
            Target::Host(h)=>CallDispatch::Host{function:h.as_u16()},
            Target::Dynamic{reason:DynamicReason::Plugin,..}=>CallDispatch::Plugin,
            Target::Unresolved{message,..}=>CallDispatch::ResolutionFault{message},
            _=>return Err("unmodeled source call resolution".into()),
        }};
        if *dispatch!=expected{return Err("call resolution differs from source declaration graph".into());}
        match dispatch {
            CallDispatch::Path{..}=>{
                for stage in &call.stages{read_child(&mut c,&mut reloads,stage.slot)?;}
                let id=match fun.code.get(c.position){Some(Op::PathCallArgs(owner,id)) if fun.calls.get(*owner).is_some_and(|other|std::ptr::eq(other,call))=>*id,_=>return Err("path call owner differs".into())};
                if fun.call_layouts.get(id)!=Some(&arguments){return Err("path call layout differs".into());}
                let owner=fun.calls.iter().position(|other|std::ptr::eq(other,call)).unwrap();
                c.op(Op::PathCallArgs(owner,id))?;
            },
            CallDispatch::ResolutionFault{message}=>{
                let index=match fun.code.get(c.position){Some(Op::RejectCallResolution(index))=>*index,_=>return Err("missing call resolution fault".into())};
                if fun.constants.get(index)!=Some(&Value::Text(message.clone())){return Err("call resolution fault payload differs".into());}
                c.op(Op::RejectCallResolution(index))?;
            },
            CallDispatch::RejectDuplicate { name } => {
                let index = match fun.code.get(c.position) {
                    Some(Op::RejectDuplicateArgument(i)) => *i,
                    _ => return Err("duplicate argument refusal missing".into()),
                };
                if fun.constants.get(index) != Some(&Value::Text(name.clone())) {
                    return Err("duplicate argument name differs".into());
                }
                c.op(Op::RejectDuplicateArgument(index))?;
            }
            CallDispatch::User {
                target,
                tag_instance,
            } => {
                if *target >= program.functions.len()
                    || *tag_instance != program.objects.iter().any(|(_, id)| id == target)
                {
                    return Err("resolved call target/tag differs".into());
                }
                for stage in &call.stages {
                    read_child(&mut c, &mut reloads, stage.slot)?;
                }
                let id = layout(&mut c, &arguments)?;
                c.op(Op::CallArgs(*target, id))?;
                if *tag_instance {
                    c.op(Op::TagInstance(*target))?;
                }
            }
            CallDispatch::Method { receiver } => {
                if call.source.receiver.as_ref() != Some(receiver)
                    && call.source.path.as_ref().and_then(|p| p.first()) != Some(receiver)
                {
                    return Err("method receiver differs from source".into());
                }
                for stage in &call.stages {
                    read_child(&mut c, &mut reloads, stage.slot)?;
                }
                let slot = fun
                    .locals
                    .iter()
                    .position(|n| n == receiver)
                    .ok_or("method receiver slot")?;
                read(&mut c, slot)?;
                let index = match fun.code.get(c.position) {
                    Some(Op::MethodCallArgs(i, _)) => *i,
                    _ => return Err("method dispatch missing".into()),
                };
                if fun.constants.get(index) != Some(&Value::Text(call.source.callee.clone())) {
                    return Err("method name differs from source".into());
                }
                let id = layout(&mut c, &arguments)?;
                c.op(Op::MethodCallArgs(index, id))?;
            }
            CallDispatch::Builtin|CallDispatch::Plugin|CallDispatch::Host{..}=>{
                for stage in &call.stages{read_child(&mut c,&mut reloads,stage.slot)?;}
                let (first,id)=match fun.code.get(c.position){Some(Op::BuiltinCallArgs(n,l)|Op::PluginCallArgs(n,l))=>(*n,*l),Some(Op::HostCallArgs(n,l))=>(*n as usize,*l),_=>return Err("missing native argument dispatch".into())};
                if fun.call_layouts.get(id)!=Some(&arguments){return Err("native argument layout differs".into());}
                match dispatch {
                    CallDispatch::Builtin=>{
                        if crate::arguments::builtin_signature(&call.source.callee).is_none() || fun.constants.get(first)!=Some(&Value::Text(call.source.callee.clone())){return Err("builtin identity differs".into());}
                        c.op(Op::BuiltinCallArgs(first,id))?;
                    },
                    CallDispatch::Plugin=>{
                        if fun.constants.get(first)!=Some(&Value::Text(call.source.callee.clone())){return Err("plugin identity differs".into());}
                        c.op(Op::PluginCallArgs(first,id))?;
                    },
                    CallDispatch::Host{function}=>{
                        let hf=crate::host::HostFn::from_name(&call.source.callee).ok_or("source host name")?;
                        if hf.as_u16()!=*function || first!=*function as usize{return Err("host primitive identity differs".into());}
                        c.op(Op::HostCallArgs(*function,id))?;
                    },_=>unreachable!(),
                }
            }

        }
        if reloads.next().is_some() || c.position != node.end {
            return Err("call instruction/expression coverage differs".into());
        }
        active.push((node, slots));
    }
    if fun
        .expressions
        .iter()
        .filter(|e| matches!(e.rule, ExpressionRule::CallBoundary(_)))
        .count()
        != owners.len()
    {
        return Err("uncovered call source expression".into());
    }
    Ok(())
}

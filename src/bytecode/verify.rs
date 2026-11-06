//! Checks every emitted instruction and a control-flow stack-height certificate.
//! This is structural validation, not source/bytecode semantic equivalence.
use super::{FnChunk, Op, Program};
use crate::value::Value;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct StackCertificate {
    pub heights: Vec<Option<usize>>,
}

impl Program {
    pub fn verify(&self) -> Result<Vec<StackCertificate>, String> {
        let n = self.functions.len();
        if self.entry >= n || self.parents.len() != n || self.children.len() != n {
            return Err("invalid program function tables".into());
        }
        for (i, parent) in self.parents.iter().enumerate() {
            if parent.is_some_and(|p| p >= n || p == i) {
                return Err("invalid function parent".into());
            }
            for &child in &self.children[i] {
                if child >= n || self.parents[child] != Some(i) {
                    return Err("inconsistent function child".into());
                }
            }
        }
        check_forest(&self.parents)?;
        for (child, parent) in self.parents.iter().enumerate() {
            if let Some(parent) = parent {
                if !self.children[*parent].contains(&child)
                    || self.functions[*parent].module_path != self.functions[child].module_path
                {
                    return Err("inconsistent parent owner".into());
                }
            }
        }
        super::names::verify(self)?;
        if self.objects.len() != self.object_bases.len() {
            return Err("invalid object tables".into());
        }
        for (i, (_, fid)) in self.objects.iter().enumerate() {
            if *fid >= n
                || self.object_bases[i]
                    .is_some_and(|base| base >= n || !self.objects.iter().any(|(_, f)| *f == base))
            {
                return Err("invalid object reference".into());
            }
        }
        if self.methods.iter().any(|(owner, _, fid)| {
            *owner >= n || *fid >= n || !self.objects.iter().any(|(_, f)| f == owner)
        }) || self.path_index.values().any(|id| *id >= n)
        {
            return Err("invalid method/export reference".into());
        }
        let object_ids = self
            .objects
            .iter()
            .enumerate()
            .map(|(i, (_, fid))| (*fid, i))
            .collect::<std::collections::HashMap<_, _>>();
        if object_ids.len() != self.objects.len() {
            return Err("duplicate object reference".into());
        }
        let bases = self
            .object_bases
            .iter()
            .map(|base| base.map(|fid| object_ids[&fid]))
            .collect::<Vec<_>>();
        check_forest(&bases)?;
        if self
            .methods
            .iter()
            .any(|(owner, _, fid)| self.parents[*fid] != Some(*owner))
        {
            return Err("inconsistent method owner".into());
        }
        self.functions
            .iter()
            .enumerate()
            .map(|(i, f)| {
                verify_chunk(self, i, f)
                    .map_err(|e| format!("bytecode verification in {}: {e}", f.name))
            })
            .collect()
    }
}

fn verify_chunk(program: &Program, _fid: usize, fun: &FnChunk) -> Result<StackCertificate, String> {
    super::expression::verify(fun)?;
    super::statement::verify(fun)?;
    super::call::verify(program,fun)?;
    super::source::verify(fun)?;
    let size = fun.code.len();
    if size == 0 || fun.spans.len() != size {
        return Err("invalid code/span table".into());
    }
    if fun.defaults.len() != fun.params.len()
        || fun.locals.get(..fun.params.len()) != Some(fun.params.as_slice())
    {
        return Err("parameters must be a local-slot prefix".into());
    }
    if fun.locals.iter().collect::<HashSet<_>>().len() != fun.locals.len() {
        return Err("duplicate local slot name".into());
    }
    let local = |i: usize| {
        if i < fun.locals.len() {
            Ok(())
        } else {
            Err("invalid local reference".to_string())
        }
    };
    let target = |i: usize| {
        if i < size {
            Ok(())
        } else {
            Err("invalid jump target".to_string())
        }
    };
    let function = |i: usize| {
        if i < program.functions.len() {
            Ok(())
        } else {
            Err("invalid call target".to_string())
        }
    };
    let text = |i: usize| {
        if matches!(fun.constants.get(i), Some(Value::Text(_))) {
            Ok(())
        } else {
            Err("invalid text constant reference".to_string())
        }
    };
    let mut shapes = Vec::with_capacity(size);
    for (ip, op) in fun.code.iter().copied().enumerate() {
        // pop count, push count, successor PCs. All opcodes are handled.
        let next = vec![ip + 1];
        let shape = match op {
            Op::Constant(i) => {
                if i >= fun.constants.len() {
                    return Err("invalid constant reference".into());
                }
                (0, 1, next)
            }
            Op::True | Op::False | Op::None_ => (0, 1, next),
            Op::GetLocal(i) => {
                local(i)?;
                (0, 1, next)
            }
            Op::SetLocal(i) | Op::CheckBinding(i) => {
                local(i)?;
                (1, 1, next)
            }
            Op::Pop | Op::Print => (1, 0, next),
            Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Div
            | Op::Equal
            | Op::NotEqual
            | Op::Greater
            | Op::GreaterEqual
            | Op::Less
            | Op::Lessequal
            | Op::Split
            | Op::Join
            | Op::GetIndex
            | Op::FootnoteGet => (2, 1, next),
            Op::ContractFootnoteGet(i) => {
                local(i)?;
                (2, 1, next)
            }
            Op::Negate
            | Op::Not
            | Op::Input
            | Op::Len
            | Op::Str
            | Op::Int
            | Op::TypeOf
            | Op::Trim
            | Op::IterItems
            | Op::CallFn => (1, 1, next),
            Op::BuildList(n) => (n, 1, next),
            Op::BuildMap(n) => (n.checked_mul(2).ok_or("map arity overflow")?, 1, next),
            Op::Jump(t) => {
                target(t)?;
                (0, 0, vec![t])
            }
            Op::JumpIfFalse(t) => {
                target(t)?;
                (1, 0, vec![ip + 1, t])
            }
            Op::JumpIfProvided(p, t) => {
                target(t)?;
                if p >= fun.params.len() {
                    return Err("invalid presence reference".into());
                }
                (0, 0, vec![ip + 1, t])
            }
            Op::ResetLoop(id)|Op::CheckLoop(id)=>{
                if !matches!(fun.statements.get(id).map(|s|&s.rule),Some(super::StatementRule::While(_))){return Err("invalid source loop reference".into());}
                (0,0,next)
            },
            Op::FillMissingParameters | Op::CheckArguments | Op::RestoreReceiver | Op::CheckFrameArguments => (0, 0, next),
            Op::CheckReturn => (1, 1, next),
            Op::Call(f, n) => {
                function(f)?;
                (n, 1, next)
            }
            Op::CallArgs(f, l) => {
                function(f)?;
                let layout = fun.call_layouts.get(l).ok_or("invalid call layout")?;
                (layout.len(), 1, next)
            }
            Op::BuiltinCallArgs(name,l)|Op::PluginCallArgs(name,l)=>{
                text(name)?;
                if matches!(op,Op::BuiltinCallArgs(_,_)) && !matches!(&fun.constants[name],Value::Text(name) if crate::arguments::builtin_signature(name).is_some()){return Err("unknown builtin signature".into());}
                let layout=fun.call_layouts.get(l).ok_or("invalid call layout")?;(layout.len(),1,next)
            },
            Op::HostCallArgs(id,l)=>{
                if crate::host::HostFn::from_u16(id).is_none(){return Err("unknown host id".into());}
                let layout=fun.call_layouts.get(l).ok_or("invalid call layout")?;(layout.len(),1,next)
            },
            Op::PathCallArgs(id,l)=>{
                if !matches!(fun.calls.get(id).and_then(|c|c.dispatch.as_ref()),Some(super::CallDispatch::Path{..})){return Err("invalid path call record".into());}
                let layout=fun.call_layouts.get(l).ok_or("invalid path call layout")?;(layout.len(),1,next)
            },
            Op::RejectCallResolution(index)=>{text(index)?;(0,0,vec![])},
            Op::MethodCall(name, n) => {
                text(name)?;
                (n.checked_add(1).ok_or("method arity overflow")?, 1, next)
            }
            Op::MethodCallArgs(name, l) => {
                text(name)?;
                let layout = fun.call_layouts.get(l).ok_or("invalid method layout")?;
                (
                    layout.len().checked_add(1).ok_or("method arity overflow")?,
                    1,
                    next,
                )
            }
            Op::PluginCall(name, n) => {
                text(name)?;
                if let Value::Text(s) = &fun.constants[name] {
                    if s.starts_with('@') && n % 2 != 0 {
                        return Err("odd named plugin arity".into());
                    }
                }
                (n, 1, next)
            }
            Op::HostCall(id, n) => {
                if crate::host::HostFn::from_u16(id).is_none() {
                    return Err("invalid host reference".into());
                }
                (n, 1, next)
            }
            Op::TagInstance(f) => {
                function(f)?;
                if !program.objects.iter().any(|(_, id)| *id == f) {
                    return Err("invalid constructor reference".into());
                }
                (1, 1, next)
            }
            Op::RejectDuplicateArgument(name) | Op::RejectArgumentExpansion(name) => {
                text(name)?;
                (0, 0, vec![])
            }
            Op::Return => (1, 0, vec![]),
        };
        shapes.push(shape);
    }
    let mut heights = vec![None; size];
    heights[0] = Some(0);
    let mut queue = VecDeque::from([0]);
    while let Some(ip) = queue.pop_front() {
        let height = heights[ip].unwrap();
        let (pop, push, edges) = &shapes[ip];
        if height < *pop {
            return Err(format!("stack underflow at {ip}"));
        }
        if matches!(fun.code[ip], Op::Return) && height != 1 {
            return Err(format!("return leaks stack values at {ip}"));
        }
        let after = (height - pop)
            .checked_add(*push)
            .ok_or("stack height overflow")?;
        for &to in edges {
            target(to)?;
            match heights[to] {
                Some(known) if known != after => {
                    return Err(format!("inconsistent stack join at {to}"))
                }
                Some(_) => {}
                None => {
                    heights[to] = Some(after);
                    queue.push_back(to);
                }
            }
        }
    }
    Ok(StackCertificate { heights })
}

fn check_forest(parents: &[Option<usize>]) -> Result<(), String> {
    let mut colors = vec![0u8; parents.len()];
    for start in 0..parents.len() {
        if colors[start] != 0 {
            continue;
        }
        let mut cursor = Some(start);
        let mut path = Vec::new();
        while let Some(node) = cursor {
            if node >= parents.len() {
                return Err("invalid forest reference".into());
            }
            if colors[node] == 1 {
                return Err("cyclic bytecode declaration graph".into());
            }
            if colors[node] == 2 {
                break;
            }
            colors[node] = 1;
            path.push(node);
            cursor = parents[node];
        }
        for node in path {
            colors[node] = 2;
        }
    }
    Ok(())
}

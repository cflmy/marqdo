//! Symbolic branch VCs for a deliberately restricted, pure integer fragment.
//! A proved proposition in the Int abstraction is NOT by itself a proof of
//! Marqdo runtime semantics (i64, dynamic values and exceptions).
use std::collections::BTreeMap;
use crate::prove::proof_check::{self,ClaimAst as A,CTok as T};
use crate::sema::ir::{BoundCall,SemanticProject};

#[derive(Debug,Clone)]
pub struct SymbolicGoal{
    pub proposition:String,
    pub guard_propositions:Vec<String>,
    pub guard_index:usize,
    pub binders:Vec<String>,
    pub runtime_domain_verified:bool,
}
fn identifier(s:&str)->bool {
    !s.is_empty()&&s.chars().all(|c|c.is_alphanumeric()||c=='_')
}
fn term(expr:&A,bindings:&BTreeMap<String,String>)->Option<(String,bool)>{
    match expr {
        A::Lit(T::Int(n))=>Some((format!("({n} : Int)"),false)),
        A::Lit(T::Bool(b))=>Some((if *b{"True"}else{"False"}.into(),true)),
        A::Var(v)=>Some((bindings.get(v)?.clone(),false)),
        A::Bin{op,left,right}=>{
            let (l,lb)=term(left,bindings)?;
            let (r,rb)=term(right,bindings)?;
            match *op{
                "=" | "≠" | "<" | "≤" | ">" | "≥" if !lb && !rb=>
                    Some((format!("({l} {op} {r})"),true)),
                "且" if lb && rb=>Some((format!("({l} ∧ {r})"),true)),
                "或" if lb && rb=>Some((format!("({l} ∨ {r})"),true)),
                _=>None
            }
        },
        A::Un{op:"非",expr}=>{
            let (inner,boolean)=term(expr,bindings)?;
            boolean.then(||(format!("(¬ {inner})"),true))
        },
        _=>None
    }
}
fn proposition(source:&str,bindings:&BTreeMap<String,String>)->Option<String>{
    let tokens=proof_check::tokenize_claim(source).ok()?;
    let (_,expr)=proof_check::parse_claim_expr(&tokens).ok()?;
    let (text,boolean)=term(&expr,bindings)?;
    boolean.then_some(text)
}
/// Source-backed branch facts are the only assumptions accepted; no theorem
/// premise provided by a model/user is silently promoted to an axiom.
pub fn plan(semantic:&SemanticProject,call:&BoundCall,premise:&str)->Option<SymbolicGoal>{
    if call.control_blocker.is_some()||call.path_guards.is_empty(){return None}
    let callee=semantic.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.target))?.signature.as_ref()?;
    let caller=semantic.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.caller))?.signature.as_ref()?;
    if callee.params.len()!=call.args.len(){return None}
    let mut caller_bind=BTreeMap::new();
    let mut binders=Vec::new();
    for (i,param) in caller.params.iter().enumerate(){
        if !identifier(&param.name){return None}
        let variable=format!("v{i}");
        caller_bind.insert(param.name.clone(),format!("({variable} : Int)"));
        binders.push(variable);
    }
    let domain=caller.params.iter().all(|p|p.ty.eq_ignore_ascii_case("int"));
    let mut callee_bind=BTreeMap::new();
    for (formal,arg) in callee.params.iter().zip(call.args.iter()){
        if arg.name.as_ref().is_some_and(|s|s!=&formal.name){return None}
        let actual=if let Some(n)=arg.integer {
            format!("({n} : Int)")
        }else{
            caller_bind.get(arg.symbol.as_ref()?)?.clone()
        };
        if !identifier(&formal.name){return None}
        callee_bind.insert(formal.name.clone(),actual);
    }
    let wanted=proposition(premise,&callee_bind)?;
    let all=call.path_guards.iter().map(|guard|proposition(guard,&caller_bind))
        .collect::<Option<Vec<_>>>()?;
    let idx=all.iter().position(|guard|guard==&wanted)?;
    Some(SymbolicGoal {
        proposition:wanted,guard_propositions:all,guard_index:idx,
        binders,runtime_domain_verified:domain
    })
}
pub fn emit_lean(goals:&[(String,SymbolicGoal)])->String{
    let mut text=String::from("-- Symbolic path theorem over Int model; not runtime equivalence.\nnamespace MarqdoSymbolic\n");
    for (i,(_,g)) in goals.iter().enumerate(){
        let vars=g.binders.iter().map(|x|format!("({x} : Int)"))
            .collect::<Vec<_>>().join(" ");
        let hypotheses=g.guard_propositions.iter().enumerate()
            .map(|(n,p)|format!("(h{n} : {p})"))
            .collect::<Vec<_>>().join(" ");
        text.push_str(&format!("theorem path_vc_{i} {vars} {hypotheses} : {} := by\n  exact h{}\n",
            g.proposition,g.guard_index));
    }
    text.push_str("end MarqdoSymbolic\n");
    text
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn symbolic_guards_cannot_be_invented_from_unsupported_calls(){
        assert!(proposition("lib.fetch(x) > 0",&BTreeMap::new()).is_none());
    }
}

//! Closed, overflow-safe callsite verification conditions. A generated
//! formula is a Lean *goal*, not evidence. Only a subsequent kernel invocation
//! can discharge it. Unknown/symbolic arguments always remain open.
use std::collections::BTreeMap;
use crate::prove::proof_check::{self,ClaimAst as A,CTok as T};
use crate::sema::ir::{BoundCall, SemanticProject};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Goal {
    /// Exact statement of the instantiated proposition.
    pub proposition: String,
    /// Conservative host-side check for early rejection only; not a proof.
    pub holds: bool,
}
enum Term {
    Int(i64,String),
    Bool(bool,String),
}
fn calc(ast:&A, actuals:&BTreeMap<String,i64>)->Option<Term>{
    match ast {
        A::Lit(T::Int(n))=>Some(Term::Int(*n,format!("({n} : Int)"))),
        A::Lit(T::Bool(b))=>Some(Term::Bool(*b,
            if *b{"True"}else{"False"}.into())),
        A::Var(v)=>actuals.get(v).map(|n|Term::Int(*n,format!("({n} : Int)"))),
        A::Un{op,expr}=>match (*op,calc(expr,actuals)?) {
            ("-",Term::Int(n,code))=>Some(Term::Int(n.checked_neg()?,format!("(-{code})"))),
            ("非",Term::Bool(b,code))=>Some(Term::Bool(!b,format!("(¬ {code})"))),
            _=>None
        },
        A::Bin{op,left,right}=>{
            let l=calc(left,actuals)?;
            let r=calc(right,actuals)?;
            match (l,r) {
                (Term::Int(x,a),Term::Int(y,b))=>{
                    let pair=match *op {
                        "+"=>Some((x.checked_add(y)?,format!("({a} + {b})"))),
                        "-"=>Some((x.checked_sub(y)?,format!("({a} - {b})"))),
                        "*"=>Some((x.checked_mul(y)?,format!("({a} * {b})"))),
                        _=>None
                    };
                    if let Some((n,code))=pair {return Some(Term::Int(n,code))}
                    let (truth,rel)=match *op {
                        "="=>(x==y,"="),"≠"=>(x!=y,"≠"),
                        "<"=>(x<y,"<"),"≤"=>(x<=y,"≤"),
                        ">"=>(x>y,">"),"≥"=>(x>=y,"≥"),_=>return None
                    };
                    Some(Term::Bool(truth,format!("({a} {rel} {b})")))
                },
                (Term::Bool(x,a),Term::Bool(y,b))=>{
                    let (truth,code)=match *op {
                        "且"=>(x&&y,format!("({a} ∧ {b})")),
                        "或"=>(x||y,format!("({a} ∨ {b})")),
                        "="=>(x==y,format!("({a} ↔ {b})")),
                        "≠"=>(x!=y,format!("(¬ ({a} ↔ {b}))")),
                        _=>return None
                    };
                    Some(Term::Bool(truth,code))
                },
                _=>None
            }
        },
        _=>None
    }
}
pub fn instantiate(semantic:&SemanticProject,call:&BoundCall,premise:&str)->Option<Goal>{
    let owner=semantic.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.target))?;
    let signature=owner.signature.as_ref()?;
    if signature.params.len()!=call.args.len(){return None}
    // No implicit defaults, no mixed positional/named ambiguity. Explicitly
    // require that the exact ordered actuals match the callee signature.
    let mut bindings=BTreeMap::new();
    for (formal,actual) in signature.params.iter().zip(call.args.iter()){
        if actual.name.as_ref().is_some_and(|name|name!=&formal.name){
            return None
        }
        bindings.insert(formal.name.clone(),actual.integer?);
    }
    let tokens=proof_check::tokenize_claim(premise).ok()?;
    let (_,expr)=proof_check::parse_claim_expr(&tokens).ok()?;
    match calc(&expr,&bindings)? {
        Term::Bool(holds,proposition)=>Some(Goal{holds,proposition}),
        Term::Int(..)=>None
    }
}
pub fn emit_lean(goals:&[(String,Goal)])->String{
    let mut out=String::from(
        "-- Generated closed obligations. These are not accepted until Lean checks them.\n");
    out.push_str("namespace MarqdoClosedCallsite\n");
    for (i,(_,goal)) in goals.iter().enumerate(){
        out.push_str(&format!(
            "theorem vc_{i} : {} := by decide\n",goal.proposition));
    }
    out.push_str("end MarqdoClosedCallsite\n");
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bool_and_arithmetic_propositions_are_kernel_goals_not_certs(){
        let ast=proof_check::parse_claim_expr(
            &proof_check::tokenize_claim("金额 > 0 且 金额 < 10").unwrap()).unwrap().1;
        let args=BTreeMap::from([("金额".into(),3)]);
        let Term::Bool(holds,prop)=calc(&ast,&args).unwrap() else {panic!("not boolean")};
        assert!(holds);
        assert!(prop.contains("∧")&&prop.contains("(3 : Int)"));
        let lean=emit_lean(&[("test".into(),Goal{proposition:prop,holds})]);
        assert!(lean.contains("theorem vc_0"));
        assert!(!lean.contains("sorry"));
    }
    #[test]
    fn overflowing_i64_expression_is_not_silently_lean_unbounded(){
        let ast=proof_check::parse_claim_expr(
            &proof_check::tokenize_claim("x + 1 > 0").unwrap()).unwrap().1;
        assert!(calc(&ast,&BTreeMap::from([("x".into(),i64::MAX)])).is_none());
    }
}

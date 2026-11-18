//! Independent assume/guarantee composition of source-backed pure arithmetic
//! expressions. All results are **Lean Int-model** theorems, NOT dynamic
//! Marqdo Number/i64 runtime equivalence. Every unsupported case fails closed.
use std::{collections::BTreeSet,fs,path::Path,time::Duration};
use anyhow::{Context,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use crate::{
    ast::{BinaryOp,Expr,Function,Stmt,UnaryOp},
    prove::{driver::{Toolchain,compile_import_module},
            proof_check::{self,ClaimAst,CTok}},
    sema::ir::{BoundCall,Compilation,SemanticArtifact,CanonicalParam}
};
use super::projection::{load_function,forwarder,closures};

const KIND:&str="marqdo_pure_expression_composition";
const SCOPE:&str="number_contract_int_model_pure_expression_forwarder";
fn sha(s:&[u8])->String{format!("sha256:{:x}",Sha256::digest(s))}

#[derive(Clone,Debug,PartialEq,Eq)]
enum Term{
    Number(i64), Param(usize), Neg(Box<Term>),
    Add(Box<Term>,Box<Term>),Sub(Box<Term>,Box<Term>),
    Mul(Box<Term>,Box<Term>)
}
impl Term{
    fn source(e:&Expr,params:&[CanonicalParam],depth:usize)->Option<Self>{
        if depth>=32{return None}
        match e{
            Expr::Literal(crate::ast::Literal::Int(n))=>Some(Self::Number(*n)),
            Expr::Var(s)=>Some(Self::Param(unique_param(params,s)?)),
            Expr::Occluded{payload:Some(inner),..}=>Self::source(inner,params,depth+1),
            Expr::Unary{op:UnaryOp::Neg,expr}=>
                Some(Self::Neg(Box::new(Self::source(expr,params,depth+1)?))),
            Expr::Binary{op,left,right}=>{
                let l=Box::new(Self::source(left,params,depth+1)?);
                let r=Box::new(Self::source(right,params,depth+1)?);
                match op{
                    BinaryOp::Add=>Some(Self::Add(l,r)),
                    BinaryOp::Sub=>Some(Self::Sub(l,r)),
                    BinaryOp::Mul=>Some(Self::Mul(l,r)),
                    _=>None
                }
            },
            _=>None
        }
    }
    fn claim(e:&ClaimAst,params:&[CanonicalParam],depth:usize)->Option<Self>{
        if depth>=32{return None}
        match e{
            ClaimAst::Lit(CTok::Int(n))=>Some(Self::Number(*n)),
            ClaimAst::Var(s)=>Some(Self::Param(unique_param(params,s)?)),
            ClaimAst::Un{op:"-",expr}=>Some(Self::Neg(Box::new(
                Self::claim(expr,params,depth+1)?))),
            ClaimAst::Bin{op,left,right}=>{
                let l=Box::new(Self::claim(left,params,depth+1)?);
                let r=Box::new(Self::claim(right,params,depth+1)?);
                match *op{
                    "+"=>Some(Self::Add(l,r)),
                    "-"=>Some(Self::Sub(l,r)),
                    "*"=>Some(Self::Mul(l,r)),
                    _=>None
                }
            }
            _=>None
        }
    }
    fn substitute(&self,args:&[Term],depth:usize)->Option<Self>{
        if depth>=64{return None}
        match self{
            Self::Number(n)=>Some(Self::Number(*n)),
            Self::Param(i)=>args.get(*i).cloned(),
            Self::Neg(a)=>Some(Self::Neg(Box::new(a.substitute(args,depth+1)?))),
            Self::Add(a,b)=>Some(Self::Add(Box::new(a.substitute(args,depth+1)?),
                Box::new(b.substitute(args,depth+1)?))),
            Self::Sub(a,b)=>Some(Self::Sub(Box::new(a.substitute(args,depth+1)?),
                Box::new(b.substitute(args,depth+1)?))),
            Self::Mul(a,b)=>Some(Self::Mul(Box::new(a.substitute(args,depth+1)?),
                Box::new(b.substitute(args,depth+1)?)))
        }
    }
    fn lean(&self)->String{
        match self{
            Self::Number(n)=>format!("({n} : Int)"),
            Self::Param(i)=>format!("v{i}"),
            Self::Neg(a)=>format!("(-{})",a.lean()),
            Self::Add(a,b)=>format!("({} + {})",a.lean(),b.lean()),
            Self::Sub(a,b)=>format!("({} - {})",a.lean(),b.lean()),
            Self::Mul(a,b)=>format!("({} * {})",a.lean(),b.lean())
        }
    }
    fn is_composite(&self)->bool{
        matches!(self,Self::Neg(..)|Self::Add(..)|Self::Sub(..)|Self::Mul(..))
    }
}
fn unique_param(params:&[CanonicalParam],name:&str)->Option<usize>{
    let i=params.iter().position(|p|p.name==name)?;
    (params.iter().filter(|p|p.name==name).count()==1).then_some(i)
}
fn claim_matches(compiled:&Compilation,artifact:&SemanticArtifact,
    params:&[CanonicalParam],want:&Term)->bool{
    let Some(owner)=artifact.symbol_id.as_ref() else{return false};
    // The declaration must unambiguously refer to the exact source equation.
    // Do not convert any arbitrary author-written proposition into an axiom.
    compiled.semantics.claims.iter().filter(|c|{
        if c.owner!=*owner||
            !c.premise.as_deref().map_or(true,|s|s.trim().is_empty()){
            return false
        }
        let Ok(tokens)=proof_check::tokenize_claim(&c.conclusion) else{return false};
        let Ok((_,tree))=proof_check::parse_claim_expr(&tokens) else{return false};
        match tree{
            ClaimAst::Bin{op:"=",left,right}
                if matches!(*left,ClaimAst::Var(ref v) if v=="结果")=>
                Term::claim(&right,params,0).as_ref()==Some(want),
            _=>false
        }
    }).count()==1
}
fn callee_expression(fun:&Function,params:&[CanonicalParam])->Option<Term>{
    match fun.body.as_slice(){
        [Stmt::Return{value,..}]=>Term::source(value,params,0),
        _=>None
    }
}
struct Prepared{
    identity:String,
    callee:String,
    caller:String
}
fn candidate(c:&Compilation,call:&BoundCall)->Option<Prepared>{
    let sem=&c.semantics;
    let ca=single_artifact(c,&call.caller)?;
    let ce=single_artifact(c,&call.target)?;
    if ca.source.file==ce.source.file||call.caller==call.target
        ||!ca.effects.is_pure()||!ce.effects.is_pure()
        ||!call.required_effects.is_pure()
        ||call.control_blocker.is_some()||!call.path_guards.is_empty(){return None}
    if sem.bound_calls.iter().filter(|x|x.caller==call.caller).count()!=1{
        return None
    }
    let cs=ca.signature.as_ref()?;
    let es=ce.signature.as_ref()?;
    let caller=load_function(c,ca)?;
    let callee=load_function(c,ce)?;
    if caller.is_object()||callee.is_object(){return None}
    if caller.params.len()!=cs.params.len()
        ||callee.params.len()!=es.params.len()
        ||call.args.len()!=es.params.len(){return None}
    if !caller.params.iter().zip(cs.params.iter()).all(|(a,b)|a.name==b.name)
        ||!callee.params.iter().zip(es.params.iter()).all(|(a,b)|a.name==b.name){
        return None
    }
    if cs.params.iter().chain(es.params.iter())
        .any(|p|p.ty!="number"||!p.required||p.default_digest.is_some()){
        return None
    }
    let body=callee_expression(&callee,&es.params)?;
    if !body.is_composite(){return None}
    if !claim_matches(c,ce,&es.params,&body){return None}
    let forwarded=forwarder(&caller)?.for_evaluation().ok()?;
    if forwarded.path.is_none()||forwarded.receiver.is_some()
        ||crate::sema::bind::call_display(&forwarded)!=call.call_site
        ||forwarded.args.len()!=call.args.len(){return None}
    let args=call.args.iter().zip(es.params.iter()).map(|(a,p)|{
        if a.name.as_ref().is_some_and(|name|name!=&p.name){return None}
        if let Some(n)=a.integer{return Some(Term::Number(n))}
        let symbol=a.symbol.as_ref()?;
        Some(Term::Param(unique_param(&cs.params,symbol)?))
    }).collect::<Option<Vec<_>>>()?;
    let expected=body.substitute(&args,0)?;
    if !claim_matches(c,ca,&cs.params,&expected){return None}
    let callee_binders=binders(es.params.len());
    let caller_binders=binders(cs.params.len());
    let callee_vars=vars(es.params.len());
    let caller_vars=vars(cs.params.len());
    let call_args=args.iter().map(Term::lean).collect::<Vec<_>>().join(" ");
    let callee_src=format!(
        "-- Source-derived callee expression; Lean Int abstraction only.\n\
         def callee {callee_binders} : Int := {}\n\
         theorem callee_contract {callee_binders} : callee {callee_vars} = {} := by rfl\n",
         body.lean(),body.lean());
    let caller_src=format!(
        "import CalleeModule\n\
         -- The caller imports the callee's theorem, never its implementation body.\n\
         def caller {caller_binders} : Int := callee {call_args}\n\
         theorem caller_contract {caller_binders} : caller {caller_vars} = {} := by\n\
         \x20\x20simpa [caller] using (callee_contract {call_args})\n",
         expected.lean());
    Some(Prepared{
        identity:format!("{}#{}",ca.artifact_id.0,ce.artifact_id.0),
        callee:callee_src,caller:caller_src
    })
}
fn single_artifact<'a>(c:&'a Compilation,
    symbol:&crate::sema::SymbolId)->Option<&'a SemanticArtifact>{
    let mut found=c.semantics.artifacts.iter().filter(|a|
        a.kind=="function"&&a.symbol_id.as_ref()==Some(symbol));
    let item=found.next()?;
    found.next().is_none().then_some(item)
}
fn binders(n:usize)->String{
    (0..n).map(|i|format!("(v{i} : Int)")).collect::<Vec<_>>().join(" ")
}
fn vars(n:usize)->String{
    (0..n).map(|i|format!("v{i}")).collect::<Vec<_>>().join(" ")
}
pub(super) fn eligible(c:&Compilation,call:&BoundCall)->bool{
    candidate(c,call).is_some()
}
pub(super) fn prove(c:&Compilation,out:&Path,lean:&Toolchain)->Result<Vec<Value>>{
    let mut results=Vec::new();
    for call in &c.semantics.bound_calls{
        if super::unified::covers(c,&call.caller){continue}
        let Some(plan)=candidate(c,call) else{continue};
        let closure=closures(c,call)?;
        let key=sha(format!("{}|{:?}|{}",plan.identity,closure,lean.version).as_bytes());
        let dir=out.join("composed").join(format!("expr-{}",&key[7..23]));
        fs::create_dir_all(&dir)?;
        let callee_src=dir.join("CalleeModule.lean");
        let caller_src=dir.join("CallerModule.lean");
        let callee_obj=dir.join("CalleeModule.olean");
        let caller_obj=dir.join("CallerModule.olean");
        fs::write(&callee_src,&plan.callee)?;
        fs::write(&caller_src,&plan.caller)?;
        let checked=(||->Result<()>{
            compile_import_module(lean,&callee_src,&callee_obj,&dir,
                Duration::from_secs(120)).context("expression callee proof")?;
            compile_import_module(lean,&caller_src,&caller_obj,&dir,
                Duration::from_secs(120)).context("expression caller composition")?;
            Ok(())
        })();
        if let Err(e)=checked{
            let _=fs::remove_file(dir.join("composition-certificate.json"));
            results.push(json!({"identity":plan.identity,"status":"lean_rejected",
                "scope":SCOPE,"error":format!("{e:#}")}));
            continue
        }
        let cert=json!({
            "schema_version":1,"kind":KIND,"scope":SCOPE,
            "status":"lean_kernel_proved_model",
            "caller":call.caller.as_str(),"callee":call.target.as_str(),
            "identity":plan.identity,"toolchain":lean.version,
            "source_closures":closure.iter().map(|(id,digest)|
                json!({"id":id,"closure":digest})).collect::<Vec<_>>(),
            "callee_source_sha256":sha(plan.callee.as_bytes()),
            "caller_source_sha256":sha(plan.caller.as_bytes()),
            "callee_compiled_sha256":sha(&fs::read(&callee_obj)?),
            "caller_compiled_sha256":sha(&fs::read(&caller_obj)?),
            "full_marqdo_runtime_semantics_proved":false,
            "independent_replay":"CalleeModule.lean then CallerModule.lean"
        });
        let receipt=dir.join("composition-certificate.json");
        fs::write(&receipt,serde_json::to_vec_pretty(&cert)?)?;
        results.push(json!({"identity":plan.identity,"status":"lean_kernel_proved_model",
            "scope":SCOPE,"artifact":receipt}));
    }
    Ok(results)
}
pub(super) fn replay(c:&Compilation,certificate:&Path,lean:&Toolchain)
    ->Result<Value>{
    let cert:Value=serde_json::from_slice(&fs::read(certificate)?)?;
    anyhow::ensure!(cert["schema_version"]==1&&cert["kind"]==KIND
        &&cert["scope"]==SCOPE&&cert["status"]=="lean_kernel_proved_model",
        "proof.expression_invalid_certificate");
    anyhow::ensure!(cert["toolchain"]==lean.version,"proof.expression_toolchain_mismatch");
    let identity=cert["identity"].as_str().context("proof.expression_missing_identity")?;
    let candidates=c.semantics.bound_calls.iter().filter_map(|call|
        candidate(c,call).and_then(|plan|(plan.identity==identity).then_some((call,plan))))
        .collect::<Vec<_>>();
    anyhow::ensure!(candidates.len()==1,"proof.expression_ambiguous_or_stale_source");
    let (call,plan)=&candidates[0];
    anyhow::ensure!(cert["caller"]==call.caller.as_str()
        &&cert["callee"]==call.target.as_str(),"proof.expression_changed_symbols");
    let expected=closures(c,call)?;
    let rows=cert["source_closures"].as_array().context("proof.expression_missing_closures")?;
    let actual=rows.iter().map(|r|Ok((
        r["id"].as_str().context("proof.expression_bad_id")?.to_owned(),
        r["closure"].as_str().context("proof.expression_bad_digest")?.to_owned()
    ))).collect::<Result<BTreeSet<_>>>()?;
    anyhow::ensure!(rows.len()==expected.len()&&actual==expected,
        "proof.expression_stale_or_ambiguous_closure");
    let dir=certificate.parent().context("proof.expression_missing_parent")?;
    let callee_src=dir.join("CalleeModule.lean");
    let caller_src=dir.join("CallerModule.lean");
    anyhow::ensure!(fs::read_to_string(&callee_src)?==plan.callee
        &&fs::read_to_string(&caller_src)?==plan.caller
        &&cert["callee_source_sha256"]==sha(plan.callee.as_bytes())
        &&cert["caller_source_sha256"]==sha(plan.caller.as_bytes()),
        "proof.expression_source_mismatch");
    let callee_obj=dir.join("CalleeModule.olean");
    let caller_obj=dir.join("CallerModule.olean");
    compile_import_module(lean,&callee_src,&callee_obj,dir,Duration::from_secs(120))?;
    compile_import_module(lean,&caller_src,&caller_obj,dir,Duration::from_secs(120))?;
    anyhow::ensure!(cert["callee_compiled_sha256"]==sha(&fs::read(callee_obj)?)
        &&cert["caller_compiled_sha256"]==sha(&fs::read(caller_obj)?),
        "proof.expression_compiled_mismatch");
    Ok(json!({"schema_version":1,"kind":"independent_composition_replay",
        "status":"lean_kernel_replayed","scope":SCOPE,
        "identity":identity,"current_source_verified":true,
        "full_marqdo_runtime_semantics_proved":false}))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn arithmetic_contract_requires_exact_structural_match(){
        let params=vec![CanonicalParam{name:"n".into(),ty:"number".into(),
            required:true,default_digest:None,unit:None}];
        let lhs=crate::parse::parse_expr("`n` + 1").unwrap();
        let want=Term::source(&lhs,&params,0).unwrap();
        let good=proof_check::parse_claim_expr(
            &proof_check::tokenize_claim("n + 1").unwrap()).unwrap().1;
        assert_eq!(Term::claim(&good,&params,0),Some(want.clone()));
        let bad=proof_check::parse_claim_expr(
            &proof_check::tokenize_claim("n + 2").unwrap()).unwrap().1;
        assert_ne!(Term::claim(&bad,&params,0),Some(want));
        assert!(Term::source(&crate::parse::parse_expr("`n` / 2").unwrap(),&params,0).is_none());
    }
}

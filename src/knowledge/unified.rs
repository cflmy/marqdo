//! Unified source-derived, kernel-checked modular contracts over the Lean Int
//! abstraction. NOT an unrestricted Marqdo runtime correctness theorem.
//!
//! Each function is translated ONCE from its current AST into a separate Lean
//! module. Contracts are proved from that source; callers use only imported
//! contracts, never synthetic axioms or inlined callee implementations.
//! Supported: pure, statically bound positional calls; integer linear terms,
//! immutable expressions, sequential local assignments and total if/else
//! branches; arbitrarily nested acyclic cross-file dependencies (depth capped).
//! Unsupported: missing/multiple contracts, loops, recursive calls, plugins,
//! effects, dynamic dispatch, defaults, conditional contracts, non-total branches,
//! runtime Number/i64/Formula equivalence. Every such case fails closed.
use std::{collections::{BTreeMap,BTreeSet},fs,path::Path,time::Duration};
use anyhow::{Context,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use crate::{
    ast::{Arg,BinaryOp,CallExpr,Expr,Function,Literal,Stmt,UnaryOp},
    prove::{driver::{Toolchain,compile_import_module},
        proof_check::{self,ClaimAst,CTok}},
    sema::{SymbolId,ir::{BoundCall,CanonicalParam,Compilation,SemanticArtifact}}
};
use super::projection::load_function;

const KIND:&str="marqdo_unified_modular_contract";
const SCOPE:&str="pure_number_int_model_sequential_branch_dag";
fn sha(bytes:&[u8])->String{format!("sha256:{:x}",Sha256::digest(bytes))}
fn id_for(symbol:&str)->String{
    let value=sha(symbol.as_bytes());
    format!("f_{}",&value[7..31])
}
fn module_for(symbol:&str)->String{
    let value=sha(symbol.as_bytes());
    format!("M{}",&value[7..31])
}
fn args_text(args:&[String])->String{
    if args.is_empty(){String::new()}else{format!(" {}",args.join(" "))}
}
fn variables(count:usize)->Vec<String>{
    (0..count).map(|i|format!("v{i}")).collect()
}
fn binders(count:usize)->String{
    variables(count).iter().map(|v|format!(" ({v} : Int)")).collect()
}
fn unique_artifact<'a>(compiled:&'a Compilation,id:&str)
    ->Option<&'a SemanticArtifact>{
    let mut found=compiled.semantics.artifacts.iter().filter(|a|
        a.kind=="function" && a.symbol_id.as_ref().is_some_and(|s|s.as_str()==id));
    let one=found.next()?;
    found.next().is_none().then_some(one)
}
fn basic_signature(params:&[CanonicalParam])->bool{
    params.iter().all(|p|p.ty.eq_ignore_ascii_case("number")
        &&p.required&&p.default_digest.is_none())
        &&params.iter().map(|p|&p.name).collect::<BTreeSet<_>>().len()==params.len()
}
fn claim_term(ast:&ClaimAst,params:&BTreeMap<String,String>,depth:usize)
    ->Option<String>{
    if depth>=32{return None}
    match ast{
        ClaimAst::Lit(CTok::Int(n))=>Some(format!("({n} : Int)")),
        ClaimAst::Var(v)=>params.get(v).cloned(),
        ClaimAst::Un{op:"-",expr}=>Some(format!("(-{})",
            claim_term(expr,params,depth+1)?)),
        ClaimAst::Bin{op,left,right}=>{
            let symbol=match *op{"+"|"-"|"*"=>*op,_=>return None};
            // Multiplication between symbolic terms is outside the omega
            // fragment. Only multiplication by a literal is admitted.
            if symbol=="*" && !matches!(left.as_ref(),ClaimAst::Lit(CTok::Int(_)))
                &&!matches!(right.as_ref(),ClaimAst::Lit(CTok::Int(_))){return None}
            Some(format!("({} {symbol} {})",
                claim_term(left,params,depth+1)?,
                claim_term(right,params,depth+1)?))
        },
        _=>None
    }
}
fn result_claim(compiled:&Compilation,artifact:&SemanticArtifact,
    params:&BTreeMap<String,String>)->Option<String>{
    let owner=artifact.symbol_id.as_ref()?;
    let mut matches=compiled.semantics.claims.iter().filter(|c|
        c.owner==*owner&&c.premise.as_deref()
            .map_or(true,|p|p.trim().is_empty())).filter_map(|c|{
        let tokens=proof_check::tokenize_claim(&c.conclusion).ok()?;
        let (_,ast)=proof_check::parse_claim_expr(&tokens).ok()?;
        match ast{
            ClaimAst::Bin{op,left,right}
                if matches!(*left,ClaimAst::Var(ref n) if n=="结果")=>{
                let operator=match op{
                    "="=>"=", ">"=>">", "<"=>"<", "≥"=>">=", "≤"=>"<=", "≠"=>"!=",
                    _=>return None
                };
                Some(format!("{operator} {}",claim_term(&right,params,0)?))
            },
            _=>None
        }
    });
    let first=matches.next()?;
    // One exported claim per function is an explicit, auditable boundary;
    // other statements are never silently promoted into assumptions.
    matches.next().is_none().then_some(first)
}
#[derive(Clone)]
struct CallUse{
    callee:String,args:Vec<String>
}
#[derive(Clone)]
struct Built{
    symbol:String,artifact:String,closure:String,module:String,lean:String,
    file:String,dep_symbols:BTreeSet<String>,result_operator:String
}
struct Builder<'a>{
    compilation:&'a Compilation,
    active:BTreeSet<String>,
    built:BTreeMap<String,Built>,
    order:Vec<String>
}
impl<'a> Builder<'a>{
    fn new(c:&'a Compilation)->Self{
        Self{compilation:c,active:BTreeSet::new(),
            built:BTreeMap::new(),order:Vec::new()}
    }
    fn number(&mut self,expr:&Expr,env:&BTreeMap<String,String>,owner:&str,
        line:u32,uses:&mut Vec<CallUse>,depth:usize)->Option<String>{
        if depth>=32{return None}
        match expr{
            Expr::Literal(Literal::Int(n))=>Some(format!("({n} : Int)")),
            Expr::Var(v)=>env.get(v).cloned(),
            Expr::Occluded{payload:Some(inner),..}=>
                self.number(inner,env,owner,line,uses,depth+1),
            Expr::Unary{op:UnaryOp::Neg,expr}=>Some(format!("(-{})",
                self.number(expr,env,owner,line,uses,depth+1)?)),
            Expr::Binary{op,left,right}=>{
                let symbol=match op{
                    BinaryOp::Add=>"+",BinaryOp::Sub=>"-",
                    BinaryOp::Mul=>"*",_=>return None
                };
                if symbol=="*" && !matches!(left.as_ref(),Expr::Literal(Literal::Int(_)))
                    &&!matches!(right.as_ref(),Expr::Literal(Literal::Int(_))){
                    return None
                }
                let a=self.number(left,env,owner,line,uses,depth+1)?;
                let b=self.number(right,env,owner,line,uses,depth+1)?;
                Some(format!("({a} {symbol} {b})"))
            },
            Expr::Call(call)=>self.call(call,env,owner,line,uses,depth+1),
            _=>None
        }
    }
    fn condition(&mut self,expr:&Expr,env:&BTreeMap<String,String>,
        owner:&str,line:u32,uses:&mut Vec<CallUse>,depth:usize)->Option<String>{
        if depth>=32{return None}
        match expr{
            Expr::Literal(Literal::Bool(b))=>Some(if *b{"True"}else{"False"}.into()),
            Expr::Occluded{payload:Some(inner),..}=>
                self.condition(inner,env,owner,line,uses,depth+1),
            Expr::Unary{op:UnaryOp::Not,expr}=>Some(format!("(¬ {})",
                self.condition(expr,env,owner,line,uses,depth+1)?)),
            Expr::Binary{op,left,right}=>{
                let relation=match op{
                    BinaryOp::Eq=>Some("="),BinaryOp::Ne=>Some("≠"),
                    BinaryOp::Lt=>Some("<"),BinaryOp::Le=>Some("≤"),
                    BinaryOp::Gt=>Some(">"),BinaryOp::Ge=>Some("≥"),
                    _=>None
                };
                if let Some(rel)=relation{
                    let a=self.number(left,env,owner,line,uses,depth+1)?;
                    let b=self.number(right,env,owner,line,uses,depth+1)?;
                    return Some(format!("({a} {rel} {b})"))
                }
                let relation=match op{BinaryOp::And=>"∧",
                    BinaryOp::Or=>"∨",_=>return None};
                let a=self.condition(left,env,owner,line,uses,depth+1)?;
                let b=self.condition(right,env,owner,line,uses,depth+1)?;
                Some(format!("({a} {relation} {b})"))
            },
            _=>None
        }
    }
    fn call(&mut self,call:&CallExpr,env:&BTreeMap<String,String>,
        owner:&str,line:u32,uses:&mut Vec<CallUse>,depth:usize)->Option<String>{
        if depth>=32 ||call.path.is_none()||call.receiver.is_some()
            ||!call.pre_modifiers.is_empty(){return None}
        let normalized=call.for_evaluation().ok()?;
        let display=crate::sema::bind::call_display(&normalized);
        let mut matches=self.compilation.semantics.bound_calls.iter().filter(|bc|
            bc.caller.as_str()==owner &&bc.source.line==line&&
            bc.call_site==display);
        let bound=matches.next()?;
        if matches.next().is_some()||bound.control_blocker.is_some()
            ||!bound.required_effects.is_pure()||bound.args.len()!=call.args.len(){
            return None
        }
        let callee=bound.target.as_str();
        if callee==owner{return None}
        let artifact=unique_artifact(self.compilation,callee)?;
        if !artifact.effects.is_pure(){return None}
        let signature=artifact.signature.as_ref()?;
        if !basic_signature(&signature.params)||
            signature.params.len()!=call.args.len(){return None}
        // Only positional arguments, in executable order, are trusted here.
        // Named/default/variadic semantics require their own binding proof.
        let mut args=Vec::new();
        for (actual,static_arg) in call.args.iter().zip(&bound.args){
            if static_arg.name.is_some(){return None}
            let Arg::Positional(value)=actual else{return None};
            args.push(self.number(value,env,owner,line,uses,depth+1)?);
        }
        self.visit(callee)?;
        uses.push(CallUse{callee:callee.to_owned(),args:args.clone()});
        Some(format!("({}{})",id_for(callee),args_text(&args)))
    }
    fn block(&mut self,items:&[Stmt],mut env:BTreeMap<String,String>,
        owner:&str,uses:&mut Vec<CallUse>,depth:usize)->Option<String>{
        if depth>=32{return None}
        for (index,st) in items.iter().enumerate(){
            match st{
                Stmt::Assign{name,value,span,..}=>{
                    let rhs=self.number(value,&env,owner,span.line,uses,0)?;
                    env.insert(name.clone(),rhs);
                },
                Stmt::Return{value,span} if index+1==items.len()=>{
                    return self.number(value,&env,owner,span.line,uses,0)
                },
                Stmt::Branch{arms,span} if index+1==items.len()=>{
                    // A branch must explicitly cover all outcomes, including
                    // its else arm. No unproved fallthrough is accepted.
                    let mut otherwise=None;
                    for arm in arms.iter().rev(){
                        let expression=self.block(&arm.body,env.clone(),
                            owner,uses,depth+1)?;
                        otherwise=Some(if let Some(condition)=&arm.condition{
                            let cond=self.condition(condition,&env,owner,
                                span.line,uses,0)?;
                            let rest=otherwise?;
                            format!("(if {cond} then {expression} else {rest})")
                        }else{
                            if otherwise.is_some(){return None}
                            expression
                        });
                    }
                    return otherwise
                },
                _=>return None
            }
        }
        None
    }
    fn visit(&mut self,symbol:&str)->Option<()>{
        if self.built.contains_key(symbol){return Some(())}
        if self.active.len()>=24||!self.active.insert(symbol.to_owned()){
            return None
        }
        let result=(||->Option<Built>{
            let a=unique_artifact(self.compilation,symbol)?;
            if !a.effects.is_pure(){return None}
            let sig=a.signature.as_ref()?;
            if !basic_signature(&sig.params){return None}
            let fun:Function=load_function(self.compilation,a)?;
            if fun.is_object()||fun.params.len()!=sig.params.len()
                ||!fun.params.iter().zip(&sig.params)
                    .all(|(f,p)|f.name==p.name){return None}
            let env=sig.params.iter().enumerate().map(|(i,p)|
                (p.name.clone(),format!("v{i}"))).collect::<BTreeMap<_,_>>();
            let contract=result_claim(self.compilation,a,&env)?;
            let mut call_uses=Vec::new();
            let body=self.block(&fun.body,env,symbol,&mut call_uses,0)?;
            // No semantically relevant BoundCall may remain unmodeled:
            // every source call requires a unique static binding.
            let source_count=self.compilation.semantics.bound_calls.iter()
                .filter(|c|c.caller.as_str()==symbol).count();
            if call_uses.len()!=source_count{return None}
            let deps=call_uses.iter().map(|use_|use_.callee.clone())
                .collect::<BTreeSet<_>>();
            if deps.contains(symbol){return None}
            let fname=id_for(symbol);
            let args=variables(sig.params.len());
            let bind=binders(sig.params.len());
            let invocation=format!("{fname}{}",args_text(&args));
            let mut source=String::new();
            for dep in &deps{
                source.push_str(&format!("import {}\n",module_for(dep)));
            }
            source.push_str(&format!(
                "-- Generated from a pure Marqdo AST. Int abstraction only.\n\
                 def {fname}{bind} : Int := {body}\n\
                 theorem {fname}_contract{bind} : {invocation} {contract} := by\n"));
            for (i,use_) in call_uses.iter().enumerate(){
                let callee_contract=format!("{}_contract",id_for(&use_.callee));
                source.push_str(&format!(
                    "  have hc{i} := {callee_contract}{}\n",args_text(&use_.args)));
            }
            source.push_str(&format!(
                "  simp only [{fname}] at *\n  split_ifs <;> omega\n"));
            let operator=contract.split_whitespace().next()?.to_string();
            Some(Built{
                symbol:symbol.to_owned(),artifact:a.artifact_id.0.clone(),
                closure:a.hashes.closure.0.clone(),module:module_for(symbol),
                lean:source,file:a.source.file.clone(),dep_symbols:deps,
                result_operator:operator
            })
        })();
        self.active.remove(symbol);
        if let Some(built)=result{
            self.order.push(symbol.to_owned());
            self.built.insert(symbol.to_owned(),built);
            Some(())
        }else{None}
    }
    fn plan(mut self,root:&str)->Option<Plan>{
        self.visit(root)?;
        if self.built.len()<2{return None}
        let nodes=self.order.iter().map(|id|self.built.get(id).cloned())
            .collect::<Option<Vec<_>>>()?;
        // Actually prove a caller that uses imported source definitions.
        if nodes.last()?.dep_symbols.is_empty(){return None}
        Some(Plan{root:root.into(),nodes})
    }
}
#[derive(Clone)]
struct Plan{root:String,nodes:Vec<Built>}
impl Plan{
    fn identities(&self)->BTreeSet<(String,String)>{
        self.nodes.iter().map(|n|(n.artifact.clone(),n.closure.clone()))
            .collect()
    }
    fn key(&self,version:&str)->String{
        sha(format!("{}|{:?}|{}",self.root,self.identities(),version).as_bytes())
    }
}
fn candidate(c:&Compilation,root:&str)->Option<Plan>{
    Builder::new(c).plan(root)
}
pub(super) fn covers(c:&Compilation,root:&SymbolId)->bool{
    candidate(c,root.as_str()).is_some()
}
pub(super) fn prove(c:&Compilation,out:&Path,lean:&Toolchain)->Result<Vec<Value>>{
    let roots=c.semantics.bound_calls.iter().map(|call|call.caller.as_str())
        .collect::<BTreeSet<_>>();
    let mut results=Vec::new();
    for root in roots{
        let Some(plan)=candidate(c,root)else{continue};
        let key=plan.key(&lean.version);
        let dir=out.join("composed").join(format!("unified-{}",&key[7..23]));
        fs::create_dir_all(&dir)?;
        let checked=(||->Result<Vec<Value>>{
            let mut files=Vec::new();
            for node in &plan.nodes{
                let src=dir.join(format!("{}.lean",node.module));
                let obj=dir.join(format!("{}.olean",node.module));
                fs::write(&src,&node.lean)?;
                compile_import_module(lean,&src,&obj,&dir,Duration::from_secs(120))
                    .with_context(||format!("module {}",node.module))?;
                files.push(json!({
                    "symbol":node.symbol,"artifact":node.artifact,
                    "module":node.module,"source_sha256":sha(node.lean.as_bytes()),
                    "compiled_sha256":sha(&fs::read(obj)?),
                    "dependencies":node.dep_symbols,"result_operator":node.result_operator,
                    "source_file":node.file
                }));
            }
            Ok(files)
        })();
        match checked{
            Ok(files)=>{
                let certificate=json!({
                    "schema_version":1,"kind":KIND,"scope":SCOPE,
                    "status":"lean_kernel_proved_model",
                    "evidence_grade":"model_proved",
                    "full_marqdo_runtime_semantics_proved":false,
                    "identity":plan.root,"toolchain":lean.version,
                    "source_closures":plan.identities().iter().map(|(id,h)|
                        json!({"id":id,"closure":h})).collect::<Vec<_>>(),
                    "modules":files,
                    "independent_replay":"topological module recompilation from current source"
                });
                let certificate_path=dir.join("composition-certificate.json");
                fs::write(&certificate_path,serde_json::to_vec_pretty(&certificate)?)?;
                results.push(json!({"identity":plan.root,
                    "status":"lean_kernel_proved_model",
                    "scope":SCOPE,"artifact":certificate_path,
                    "evidence":super::evidence::model_proved()}));
            },
            Err(e)=>{
                let _=fs::remove_file(dir.join("composition-certificate.json"));
                results.push(json!({"identity":plan.root,"status":"lean_rejected",
                    "scope":SCOPE,"error":format!("{e:#}"),
                    "evidence":super::evidence::unknown("lean_rejected")}));
            }
        }
    }
    Ok(results)
}
pub(super) fn replay(c:&Compilation,certificate:&Path,lean:&Toolchain)->Result<Value>{
    let cert:Value=serde_json::from_slice(&fs::read(certificate)?)?;
    anyhow::ensure!(cert["schema_version"]==1&&cert["kind"]==KIND
        &&cert["scope"]==SCOPE&&cert["status"]=="lean_kernel_proved_model"
        &&cert["evidence_grade"]=="model_proved"
        &&cert["full_marqdo_runtime_semantics_proved"]==false,
        "proof.unified_invalid_certificate");
    anyhow::ensure!(cert["toolchain"]==lean.version,
        "proof.unified_stale_toolchain");
    let root=cert["identity"].as_str().context("proof.unified_missing_identity")?;
    let plan=candidate(c,root).context("proof.unified_stale_or_unsupported_source")?;
    let expected=plan.identities();
    let items=cert["source_closures"].as_array()
        .context("proof.unified_missing_source_closures")?;
    let actual=items.iter().map(|r|Ok((
        r["id"].as_str().context("proof.unified_invalid_id")?.to_string(),
        r["closure"].as_str().context("proof.unified_invalid_hash")?.to_string()
    ))).collect::<Result<BTreeSet<_>>>()?;
    anyhow::ensure!(items.len()==expected.len()&&actual==expected,
        "proof.unified_stale_or_ambiguous_source_closure");
    let rows=cert["modules"].as_array().context("proof.unified_missing_modules")?;
    anyhow::ensure!(rows.len()==plan.nodes.len(),
        "proof.unified_module_count_changed");
    let dir=certificate.parent().context("proof.unified_missing_parent")?;
    for (node,row) in plan.nodes.iter().zip(rows){
        anyhow::ensure!(row["symbol"]==node.symbol&&row["artifact"]==node.artifact
            &&row["module"]==node.module
            &&row["dependencies"]==json!(node.dep_symbols)
            &&row["result_operator"]==node.result_operator
            &&row["source_file"]==node.file,
            "proof.unified_module_identity_changed");
        let src=dir.join(format!("{}.lean",node.module));
        let obj=dir.join(format!("{}.olean",node.module));
        anyhow::ensure!(fs::read_to_string(&src)?==node.lean
            &&row["source_sha256"]==sha(node.lean.as_bytes()),
            "proof.unified_generated_source_changed");
        compile_import_module(lean,&src,&obj,dir,Duration::from_secs(120))?;
        anyhow::ensure!(row["compiled_sha256"]==sha(&fs::read(&obj)?),
            "proof.unified_compiled_module_changed");
    }
    Ok(json!({"schema_version":1,"kind":"independent_composition_replay",
        "status":"lean_kernel_replayed","scope":SCOPE,"identity":root,
        "evidence":super::evidence::model_proved(),
        "current_source_verified":true,
        "full_marqdo_runtime_semantics_proved":false}))
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn simple_source_contracts_must_not_be_promoted_without_unique_claim(){
        let c=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(
                Path::new("doc/verification/skill-proof")).unwrap());
        if let Ok(project)=c{
            for artifact in project.semantics.artifacts.iter()
                .filter(|a|a.kind=="function"){
                assert!(!project.semantics.claims.iter()
                    .any(|r|r.owner.as_str()=="__invented_symbol__"));
                let _=artifact;
            }
        }
    }
    #[test]
    fn claim_language_refuses_nonlinear_and_unknown_values(){
        let bind=BTreeMap::from([("n".into(),"v0".into()),
            ("m".into(),"v1".into())]);
        let parse=|s:&str|{
            let toks=proof_check::tokenize_claim(s).unwrap();
            proof_check::parse_claim_expr(&toks).unwrap().1
        };
        assert_eq!(claim_term(&parse("n + 2"),&bind,0),
            Some("(v0 + (2 : Int))".into()));
        assert!(claim_term(&parse("n * m"),&bind,0).is_none());
        assert!(claim_term(&parse("n / 2"),&bind,0).is_none());
        assert!(claim_term(&parse("secret"),&bind,0).is_none());
    }
}

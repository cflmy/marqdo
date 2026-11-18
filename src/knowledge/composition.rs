//! Reusable, independently replayable **restricted** module composition.
//!
//! Scope: pure typed-int forwarding caller and source-constant callee. Two
//! independently compiled Lean modules: caller imports the callee's theorem,
//! not its implementation. Every unmodeled construct fails closed.
//! This is not a theorem about arbitrary Marqdo programs.
use std::{fs,path::Path,time::Duration};
use anyhow::{Context,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use crate::{
    ast::{Arg,CallExpr,Expr,Function,Literal,Stmt},
    prove::driver::{Toolchain,compile_import_module},
    sema::ir::{Compilation,SemanticArtifact,BoundCall}
};
fn sha(input:&[u8])->String{format!("sha256:{:x}",Sha256::digest(input))}
fn lookup<'a>(items:&'a [Function],path:&[&str])->Option<&'a Function>{
    let (first,rest)=path.split_first()?;
    let fun=items.iter().find(|f|f.name==*first)?;
    if rest.is_empty(){Some(fun)}else{lookup(&fun.children,rest)}
}
fn function(compiled:&Compilation,a:&SemanticArtifact)->Option<Function>{
    let raw=fs::read_to_string(compiled.semantics.root.join(&a.source.file)).ok()?;
    let doc=crate::parse::parse_source(&raw).ok()?;
    let path=a.display_name.split('.').collect::<Vec<_>>();
    lookup(&doc.functions,&path).cloned()
}
fn call_inside(expr:&Expr)->Option<&CallExpr>{
    match expr{
        Expr::Call(c)=>Some(c),
        Expr::Occluded{payload:Some(inner),..}=>call_inside(inner),
        _=>None
    }
}
fn forwarded(fun:&Function)->Option<&CallExpr>{
    match fun.body.as_slice(){
        [Stmt::Return{value,..}]=>call_inside(value),
        [Stmt::Assign{name,value,..},Stmt::Return{value:Expr::Var(v),..}]
            if name==v=>call_inside(value),
        _=>None
    }
}
fn constant_return(fun:&Function)->Option<i64>{
    match fun.body.as_slice(){
        [Stmt::Return{value:Expr::Literal(Literal::Int(n)),..}]=>Some(*n),
        [Stmt::Return{value:Expr::Occluded{payload:Some(v),..},..}] =>
            if let Expr::Literal(Literal::Int(n))=v.as_ref(){Some(*n)}else{None},
        _=>None
    }
}
fn claim_is_constant(compiled:&Compilation,owner:&SemanticArtifact,n:i64)->bool{
    owner.symbol_id.as_ref().is_some_and(|id|
        compiled.semantics.claims.iter().any(|c|
            c.owner==*id
            && c.premise.as_deref().is_none_or(|p|p.trim().is_empty())
            && c.conclusion.trim()==format!("结果 = {n}")))
}
fn candidate(compiled:&Compilation,call:&BoundCall)
  ->Option<(String,String,String,i64)>{
    let ca=compiled.semantics.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.caller) && a.kind=="function")?;
    let ce=compiled.semantics.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.target) && a.kind=="function")?;
    if !ca.effects.is_pure()||!ce.effects.is_pure()
        ||!call.required_effects.is_pure()
        ||call.control_blocker.is_some()||!call.path_guards.is_empty(){return None}
    let caller=function(compiled,ca)?;
    let callee=function(compiled,ce)?;
    let forwarded=forwarded(&caller)?;
    let runtime=forwarded.for_evaluation().ok()?;
    if runtime.path.as_ref().is_none()||runtime.receiver.is_some()
        ||caller.is_object()||callee.is_object(){return None}
    let n=constant_return(&callee)?;
    if !claim_is_constant(compiled,ca,n)||!claim_is_constant(compiled,ce,n){
        return None
    }
    let cs=ca.signature.as_ref()?;
    let es=ce.signature.as_ref()?;
    if cs.params.len()!=caller.params.len()
        ||es.params.len()!=callee.params.len()
        ||es.params.len()!=call.args.len()
        ||call.args.len()!=runtime.args.len(){return None}
    if cs.params.iter().chain(es.params.iter())
        .any(|p|!p.ty.eq_ignore_ascii_case("int")||!p.required){return None}
    let arg_expr=call.args.iter().zip(es.params.iter()).map(|(a,p)|{
        if a.name.as_ref().is_some_and(|x|x!=&p.name){return None}
        if let Some(v)=a.integer {return Some(format!("({v} : Int)"))}
        let symbol=a.symbol.as_ref()?;
        let position=cs.params.iter().position(|x|x.name==*symbol)?;
        Some(format!("v{position}"))
    }).collect::<Option<Vec<_>>>()?.join(" ");
    let callee_binders=(0..es.params.len()).map(|i|format!("(v{i} : Int)"))
        .collect::<Vec<_>>().join(" ");
    let caller_binders=(0..cs.params.len()).map(|i|format!("(v{i} : Int)"))
        .collect::<Vec<_>>().join(" ");
    let callee_args=(0..es.params.len()).map(|i|format!("v{i}"))
        .collect::<Vec<_>>().join(" ");
    let caller_args=(0..cs.params.len()).map(|i|format!("v{i}"))
        .collect::<Vec<_>>().join(" ");
    // This callee theorem is verified from the translated source *inside its
    // own module*. The caller only imports its proved interface theorem.
    let callee_src=format!(
        "-- pure typed-int fragment of {}\\n\
         def callee {callee_binders} : Int := ({n} : Int)\\n\
         theorem callee_contract {callee_binders} : callee {callee_args} = ({n} : Int) := by rfl\\n",
        ce.artifact_id.0);
    let caller_src=format!(
        "import CalleeModule\\n\
         -- exact forwarded import call from {}\\n\
         def caller {caller_binders} : Int := callee {arg_expr}\\n\
         theorem caller_contract {caller_binders} : caller {caller_args} = ({n} : Int) := by\\n\
         \\x20 simpa [caller] using (callee_contract {arg_expr})\\n",
        ca.artifact_id.0);
    Some((callee_src,caller_src,format!("{}#{}",ca.artifact_id.0,
        ce.artifact_id.0),n))
}
/// Emit one sealed bundle per eligible call. An unsuccessful Lean command
/// never creates a certificate; the JSON report carries the failure.
pub fn prove(compiled:&Compilation,out:&Path,toolchain:&Toolchain)->Result<Value>{
    let mut records=Vec::new();
    for call in &compiled.semantics.bound_calls{
        let Some((source,client,identity,n))=candidate(compiled,call) else {
            records.push(json!({"caller":call.caller.as_str(),
                "callee":call.target.as_str(),"status":"unsupported_fragment",
                "reason":"requires_pure_typed_int_forwarder_with_constant_claims"}));
            continue;
        };
        let key=sha(format!("{}|{}|{}|{}",identity,
            compiled.semantics.artifacts.iter().find(|a|
                a.symbol_id.as_ref()==Some(&call.caller))
                .map(|a|a.hashes.closure.0.as_str()).unwrap_or(""),
            compiled.semantics.artifacts.iter().find(|a|
                a.symbol_id.as_ref()==Some(&call.target))
                .map(|a|a.hashes.closure.0.as_str()).unwrap_or(""),
            toolchain.version).as_bytes());
        let folder=out.join("composed").join(&key[7..23]);
        fs::create_dir_all(&folder)?;
        let callee=folder.join("CalleeModule.lean");
        let caller=folder.join("CallerModule.lean");
        let built=folder.join("CalleeModule.olean");
        fs::write(&callee,&source)?;
        fs::write(&caller,&client)?;
        let run=(|| ->Result<()>{
            compile_import_module(toolchain,&callee,&built,&folder,
                Duration::from_secs(120)).context("callee interface")?;
            let verified=folder.join("CallerModule.olean");
            compile_import_module(toolchain,&caller,&verified,&folder,
                Duration::from_secs(120)).context("caller composition")?;
            Ok(())
        })();
        if let Err(e)=run{
            let _=fs::remove_file(folder.join("composition-certificate.json"));
            records.push(json!({"identity":identity,
                "status":"lean_rejected","error":format!("{e:#}")}));
            continue
        }
        let cert=json!({
            "schema_version":1,"kind":"marqdo_restricted_composition",
            "scope":"typed_int_pure_forwarder_constant_postcondition",
            "status":"lean_kernel_proved_model",
            "identity":identity,"caller":call.caller.as_str(),
            "callee":call.target.as_str(),
            "claimed_result":n,
            "callee_source_sha256":sha(source.as_bytes()),
            "caller_source_sha256":sha(client.as_bytes()),
            "callee_compiled_sha256":sha(&fs::read(&built)?),
            "caller_compiled_sha256":sha(&fs::read(folder.join("CallerModule.olean"))?),
            "toolchain":toolchain.version,
            "source_closures":compiled.semantics.artifacts.iter()
                .filter(|a|a.symbol_id.as_ref()==Some(&call.caller)
                    ||a.symbol_id.as_ref()==Some(&call.target))
                .map(|a|json!({"id":a.artifact_id.0,
                    "closure":a.hashes.closure.0})).collect::<Vec<_>>(),
            "independent_replay":"CalleeModule.lean then CallerModule.lean"
        });
        let receipt=folder.join("composition-certificate.json");
        fs::write(&receipt,serde_json::to_vec_pretty(&cert)?)?;
        records.push(json!({"identity":identity,
            "status":"lean_kernel_proved_model","artifact":receipt,
            "scope":"typed_int_pure_forwarder_constant_postcondition"}));
    }
    Ok(json!({"schema_version":1,"role":"restricted_composed_lean_module",
        "results":records,
        "full_marqdo_runtime_semantics_proved":false}))
}

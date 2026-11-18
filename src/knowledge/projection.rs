//! Cross-module projection contracts for the *abstract* Lean Int model.
//!
//! This is a source-regenerated, kernel-replayed proof: the callee returns
//! exactly one of its own arguments and the caller forwards one static import.
//! No arithmetic, dynamic dispatch, effects, exceptions or runtime Number
//! equivalence is inferred. Unmodeled programs never get a certificate.
use std::{collections::BTreeSet,fs,path::Path,time::Duration};
use anyhow::{Context,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use crate::{
    ast::{CallExpr,Expr,Function,Stmt},
    prove::driver::{Toolchain,compile_import_module},
    sema::ir::{BoundCall,Compilation,SemanticArtifact}
};

const SCOPE:&str="number_contract_int_model_pure_forwarder_parameter_projection";
const KIND:&str="marqdo_parameter_projection_composition";
fn sha(bytes:&[u8])->String{format!("sha256:{:x}",Sha256::digest(bytes))}

struct Prepared {
    callee_source:String,
    caller_source:String,
    identity:String,
    result_label:String,
    parameter_index:usize,
}
fn lookup<'a>(functions:&'a [Function],segments:&[&str])->Option<&'a Function>{
    let (head,tail)=segments.split_first()?;
    let mut candidates=functions.iter().filter(|f|f.name==*head);
    let found=candidates.next()?;
    if candidates.next().is_some(){return None}
    if tail.is_empty(){Some(found)}else{lookup(&found.children,tail)}
}
pub(super) fn load_function(compiled:&Compilation,artifact:&SemanticArtifact)->Option<Function>{
    let text=fs::read_to_string(compiled.semantics.root.join(&artifact.source.file)).ok()?;
    let parsed=crate::parse::parse_source(&text).ok()?;
    lookup(&parsed.functions,&artifact.display_name.split('.').collect::<Vec<_>>()).cloned()
}
fn unwrap_var(expr:&Expr)->Option<&str>{
    match expr{
        Expr::Var(v)=>Some(v.as_str()),
        Expr::Occluded{payload:Some(inner),..}=>unwrap_var(inner),
        _=>None
    }
}
fn unwrap_call(expr:&Expr)->Option<&CallExpr>{
    match expr{
        Expr::Call(c)=>Some(c),
        Expr::Occluded{payload:Some(inner),..}=>unwrap_call(inner),
        _=>None
    }
}
fn projection(fun:&Function)->Option<&str>{
    match fun.body.as_slice(){
        [Stmt::Return{value,..}]=>unwrap_var(value),
        _=>None
    }
}
pub(super) fn forwarder(fun:&Function)->Option<&CallExpr>{
    match fun.body.as_slice(){
        [Stmt::Return{value,..}]=>unwrap_call(value),
        [Stmt::Assign{name,value,..},Stmt::Return{value:Expr::Var(result),..}]
            if name==result=>unwrap_call(value),
        _=>None
    }
}
fn exact_unconditional_claim(compiled:&Compilation,artifact:&SemanticArtifact,
    expected:&str)->bool{
    artifact.symbol_id.as_ref().is_some_and(|owner|
        compiled.semantics.claims.iter().filter(|c|
            c.owner==*owner &&
            c.premise.as_deref().map_or(true,|p|p.trim().is_empty()) &&
            c.conclusion.trim()==expected
        ).count()==1)
}
fn candidate(compiled:&Compilation,call:&BoundCall)->Option<Prepared>{
    let sem=&compiled.semantics;
    let mut caller_artifacts=sem.artifacts.iter().filter(|a|
        a.kind=="function"&&a.symbol_id.as_ref()==Some(&call.caller));
    let ca=caller_artifacts.next()?;
    if caller_artifacts.next().is_some(){return None}
    let mut callee_artifacts=sem.artifacts.iter().filter(|a|
        a.kind=="function"&&a.symbol_id.as_ref()==Some(&call.target));
    let ce=callee_artifacts.next()?;
    if callee_artifacts.next().is_some(){return None}
    if ca.source.file==ce.source.file||call.caller==call.target||
        !ca.effects.is_pure()||!ce.effects.is_pure()||
        !call.required_effects.is_pure()||
        call.control_blocker.is_some()||!call.path_guards.is_empty(){
        return None
    }
    let caller=load_function(compiled,ca)?;
    let callee=load_function(compiled,ce)?;
    if caller.is_object()||callee.is_object(){return None}
    let return_parameter=projection(&callee)?;
    let forwarded=forwarder(&caller)?.for_evaluation().ok()?;
    if forwarded.path.is_none()||forwarded.receiver.is_some()||
        crate::sema::bind::call_display(&forwarded)!=call.call_site ||
        sem.bound_calls.iter().filter(|c|c.caller==call.caller).count()!=1{
        return None
    }
    let cs=ca.signature.as_ref()?;
    let es=ce.signature.as_ref()?;
    if cs.params.len()!=caller.params.len()||
        es.params.len()!=callee.params.len()||
        es.params.len()!=call.args.len()||
        forwarded.args.len()!=call.args.len(){return None}
    // Canonical "number" includes non-integer variants at runtime.
    // Our result is exclusively about the mathematical Int abstraction.
    if cs.params.iter().chain(es.params.iter())
        .any(|p|!p.ty.eq_ignore_ascii_case("number")||!p.required||
            p.default_digest.is_some()){return None}
    if !cs.params.iter().zip(caller.params.iter())
        .all(|(a,b)|a.name==b.name) ||
       !es.params.iter().zip(callee.params.iter())
        .all(|(a,b)|a.name==b.name){return None}
    let k=es.params.iter().position(|p|p.name==return_parameter)?;
    if es.params.iter().filter(|p|p.name==return_parameter).count()!=1{
        return None
    }
    if !exact_unconditional_claim(compiled,ce,
        &format!("结果 = {return_parameter}")){return None}
    let mut args=Vec::new();
    let mut caller_result=None;
    for (i,(formal,arg)) in es.params.iter().zip(call.args.iter()).enumerate(){
        if arg.name.as_ref().is_some_and(|name|name!=&formal.name){return None}
        let (term,label)=if let Some(value)=arg.integer{
            (format!("({value} : Int)"),value.to_string())
        }else{
            let symbol=arg.symbol.as_ref()?;
            let index=cs.params.iter().position(|p|p.name==*symbol)?;
            if cs.params.iter().filter(|p|p.name==*symbol).count()!=1{return None}
            (format!("v{index}"),symbol.clone())
        };
        args.push(term);
        if i==k{caller_result=Some(label)}
    }
    let result_label=caller_result?;
    if !exact_unconditional_claim(compiled,ca,
        &format!("结果 = {result_label}")){return None}
    let callee_binders=(0..es.params.len())
        .map(|i|format!("(v{i} : Int)")).collect::<Vec<_>>().join(" ");
    let caller_binders=(0..cs.params.len())
        .map(|i|format!("(v{i} : Int)")).collect::<Vec<_>>().join(" ");
    let callee_vars=(0..es.params.len())
        .map(|i|format!("v{i}")).collect::<Vec<_>>().join(" ");
    let caller_vars=(0..cs.params.len())
        .map(|i|format!("v{i}")).collect::<Vec<_>>().join(" ");
    let call_terms=args.join(" ");
    let projected_term=&args[k];
    // No author-controlled identifiers appear in generated Lean code.
    let callee_source=format!(
        "def callee {callee_binders} : Int := v{k}\n\
         theorem callee_contract {callee_binders} : callee {callee_vars} = v{k} := by rfl\n");
    let caller_source=format!(
        "import CalleeModule\n\
         def caller {caller_binders} : Int := callee {call_terms}\n\
         theorem caller_contract {caller_binders} : caller {caller_vars} = {projected_term} := by\n\
         \x20\x20simpa [caller] using (callee_contract {call_terms})\n");
    Some(Prepared{
        callee_source,caller_source,
        identity:format!("{}#{}",ca.artifact_id.0,ce.artifact_id.0),
        result_label,parameter_index:k
    })
}
pub(super) fn eligible(compiled:&Compilation,call:&BoundCall)->bool{
    candidate(compiled,call).is_some()
}
pub(super) fn closures(compiled:&Compilation,call:&BoundCall)->Result<BTreeSet<(String,String)>>{
    let set=compiled.semantics.artifacts.iter()
        .filter(|a|a.symbol_id.as_ref()==Some(&call.caller)||
                    a.symbol_id.as_ref()==Some(&call.target))
        .map(|a|(a.artifact_id.0.clone(),a.hashes.closure.0.clone()))
        .collect::<BTreeSet<_>>();
    anyhow::ensure!(set.len()==2,"proof.projection_ambiguous_dependency");
    Ok(set)
}
pub(super) fn prove(compiled:&Compilation,out:&Path,lean:&Toolchain)->Result<Vec<Value>>{
    let mut rows=Vec::new();
    for call in &compiled.semantics.bound_calls{
        let Some(plan)=candidate(compiled,call) else{continue};
        let dependencies=closures(compiled,call)?;
        let key=sha(format!("{}|{:?}|{}",plan.identity,dependencies,lean.version).as_bytes());
        let dir=out.join("composed").join(format!("projection-{}",&key[7..23]));
        fs::create_dir_all(&dir)?;
        let callee=dir.join("CalleeModule.lean");
        let caller=dir.join("CallerModule.lean");
        let callee_obj=dir.join("CalleeModule.olean");
        let caller_obj=dir.join("CallerModule.olean");
        fs::write(&callee,&plan.callee_source)?;
        fs::write(&caller,&plan.caller_source)?;
        let checked=(||->Result<()>{
            compile_import_module(lean,&callee,&callee_obj,&dir,
                Duration::from_secs(120)).context("projection callee")?;
            compile_import_module(lean,&caller,&caller_obj,&dir,
                Duration::from_secs(120)).context("projection caller")?;
            Ok(())
        })();
        if let Err(e)=checked{
            let _=fs::remove_file(dir.join("composition-certificate.json"));
            rows.push(json!({"identity":plan.identity,"status":"lean_rejected",
                "scope":SCOPE,"error":format!("{e:#}")}));
            continue
        }
        let cert=json!({
            "schema_version":1,"kind":KIND,"scope":SCOPE,
            "status":"lean_kernel_proved_model",
            "identity":plan.identity,
            "caller":call.caller.as_str(),"callee":call.target.as_str(),
            "result_label":plan.result_label,
            "parameter_index":plan.parameter_index,
            "callee_source_sha256":sha(plan.callee_source.as_bytes()),
            "caller_source_sha256":sha(plan.caller_source.as_bytes()),
            "callee_compiled_sha256":sha(&fs::read(&callee_obj)?),
            "caller_compiled_sha256":sha(&fs::read(&caller_obj)?),
            "toolchain":lean.version,
            "source_closures":dependencies.iter().map(|(id,closure)|
                json!({"id":id,"closure":closure})).collect::<Vec<_>>(),
            "runtime_domain_verified":false,
            "independent_replay":"CalleeModule.lean then CallerModule.lean"
        });
        let receipt=dir.join("composition-certificate.json");
        fs::write(&receipt,serde_json::to_vec_pretty(&cert)?)?;
        rows.push(json!({"identity":plan.identity,"status":"lean_kernel_proved_model",
            "artifact":receipt,"scope":SCOPE}));
    }
    Ok(rows)
}
pub(super) fn replay(compiled:&Compilation,certificate:&Path,lean:&Toolchain)
    ->Result<Value>{
    let cert:Value=serde_json::from_slice(&fs::read(certificate)?)?;
    anyhow::ensure!(cert["schema_version"]==1&&cert["kind"]==KIND&&
        cert["scope"]==SCOPE&&cert["status"]=="lean_kernel_proved_model",
        "proof.projection_invalid_certificate");
    anyhow::ensure!(cert["toolchain"]==lean.version,
        "proof.projection_stale_toolchain");
    let identity=cert["identity"].as_str().context("missing identity")?;
    let matches=compiled.semantics.bound_calls.iter().filter_map(|call|
        candidate(compiled,call).and_then(|p|
            (p.identity==identity).then_some((call,p)))).collect::<Vec<_>>();
    anyhow::ensure!(matches.len()==1,
        "proof.projection_ambiguous_or_stale_source");
    let (call,plan)=&matches[0];
    anyhow::ensure!(cert["caller"]==call.caller.as_str()&&
        cert["callee"]==call.target.as_str()&&
        cert["parameter_index"]==plan.parameter_index&&
        cert["result_label"]==plan.result_label,
        "proof.projection_changed_proof_subject");
    let expected=closures(compiled,call)?;
    let rows=cert["source_closures"].as_array()
        .context("proof.projection_missing_closures")?;
    let actual=rows.iter().map(|r|Ok((
        r["id"].as_str().context("proof.projection_bad_id")?.to_string(),
        r["closure"].as_str().context("proof.projection_bad_closure")?.to_string()
    ))).collect::<Result<BTreeSet<_>>>()?;
    anyhow::ensure!(rows.len()==expected.len()&&actual==expected,
        "proof.projection_stale_or_ambiguous_closure");
    let dir=certificate.parent().context("missing certificate parent")?;
    let callee=dir.join("CalleeModule.lean");
    let caller=dir.join("CallerModule.lean");
    anyhow::ensure!(fs::read_to_string(&callee)?==plan.callee_source&&
        fs::read_to_string(&caller)?==plan.caller_source&&
        cert["callee_source_sha256"]==sha(plan.callee_source.as_bytes())&&
        cert["caller_source_sha256"]==sha(plan.caller_source.as_bytes()),
        "proof.projection_generated_source_mismatch");
    let callee_obj=dir.join("CalleeModule.olean");
    let caller_obj=dir.join("CallerModule.olean");
    compile_import_module(lean,&callee,&callee_obj,dir,
        Duration::from_secs(120))?;
    compile_import_module(lean,&caller,&caller_obj,dir,
        Duration::from_secs(120))?;
    anyhow::ensure!(cert["callee_compiled_sha256"]==sha(&fs::read(callee_obj)?)&&
        cert["caller_compiled_sha256"]==sha(&fs::read(caller_obj)?),
        "proof.projection_replay_output_changed");
    Ok(json!({"schema_version":1,"kind":"independent_composition_replay",
        "status":"lean_kernel_replayed","scope":SCOPE,"identity":identity,
        "current_source_verified":true,
        "full_marqdo_runtime_semantics_proved":false}))
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn symbolic_projection_uses_only_the_selected_actual(){
        let root=std::env::temp_dir().join(format!(
            "marqdo-projection-unit-{}",std::process::id()));
        let _=fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let callee=r#"---
markup: occlusion
---
# math
## select
+ `first`
+ `second`

| 参数 | 类型 | 说明 |
|---|---|---|
| first | int | integer |
| second | int | integer |

| 定理 | 结论 | 策略 |
|---|---|---|
| proj | 结果 = second | auto |

*`second`*
"#;
        let caller=r#"---
markup: occlusion
import:
  math: math.mq.md
---
# main
## use
+ `x`
+ `y`

| 参数 | 类型 | 说明 |
|---|---|---|
| x | int | integer |
| y | int | integer |

| 定理 | 结论 | 策略 |
|---|---|---|
| forwarded | 结果 = x | auto |

**v = [选择](math.math.select(`y`, `x`))**
*v*
"#;
        fs::write(root.join("math.mq.md"),callee).unwrap();
        fs::write(root.join("main.mq.md"),caller).unwrap();
        let project=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(&root).unwrap()).unwrap();
        let call=project.semantics.bound_calls.iter().find(|c|
            c.call_site=="math.math.select").expect("static call");
        let proof=candidate(&project,call).expect("symbolic projection");
        assert_eq!(proof.parameter_index,1);
        assert_eq!(proof.result_label,"x");
        assert!(proof.callee_source.contains(":= v1"));
        assert!(proof.caller_source.contains("callee v1 v0"));
        assert!(proof.caller_source.contains("= v0"));
        assert!(!proof.caller_source.contains("sorry"));

        // Reordering the actuals without changing the asserted result must
        // invalidate the caller contract; source is the authority.
        let incorrect=caller.replace(
            "math.math.select(`y`, `x`)",
            "math.math.select(`x`, `y`)");
        fs::write(root.join("main.mq.md"),incorrect).unwrap();
        let changed=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(&root).unwrap()).unwrap();
        let call=changed.semantics.bound_calls.iter().find(|c|
            c.call_site=="math.math.select").unwrap();
        assert!(candidate(&changed,call).is_none());

        // A declaration with an unsatisfied precondition is NOT a proof.
        fs::write(root.join("main.mq.md"),caller).unwrap();
        fs::write(root.join("math.mq.md"),
            callee.replace("| proj | 结果 = second | auto |",
                "| proj | 结果 = second | auto |\n| other | 结果 = second | auto |")).unwrap();
        let ambiguous=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(&root).unwrap()).unwrap();
        let call=ambiguous.semantics.bound_calls.iter().find(|c|
            c.call_site=="math.math.select").unwrap();
        assert!(candidate(&ambiguous,call).is_none());
        fs::remove_dir_all(root).unwrap();
    }
}

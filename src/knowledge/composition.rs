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
    ast::{CallExpr,Expr,Function,Literal,Stmt},
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
    // Treat duplicate eligible claims as ambiguity, never as evidence.
    // A single matching declaration is required at the exact owner SymbolId.
    owner.symbol_id.as_ref().is_some_and(|id|
        compiled.semantics.claims.iter().filter(|c|
            c.owner==*id
            && c.premise.as_deref().map_or(true,|p|p.trim().is_empty())
            && c.conclusion.trim()==format!("结果 = {n}")).count()==1)
}
fn candidate(compiled:&Compilation,call:&BoundCall)
  ->Option<(String,String,String,i64)>{
    let ca=compiled.semantics.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.caller) && a.kind=="function")?;
    let ce=compiled.semantics.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.target) && a.kind=="function")?;
    // The contract is strictly inter-module: local recursion and aliases
    // must not inherit an import-composition certificate.
    if ca.source.file==ce.source.file || call.caller==call.target{return None}
    if !ca.effects.is_pure()||!ce.effects.is_pure()
        ||!call.required_effects.is_pure()
        ||call.control_blocker.is_some()||!call.path_guards.is_empty(){return None}
    let caller=function(compiled,ca)?;
    let callee=function(compiled,ce)?;
    let forwarded=forwarded(&caller)?;
    let runtime=forwarded.for_evaluation().ok()?;
    if crate::sema::bind::call_display(&runtime)!=call.call_site
       ||compiled.semantics.bound_calls.iter().filter(|item|
            item.caller==call.caller).count()!=1 {return None}
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
        // Marqdo currently canonicalizes int/num/float to "number".
        // This certificate is explicitly about the *Lean Int model*, NOT
        // all runtime Number variants. Runtime-equivalence remains false.
        .any(|p|!p.ty.eq_ignore_ascii_case("number")||!p.required){return None}
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
        "-- pure typed-int fragment of {}\n\
         def callee {callee_binders} : Int := ({n} : Int)\n\
         theorem callee_contract {callee_binders} : callee {callee_args} = ({n} : Int) := by rfl\n",
        ce.artifact_id.0);
    let caller_src=format!(
        "import CalleeModule\n\
         -- exact forwarded import call from {}\n\
         def caller {caller_binders} : Int := callee {arg_expr}\n\
         theorem caller_contract {caller_binders} : caller {caller_args} = ({n} : Int) := by\n\
         \x20\x20simpa [caller] using (callee_contract {arg_expr})\n",
        ca.artifact_id.0);
    Some((callee_src,caller_src,format!("{}#{}",ca.artifact_id.0,
        ce.artifact_id.0),n))
}
/// Explain why a caller cannot receive a scoped composition certificate.
/// Missing evidence is reported, never fixed up by guessing call identities.
fn rejection_diagnostics(c:&Compilation,call:&BoundCall)->Value{
    let caller=c.semantics.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.caller));
    let callee=c.semantics.artifacts.iter().find(|a|
        a.symbol_id.as_ref()==Some(&call.target));
    let cf=caller.and_then(|a|function(c,a));
    let ef=callee.and_then(|a|function(c,a));
    let constant=ef.as_ref().and_then(constant_return);
    json!({
        "caller_semantics_resolved":caller.is_some(),
        "callee_semantics_resolved":callee.is_some(),
        "caller_effects":caller.map(|a|a.effects.values()),
        "callee_effects":callee.map(|a|a.effects.values()),
        "call_effects":call.required_effects.values(),
        "caller_forward_shape":cf.as_ref().and_then(forwarded).is_some(),
        "caller_ast_body":cf.as_ref().map(|f|format!("{:?}",f.body)),
        "callee_ast_body":ef.as_ref().map(|f|format!("{:?}",f.body)),
        "callee_constant_return":constant,
        "runtime_forward_name":cf.as_ref().and_then(forwarded)
            .and_then(|f|f.for_evaluation().ok())
            .map(|c|crate::sema::bind::call_display(&c)),
        "bound_call_name":call.call_site,
        "caller_bound_call_count":c.semantics.bound_calls.iter()
            .filter(|item|item.caller==call.caller).count(),
        "caller_has_expected_claim":caller.zip(constant).is_some_and(|(a,n)|
            claim_is_constant(c,a,n)),
        "callee_has_expected_claim":callee.zip(constant).is_some_and(|(a,n)|
            claim_is_constant(c,a,n)),
        "caller_signature":caller.and_then(|a|a.signature.as_ref())
            .map(|s|s.params.iter().map(|p|json!({"name":p.name,"type":p.ty,
                "required":p.required})).collect::<Vec<_>>()),
        "callee_signature":callee.and_then(|a|a.signature.as_ref())
            .map(|s|s.params.iter().map(|p|json!({"name":p.name,"type":p.ty,
                "required":p.required})).collect::<Vec<_>>()),
        "actual_arguments":call.args.iter().map(|a|json!({
            "name":a.name,"symbol":a.symbol,"integer":a.integer})).collect::<Vec<_>>(),
        "path_guards":call.path_guards,"control_blocker":call.control_blocker
    })
}
/// Emit one sealed bundle per eligible call. An unsuccessful Lean command
/// never creates a certificate; the JSON report carries the failure.
pub fn prove(compiled:&Compilation,out:&Path,toolchain:&Toolchain)->Result<Value>{
    let mut records=Vec::new();
    for call in &compiled.semantics.bound_calls{
        let Some((source,client,identity,n))=candidate(compiled,call) else {
            records.push(json!({"caller":call.caller.as_str(),
                "callee":call.target.as_str(),"status":"unsupported_fragment",
                "reason":"requires_pure_number_contract_int_model_forwarder_with_constant_claims",
                "diagnostics":rejection_diagnostics(compiled,call)}));
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
            "scope":"number_contract_int_model_pure_forwarder_constant_result",
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
            "scope":"number_contract_int_model_pure_forwarder_constant_result"}));
    }
    Ok(json!({"schema_version":1,"role":"restricted_composed_lean_module",
        "results":records,
        "full_marqdo_runtime_semantics_proved":false}))
}



/// Strict receipt-to-source correspondence; any duplicate row is a hard failure.
fn ensure_exact_closures(
    rows:usize,
    received:&std::collections::BTreeSet<(String,String)>,
    expected:&std::collections::BTreeSet<(String,String)>
)->Result<()>{
    anyhow::ensure!(rows==expected.len() && received==expected,
        "proof.compose_stale_or_ambiguous_closure");
    Ok(())
}

/// Replay a composition certificate *against the current project source*.
/// The certificate never supplies Lean source code: it must byte-match source
/// recompiled from the current Marqdo AST, and both Lean modules are rechecked.
pub fn replay(compiled:&Compilation,certificate:&Path,lean:&Toolchain)->Result<Value>{
    let receipt:Value=serde_json::from_slice(&fs::read(certificate)?)?;
    anyhow::ensure!(receipt["schema_version"]==1
        &&receipt["kind"]=="marqdo_restricted_composition"
        &&receipt["scope"]=="number_contract_int_model_pure_forwarder_constant_result"
        &&receipt["status"]=="lean_kernel_proved_model",
        "proof.compose_invalid_certificate: unexpected status or scope");
    anyhow::ensure!(receipt["toolchain"]==lean.version,
        "proof.compose_stale_toolchain");
    let identity=receipt["identity"].as_str().context("missing identity")?;
    // A certificate identity is a unique proof subject, not a search hint.
    // Reject ambiguous candidates instead of silently selecting the first.
    let matching=compiled.semantics.bound_calls.iter()
        .filter_map(|call|{
            let (a,b,id,n)=candidate(compiled,call)?;
            (id==identity).then_some((a,b,id,n,call))
        }).collect::<Vec<_>>();
    anyhow::ensure!(matching.len()==1,
        "proof.compose_ambiguous_or_stale_source: expected exactly one callsite");
    let (callee,caller,_,n,call)=matching.into_iter().next()
        .context("proof.compose_missing_candidate")?;
    anyhow::ensure!(receipt["claimed_result"]==n,
        "proof.compose_claim_changed");
    anyhow::ensure!(receipt["caller"]==call.caller.as_str()
        &&receipt["callee"]==call.target.as_str(),
        "proof.compose_symbol_changed");
    // Exact-set comparison blocks duplicate, injected, and omitted dependency
    // rows. The receipt does not get to choose the dependency closure.
    let expected_closures=compiled.semantics.artifacts.iter()
        .filter(|a|a.symbol_id.as_ref()==Some(&call.caller)
            ||a.symbol_id.as_ref()==Some(&call.target))
        .map(|a|(a.artifact_id.0.clone(),a.hashes.closure.0.clone()))
        .collect::<std::collections::BTreeSet<_>>();
    anyhow::ensure!(expected_closures.len()==2,
        "proof.compose_ambiguous_artifact");
    let received=receipt["source_closures"].as_array()
        .context("proof.compose_missing_source_closures")?;
    let actual_closures=received.iter().map(|r|{
        Ok((r["id"].as_str().context("proof.compose_invalid_closure_id")?.to_owned(),
            r["closure"].as_str().context("proof.compose_invalid_closure_hash")?.to_owned()))
    }).collect::<Result<std::collections::BTreeSet<_>>>()?;
    ensure_exact_closures(received.len(),&actual_closures,&expected_closures)?;
    let folder=certificate.parent().context("missing certificate parent")?;
    let callee_path=folder.join("CalleeModule.lean");
    let caller_path=folder.join("CallerModule.lean");
    anyhow::ensure!(fs::read_to_string(&callee_path)?==callee
        &&fs::read_to_string(&caller_path)?==caller,
        "proof.compose_generated_source_mismatch");
    anyhow::ensure!(receipt["callee_source_sha256"]==sha(callee.as_bytes())
        &&receipt["caller_source_sha256"]==sha(caller.as_bytes()),
        "proof.compose_hash_mismatch");
    let callee_obj=folder.join("CalleeModule.olean");
    let caller_obj=folder.join("CallerModule.olean");
    compile_import_module(lean,&callee_path,&callee_obj,folder,
        Duration::from_secs(120))?;
    compile_import_module(lean,&caller_path,&caller_obj,folder,
        Duration::from_secs(120))?;
    anyhow::ensure!(receipt["callee_compiled_sha256"]==sha(&fs::read(&callee_obj)?)
        &&receipt["caller_compiled_sha256"]==sha(&fs::read(&caller_obj)?),
        "proof.compose_replay_output_changed");
    Ok(json!({"schema_version":1,"kind":"independent_composition_replay",
        "status":"lean_kernel_replayed","scope":receipt["scope"],
        "identity":identity,"current_source_verified":true,
        "full_marqdo_runtime_semantics_proved":false}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn certificate_closures_must_be_exact_and_unique(){
        use std::collections::BTreeSet;
        let expected=BTreeSet::from([
            ("caller".into(),"sha256:a".into()),
            ("callee".into(),"sha256:b".into())
        ]);
        assert!(ensure_exact_closures(2,&expected,&expected).is_ok());
        assert!(ensure_exact_closures(3,&expected,&expected).is_err());
        let missing=BTreeSet::from([("caller".into(),"sha256:a".into())]);
        assert!(ensure_exact_closures(1,&missing,&expected).is_err());
        let mut injected=expected.clone();
        injected.insert(("third".into(),"sha256:c".into()));
        assert!(ensure_exact_closures(3,&injected,&expected).is_err());
        let mut changed=expected.clone();
        changed.remove(&("callee".into(),"sha256:b".into()));
        changed.insert(("callee".into(),"sha256:wrong".into()));
        assert!(ensure_exact_closures(2,&changed,&expected).is_err());
    }
    #[test]
    fn pure_forwarder_can_be_compiled_into_two_independent_lean_sources(){
        let root=std::env::temp_dir().join(
            format!("marqdo-compose-unit-{}",std::process::id()));
        let _=fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("math.mq.md"),r#"---
markup: occlusion
---

# math

## exact
+ `n`

| 参数 | 类型 | 说明 |
|---|---|---|
| n | int | integer |

| 定理 | 结论 | 策略 |
|---|---|---|
| fixed | 结果 = 2 | auto |

*2*
"#).unwrap();
        fs::write(root.join("main.mq.md"),r#"---
markup: occlusion
import:
  math: math.mq.md
---

# main

## forward
+ `n`

| 参数 | 类型 | 说明 |
|---|---|---|
| n | int | integer |

| 定理 | 结论 | 策略 |
|---|---|---|
| same | 结果 = 2 | auto |

**v = [调用](math.math.exact(`n`))**
*v*
"#).unwrap();
        let c=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(&root).unwrap()).unwrap();
        let call=c.semantics.bound_calls.iter().find(|x|
            x.call_site=="math.math.exact").unwrap();
        let (callee,caller,_,result)=candidate(&c,call)
            .unwrap_or_else(||panic!("composition candidate rejected: {}",
                rejection_diagnostics(&c,call)));
        assert_eq!(result,2);
        assert!(callee.contains("theorem callee_contract"));
        assert!(caller.starts_with("import CalleeModule"));
        assert!(caller.contains("using (callee_contract"));
        // Branching is explicitly outside the certificate scope.
        fs::write(root.join("main.mq.md"),r#"---
import:
  math: math.mq.md
---

# main

## forward
+ `n`

| 参数 | 类型 | 说明 |
|---|---|---|
| n | int | integer |

| 定理 | 结论 | 策略 |
|---|---|---|
| same | 结果 = 2 | auto |

1. `n` > 0
  **v = [调用](math.math.exact(`n`))**
2. *
  **v = 0**

*v*
"#).unwrap();
        let changed=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(&root).unwrap()).unwrap();
        let guarded=changed.semantics.bound_calls.iter().find(|x|
            x.call_site=="math.math.exact").unwrap();
        assert!(candidate(&changed,guarded).is_none());
        fs::remove_dir_all(root).unwrap();
    }
}

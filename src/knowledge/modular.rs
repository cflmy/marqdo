//! Conservative inter-module proof dependency planning.
//!
//! This is NOT a rule that a proved callee theorem magically proves its
//! caller. It records which Lean-checked callee contracts a caller could rely
//! on and refuses to discharge parameter-dependent preconditions by guesswork.
//! The independently proved caller claim remains a separate obligation.

use std::collections::{BTreeMap, BTreeSet};
use serde_json::{json, Value};
use crate::sema::ir::SemanticProject;

fn linked_claims<'a>(proofs:&'a Value, symbol:&str)->Vec<&'a Value>{
    proofs["claims"].as_array().into_iter()
        .flat_map(|rows|rows.iter())
        .filter(|r|r["owner_symbol_id"].as_str()==Some(symbol))
        .collect()
}

pub fn analyze(semantic:&SemanticProject, proof_index:&Value)->Value{
    analyze_with_verified(semantic,proof_index,&BTreeSet::new())
}

/// The accepted set must come from the real Lean kernel, never from the host
/// calculator or an untrusted JSON status.
pub fn analyze_with_verified(semantic:&SemanticProject, proof_index:&Value,
    kernel_verified:&BTreeSet<String>)->Value{
    let known:BTreeSet<&str>=semantic.artifacts.iter()
        .filter_map(|a|a.symbol_id.as_ref().map(|id|id.as_str())).collect();
    let mut obligations=Vec::new();
    let mut links:BTreeMap<String,Vec<String>>=BTreeMap::new();
    for call in &semantic.bound_calls {
        let callee=call.target.as_str();
        let caller_id=call.caller.as_str();
        let caller=Some(caller_id);
        let assumption=semantic.assumptions.iter()
            .find(|a|a.owner.as_str()==caller_id &&
                (a.operation==call.call_site || a.operation==callee));
        let id=format!("{}:{}:{}",call.source.file,call.source.line,callee);
        let exported=semantic.claims.iter().any(|c|c.owner.as_str()==callee);
        let receipts=linked_claims(proof_index,callee);
        let contracts:Vec<Value>=semantic.claims.iter()
            .filter(|c|c.owner.as_str()==callee)
            .map(|c|{
                let matched=receipts.iter().find(|r|
                    r["claim_anchor_id"]==c.anchor_id
                    && r["conclusion"]==c.conclusion
                    && r["premise"]==json!(c.premise));
                let verified=matched.is_some_and(|r|r["status"]=="proved"
                    && r["usable"]==true);
                let unconditional=c.premise.as_deref()
                    .map_or(true,|s|s.trim().is_empty());
                let goal=if unconditional {None} else {
                    c.premise.as_deref().and_then(|pre|
                        super::vc::instantiate(semantic,call,pre))
                };
                let symbolic=if unconditional {None} else {
                    c.premise.as_deref().and_then(|pre|
                        super::symbolic::plan(semantic,call,pre))
                };
                let key=format!("{}#{}",id,c.anchor_id);
                let kernel_passed=goal.as_ref().is_some_and(|g|
                    g.holds && kernel_verified.contains(&key));
                json!({
                    "claim_anchor_id":c.anchor_id,
                    "premise":c.premise,
                    "conclusion":c.conclusion,
                    "callee_certificate_valid":verified,
                    // An unbound premise is a caller-side proof obligation;
                    // it is never discharged merely because callee was proved.
                    "precondition_goal_key":if unconditional {None} else {Some(key)},
                    "lean_proposition":goal.as_ref().map(|g|g.proposition.as_str()),
                    "closed_host_check":goal.as_ref().map(|g|g.holds),
                    "symbolic_model_proposition":symbolic.as_ref().map(|g|g.proposition.as_str()),
                    "symbolic_model_guard_count":symbolic.as_ref().map(|g|g.guard_propositions.len()),
                    "symbolic_model_runtime_domain_verified":symbolic.as_ref().map(|g|g.runtime_domain_verified),
                    "path_guards":call.path_guards,
                    "control_blocker":call.control_blocker,
                    "call_precondition_status":if unconditional {"none"}
                        else if kernel_passed {"lean_kernel_verified"}
                        else if goal.as_ref().is_some_and(|g|!g.holds) {"refuted_closed_precondition"}
                        else if goal.is_some() {"requires_lean_kernel"}
                        else if symbolic.is_some() {"symbolic_path_lean_model_pending"}
                        else {"requires_symbolic_callsite_proof"},
                    "status":if !verified {"missing_verified_callee_claim"}
                        else if unconditional || kernel_passed {"available_as_contract"}
                        else if goal.as_ref().is_some_and(|g|!g.holds) {"precondition_refuted"}
                        else {"precondition_pending"}
                })
            }).collect();
        let controls_sufficient=call.control_blocker.is_none()
            && call.required_effects.is_pure();
        let all_available=controls_sufficient && exported && !contracts.is_empty()
            && contracts.iter().all(|x|x["status"]=="available_as_contract");
        let status=if !known.contains(callee) {
            "external_without_semantic_definition"
        } else if !controls_sufficient {
            "unmodeled_control_or_effect"
        } else if !exported {
            "callee_without_declared_contract"
        } else if all_available {
            "callee_contracts_verified"
        } else {
            "proof_obligations_pending"
        };
        obligations.push(json!({
            "id":id,"caller":caller,"callee":callee,
            "call_site":call.call_site,
            "source":{"file":call.source.file,"line":call.source.line},
            "status":status,"contracts":contracts,
            "caller_proof_required":true,
            "runtime_link_required":true,
            "path_guards":call.path_guards,
            "control_blocker":call.control_blocker,
            "effects":call.required_effects.values(),
            "runtime_effect_model":"not_established_by_knowledge_audit",
            "caller_assumption":assumption.map(|a|json!({
                "id":a.id,"premise":a.premise,
                "postcondition":a.postcondition,
                "verified":false,
                "status":"requires_lean_callsite_derivation"
            })),
            "caller_assumption_binding":if assumption.is_some(){"declared_unproved"}
                else {"absent"}
        }));
        if let Some(caller)=caller {
            links.entry(caller.to_string()).or_default().push(callee.into());
        }
    }
    for call in &semantic.dynamic_calls {
        obligations.push(json!({
            "id":format!("{}:{}:dynamic",call.source.file,call.source.line),
            "source":{"file":call.source.file,"line":call.source.line},
            "status":"dynamic_or_unresolved_no_modular_proof",
            "reason":call.reason,"caller_proof_required":true
        }));
    }
    obligations.sort_by(|a,b|a["id"].as_str().cmp(&b["id"].as_str()));
    let cycles=find_cycles(&links);
    let verified_claims:Vec<Value>=semantic.claims.iter().map(|c|{
        let found=linked_claims(proof_index,c.owner.as_str()).iter().any(|r|
            r["claim_anchor_id"]==c.anchor_id &&
            r["conclusion"]==c.conclusion &&
            r["premise"]==json!(c.premise) &&
            r["status"]=="proved" && r["usable"]==true);
        json!({"claim_anchor_id":c.anchor_id,"owner":c.owner.as_str(),
            "status":if found {"proved_with_local_lean_receipt"} else {"unverified"}})
    }).collect();
    let linked=obligations.iter().all(|o|o["status"]=="callee_contracts_verified")
        && cycles.is_empty();
    let declarations_proved=!verified_claims.is_empty()
        && verified_claims.iter().all(|c|c["status"]=="proved_with_local_lean_receipt");
    let audit_passes=linked && declarations_proved;
    json!({
        "schema_version":1,
        "kind":"modular_proof_obligations",
        "status":if audit_passes {"dependency_receipts_valid"}
            else {"open_obligations"},
        "audit_passes":audit_passes,
        "verified_claims":verified_claims,
        // Even a successful audit is NOT a modular Lean theorem:
        // the runtime-link and caller entailment obligations remain.
        "proof_completion":"not_established_by_dependency_analysis",
        "obligations":obligations,
        "cycles":cycles
    })
}

fn find_cycles(graph:&BTreeMap<String,Vec<String>>)->Vec<Vec<String>>{
    fn visit<'a>(node:&'a str,g:&'a BTreeMap<String,Vec<String>>,
        visiting:&mut BTreeSet<String>,done:&mut BTreeSet<String>,
        stack:&mut Vec<String>,cycles:&mut Vec<Vec<String>>){
        if done.contains(node){return}
        if !visiting.insert(node.to_string()){
            if let Some(start)=stack.iter().position(|x|x==node){
                let mut cycle=stack[start..].to_vec();
                cycle.push(node.to_string());
                cycles.push(cycle);
            }
            return
        }
        stack.push(node.to_string());
        if let Some(next)=g.get(node){
            for target in next {visit(target,g,visiting,done,stack,cycles)}
        }
        stack.pop();visiting.remove(node);done.insert(node.to_string());
    }
    let mut visiting=BTreeSet::new();
    let mut done=BTreeSet::new();
    let mut cycles=Vec::new();
    for key in graph.keys(){visit(key,graph,&mut visiting,&mut done,&mut Vec::new(),&mut cycles)}
    cycles
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contracts_never_discharge_callsite_preconditions_implicitly(){
        let source=std::env::temp_dir().join(format!("marqdo-modular-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&source);
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("callee.mq.md"),
            "# callee\n\n## f\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| c | 结果 = 2 | x > 0 | auto |\n\n*2*\n").unwrap();
        std::fs::write(source.join("main.mq.md"),
            "---\nimport:\n  callee: callee.mq.md\n---\n# main\n\n## call\n\n**x = [外部调用](callee.callee.f())**\n*x*\n").unwrap();
        let compilation=crate::sema::ir::compile_project(
            &crate::sema::ir::CompileOptions::for_path(&source).unwrap()).unwrap();
        let report=analyze(&compilation.semantics,&json!({"claims":[]}));
        let obligations=report["obligations"].as_array().unwrap();
        assert!(!obligations.is_empty());
        assert!(obligations.iter().all(|x|x["status"]!="proved"));
        assert_eq!(report["proof_completion"],"not_established_by_dependency_analysis");
        // Even a caller-supplied "valid" callee receipt cannot discharge the
        // actual-argument bridge for a conditional requires clause.
        let callee=compilation.semantics.claims.iter().find(|c|c.label=="c").unwrap();
        let forged=json!({"claims":[{
            "owner_symbol_id":callee.owner.as_str(),
            "claim_anchor_id":callee.anchor_id,
            "conclusion":callee.conclusion,
            "premise":callee.premise,
            "status":"proved","usable":true
        }]});
        let conditional=analyze(&compilation.semantics,&forged);
        let first=conditional["obligations"].as_array().unwrap()
            .iter().find(|x|x["callee"]==callee.owner.as_str()).unwrap();
        assert_eq!(first["contracts"][0]["status"],"precondition_pending");
        assert_eq!(first["contracts"][0]["call_precondition_status"],
            "requires_symbolic_callsite_proof");
        assert_eq!(conditional["audit_passes"],false);
        std::fs::remove_dir_all(source).unwrap();
    }
}

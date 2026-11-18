//! Bind a proved function claim to one semantic symbol, never to every
//! marked step. Unmatched identities and stale evidence fail closed.

use std::{collections::BTreeMap, fs, path::Path};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use crate::{prove::{Report, Verdict}, sema::ir::{Compilation, SemanticArtifact, SemanticProject}};

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn generated_module_valid(folder:&Path,expected:&Value)->bool {
    let source_dir=folder.join("Generated");
    let Ok(files)=fs::read_dir(&source_dir) else {return false};
    let mut candidates=files.flatten().filter(|f|
        f.path().extension().is_some_and(|ext|ext=="lean")).collect::<Vec<_>>();
    if candidates.len()!=1{return false}
    let Some(generated)=candidates.pop() else{return false};
    let Ok(source)=fs::read(generated.path()) else{return false};
    sha(&source).trim_start_matches("sha256:")==expected.as_str().unwrap_or("")
}
fn certificate_identity_valid(folder:&Path,cert:&Value)->bool {
    let Ok(plan)=fs::read(folder.join("plan.json")) else {return false};
    let source=cert["source_sha256"].as_str().unwrap_or("");
    let library=cert["library_sha256"].as_str().unwrap_or("");
    let toolchain=cert["toolchain"].as_str().unwrap_or("");
    let actual_library=crate::prove::cache::sha256_hex(
        crate::prove::driver::LIBRARY_SRC.as_bytes());
    let expected_key=crate::prove::cache::cache_key(
        source,&crate::prove::cache::sha256_hex(&plan),
        library,toolchain,env!("CARGO_PKG_VERSION"));
    library==actual_library
        && cert["marqdo"]==env!("CARGO_PKG_VERSION")
        && cert["key"]==expected_key
        && generated_module_valid(folder,&cert["module_sha256"])
}

fn certificate(dir: &Path, report: &Report) -> Option<(Value, String)> {
    let bytes = fs::read(dir.join("certificate.json")).ok()?;
    let cert: Value = serde_json::from_slice(&bytes).ok()?;
    let lean = report.lean.as_ref()?;
    if cert["schema"] != 1 || cert["source_sha256"] != report.source_sha256
       || cert["tier"] != "lean" || cert["library_sha256"] != lean.library_sha256
       || cert["toolchain"] != lean.toolchain || cert["module_sha256"] != lean.module_sha256 {
        return None;
    }
    Some((cert, sha(&bytes)))
}

fn matching<'a>(compiled: &'a Compilation, path: &Path, name: &str)
 -> Option<&'a SemanticArtifact> {
    let file=path.canonicalize().ok()?;
    let mut all=compiled.semantics.artifacts.iter()
      .filter(|a|a.kind=="function" && a.display_name==name)
      .filter(|a|compiled.semantics.root.join(&a.source.file)
          .canonicalize().ok().as_deref()==Some(file.as_path()));
    let first=all.next()?;
    all.next().is_none().then_some(first)
}

/// Exact owner binding; related step anchors are navigation only, not proved.
pub fn link_report(compiled: &Compilation, file: &Path, report: &Report, dir: &Path) -> Value {
    let cert=certificate(dir,report);
    let mut claims=Vec::new();
    let mut unresolved=Vec::new();
    for f in &report.functions {
        let Some(owner)=matching(compiled,file,&f.name) else {
            unresolved.push(json!({"function":f.name,"reason":"unbound_or_ambiguous"}));
            continue;
        };
        let Some(symbol)=owner.symbol_id.as_ref() else {
            unresolved.push(json!({"function":f.name,"reason":"missing_symbol_id"}));
            continue;
        };
        let steps:Vec<&str>=compiled.semantics.steps.iter()
            .filter(|s|s.id.0.starts_with(&format!("{}#step-",symbol.as_str())))
            .filter_map(|s|s.anchor_id.as_deref()).collect();
        for claim in &f.claims {
            // A proof receipt must be bound to the exact theorem declaration
            // present in the same AST, not merely to a similarly named function.
            let mut matching_claims=compiled.semantics.claims.iter().filter(|decl|
                decl.owner==*symbol && decl.label==claim.id
                && decl.conclusion==claim.conclusion
                && decl.premise==claim.premise);
            let Some(declared)=matching_claims.next() else {
                unresolved.push(json!({"function":f.name,"claim":claim.id,
                    "reason":"theorem_declaration_missing_or_changed"}));
                continue;
            };
            if matching_claims.next().is_some() {
                unresolved.push(json!({"function":f.name,"claim":claim.id,
                    "reason":"ambiguous_theorem_declaration"}));
                continue;
            }
            let cert_ok=cert.as_ref().is_some_and(|(v,_)|
                v["claims"].as_array().is_some_and(|records|
                    records.iter().any(|r|
                        r["id"]==claim.id && r["function"]==f.name
                        && r["status"]=="proved")));
            let proved=claim.status==Verdict::Proved && claim.decided_by=="lean" && cert_ok;
            let status=if claim.status==Verdict::Proved && !proved {
                "unverified"
            } else { claim.status.as_str() };
            claims.push(json!({
                "claim_anchor_id":declared.anchor_id,
                "owner_symbol_id":symbol.as_str(),
                "function":f.name,"claim_id":claim.id,
                "premise":claim.premise,"conclusion":claim.conclusion,
                "source_line":claim.source_line,
                "status":status,
                "decided_by":if proved {"lean"} else if status=="checked" {"offline"} else {"none"},
                "certificate_digest":if proved {cert.as_ref().map(|(_,h)|h.as_str())} else {None},
                "proof_scope":"function",
                "related_anchor_ids":steps,
                "related_anchor_relation":"defined_in_same_function_not_individually_proved",
                "semantic_hashes":{"interface":owner.hashes.interface.0,
                    "implementation":owner.hashes.implementation.0,
                    "closure":owner.hashes.closure.0},
                "source_file":owner.source.file,
                "source_sha256":report.source_sha256,
                "library_sha256":report.lean.as_ref().map(|l|l.library_sha256.as_str()),
                "toolchain":report.lean.as_ref().map(|l|l.toolchain.as_str()),
                "module_sha256":report.lean.as_ref().map(|l|l.module_sha256.as_str())
            }));
        }
    }
    claims.sort_by(|a,b|a["claim_anchor_id"].as_str().cmp(&b["claim_anchor_id"].as_str()));
    let assumptions:Vec<Value>=compiled.semantics.assumptions.iter()
        .filter(|a|compiled.semantics.root.join(&a.source.file)
            .canonicalize().ok().as_deref()==file.canonicalize().ok().as_deref())
        .map(|a|json!({"id":a.id,"owner":a.owner.as_str(),
            "operation":a.operation,"premise":a.premise,
            "postcondition":a.postcondition,
            "status":"declared_not_disclosed_as_proved"}))
        .collect();
    json!({"schema_version":1,"kind":"knowledge_proof_linkage",
        "source_sha256":report.source_sha256,"file":report.file,
        "claims":claims,"assumptions":assumptions,"unresolved":unresolved})
}

pub fn emit_links(file: &Path, report: &Report, dir: &Path) -> Result<()> {
    let result=match crate::sema::ir::compile_file_closure(file) {
        Ok(compiled)=>link_report(&compiled,file,report,dir),
        Err(err)=>json!({"schema_version":1,"kind":"knowledge_proof_linkage",
            "status":"unbound","source_sha256":report.source_sha256,"claims":[],
            "unresolved":[{"reason":"semantic_compile_failed","detail":format!("{err:#}")}]}),
    };
    let path=dir.join("knowledge-links.json");
    fs::write(&path,serde_json::to_vec_pretty(&result)?)
        .with_context(||format!("write {}",path.display()))
}

/// Invalidate proof links against both semantic owner hashes and source bytes.
/// Certificate integrity is rechecked; duplicate claim identities are refused.
pub fn index_links(prove_dir:&Path, semantic:&SemanticProject)->Value {
    let owners:BTreeMap<_,_>=semantic.artifacts.iter()
        .filter_map(|a|a.symbol_id.as_ref().map(|id|(id.as_str().to_string(),a)))
        .collect();
    let mut entries=Vec::new();
    if let Ok(folders)=fs::read_dir(prove_dir) {
        for folder in folders.flatten().filter(|f|f.path().is_dir()) {
            let Ok(bytes)=fs::read(folder.path().join("knowledge-links.json")) else {continue};
            let Ok(link) = serde_json::from_slice::<Value>(&bytes) else {continue};
            let Some(claims)=link["claims"].as_array() else {continue};
            for claim in claims {
                let mut entry=claim.clone();
                entry["receipt_artifact"]=json!(folder.file_name().to_string_lossy().to_string());
                let owner=claim["owner_symbol_id"].as_str().and_then(|id|owners.get(id));
                let theorem_present=claim["owner_symbol_id"].as_str().is_some_and(|id|
                    semantic.claims.iter().any(|c|
                        c.owner.as_str()==id
                        && claim["claim_anchor_id"]==c.anchor_id
                        && claim["claim_id"]==c.label
                        && claim["conclusion"]==c.conclusion
                        && claim["premise"]==json!(c.premise)));
                let fresh=theorem_present && owner.is_some_and(|a| {
                    claim["source_file"]==a.source.file
                    && claim["semantic_hashes"]["interface"]==a.hashes.interface.0
                    && claim["semantic_hashes"]["implementation"]==a.hashes.implementation.0
                    && claim["semantic_hashes"]["closure"]==a.hashes.closure.0
                    && fs::read(semantic.root.join(&a.source.file)).ok().is_some_and(
                        |raw|sha(&raw).trim_start_matches("sha256:")==claim["source_sha256"])
                });
                let certified=entry["status"]=="proved" && entry["decided_by"]=="lean"
                    && entry["certificate_digest"].as_str().is_some_and(|digest|
                        fs::read(folder.path().join("certificate.json")).ok().is_some_and(|bytes| {
                            if sha(&bytes)!=digest { return false; }
                            let Ok(cert)=serde_json::from_slice::<Value>(&bytes) else {
                                return false
                            };
                            cert["schema"]==1 && cert["tier"]=="lean"
                                && certificate_identity_valid(&folder.path(),&cert)
                                && cert["source_sha256"]==entry["source_sha256"]
                                && cert["library_sha256"]==entry["library_sha256"]
                                && cert["toolchain"]==entry["toolchain"]
                                && cert["module_sha256"]==entry["module_sha256"]
                                && cert["claims"].as_array().is_some_and(|records|
                                    records.iter().any(|r|
                                        r["id"]==entry["claim_id"]
                                        && r["function"]==entry["function"]
                                        && r["status"]=="proved"))
                        }));
                if !fresh {
                    entry["status"]=json!("stale");
                    entry["stale_reason"]=json!("source_or_semantic_dependency_changed");
                } else if entry["status"]=="proved" && !certified {
                    entry["status"]=json!("unverified");
                    entry["stale_reason"]=json!("missing_or_changed_lean_certificate");
                }
                entry["usable"]=json!(fresh && certified);
                entries.push(entry);
            }
        }
    }
    entries.sort_by(|a,b|a["claim_anchor_id"].as_str().cmp(&b["claim_anchor_id"].as_str()));
    let mut freq=BTreeMap::new();
    for e in &entries {
        if let Some(id)=e["claim_anchor_id"].as_str() {
            *freq.entry(id.to_string()).or_insert(0usize)+=1;
        }
    }
    for e in &mut entries {
        if e["claim_anchor_id"].as_str().and_then(|id|freq.get(id))
            .is_some_and(|n|*n>1) {
            e["status"]=json!("ambiguous");
            e["stale_reason"]=json!("multiple_proof_link_records");
            e["usable"]=json!(false);
        }
    }
    json!({"schema_version":1,"role":"proof_receipts_not_execution_authority",
        "claims":entries})
}


/// Recheck every locally cached *usable* Lean receipt with an actual Lean
/// invocation. Hash-only certificates are not trusted for --verify. An error
/// at one folder revokes all its receipts; no JSON proof flag can override it.
pub fn replay_index(
    prove_dir:&Path, index:&mut Value,
    toolchain:&crate::prove::driver::Toolchain,
    timeout:std::time::Duration
)->Result<()> {
    let (libdir,_)=crate::prove::driver::ensure_library(prove_dir,toolchain)?;
    let mut replayed:BTreeMap<String,Vec<(String,String)>>=BTreeMap::new();
    let mut rejected:std::collections::BTreeSet<String>=std::collections::BTreeSet::new();
    let Some(entries)=index["claims"].as_array_mut() else {return Ok(())};
    let artifacts:std::collections::BTreeSet<_>=entries.iter()
        .filter(|e|e["usable"]==true)
        .filter_map(|e|e["receipt_artifact"].as_str().map(str::to_owned))
        .collect();
    for id in artifacts {
        if id=="." || id==".." || id.contains('/') || id.contains('\\'){
            rejected.insert(id);
            continue;
        }
        let src_dir=prove_dir.join(&id).join("Generated");
        let files=fs::read_dir(&src_dir).ok().into_iter()
            .flat_map(|items|items.flatten())
            .filter(|f|f.path().extension().is_some_and(|ext|ext=="lean"))
            .map(|f|f.path()).collect::<Vec<_>>();
        if files.len()!=1 {rejected.insert(id);continue;}
        let path=&files[0];
        let Ok(source)=fs::read_to_string(path) else {rejected.insert(id);continue};
        let Ok(outcome)=crate::prove::driver::check_module(
            toolchain,&libdir,path,&source,timeout,None) else {
                rejected.insert(id);continue;
            };
        if outcome.status!=Verdict::Proved ||
            fs::read_to_string(path).ok().as_deref()!=Some(source.as_str()) {
            rejected.insert(id);continue;
        }
        replayed.insert(id,outcome.claims.into_iter()
            .filter(|c|c.status==Verdict::Proved)
            .map(|c|(c.function,c.claim)).collect());
    }
    for entry in entries {
        if entry["usable"]!=true {continue;}
        let id=entry["receipt_artifact"].as_str().unwrap_or("");
        let found=replayed.get(id).is_some_and(|claims|claims.iter().any(|(fun,claim)|
            entry["function"]==fun.as_str() && entry["claim_id"]==claim.as_str()));
        if !found || rejected.contains(id) {
            entry["usable"]=json!(false);
            entry["status"]=json!("unverified");
            entry["stale_reason"]=json!("lean_kernel_replay_failed");
        } else {
            entry["kernel_replayed"]=json!(true);
        }
    }
    Ok(())
}

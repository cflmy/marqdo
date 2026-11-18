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
                "toolchain":report.lean.as_ref().map(|l|l.toolchain.as_str())
            }));
        }
    }
    claims.sort_by(|a,b|a["claim_anchor_id"].as_str().cmp(&b["claim_anchor_id"].as_str()));
    json!({"schema_version":1,"kind":"knowledge_proof_linkage",
        "source_sha256":report.source_sha256,"file":report.file,
        "claims":claims,"unresolved":unresolved})
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
                let owner=claim["owner_symbol_id"].as_str().and_then(|id|owners.get(id));
                let fresh=owner.is_some_and(|a| {
                    claim["source_file"]==a.source.file
                    && claim["semantic_hashes"]["interface"]==a.hashes.interface.0
                    && claim["semantic_hashes"]["implementation"]==a.hashes.implementation.0
                    && claim["semantic_hashes"]["closure"]==a.hashes.closure.0
                    && fs::read(semantic.root.join(&a.source.file)).ok().is_some_and(
                        |raw|sha(&raw).trim_start_matches("sha256:")==claim["source_sha256"])
                });
                let certified=entry["status"]=="proved" && entry["decided_by"]=="lean"
                    && entry["certificate_digest"].as_str().is_some_and(|digest|
                        fs::read(folder.path().join("certificate.json")).ok().is_some_and(
                            |bytes|sha(&bytes)==digest));
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

//! Knowledge-anchored proof receipts are a *narrow* verified linkage layer.
//! No synthetic test here is evidence that Lean proved a statement.

use std::{fs, path::PathBuf};
use marqdo::knowledge::proof::{link_report, index_links};
use marqdo::prove::{ClaimReport, FunctionReport, LeanInfo, Report, Tier, Verdict};
use marqdo::sema::ir::{compile_project, CompileOptions};
use sha2::{Digest, Sha256};

fn root(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64=AtomicU64::new(0);
    let dir=std::env::temp_dir().join(format!("mq-proof-links-{}-{}-{}",
        label,std::process::id(),SEQ.fetch_add(1,Ordering::Relaxed)));
    fs::create_dir_all(&dir).unwrap();dir
}
fn sha(bytes:&[u8])->String {format!("{:x}",Sha256::digest(bytes))}
fn report(file:&PathBuf, status:Verdict, decided_by:&str, lean:Option<LeanInfo>)->Report {
    Report {
        file:"main.mq.md".into(),source_sha256:sha(&fs::read(file).unwrap()),
        artifact_id:None,status,
        functions:vec![FunctionReport{
            name:"main.compute".into(),verdict:status,
            tier:if lean.is_some(){Tier::Lean}else{Tier::Offline},
            ops:vec![],rejected:vec![],
            claims:vec![ClaimReport{
                id:"rule".into(),premise:None,conclusion:"结果 = 6".into(),
                tactic:None,status,decided_by:decided_by.into(),
                source_line:Some(5),evidence:None,
            }]
        }],
        diagnostics:vec![],emitted:vec![],lean,duration_ms:0
    }
}
#[test]
fn links_bind_to_exact_owner_and_stale_after_source_change() {
    let dir=root("stale");
    let file=dir.join("main.mq.md");
    fs::write(&file,"# main\n\n## compute\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| rule | 结果 = 6 | | auto |\n\n**x = [演算](2 * 3)**\n*x*\n").unwrap();
    let compiled=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let receipt=link_report(&compiled,&file,&report(&file,Verdict::Checked,"trace",None),&dir);
    assert_eq!(receipt["claims"][0]["status"],"checked");
    assert_eq!(receipt["claims"][0]["proof_scope"],"function");
    assert!(receipt["claims"][0]["owner_symbol_id"].as_str().unwrap().len()>10);
    assert_eq!(receipt["claims"][0]["certificate_digest"],serde_json::Value::Null);
    assert_eq!(receipt["claims"][0]["related_anchor_ids"].as_array().unwrap().len(),1);
    let sidecar=dir.join(".marqdo/prove/one");
    fs::create_dir_all(&sidecar).unwrap();
    fs::write(sidecar.join("knowledge-links.json"),serde_json::to_vec(&receipt).unwrap()).unwrap();
    let current=index_links(&dir.join(".marqdo/prove"),&compiled.semantics);
    assert_eq!(current["claims"][0]["status"],"checked");
    assert_eq!(current["claims"][0]["usable"],false);
    fs::write(&file,"# main\n\n## compute\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| rule | 结果 = 6 | | auto |\n\n**x = [演算](2 * 4)**\n*x*\n").unwrap();
    let changed=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let stale=index_links(&dir.join(".marqdo/prove"),&changed.semantics);
    assert_eq!(stale["claims"][0]["status"],"stale");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn altered_theorem_conclusion_cannot_reuse_a_matching_function_name() {
    let dir=root("altered-claim");
    let file=dir.join("main.mq.md");
    fs::write(&file,"# main\n\n## compute\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| rule | 结果 = 6 | | auto |\n\n**x = [演算](2 * 3)**\n*x*\n").unwrap();
    let c=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let mut fake=report(&file,Verdict::Proved,"lean",None);
    fake.functions[0].claims[0].conclusion="结果 = 999".into();
    let linkage=link_report(&c,&file,&fake,&dir);
    assert!(linkage["claims"].as_array().unwrap().is_empty());
    assert_eq!(linkage["unresolved"][0]["reason"],"theorem_declaration_missing_or_changed");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn dependency_change_invalidates_caller_proof_even_if_caller_source_is_unchanged() {
    let dir=root("transitive");
    let lib=dir.join("rates.mq.md");
    let app=dir.join("main.mq.md");
    fs::write(&lib,"# rates\n\n## lookup\n\n*2*\n").unwrap();
    fs::write(&app,r#"---
import:
  rates: rates.mq.md
---

# main

## compute

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| rule | 结果 = 6 | | auto |

**tax = [计算税额](rates.rates.lookup())**
*tax*
"#).unwrap();
    let before=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let owner=before.semantics.artifacts.iter()
        .find(|a|a.display_name=="main.compute").unwrap();
    assert!(!owner.dependencies.is_empty(),
        "imported call must be a bound dependency; check full class path");
    let old_closure=owner.hashes.closure.clone();
    let receipt=link_report(&before,&app,&report(&app,Verdict::Checked,"trace",None),&dir);
    assert_eq!(receipt["claims"].as_array().unwrap().len(),1,
        "function identity must resolve exactly");
    let proof=dir.join(".marqdo/prove/first");
    fs::create_dir_all(&proof).unwrap();
    fs::write(proof.join("knowledge-links.json"),
        serde_json::to_vec(&receipt).unwrap()).unwrap();
    fs::write(&lib,"# rates\n\n## lookup\n\n*3*\n").unwrap();
    let after=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let revised=after.semantics.artifacts.iter()
        .find(|a|a.display_name=="main.compute").unwrap();
    assert_ne!(old_closure,revised.hashes.closure,
        "callee implementation changes must invalidate caller closure");
    let index=index_links(&dir.join(".marqdo/prove"),&after.semantics);
    assert_eq!(index["claims"][0]["status"],"stale");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn complete_knowledge_publish_contains_anchors_refs_and_proof_receipts() {
    let dir=root("publish");
    let file=dir.join("main.mq.md");
    fs::write(&file,"# main\n\n## compute\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| rule | 结果 = 6 | | auto |\n\n[查看知识](#@演算)\n[查看定理](#@rule)\n**x = [演算](2 * 3)**\n*x*\n").unwrap();
    let out=dir.join(".marqdo");
    marqdo::knowledge::write_knowledge(marqdo::knowledge::KnowledgeOptions{
        path:dir.clone(),out_dir:out.clone()
    }).unwrap();
    let anchors:serde_json::Value=serde_json::from_slice(
        &fs::read(out.join("anchors.json")).unwrap()).unwrap();
    assert_eq!(anchors["anchors"].as_array().unwrap().len(),1);
    let refs:serde_json::Value=serde_json::from_slice(
        &fs::read(out.join("refs.json")).unwrap()).unwrap();
    assert!(refs["refs"].as_array().unwrap().iter()
        .any(|r|r["label"]=="查看知识" && r["resolution"]=="anchor"));
    let claims:serde_json::Value=serde_json::from_slice(
        &fs::read(out.join("claims.json")).unwrap()).unwrap();
    assert_eq!(claims["claims"].as_array().unwrap().len(),1);
    assert_eq!(claims["claims"][0]["verification"],"declared_unverified");
    assert!(refs["refs"].as_array().unwrap().iter()
        .any(|r|r["label"]=="查看定理" && r["resolution"]=="claim"));
    let index:serde_json::Value=serde_json::from_slice(
        &fs::read(out.join("proof-index.json")).unwrap()).unwrap();
    assert_eq!(index["claims"].as_array().unwrap().len(),0);
    let generated=fs::read_to_string(out.join("CURRENT")).unwrap();
    assert!(out.join("generations").join(generated.trim())
        .join("anchors.json").is_file());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn fake_lean_status_with_no_matching_certificate_cannot_be_proved() {
    let dir=root("forge");
    let file=dir.join("main.mq.md");
    fs::write(&file,"# main\n\n## compute\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| rule | 结果 = 6 | | auto |\n\n**x = [演算](2 * 3)**\n*x*\n").unwrap();
    let compiled=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let lean=LeanInfo{
        toolchain:"Lean 4.35".into(),library_sha256:"lib".into(),
        module_sha256:"module".into(),cache:"miss"
    };
    let receipt=link_report(&compiled,&file,
        &report(&file,Verdict::Proved,"lean",Some(lean)),&dir);
    assert_eq!(receipt["claims"][0]["status"],"unverified");
    assert_eq!(receipt["claims"][0]["certificate_digest"],serde_json::Value::Null);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn real_prove_pipeline_emits_function_bound_knowledge_sidecar_without_lean() {
    let dir=root("pipeline");
    let file=dir.join("main.mq.md");
    fs::write(&file,r#"---
markup: occlusion
---

# main

## compute

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 无外部操作 | 长度(轨迹) = 0 | | auto |

**x = [演算](2 * 3)**
*x*
"#).unwrap();
    let out=dir.join(".marqdo/prove/manual");
    let opts=marqdo::prove::ProveOptions {
        emit_dir:Some(out.clone()),judge:false,..Default::default()
    };
    let report=marqdo::prove::prove_path(&file,&opts).unwrap();
    assert!(report.functions.iter().any(|f|!f.claims.is_empty()));
    let linkage:serde_json::Value=serde_json::from_slice(
        &fs::read(out.join("knowledge-links.json")).unwrap()).unwrap();
    assert_eq!(linkage["schema_version"],1);
    assert_eq!(linkage["claims"][0]["claim_id"],"无外部操作");
    assert_eq!(linkage["claims"][0]["proof_scope"],"function");
    assert_ne!(linkage["claims"][0]["status"],"proved");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn stale_duplicate_claims_are_never_selected_by_position() {
    let dir=root("duplicate");
    let file=dir.join("main.mq.md");
    fs::write(&file,"# main\n\n## compute\n\n| 定理 | 结论 | 前提 | 策略 |\n|---|---|---|---|\n| rule | 结果 = 6 | | auto |\n\n**x = [演算](2 * 3)**\n*x*\n").unwrap();
    let compiled=compile_project(&CompileOptions::for_path(&dir).unwrap()).unwrap();
    let receipt=link_report(&compiled,&file,&report(&file,Verdict::Checked,"trace",None),&dir);
    for name in ["a","b"] {
        let out=dir.join(".marqdo/prove").join(name);
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("knowledge-links.json"),serde_json::to_vec(&receipt).unwrap()).unwrap();
    }
    let index=index_links(&dir.join(".marqdo/prove"),&compiled.semantics);
    assert_eq!(index["claims"][0]["status"],"ambiguous");
    assert_eq!(index["claims"][1]["status"],"ambiguous");
    assert_eq!(index["claims"][0]["usable"],false);
    fs::remove_dir_all(dir).unwrap();
}

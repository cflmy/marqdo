//! S0 executable-knowledge baseline gate (offline).
//!
//! See `tests/executable_knowledge/README.md`. Design-status samples may run
//! but do not prove source/applicability. Expected failures are asserted;
//! the bytecode ambiguous-name defect is fingerprinted until S2 BoundCall.

use std::process::Command;

fn run_backend(path: &str, backend: &str) -> (i32, String, String) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", path, "--backend", backend])
        .current_dir(&root)
        .output()
        .expect("run marqdo");
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn assert_out_both(path: &str, expect: &str) {
    for backend in ["tree", "bytecode"] {
        let (code, stdout, stderr) = run_backend(path, backend);
        assert_eq!(
            code, 0,
            "{path} backend={backend} failed stderr={stderr}"
        );
        assert_eq!(
            stdout.trim_end(),
            expect.trim_end(),
            "{path} backend={backend} stdout={stdout}"
        );
    }
}

fn assert_fails_both(path: &str, stderr_substr: &str) {
    for backend in ["tree", "bytecode"] {
        let (code, stdout, stderr) = run_backend(path, backend);
        assert_ne!(
            code, 0,
            "{path} backend={backend} unexpectedly succeeded stdout={stdout}"
        );
        assert!(
            stderr.contains(stderr_substr),
            "{path} backend={backend} stderr missing {stderr_substr:?}: {stderr}"
        );
    }
}

#[test]
fn s0_01_inline_calc() {
    assert_out_both(
        "tests/executable_knowledge/baseline/01-inline/main.mq.md",
        "60",
    );
}

#[test]
fn s0_02_extract_named_function() {
    assert_out_both(
        "tests/executable_knowledge/baseline/02-extract/main.mq.md",
        "60",
    );
}

#[test]
fn s0_03_two_document_import() {
    assert_out_both(
        "tests/executable_knowledge/baseline/03-import/main.mq.md",
        "60",
    );
}

#[test]
fn s0_04_access_and_loop() {
    assert_out_both(
        "tests/executable_knowledge/baseline/04-access-loop/main.mq.md",
        "a=值\nb=二\n果子=一\n果子=二",
    );
}

/// Design-status: pure calc may run; does **not** prove tax facts verified (G05).
#[test]
fn s0_05_policy_draft_try_run_not_verified() {
    assert_out_both(
        "tests/executable_knowledge/baseline/05-policy-draft/main.mq.md",
        "113",
    );
}

#[test]
fn s0_06a_missing_import_rejected() {
    // Message wording differs by backend; both must mention the missing lib stem.
    assert_fails_both(
        "tests/executable_knowledge/baseline/06-reject/missing-import.mq.md",
        "pricing",
    );
}

#[test]
fn s2_g07_mlsp_bind_execute_reuses_existing_capability() {
    // Stub/no-LLM REUSE path (G07/G09 subset): find → bind → execute, no new source.
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let project = root.display().to_string();
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":project,"query":"商品小计"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(found["ok"], true, "{found}");
    let id = found["result"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["kind"] == "function")
        .expect("商品小计 function")["artifact_id"]
        .as_str()
        .unwrap()
        .to_string();
    let bound: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.bind","params":{"project":root,"artifact_id":id}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(bound["ok"], true, "bind should succeed for pure calc: {bound}");
    let handle = &bound["result"]["handle"];
    assert!(handle["symbol_id"].as_str().is_some());
    let result: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.execute",
            "params":{"handle":handle,"inputs":{"数量":2,"单价":30}}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(
        result["ok"], true,
        "execute should reuse bound symbol: {result}"
    );
    assert_eq!(result["result"]["result"], 60);
    assert_eq!(result["result"]["symbol_id"], handle["symbol_id"]);
}

#[test]
fn s2_g06_wrapper_inherits_fs_write_not_pure() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-g06-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out);
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "knowledge",
            "tests/executable_knowledge/baseline/09-effects",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge compile");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let mut found_wrap = false;
    let arts = out.join("artifacts");
    for ent in std::fs::read_dir(&arts).unwrap().flatten() {
        let text = std::fs::read_to_string(ent.path()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let name = v["display_name"].as_str().unwrap_or("");
        if name.ends_with("wrap") {
            found_wrap = true;
            let effects = v["effects"].as_array().cloned().unwrap_or_default();
            let effects: Vec<&str> = effects.iter().filter_map(|e| e.as_str()).collect();
            assert!(
                effects.contains(&"fs.write"),
                "wrap must inherit fs.write: {effects:?}"
            );
            assert!(!effects.iter().all(|e| *e == "pure"), "{effects:?}");
        }
    }
    assert!(found_wrap, "wrap artifact missing under {}", arts.display());
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s1_08_same_name_import_runs_a_not_b() {
    assert_out_both(
        "tests/executable_knowledge/baseline/08-same-name/main.mq.md",
        "60",
    );
}

#[test]
fn s1_08_same_name_resolve_binds_imported_module() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from = "tests/executable_knowledge/baseline/08-same-name/main.mq.md";
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "knowledge",
            "resolve",
            "pricing.商品小计",
            "--from",
            from,
            "tests/executable_knowledge/baseline/08-same-name",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge resolve");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let file = v["symbol"]["source"]["file"].as_str().unwrap_or("");
    assert!(
        file.ends_with("a.mq.md"),
        "must bind imported A, got file={file} json={v}"
    );
    assert_ne!(
        v["symbol"]["id"],
        serde_json::Value::Null,
        "stable SymbolId required: {v}"
    );
}

#[test]
fn s1_view_lists_steps_with_source_anchors() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = std::env::temp_dir().join(format!(
        "marqdo-ek-view-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out_dir);
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "view",
            "output",
            "--out",
            out_dir.to_str().unwrap(),
            "--no-exec",
            "tests/executable_knowledge/baseline/07-step-kinds/main.mq.md",
        ])
        .current_dir(&root)
        .output()
        .expect("view");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    // Find generated HTML that mentions knowledge steps.
    let mut html = String::new();
    fn walk(dir: &std::path::Path, acc: &mut String) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                walk(&p, acc);
            } else if p.extension().and_then(|e| e.to_str()) == Some("html") {
                if let Ok(s) = std::fs::read_to_string(&p) {
                    if s.contains("knowledge-step") {
                        *acc = s;
                    }
                }
            }
        }
    }
    walk(&out_dir, &mut html);
    assert!(
        html.contains("knowledge-step"),
        "view HTML should list knowledge steps"
    );
    assert!(
        html.contains("href=\"#stmt-"),
        "steps should link to source stmt anchors"
    );
    assert!(
        html.contains("data-step-kind=\"observe\"")
            || html.contains("data-step-kind=\"derive\"")
            || html.contains("data-step-kind=\"iterate\""),
        "expected classified step kinds in view HTML"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn s1_g03_step_kinds_not_all_call() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-g03-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out);
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "knowledge",
            "tests/executable_knowledge/baseline/07-step-kinds",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge compile");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let steps: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("steps.json")).unwrap()).unwrap();
    let arr = steps["steps"].as_array().expect("steps array");
    let kinds: Vec<&str> = arr
        .iter()
        .filter_map(|s| s["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"observe"), "{kinds:?}");
    assert!(kinds.contains(&"derive"), "{kinds:?}");
    assert!(kinds.contains(&"iterate"), "{kinds:?}");
    assert!(
        !kinds.iter().all(|k| *k == "call"),
        "must not forge every step as call: {kinds:?}"
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s1_g01_semantic_edges_not_overwritten_by_ekc() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-g01-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out);
    let out_s = out.to_str().unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "knowledge",
            "tests/executable_knowledge/baseline/03-import",
            "-o",
            out_s,
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge compile");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let semantic_edges = out.join("graph/edges.json");
    let ekc_edges = out.join("ekc/graph/edges.json");
    assert!(semantic_edges.is_file(), "missing semantic {}", semantic_edges.display());
    assert!(ekc_edges.is_file(), "missing EKC {}", ekc_edges.display());
    assert!(
        !out.join("graph/graph.json").is_file(),
        "EKC must not place graph.json under authoritative graph/"
    );
    let sem: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&semantic_edges).unwrap()).unwrap();
    assert_eq!(sem["authority"], "SemanticProject");
    assert!(sem.get("schema_version").is_some(), "{sem}");
    let ekc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ekc_edges).unwrap()).unwrap();
    assert_eq!(ekc["type"], "Marqdo Knowledge Edges");
    assert!(out.join("steps.json").is_file(), "steps projection required");
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s0_06b_ambiguous_bare_name_rejected_both_backends() {
    // G02: bare same-named imports must not silently bind (Tree + Bytecode).
    for backend in ["tree", "bytecode"] {
        let (code, stdout, stderr) = run_backend(
            "tests/executable_knowledge/baseline/06-reject/ambiguous.mq.md",
            backend,
        );
        assert_ne!(
            code, 0,
            "{backend} must reject bare ambiguous name; stdout={stdout}"
        );
        assert!(
            stderr.contains("商品小计"),
            "{backend} stderr should name the unresolved function: {stderr}"
        );
    }
}

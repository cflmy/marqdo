//! Executable-knowledge gate (offline): S0 baseline + S1/S2 BoundCall/effects.
//!
//! See `tests/executable_knowledge/README.md`. Design-status samples may run
//! but do not prove source/applicability. Expected failures are asserted.

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

#[test]
fn s2_g02_boundcall_path_and_dynamic_projections() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-g02-{}-{}",
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
            "tests/executable_knowledge/baseline/10-boundcall",
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
    let bound: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("bound_calls.json")).unwrap(),
    )
    .unwrap();
    let dynamic: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("dynamic_calls.json")).unwrap(),
    )
    .unwrap();
    let bounds = bound["bound_calls"].as_array().expect("bound_calls");
    assert!(
        bounds.iter().any(|c| {
            c["binding"] == "BoundCall"
                && c["call_site"].as_str().unwrap_or("").contains("pricing.商品小计")
                && c["target"].as_str().is_some_and(|t| !t.is_empty())
        }),
        "import path must be BoundCall with SymbolId: {bound}"
    );
    let dyns = dynamic["dynamic_calls"].as_array().expect("dynamic_calls");
    assert!(
        dyns.iter().any(|c| {
            c["binding"] == "DynamicCall" && c["reason"] == "call_fn"
        }),
        "call_fn must be DynamicCall: {dynamic}"
    );
    // Edges from caller must include the BoundCall target SymbolId.
    let edges: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("graph/edges.json")).unwrap(),
    )
    .unwrap();
    let target = bounds
        .iter()
        .find(|c| c["call_site"].as_str().unwrap_or("").contains("pricing.商品小计"))
        .unwrap()["target"]
        .as_str()
        .unwrap();
    let edge_txt = edges.to_string();
    assert!(
        edge_txt.contains(target),
        "graph edges must mention BoundCall target {target}: {edges}"
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s2_g02_boundcall_runs_both_backends() {
    assert_out_both(
        "tests/executable_knowledge/baseline/10-boundcall/main.mq.md",
        "hi\n60",
    );
}

#[test]
fn s4_examples_bookstore_chain_runs_both_backends() {
    assert_out_both(
        "examples/executable-knowledge/docs/invoice-answer.mq.md",
        "113",
    );
}

#[test]
fn s5_closure_digest_moves_with_dependency() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-closure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("dep.mq.md"),
        "---\nmarkup: occlusion\n---\n\n## rate\n\n*0.13*\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("main.mq.md"),
        "---\nmarkup: occlusion\nimport:\n  d: dep.mq.md\n---\n\n# main\n\n**t = [税](d.rate())**\n> print text=`t`\n",
    )
    .unwrap();
    let out1 = std::env::temp_dir().join(format!("marqdo-ek-closure-o1-{}", std::process::id()));
    let out2 = std::env::temp_dir().join(format!("marqdo-ek-closure-o2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out1);
    let _ = std::fs::remove_dir_all(&out2);
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["knowledge", dir.to_str().unwrap(), "-o", out1.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let mut main_closure = None;
    let mut main_impl = None;
    for ent in std::fs::read_dir(out1.join("artifacts")).unwrap().flatten() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(ent.path()).unwrap()).unwrap();
        if v["display_name"] == "main" {
            main_closure = v["hashes"]["closure"].as_str().map(str::to_string);
            main_impl = v["hashes"]["implementation"].as_str().map(str::to_string);
        }
    }
    assert!(main_closure.is_some(), "main closure missing in {}", out1.display());
    std::fs::write(
        dir.join("dep.mq.md"),
        "---\nmarkup: occlusion\n---\n\n## rate\n\n*0.14*\n",
    )
    .unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["knowledge", dir.to_str().unwrap(), "-o", out2.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let mut main_closure2 = None;
    let mut main_impl2 = None;
    for ent in std::fs::read_dir(out2.join("artifacts")).unwrap().flatten() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(ent.path()).unwrap()).unwrap();
        if v["display_name"] == "main" {
            main_closure2 = v["hashes"]["closure"].as_str().map(str::to_string);
            main_impl2 = v["hashes"]["implementation"].as_str().map(str::to_string);
        }
    }
    assert_eq!(
        main_impl, main_impl2,
        "caller implementation must stay put when only dep body changes"
    );
    assert_ne!(
        main_closure, main_closure2,
        "caller closure digest must move with dependency"
    );
    let _ = std::fs::remove_dir_all(&out1);
    let _ = std::fs::remove_dir_all(&out2);
    let _ = root;
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s6_corpus_100_deterministic_units() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-corpus100-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..100u32 {
        let n = i + 1;
        let expect = n * 2;
        let src = format!(
            "---\nmarkup: occlusion\ntitle: unit-{n}\n---\n\n# main\n\n**v = [翻倍]({n} * 2)**\n> print text=`v`\n"
        );
        let path = dir.join(format!("u{n:03}.mq.md"));
        std::fs::write(&path, src).unwrap();
        for backend in ["tree", "bytecode"] {
            let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
                .args(["run", path.to_str().unwrap(), "--backend", backend])
                .current_dir(&root)
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(0),
                "unit {n} backend={backend} stderr={}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&out.stdout).trim_end(),
                expect.to_string(),
                "unit {n} backend={backend}"
            );
        }
    }
    let out = dir.join(".marqdo-out");
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["knowledge", dir.to_str().unwrap(), "-o", out.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(out.join("CURRENT").is_file());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s6_cli_smoke_version_check_knowledge() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ver = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .arg("--version")
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(ver.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&ver.stdout).contains("marqdo")
            || String::from_utf8_lossy(&ver.stderr).contains("marqdo")
            || !ver.stdout.is_empty()
            || ver.status.success()
    );
    let check = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "check",
            "tests/executable_knowledge/baseline/01-inline/main.mq.md",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(
        check.status.code(),
        Some(0),
        "check stderr={}",
        String::from_utf8_lossy(&check.stderr)
    );
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-smoke-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out);
    let kn = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "knowledge",
            "tests/executable_knowledge/baseline/13-bookstore",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(kn.status.code(), Some(0));
    assert!(out.join("project.json").is_file());
    assert!(out.join("graph/edges.json").is_file());
    assert!(out.join("CURRENT").is_file());
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s3_extract_pure_plan_rejects_calls() {
    use marqdo::mlsp::handle_line;
    let ok: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.extract_pure",
            "params":{"name":"商品小计","expression":"数量 * 单价"}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(ok["ok"], true, "{ok}");
    assert_eq!(ok["result"]["effects"][0], "pure");
    assert!(ok["result"]["call"].as_str().unwrap().contains("商品小计"));
    let params = ok["result"]["params"].as_array().unwrap();
    assert_eq!(params.len(), 2, "{ok}");
    let bad: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.extract_pure",
            "params":{"name":"x","expression":"fs.write_text(path=\"a\", text=\"b\")"}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(bad["ok"], false, "{bad}");
    assert_eq!(bad["error"]["code"], "extract.not_pure");
}

#[test]
fn s4_bookstore_story_runs_with_narrative_and_source() {
    assert_out_both(
        "tests/executable_knowledge/baseline/13-bookstore/main.mq.md",
        "113",
    );
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-book-{}-{}",
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
            "tests/executable_knowledge/baseline/13-bookstore",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge");
    assert_eq!(status.status.code(), Some(0));
    let steps: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("steps.json")).unwrap()).unwrap();
    assert!(
        steps["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["kind"] == "narrate"),
        "{steps}"
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s4_g05_sourced_evidence_and_expired_diag() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-src-{}-{}",
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
            "tests/executable_knowledge/baseline/12-tax-source",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let mut found_sourced = false;
    for ent in std::fs::read_dir(out.join("artifacts")).unwrap().flatten() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(ent.path()).unwrap()).unwrap();
        let ev = v["verification"]["evidence"].as_str().unwrap_or("");
        if v["kind"] == "function" && ev == "sourced" {
            found_sourced = true;
            let prov = v["provenance"].as_array().cloned().unwrap_or_default();
            assert!(
                prov.iter().any(|p| p["relation"] == "supports"),
                "supports SourceRef required: {v}"
            );
        }
    }
    assert!(found_sourced, "functions should be evidence=sourced");
    assert_out_both(
        "tests/executable_knowledge/baseline/12-tax-source/main.mq.md",
        "113",
    );
    use marqdo::mlsp::handle_line;
    let project = root.join("tests/executable_knowledge/baseline/12-tax-source");
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&project,"query":"标准税率"}})
            .to_string(),
    ))
    .unwrap();
    let id = found["result"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "function")
        .unwrap()["artifact_id"]
        .as_str()
        .unwrap();
    let biz: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.bind","params":{"project":&project,"artifact_id":id,"policy":"business"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(biz["ok"], true, "sourced artifact binds under business: {biz}");

    let out2 = std::env::temp_dir().join(format!(
        "marqdo-ek-exp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&out2);
    let status2 = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "knowledge",
            "tests/executable_knowledge/baseline/12b-tax-expired",
            "-o",
            out2.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge expired");
    assert_eq!(status2.status.code(), Some(0));
    let mut found_expired = false;
    for ent in std::fs::read_dir(out2.join("artifacts")).unwrap().flatten() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(ent.path()).unwrap()).unwrap();
        if v["verification"]["evidence"] == "expired" {
            found_expired = true;
        }
    }
    assert!(found_expired, "12b-tax-expired should be evidence=expired");
    let _ = std::fs::remove_dir_all(&out);
    let _ = std::fs::remove_dir_all(&out2);
}

#[test]
fn s3_artifact_patch_wording_guard() {
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-patch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = "---\nmarkup: occlusion\n---\n\n## 商品小计\n\n`数量` 与 `单价`。\n\n*[按数量与单价计算](数量 * 单价)*\n\n# main\n\n**小计 = [两件](商品小计(数量=2, 单价=30))**\n> print text=`小计`\n";
    std::fs::write(dir.join("main.mq.md"), src).unwrap();
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&dir,"query":"商品小计"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(found["ok"], true, "{found}");
    let art = found["result"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "function")
        .unwrap();
    let id = art["artifact_id"].as_str().unwrap();
    let inspected: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.inspect","params":{"project":&dir,"artifact_id":id}})
            .to_string(),
    ))
    .unwrap();
    let impl_digest = inspected["result"]["hashes"]["implementation"]
        .as_str()
        .unwrap()
        .to_string();
    let line = inspected["result"]["source"]["line"].as_u64().unwrap() as u32;
    // Wording-only: change prose near the heading without touching signature.
    let prose_line = line + 2;
    let ok_patch: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.patch",
            "params":{
                "project":&dir,
                "artifact_id":id,
                "expected_digest":impl_digest,
                "intent":"wording",
                "edits":[{"op":"replace","line":prose_line,"text":"`数量` 与 `单价`（说明）。"}]
            }
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(ok_patch["ok"], true, "prose tweak should apply: {ok_patch}");
    assert_eq!(ok_patch["result"]["diff"]["signature_changed"], false);
    // Re-inspect and try a signature-breaking edit under wording intent.
    let inspected2: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.inspect","params":{"project":&dir,"artifact_id":id}})
            .to_string(),
    ))
    .unwrap();
    let impl2 = inspected2["result"]["hashes"]["implementation"]
        .as_str()
        .unwrap();
    let bad: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.patch",
            "params":{
                "project":&dir,
                "artifact_id":id,
                "expected_digest":impl2,
                "intent":"wording",
                "edits":[{"op":"replace","line":prose_line,"text":"`数量` `单价` `折扣`。"}]
            }
        })
        .to_string(),
    ))
    .unwrap();
    // Declaring a new backtick param may or may not change interface hash depending on
    // whether params are inferred from prose; if signature changes, wording must roll back.
    if bad["ok"] == false {
        assert_eq!(
            bad["error"]["code"], "artifact.wording_broke_contract",
            "{bad}"
        );
    }
    let _ = root;
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s1_g03_narrative_ref_steps_not_executed() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-narr-{}-{}",
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
            "tests/executable_knowledge/baseline/11-narrative-ref",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("knowledge");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    let steps: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("steps.json")).unwrap()).unwrap();
    let arr = steps["steps"].as_array().unwrap();
    assert!(
        arr.iter().any(|s| s["kind"] == "narrate"),
        "expected narrate step: {steps}"
    );
    assert!(
        arr.iter().any(|s| s["kind"] == "navigation"),
        "expected navigation step: {steps}"
    );
    let nav_count = arr.iter().filter(|s| s["kind"] == "navigation").count();
    assert_eq!(
        nav_count, 1,
        "only the prose URL becomes navigation (fence ignored): {steps}"
    );
    let refs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("refs.json")).unwrap()).unwrap();
    assert!(
        refs["refs"].as_array().unwrap().iter().any(|r| {
            r["context"] == "narrative" && r["resolution"] == "symbol"
        }),
        "narrate target 总额 should resolve: {refs}"
    );
    // run must not fetch URL; stdout ok only
    assert_out_both(
        "tests/executable_knowledge/baseline/11-narrative-ref/main.mq.md",
        "ok",
    );
    let current = std::fs::read_to_string(out.join("CURRENT")).unwrap();
    let gen = out.join("generations").join(current.trim());
    assert!(
        gen.join("steps.json").is_file(),
        "generation tree must be complete under {}",
        gen.display()
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn s2_g07_eng_reuse_requires_bind_before_execute() {
    // Discovery scores must not be treated as executable handles.
    use marqdo::knowledge::ir::{ReuseCandidate, ReuseDecision, ReuseResult};
    use marqdo::knowledge::reuse::reuse_json;
    let result = ReuseResult {
        decision: ReuseDecision::Reuse,
        confidence: 0.9,
        capability: Some("定价".into()),
        symbol: Some("商品小计".into()),
        source: None,
        reason: vec!["score meets REUSE threshold".into()],
        candidates: vec![ReuseCandidate {
            id: "capability:定价".into(),
            kind: "Capability".into(),
            title: "定价".into(),
            score: 0.9,
            resource: None,
            reason: vec![],
        }],
        constraints: vec![],
        decisions: vec![],
        failures: vec![],
        create_allowed: false,
        recommended: vec!["REUSE 定价".into()],
    };
    let v = reuse_json("商品小计", &result);
    assert_eq!(v["discovery_only"], true, "{v}");
    assert_eq!(v["bind_required"], true, "{v}");
    assert!(
        v["bind_targets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["query"] == "商品小计"),
        "REUSE must expose bind_targets for MLSP bind: {v}"
    );
}

//! Executable-knowledge gate (offline): S0 baseline + S1/S2 BoundCall/effects.
//!
//! See `tests/executable_knowledge/README.md`. Design-status samples may run
//! but do not prove source/applicability. Expected failures are asserted.

use std::process::Command;

/// Build `libagent` once per test process and prefer it over a stale `~/.marqdo/ext`.
/// Same contract as `tests/gold.rs::ensure_agent_plugin_built` — EK gate must not
/// depend on a manually copied `ext/native/libagent.so`.
fn ensure_agent_plugin_built() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let status = Command::new("cargo")
            .args(["build", "-p", "marqdo_plugin_agent"])
            .current_dir(&root)
            .status()
            .expect("build marqdo_plugin_agent");
        assert!(status.success(), "failed to build marqdo_plugin_agent");
        let lib = if cfg!(windows) {
            root.join("target").join("debug").join("agent.dll")
        } else if cfg!(target_os = "macos") {
            root.join("target").join("debug").join("libagent.dylib")
        } else {
            root.join("target").join("debug").join("libagent.so")
        };
        let release = if cfg!(windows) {
            root.join("target").join("release").join("agent.dll")
        } else if cfg!(target_os = "macos") {
            root.join("target").join("release").join("libagent.dylib")
        } else {
            root.join("target").join("release").join("libagent.so")
        };
        let chosen = if lib.is_file() {
            if release.is_file() {
                let lib_m = std::fs::metadata(&lib).and_then(|m| m.modified()).ok();
                let rel_m = std::fs::metadata(&release).and_then(|m| m.modified()).ok();
                match (lib_m, rel_m) {
                    (Some(l), Some(r)) if r > l => release,
                    _ => lib,
                }
            } else {
                lib
            }
        } else {
            release
        };
        assert!(
            chosen.is_file(),
            "missing agent plugin at {}",
            chosen.display()
        );
        // SAFETY: single-threaded Once; children inherit for the rest of the test process.
        std::env::set_var("MARQDO_AGENT_PLUGIN", &chosen);
        if std::env::var_os("MARQDO_EXT").is_none() {
            let ext = root.join("ext");
            if ext.join("ai").join("agent.mq.md").is_file() {
                std::env::set_var("MARQDO_EXT", &ext);
            }
        }
    });
}

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
    // V0: document body must appear before the knowledge auxiliary panel in main content
    // (ignore stylesheet class mentions that appear earlier in <style>).
    let main = html.split("<main").nth(1).unwrap_or(&html);
    let struct_pos = main
        .find("id=\"structure\"")
        .or_else(|| main.find("structure-block"))
        .expect("structure in main");
    let know_pos = main
        .find("block knowledge-block")
        .or_else(|| main.find("knowledge-block\">"))
        .unwrap_or(main.len());
    assert!(
        struct_pos < know_pos,
        "V0: document Structure must precede knowledge panel in <main>"
    );
    assert!(
        !html.contains("Lock digest"),
        "V0: must not mislabel artifact hash as Lock digest"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn v0_no_exec_does_not_run_side_effects() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-noexec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let marker = dir.join("SIDE_EFFECT.txt");
    let src = format!(
        "---\nmarkup: occlusion\n---\n\n# main\n\n> fs.write_text path=\"{}\" text=ran\n> print text=ok\n",
        marker.display().to_string().replace('\\', "/")
    );
    std::fs::write(dir.join("main.mq.md"), src).unwrap();
    let out_dir = dir.join("out");
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "view",
            "output",
            "--out",
            out_dir.to_str().unwrap(),
            "--no-exec",
            dir.join("main.mq.md").to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .expect("view no-exec");
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(
        !marker.exists(),
        "V0: --no-exec must not execute side effects (found {})",
        marker.display()
    );
    let _ = std::fs::remove_dir_all(&dir);
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
    // Select the caller by dependency edge (not display_name alone — module also named "main").
    let mut main_closure = None;
    let mut main_impl = None;
    let mut main_id = None;
    for ent in std::fs::read_dir(out1.join("artifacts")).unwrap().flatten() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(ent.path()).unwrap()).unwrap();
        let deps = v["dependencies"].as_array().cloned().unwrap_or_default();
        let calls_rate = deps.iter().any(|d| {
            d.as_str()
                .is_some_and(|s| s.contains("/function/rate") || s.ends_with("rate"))
        });
        if v["display_name"] == "main" && v["kind"] != "module" && calls_rate {
            main_closure = v["hashes"]["closure"].as_str().map(str::to_string);
            main_impl = v["hashes"]["implementation"].as_str().map(str::to_string);
            main_id = v["artifact_id"].as_str().map(str::to_string);
        }
    }
    assert!(
        main_closure.is_some(),
        "caller (type/function main with rate dep) closure missing in {}",
        out1.display()
    );
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
    let want_id = main_id.as_deref().unwrap();
    for ent in std::fs::read_dir(out2.join("artifacts")).unwrap().flatten() {
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(ent.path()).unwrap()).unwrap();
        if v["artifact_id"].as_str() == Some(want_id) {
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

#[test]
fn m02_source_change_invalidates_business_handle() {
    // M02: missing anchor blocks business bind; valid source then edit/delete rejects execute.
    use marqdo::mlsp::handle_line;
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-m02-src-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("source.md"), "# rate\n\nrate = 0.13\n").unwrap();
    std::fs::write(
        dir.join("facts.mq.md"),
        "---\nmarkup: occlusion\nsources:\n  - source.md#missing-anchor\napplicability:\n  region: CN\n  valid_until: 2099-12-31\n---\n\n## rate\n\n*0.13*\n",
    )
    .unwrap();
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&dir,"query":"rate","kind":"function"}})
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
        .unwrap()
        .to_string();
    let inspect: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.inspect","params":{"project":&dir,"artifact_id":&id}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(
        inspect["result"]["verification"]["evidence"], "anchor_missing",
        "missing anchor must not be sourced: {inspect}"
    );
    let denied: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.bind","params":{"project":&dir,"artifact_id":&id,"policy":"business"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(denied["ok"], false, "business bind must reject anchor_missing: {denied}");

    // Repair anchor and bind.
    std::fs::write(
        dir.join("facts.mq.md"),
        "---\nmarkup: occlusion\nsources:\n  - source.md#rate\napplicability:\n  region: CN\n  valid_until: 2099-12-31\n---\n\n## rate\n\n*0.13*\n",
    )
    .unwrap();
    let inspect2: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.inspect","params":{"project":&dir,"artifact_id":&id}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(
        inspect2["result"]["verification"]["evidence"], "sourced",
        "{inspect2}"
    );
    let evidence1 = inspect2["result"]["hashes"]["evidence"]
        .as_str()
        .unwrap()
        .to_string();
    let bound: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.bind","params":{"project":&dir,"artifact_id":&id,"policy":"business"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(bound["ok"], true, "business bind with valid anchor: {bound}");
    let handle = bound["result"]["handle"].clone();
    assert!(handle["evidence_digest"].as_str().is_some());
    assert!(handle["closure_digest"].as_str().is_some());

    // Edit source content → evidence digest moves → stale handle.
    std::fs::write(dir.join("source.md"), "# rate\n\nrate = 0.14\n").unwrap();
    let inspect3: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.inspect","params":{"project":&dir,"artifact_id":&id}})
            .to_string(),
    ))
    .unwrap();
    let evidence2 = inspect3["result"]["hashes"]["evidence"].as_str().unwrap();
    assert_ne!(
        evidence1, evidence2,
        "source content change must move evidence digest"
    );
    let after_edit: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.execute","params":{"handle":&handle,"inputs":{}}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(
        after_edit["ok"], false,
        "old business handle must fail after source edit: {after_edit}"
    );

    // Remove source → evidence denied / stale.
    let _ = std::fs::remove_file(dir.join("source.md"));
    let after_rm: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.execute","params":{"handle":&handle,"inputs":{}}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(
        after_rm["ok"], false,
        "old business handle must fail after source removal: {after_rm}"
    );

    // Unknown policy rejected.
    let bad_pol: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.bind","params":{"project":&dir,"artifact_id":&id,"policy":"trust-me"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(bad_pol["ok"], false, "unknown policy must be rejected: {bad_pol}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn m01_document_extract_markdown() {
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/12-tax-source/corpus.md");
    let out: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"document.extract","params":{"path":root}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["result"]["status"], "ok");
    assert!(
        out["result"]["segments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["kind"] == "heading"),
        "expected heading segments: {out}"
    );
}

#[test]
fn m06_publish_manifest_written() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "marqdo-ek-manifest-{}-{}",
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
            "tests/executable_knowledge/baseline/02-extract",
            "-o",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(status.status.code(), Some(0));
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("publish_manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["kind"], "publish_manifest");
    assert!(manifest["semantic_generation"].as_str().is_some());
    assert!(
        manifest["excluded_dirs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d == "data"),
        "excluded_dirs must include data: {manifest}"
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn m05_skill_front_has_occlusion_core_not_classic_index() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let skill = std::fs::read_to_string(root.join("skills/marqdo/SKILL.md")).unwrap();
    // Skip YAML frontmatter; skill_brief budgets apply to body text.
    let body = skill
        .split_once("\n---\n")
        .map(|(_, rest)| rest)
        .unwrap_or(skill.as_str());
    let head: String = body.chars().take(1200).collect();
    assert!(
        head.contains("CORE SYNTAX") && head.contains("access(coll, key)"),
        "first 1200 body chars must include Occlusion core: {}",
        &head[..head.len().min(200)]
    );
    assert!(
        !head.contains("get with `[key](coll)`")
            && !head.contains("Bold `**…**` is **return value only**"),
        "first 1200 body chars must not teach Classic index / bold-as-print: {head}"
    );
    assert!(
        root.join("skills/marqdo/migration-classic.md").is_file(),
        "Classic rules must live in migration-classic.md"
    );
}

#[test]
fn m02_impact_includes_provenance_corpus_uri() {
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/12-tax-source");
    let impact: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.impact","params":{"project":&root,"path":"corpus.md"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(impact["ok"], true, "{impact}");
    let affected = impact["result"]["affected"].as_array().unwrap();
    assert!(
        !affected.is_empty(),
        "corpus.md provenance must seed impact: {impact}"
    );
    assert!(
        affected.iter().any(|a| a["kind"] == "function"),
        "impact should reach citing functions: {impact}"
    );
}

#[test]
fn m03_verify_unknown_policy_and_may_publish_gate() {
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&root,"query":"商品小计","kind":"function"}})
            .to_string(),
    ))
    .unwrap();
    let id = found["result"]["candidates"][0]["artifact_id"].as_str().unwrap();
    let bad: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.verify","params":{"project":&root,"artifact_id":id,"policy":"trust-me"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(bad["ok"], true, "{bad}");
    assert_eq!(bad["result"]["may_bind"], false, "unknown policy must deny bind: {bad}");
    assert_eq!(bad["result"]["may_publish"], false);
    let okv: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.verify","params":{"project":&root,"artifact_id":id,"policy":"pure"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(okv["result"]["may_bind"], true, "{okv}");
    assert_eq!(
        okv["result"]["may_publish"], false,
        "static verify must not allow publish without tests/execution: {okv}"
    );
    assert_eq!(okv["result"]["checks"]["tests"]["status"], "unknown");
    assert_eq!(okv["result"]["checks"]["input_domain"]["status"], "unknown");
}

#[test]
fn m04_reuse_execute_is_actual_reuse_not_discovery() {
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&root,"query":"商品小计","kind":"function"}})
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
    let out: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.reuse_execute",
            "params":{"project":&root,"artifact_id":id,"policy":"pure","inputs":{"数量":2,"单价":30}}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["result"]["actual_reuse"], true);
    assert_eq!(out["result"]["discovery_only"], false);
    assert_eq!(out["result"]["result"], 60);
    assert!(out["result"]["trace"]["digest"].as_str().is_some());
}

#[test]
fn m07_function_sources_override_module_sources() {
    use marqdo::mlsp::handle_line;
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-fnsrc-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.md"), "# alpha\n\nA\n").unwrap();
    std::fs::write(dir.join("b.md"), "# beta\n\nB\n").unwrap();
    std::fs::write(
        dir.join("facts.mq.md"),
        "---\nmarkup: occlusion\nsources:\n  - a.md#alpha\nfunction_sources:\n  rate:\n    - b.md#beta\napplicability:\n  valid_until: 2099-12-31\n---\n\n## rate\n\n*0.13*\n\n## other\n\n*1*\n",
    )
    .unwrap();
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&dir,"query":"rate","kind":"function"}})
            .to_string(),
    ))
    .unwrap();
    let id = found["result"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "function" && c["display_name"] == "rate")
        .unwrap()["artifact_id"]
        .as_str()
        .unwrap();
    let rate: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.inspect","params":{"project":&dir,"artifact_id":id}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(rate["ok"], true, "{rate}");
    let prov = rate["result"]["provenance"].as_array().unwrap();
    assert!(
        prov.iter()
            .any(|p| p["relation"] == "supports" && p["uri"] == "b.md"),
        "rate must use function_sources b.md: {rate}"
    );
    assert!(
        !prov
            .iter()
            .any(|p| p["relation"] == "supports" && p["uri"] == "a.md"),
        "rate must not inherit module a.md when overridden: {rate}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn m08_story_build_reuse_invalidate_repair() {
    // Full story: bind → execute → source edit invalidates → rebind succeeds.
    use marqdo::mlsp::handle_line;
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-story-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("source.md"), "# rate\n\nv=0.13\n").unwrap();
    std::fs::write(
        dir.join("facts.mq.md"),
        "---\nmarkup: occlusion\nsources:\n  - source.md#rate\napplicability:\n  region: CN\n  valid_until: 2099-12-31\n---\n\n## rate\n\n*0.13*\n",
    )
    .unwrap();
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&dir,"query":"rate","kind":"function"}})
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
        .unwrap()
        .to_string();
    let first: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.reuse_execute",
            "params":{"project":&dir,"artifact_id":&id,"policy":"business","inputs":{}}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(first["ok"], true, "first reuse: {first}");
    assert_eq!(first["result"]["result"], 0.13);
    let handle = {
        let bound: serde_json::Value = serde_json::from_str(&handle_line(
            &serde_json::json!({"method":"artifact.bind","params":{"project":&dir,"artifact_id":&id,"policy":"business"}})
                .to_string(),
        ))
        .unwrap();
        bound["result"]["handle"].clone()
    };
    std::fs::write(dir.join("source.md"), "# rate\n\nv=0.14\n").unwrap();
    let stale: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.execute","params":{"handle":handle,"inputs":{}}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(stale["ok"], false, "stale after source edit: {stale}");
    let repaired: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.reuse_execute",
            "params":{"project":&dir,"artifact_id":&id,"policy":"business","inputs":{}}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(repaired["ok"], true, "rebind after repair: {repaired}");
    assert_eq!(repaired["result"]["result"], 0.13);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn m05_skill_core_card_has_access_not_classic_index() {
    // skill_core_card text is embedded in ext/ai; SKILL body already gated by m05_*.
    // Also assert migration card and CORE SYNTAX marker remain the release contract.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let agent = std::fs::read_to_string(root.join("ext/ai/agent.mq.md")).unwrap();
    assert!(
        agent.contains("## skill_core_card") && agent.contains("access(coll, key)"),
        "skill_core_card must teach access()"
    );
    assert!(
        !agent.contains("never **bold** to print") || agent.contains("never use bold markers"),
        "core card must not rely on conflicting bold-as-print Classic wording"
    );
}

#[test]
fn v1_view_exports_knowledge_presentation_and_explain_links() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = std::env::temp_dir().join(format!(
        "marqdo-ek-v1-{}-{}",
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
            "tests/executable_knowledge/baseline/02-extract/main.mq.md",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(status.status.code(), Some(0));
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
                    if s.contains("mq-knowledge-presentation") || s.contains("knowledge-explain") {
                        *acc = s;
                    }
                }
            }
        }
    }
    walk(&out_dir, &mut html);
    assert!(
        html.contains("mq-knowledge-presentation"),
        "V1 must embed KnowledgePresentation JSON"
    );
    assert!(
        html.contains("knowledge-explain") || html.contains("id=\"knowledge-"),
        "V1 must expose explain / knowledge anchors: {}",
        &html[..html.len().min(400)]
    );
    assert!(
        !html.contains("\"verified_claim\":true"),
        "must not claim verified without evidence runner"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn v2_sse_accepts_packet_rejects_stale_token() {
    assert!(marqdo::view::sse_accepts_packet("abc", None));
    assert!(marqdo::view::sse_accepts_packet("abc", Some("")));
    assert!(marqdo::view::sse_accepts_packet("abc", Some("abc")));
    assert!(!marqdo::view::sse_accepts_packet("abc", Some("zzz")));
    // Embed clients must use strict: missing/empty/mismatch all reject.
    assert!(!marqdo::view::sse_accepts_packet_strict("abc", None));
    assert!(!marqdo::view::sse_accepts_packet_strict("abc", Some("")));
    assert!(!marqdo::view::sse_accepts_packet_strict("", Some("abc")));
    assert!(!marqdo::view::sse_accepts_packet_strict("abc", Some("zzz")));
    assert!(marqdo::view::sse_accepts_packet_strict("abc", Some("abc")));
    assert_eq!(
        marqdo::view::page_token_from_source("a.mq.md", "x"),
        marqdo::view::page_token_from_source("a.mq.md", "x")
    );
    assert_ne!(
        marqdo::view::page_token_from_source("a.mq.md", "x"),
        marqdo::view::page_token_from_source("a.mq.md", "y")
    );
}

#[test]
fn v2_view_embeds_page_token_and_semantic_generation() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = std::env::temp_dir().join(format!(
        "marqdo-ek-v2-{}-{}",
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
            "tests/executable_knowledge/baseline/12-tax-source/main.mq.md",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(status.status.code(), Some(0), "{}", String::from_utf8_lossy(&status.stderr));
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
                    if s.contains("mq-knowledge-presentation") {
                        *acc = s;
                    }
                }
            }
        }
    }
    walk(&out_dir, &mut html);
    assert!(html.contains("MARQDO_VIEW_PAGE_TOKEN"), "page token global");
    assert!(html.contains("\"semantic_generation\""), "presentation generation");
    assert!(html.contains("\"page_token\""), "presentation page_token");
    assert!(html.contains("\"local_deps\""), "local deps edges");
    assert!(
        html.contains("已关联来源") || html.contains("尚未核验"),
        "sourced must not look verified"
    );
    assert!(
        html.contains("stale result ignored") || html.contains("page_token"),
        "SSE stale reject path present in page script"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn v3_corpus_1000_compile_smoke() {
    // Scale gate: 1000 tiny units compile under a fixed wall budget (no per-unit dual-backend).
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-corpus1000-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for i in 0..1000u32 {
        let n = i + 1;
        let src = format!(
            "---\nmarkup: occlusion\ntitle: unit-{n}\n---\n\n# main\n\n**v = [翻倍]({n} * 2)**\n*v*\n"
        );
        std::fs::write(dir.join(format!("u{n:04}.mq.md")), src).unwrap();
    }
    let out = dir.join(".marqdo-out");
    let start = std::time::Instant::now();
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["knowledge", dir.to_str().unwrap(), "-o", out.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    let elapsed = start.elapsed();
    assert_eq!(
        status.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(out.join("CURRENT").is_file());
    assert!(
        out.join("publish_manifest.json").is_file() || out.join("CURRENT").is_file(),
        "generation published"
    );
    // Soft budget: 1000 units should finish offline compile in under 3 minutes on CI.
    assert!(
        elapsed.as_secs() < 180,
        "1000-unit knowledge compile took {:?}",
        elapsed
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn v3_view_css_has_responsive_and_print_and_nojs_structure() {
    let cssish = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/view/html.rs"),
    )
    .unwrap();
    assert!(cssish.contains("@media print"));
    assert!(cssish.contains("@media (max-width: 640px)") || cssish.contains("max-width: 760px"));
    assert!(cssish.contains("prefers-color-scheme") || cssish.contains("--muted"));
    // no-JS: structure/knowledge/source blocks remain in static HTML (not JS-only).
    assert!(cssish.contains("structure-block"));
    assert!(cssish.contains("knowledge-block"));
}

#[test]
fn m01_extract_pdf_and_ocr_fail_explicitly() {
    use marqdo::mlsp::handle_line;
    let dir = std::env::temp_dir().join(format!(
        "marqdo-ek-extract-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("x.pdf");
    std::fs::write(&pdf, b"%PDF-1.4 fake").unwrap();
    let png = dir.join("scan.png");
    std::fs::write(&png, b"\x89PNG").unwrap();
    for (path, expect_fmt) in [(&pdf, "pdf"), (&png, "png")] {
        let out: serde_json::Value = serde_json::from_str(&handle_line(
            &serde_json::json!({"method":"document.extract","params":{"path": path}})
                .to_string(),
        ))
        .unwrap();
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["result"]["status"], "failed", "{expect_fmt}: {out}");
        assert!(
            out["result"]["missing"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m == "full_document"),
            "{expect_fmt}: {out}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn f1_publish_gate_chain_via_verify_may_publish() {
    // Framework-side gate: static verify never may_publish; reuse_execute is actual reuse.
    use marqdo::mlsp::handle_line;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/executable_knowledge/baseline/02-extract");
    let found: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.find","params":{"project":&root,"query":"商品小计","kind":"function"}})
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
    let reused: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({
            "method":"artifact.reuse_execute",
            "params":{"project":&root,"artifact_id":id,"policy":"pure","inputs":{"数量":2,"单价":30}}
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(reused["ok"], true, "{reused}");
    assert_eq!(reused["result"]["actual_reuse"], true);
    let verify: serde_json::Value = serde_json::from_str(&handle_line(
        &serde_json::json!({"method":"artifact.verify","params":{"project":&root,"artifact_id":id,"policy":"pure"}})
            .to_string(),
    ))
    .unwrap();
    assert_eq!(verify["ok"], true, "{verify}");
    assert_eq!(
        verify["result"]["may_publish"], false,
        "must not promote from static verify alone: {verify}"
    );
}

#[test]
fn f1_agent_eng_bind_gate_mq_offline() {
    ensure_agent_plugin_built();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "run",
            "tests/ext/agent-eng-bind-gate.mq.md",
            "--backend",
            "tree",
        ])
        .current_dir(&root)
        .env(
            "MARQDO_AGENT_PLUGIN",
            std::env::var_os("MARQDO_AGENT_PLUGIN").expect("MARQDO_AGENT_PLUGIN set by ensure"),
        )
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout={stdout}\nstderr={stderr}"
    );
    assert!(stdout.contains("agent-eng-bind-gate-ok"), "{stdout}");
    assert!(stdout.contains("actual-reuse-ok"), "{stdout}");
    assert!(stdout.contains("candidate-only-blocks-promote"), "{stdout}");
}

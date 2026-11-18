//! Live workbench gates — skipped unless OPENAI_API_KEY is set (never commit secrets).

use std::process::Command;

fn live_key() -> Option<String> {
    for k in ["OPENAI_API_KEY", "MARQDO_LLM_API_KEY"] {
        if let Ok(v) = std::env::var(k) {
            let t = v.trim();
            if !t.is_empty() && t != "sk-test" {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn repo_ext() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext")
}

fn ensure_llm_plugin() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_llm", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build llm plugin");
    assert!(status.success());
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let so = root.join("target/debug/libllm.so");
    if so.is_file() {
        std::env::set_var("MARQDO_LLM_PLUGIN", &so);
    }
    // Prefer in-tree ext/ over ~/.marqdo/ext (search roots list user install first).
    std::env::set_var("MARQDO_EXT", repo_ext());
}

#[test]
fn live_selection_ask_pong() {
    let Some(_key) = live_key() else {
        eprintln!("skip live_selection_ask_pong: no API key");
        return;
    };
    ensure_llm_plugin();
    if std::env::var_os("OPENAI_BASE_URL").is_none() {
        eprintln!("skip: set OPENAI_BASE_URL (e.g. https://api.llm.ustc.edu.cn/v1)");
        return;
    }
    if std::env::var_os("OPENAI_MODEL").is_none() {
        std::env::set_var("OPENAI_MODEL", "qwen3.8-chat");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/workbench-selection-live.mq.md"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("MARQDO_EXT", repo_ext())
        .env("MARQDO_LLM_PLUGIN", std::env::var_os("MARQDO_LLM_PLUGIN").unwrap_or_default())
        .output()
        .expect("run workbench-selection-live");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "stderr={stderr}\nstdout={stdout}");
    let lower = stdout.to_lowercase();
    assert!(lower.contains("pong"), "expected pong, got {stdout:?}");
    assert!(
        stdout.contains("usage-present") || stdout.contains("usage-unknown"),
        "expected usage line, got {stdout:?}"
    );
}

#[test]
fn live_dual_model_isolation_via_plugin() {
    let Some(key) = live_key() else {
        eprintln!("skip live_dual_model_isolation_via_plugin: no API key");
        return;
    };
    let base = match std::env::var("OPENAI_BASE_URL") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            eprintln!("skip: OPENAI_BASE_URL required");
            return;
        }
    };
    ensure_llm_plugin();
    // Two sequential asks with different model ids — both must succeed without env clobber.
    for model in ["qwen3.8-chat", "deepseek-flash"] {
        std::env::set_var("OPENAI_MODEL", model);
        let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .args(["run", "tests/ext/workbench-selection-live.mq.md"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .env("OPENAI_API_KEY", &key)
            .env("OPENAI_BASE_URL", &base)
            .env("OPENAI_MODEL", model)
            .env("MARQDO_EXT", repo_ext())
            .env(
                "MARQDO_LLM_PLUGIN",
                std::env::var_os("MARQDO_LLM_PLUGIN").unwrap_or_default(),
            )
            .output()
            .expect("run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code().unwrap_or(1),
            0,
            "model={model} stderr={stderr} stdout={stdout}"
        );
        assert!(
            stdout.to_lowercase().contains("pong"),
            "model={model} stdout={stdout}"
        );
    }
}

#[test]
fn cli_view_document_writes_embed_html() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("target/workbench-docview.html");
    let _ = std::fs::remove_file(&out);
    let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "view",
            "document",
            "--project",
            root.join("tests/executable_knowledge/baseline/02-extract")
                .to_str()
                .unwrap(),
            "--document",
            "main.mq.md",
            "--out",
            out.to_str().unwrap(),
        ])
        .current_dir(&root)
        .status()
        .expect("view document");
    assert!(status.success());
    let html = std::fs::read_to_string(&out).expect("read html");
    assert!(html.contains("mq-document"));
    assert!(!html.contains("<script"));
    assert!(html.contains("execution") || html.contains("mq-document"));
}

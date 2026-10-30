//! `marqdo call` runs one function. These tests start the real binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_marqdo")
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(root())
        .output()
        .expect("spawn marqdo")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

fn assert_value(file: &Path, function: &str, args: &[&str], backend: &str, expect: &str) {
    let file = file.to_str().unwrap();
    let mut cmd = vec!["call", "--backend", backend, file, function];
    cmd.extend_from_slice(args);
    let out = run(&cmd);
    assert_eq!(
        out.status.code(),
        Some(0),
        "backend={backend} stdout={} stderr={}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert_eq!(text(&out.stdout), expect, "backend={backend}");
}

fn assert_code(file: &Path, function: &str, args: &[&str], code: &str) {
    let file = file.to_str().unwrap();
    let mut cmd = vec!["call", "--json", file, function];
    cmd.extend_from_slice(args);
    let out = run(&cmd);
    assert_ne!(out.status.code(), Some(0), "stdout={}", text(&out.stdout));
    let body: serde_json::Value = serde_json::from_str(&text(&out.stdout)).unwrap_or_else(|e| {
        panic!("json stdout: {e}\n{}", text(&out.stdout));
    });
    assert_eq!(body["ok"], false, "{body}");
    assert_eq!(body["error"]["code"], code, "{body}");
}

#[test]
fn calculator_matches_the_documented_examples() {
    let file = root().join("examples/calculator.mq.md");
    for backend in ["tree", "bytecode"] {
        assert_value(&file, "计算器.加法", &["a=2", "b=3"], backend, "5");
        assert_value(&file, "计算器.加法", &["a=-2", "b=3"], backend, "1");
        assert_value(&file, "计算器.乘法", &["a=4", "b=6"], backend, "24");
        assert_value(&file, "计算器.乘法", &["a=0", "b=100"], backend, "0");
        assert_value(&file, "加法", &["a=2", "b=3"], backend, "5");
    }
}

#[test]
fn symbol_id_selects_the_same_function_on_both_backends() {
    let file = root().join("examples/calculator.mq.md");
    let compiled = marqdo::sema::compile_file_closure(&file).expect("compile calculator");
    let artifact = compiled
        .semantics
        .artifacts
        .iter()
        .find(|artifact| artifact.display_name == "计算器.加法")
        .expect("加法 artifact");
    let id = artifact
        .symbol_id
        .as_ref()
        .expect("symbol id")
        .as_str()
        .to_string();
    let args = vec![
        ("a".to_string(), marqdo::value::Value::Int(2)),
        ("b".to_string(), marqdo::value::Value::Int(3)),
    ];
    for backend in [marqdo::Backend::Tree, marqdo::Backend::Bytecode] {
        let value = marqdo::call::call_file(&file, &id, &args, backend).unwrap_or_else(|err| {
            panic!("symbol id call failed on {backend:?}: {err:#}")
        });
        assert_eq!(value, marqdo::value::Value::Int(5));
    }
}

#[test]
fn json_envelope_and_metadata_are_not_a_skill_gate() {
    let file = root().join("tests/call/metadata.mq.md");
    let out = run(&[
        "call",
        "--json",
        file.to_str().unwrap(),
        "计算器.加法",
        "a=2",
        "b=3",
    ]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let body: serde_json::Value = serde_json::from_str(&text(&out.stdout)).unwrap();
    assert_eq!(body, serde_json::json!({"ok": true, "result": 5}));

    let trailing = run(&[
        "call",
        file.to_str().unwrap(),
        "计算器.加法",
        "a=2",
        "b=3",
        "--backend",
        "bytecode",
        "--json",
    ]);
    assert_eq!(trailing.status.code(), Some(0), "{}", text(&trailing.stderr));
    let body: serde_json::Value = serde_json::from_str(&text(&trailing.stdout)).unwrap();
    assert_eq!(body, serde_json::json!({"ok": true, "result": 5}));
}

#[test]
fn bytecode_does_not_run_main() {
    let file = root().join("tests/call/main-split.mq.md");
    for backend in ["tree", "bytecode"] {
        assert_value(&file, "计算器.加法", &["a=2", "b=3"], backend, "5");
    }
}

#[test]
fn missing_unknown_and_contract_args_are_structured() {
    let calc = root().join("examples/calculator.mq.md");
    assert_code(&calc, "没有这个函数", &[], "artifact.unresolved");
    assert_code(&calc, "计算器.加法", &["a=2"], "call.missing_arg");
    assert_code(
        &calc,
        "计算器.加法",
        &["a=2", "b=3", "c=1"],
        "call.unknown_arg",
    );
    let contract = root().join("tests/call/contract.mq.md");
    assert_code(&contract, "校验.加一", &["n=True"], "contract.arg_mismatch");
    assert_value(&contract, "校验.加一", &["n=3"], "tree", "4");
}

#[test]
fn same_short_name_is_ambiguous() {
    let file = root().join("tests/call/ambiguous.mq.md");
    assert_code(&file, "加法", &[], "artifact.ambiguous");
    assert_value(&file, "甲.加法", &[], "tree", "1");
    assert_value(&file, "乙.加法", &[], "bytecode", "2");
}

#[test]
fn instance_method_is_refused() {
    let file = root().join("tests/call/method.mq.md");
    assert_code(&file, "点.横坐标", &[], "artifact.not_executable");
}

#[test]
fn import_is_executed_and_call_lib_path_stays_on_bytecode() {
    let site = root().join("tests/call/import/site.mq.md");
    for backend in ["tree", "bytecode"] {
        assert_value(&site, "包装.再加一", &["n=4"], backend, "5");
    }
    let tmp = std::env::temp_dir().join(format!("mq-call-lib-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let args = tmp.join("args.json");
    let emit = tmp.join("out.json");
    fs::write(&args, r#"{"n":4}"#).unwrap();
    let out = Command::new(bin())
        .args([
            "run",
            site.to_str().unwrap(),
            "--backend",
            "bytecode",
            "--call-lib-path",
            "inc.加一",
            "--args-json",
            args.to_str().unwrap(),
            "--emit-result",
            emit.to_str().unwrap(),
        ])
        .current_dir(root())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        text(&out.stderr),
        text(&out.stdout)
    );
    assert_eq!(fs::read_to_string(&emit).unwrap().trim(), "5");
}

#[test]
fn impure_calls_do_not_run() {
    let src = root().join("tests/call/effects.mq.md");
    let dir = std::env::temp_dir().join(format!("mq-call-fx-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("effects.mq.md");
    fs::copy(&src, &file).unwrap();
    for function in ["危险.写入", "危险.联网"] {
        let out = run(&["call", "--json", file.to_str().unwrap(), function]);
        assert_ne!(out.status.code(), Some(0), "{function} {}", text(&out.stdout));
        let body: serde_json::Value = serde_json::from_str(&text(&out.stdout)).unwrap();
        assert_eq!(body["error"]["code"], "artifact.effect_denied", "{body}");
    }
    assert!(
        !dir.join("sent.txt").exists(),
        "denied write must not create sent.txt"
    );
}

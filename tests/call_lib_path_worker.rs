//! Phase A: `marqdo run --call-lib-path` writes `--emit-result` without parent INVOKE_LOCK.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn call_lib_path_emits_json_result() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let site = root.join("tests/ext/isolated-execute-site");
    let entry = site.join("serve.mq.md");
    let tmp = std::env::temp_dir().join(format!("mq-clp-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    let args = tmp.join("args.json");
    let emit = tmp.join("out.json");
    std::fs::write(&args, r#"{"payload":null}"#).unwrap();

    let bin = env!("CARGO_BIN_EXE_marqdo");
    let out = Command::new(bin)
        .arg("run")
        .arg(&entry)
        .arg("--call-lib-path")
        .arg("site.models") // entry is serve.mq.md which imports site
        .arg("--fs-root")
        .arg(&site)
        .arg("--args-json")
        .arg(&args)
        .arg("--emit-result")
        .arg(&emit)
        .current_dir(&site)
        .env("MARQDO_EXT", root.join("ext"))
        .output()
        .expect("spawn worker");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code().unwrap_or(1),
        0,
        "stderr={stderr}\nstdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let body = std::fs::read_to_string(&emit).expect("emit-result");
    assert!(
        body.contains("list") || body.contains("ok"),
        "emit={body}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

//! Browser WASM integration smoke (Node + wasm artifact).

use std::path::PathBuf;
use std::process::Command;

fn wasm_artifact() -> Option<PathBuf> {
    let mut cands = Vec::new();
    if let Ok(td) = std::env::var("CARGO_TARGET_DIR") {
        cands.push(PathBuf::from(td).join("wasm32-unknown-unknown/release-wasm/marqdo_wasm.wasm"));
    }
    cands.push(PathBuf::from(
        "target/wasm32-unknown-unknown/release-wasm/marqdo_wasm.wasm",
    ));
    cands.push(PathBuf::from("examples/browser-hello/marqdo_wasm.wasm"));
    cands.into_iter().find(|p| p.is_file())
}

#[test]
fn wasm_node_abi_smoke() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("skip: node not installed");
        return;
    }
    let wasm = match wasm_artifact() {
        Some(p) => p,
        None => {
            if std::env::var("MARQDO_WASM_SMOKE").ok().as_deref() != Some("1") {
                eprintln!("skip: no wasm artifact (run `marqdo wasm build` or set MARQDO_WASM_SMOKE=1)");
                return;
            }
            let status = Command::new(env!("CARGO_BIN_EXE_marqdo"))
                .args(["wasm", "build", "-o", "examples/browser-hello"])
                .status()
                .expect("marqdo wasm build");
            assert!(status.success(), "marqdo wasm build failed");
            PathBuf::from("examples/browser-hello/marqdo_wasm.wasm")
        }
    };
    assert!(wasm.is_file(), "missing {}", wasm.display());
    let out = Command::new("node")
        .args(["tests/wasm/smoke.mjs", wasm.to_str().unwrap()])
        .output()
        .expect("node smoke");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "smoke failed: status={} stdout={} stderr={}",
        out.status,
        stdout,
        stderr
    );
    assert!(stdout.contains("wasm smoke ok"), "stdout={stdout}");
}

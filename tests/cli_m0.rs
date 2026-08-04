use std::process::Command;

#[test]
fn cli_runs_and_reports_unimplemented() {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "examples/hello.mq.md"])
        .output()
        .expect("failed to run marqdo");

    assert!(
        !output.status.success(),
        "M0 should exit non-zero until eval exists"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not implemented") || stderr.contains("evaluation not implemented"),
        "stderr was: {stderr}"
    );
}

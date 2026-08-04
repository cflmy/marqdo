use std::process::Command;

#[test]
fn cli_runs_and_reports_unimplemented() {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "examples/hello.mq.md"])
        .output()
        .expect("failed to run marqdo");

    assert!(
        !output.status.success(),
        "M0/M1 should exit non-zero until eval exists"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not implemented") || stderr.contains("evaluation not implemented"),
        "stderr was: {stderr}"
    );
}

#[test]
fn cli_dump_lines_shows_classification() {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "examples/hello.mq.md", "--dump-lines"])
        .output()
        .expect("failed to run marqdo");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("=== marqdo: lines"),
        "stdout was: {stdout}"
    );
    assert!(stdout.contains("Code"), "expected Code line: {stdout}");
    assert!(stdout.contains("Comment") || stdout.contains("Blank"), "{stdout}");
    assert!(stdout.contains("print"), "{stdout}");
    assert!(!output.status.success());
}

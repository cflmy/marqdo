use std::process::Command;

fn run(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(args)
        .output()
        .expect("failed to run marqdo");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn hello_prints_greeting() {
    let (code, stdout, stderr) = run(&["run", "examples/hello.mq.md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "Hello World!");
}

#[test]
fn index_nested_call() {
    let (code, stdout, stderr) = run(&["run", "examples/index.mq.md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "Hello World!");
}

#[test]
fn branch_else_arm() {
    let (code, stdout, stderr) = run(&["run", "examples/branch.mq.md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "零");
}

#[test]
fn loop_while_and_foreach() {
    let (code, stdout, stderr) = run(&["run", "examples/loop.mq.md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(
        stdout.trim_end(),
        "3\n2\n1\n今天吃苹果\n今天吃梨"
    );
}

#[test]
fn collection_foreach() {
    let (code, stdout, stderr) = run(&["run", "examples/collection.mq.md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "苹果\n梨\n桃");
}

#[test]
fn with_import() {
    let (code, stdout, stderr) = run(&["run", "examples/with-import.mq.md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "42\n你好，Marqdo!");
}

#[test]
fn hello_dump_lines_still_runs() {
    let (code, stdout, stderr) = run(&["run", "examples/hello.mq.md", "--dump-lines"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("=== marqdo: lines"));
    assert!(stdout.contains("Hello World!"));
}

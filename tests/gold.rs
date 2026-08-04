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

fn assert_out(path: &str, expect: &str) {
    let (code, stdout, stderr) = run(&["run", path]);
    assert_eq!(code, 0, "{path} stderr={stderr}");
    assert_eq!(stdout.trim_end(), expect.trim_end(), "{path}");
}

fn assert_err(path: &str, substr: &str) {
    let (code, stdout, stderr) = run(&["run", path]);
    assert_ne!(code, 0, "{path} unexpectedly succeeded stdout={stdout}");
    assert!(
        stderr.contains(substr),
        "{path} stderr missing {substr:?}: {stderr}"
    );
}

#[test]
fn structure_hello() {
    assert_out("examples/structure/hello.mq.md", "Hello World!");
}

#[test]
fn structure_nested_call() {
    assert_out("examples/structure/nested-call.mq.md", "Hello World!");
}

#[test]
fn structure_positional_call() {
    assert_out(
        "examples/structure/positional-call.mq.md",
        "Hello Marqdo!\nHello World!",
    );
}

#[test]
fn structure_branch() {
    assert_out("examples/structure/branch.mq.md", "零");
}

#[test]
fn structure_loop() {
    assert_out(
        "examples/structure/loop.mq.md",
        "3\n2\n1\n今天吃苹果\n今天吃梨",
    );
}

#[test]
fn structure_collection() {
    assert_out("examples/structure/collection.mq.md", "苹果\n梨\n桃");
}

#[test]
fn structure_import() {
    assert_out(
        "examples/structure/import/main.mq.md",
        "42\n你好，Marqdo!",
    );
}

#[test]
fn keywords_print() {
    assert_out(
        "examples/keywords/print.mq.md",
        "named form\npositional",
    );
}

#[test]
fn keywords_bool_logic() {
    assert_out(
        "examples/keywords/bool-logic.mq.md",
        "ok\nnone-falsy",
    );
}

#[test]
fn error_undefined_var() {
    assert_err(
        "examples/errors/undefined-var.mq.md",
        "undefined variable `missing`",
    );
}

#[test]
fn error_unknown_fn() {
    assert_err(
        "examples/errors/unknown-fn.mq.md",
        "unknown function `no_such_fn`",
    );
}

#[test]
fn error_bad_arity() {
    assert_err(
        "examples/errors/bad-arity.mq.md",
        "missing argument for parameter `x`",
    );
}

#[test]
fn error_syntax_bad_line() {
    assert_err(
        "examples/errors/syntax-bad-line.mq.md",
        "unrecognized statement",
    );
}

fn assert_out_backend(path: &str, backend: &str, expect: &str) {
    let (code, stdout, stderr) = run(&["run", path, "--backend", backend]);
    assert_eq!(code, 0, "{path} backend={backend} stderr={stderr}");
    assert_eq!(
        stdout.trim_end(),
        expect.trim_end(),
        "{path} backend={backend}"
    );
}

#[test]
fn bytecode_hello() {
    assert_out_backend(
        "examples/structure/hello.mq.md",
        "bytecode",
        "Hello World!",
    );
}

#[test]
fn bytecode_branch() {
    assert_out_backend("examples/structure/branch.mq.md", "bytecode", "零");
}

#[test]
fn bytecode_loop() {
    assert_out_backend(
        "examples/structure/loop.mq.md",
        "bytecode",
        "3\n2\n1\n今天吃苹果\n今天吃梨",
    );
}

#[test]
fn bytecode_collection() {
    assert_out_backend(
        "examples/structure/collection.mq.md",
        "bytecode",
        "苹果\n梨\n桃",
    );
}

#[test]
fn bytecode_print_keyword() {
    assert_out_backend(
        "examples/keywords/print.mq.md",
        "bytecode",
        "named form\npositional",
    );
}

#[test]
fn bytecode_bool_logic() {
    assert_out_backend(
        "examples/keywords/bool-logic.mq.md",
        "bytecode",
        "ok\nnone-falsy",
    );
}

#[test]
fn bytecode_nested_call() {
    assert_out_backend(
        "examples/structure/nested-call.mq.md",
        "bytecode",
        "Hello World!",
    );
}

#[test]
fn bytecode_positional_call() {
    assert_out_backend(
        "examples/structure/positional-call.mq.md",
        "bytecode",
        "Hello Marqdo!\nHello World!",
    );
}

#[test]
fn bytecode_import() {
    assert_out_backend(
        "examples/structure/import/main.mq.md",
        "bytecode",
        "42\n你好，Marqdo!",
    );
}

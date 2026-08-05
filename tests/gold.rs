use std::io::Write;
use std::process::{Command, Stdio};

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

fn run_with_stdin(args: &[&str], stdin: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn marqdo");
    {
        let mut pipe = child.stdin.take().expect("stdin");
        pipe.write_all(stdin.as_bytes()).expect("write stdin");
    }
    let output = child.wait_with_output().expect("wait marqdo");
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

/// Assert failure with `path:line:col:` prefix and message substring (both backends).
fn assert_err(path: &str, line_col: &str, substr: &str) {
    for backend in ["tree", "bytecode"] {
        let (code, stdout, stderr) = run(&["run", path, "--backend", backend]);
        assert_ne!(
            code, 0,
            "{path} backend={backend} unexpectedly succeeded stdout={stdout}"
        );
        let loc = format!("{path}:{line_col}:");
        assert!(
            stderr.contains(&loc),
            "{path} backend={backend} stderr missing location {loc:?}: {stderr}"
        );
        assert!(
            stderr.contains(substr),
            "{path} backend={backend} stderr missing {substr:?}: {stderr}"
        );
    }
}

#[test]
fn structure_hello() {
    assert_out("tests/structure/hello.mq.md", "Hello World!");
}

#[test]
fn structure_nested_call() {
    assert_out("tests/structure/nested-call.mq.md", "Hello World!");
}

#[test]
fn structure_positional_call() {
    assert_out(
        "tests/structure/positional-call.mq.md",
        "Hello Marqdo!\nHello World!",
    );
}

#[test]
fn structure_branch() {
    assert_out("tests/structure/branch.mq.md", "零");
}

#[test]
fn structure_loop() {
    assert_out(
        "tests/structure/loop.mq.md",
        "3
2
1
今天吃苹果
今天吃梨",
    );
}

#[test]
fn structure_collection() {
    assert_out("tests/structure/collection.mq.md", "苹果
梨
桃");
}

#[test]
fn structure_import() {
    assert_out(
        "tests/structure/import/main.mq.md",
        "42
你好，Marqdo!",
    );
}


#[test]
fn structure_fn_end_hr() {
    assert_out("tests/structure/fn-end-hr.mq.md", "Hello World!");
}

#[test]
fn structure_fn_end_empty_return() {
    assert_out("tests/structure/fn-end-empty-return.mq.md", "Hi Marqdo!");
}

#[test]
fn structure_paragraph_comment() {
    assert_out("tests/structure/paragraph-comment.mq.md", "ok");
}

#[test]
fn keywords_print() {
    assert_out(
        "tests/keywords/print.mq.md",
        "named form\npositional",
    );
}

#[test]
fn keywords_bool_logic() {
    assert_out(
        "tests/keywords/bool-logic.mq.md",
        "ok\nnone-falsy",
    );
}

#[test]
fn keywords_stdlib() {
    assert_out(
        "tests/keywords/stdlib.mq.md",
        "5
3
42
42
1",
    );
}

#[test]
fn bytecode_stdlib() {
    assert_out_backend(
        "tests/keywords/stdlib.mq.md",
        "bytecode",
        "5
3
42
42
1",
    );
}


#[test]
fn keywords_stdlib_more() {
    assert_out(
        "tests/keywords/stdlib-more.mq.md",
        "hi
3
b
a-b-c
list",
    );
}

#[test]
fn bytecode_stdlib_more() {
    assert_out_backend(
        "tests/keywords/stdlib-more.mq.md",
        "bytecode",
        "hi
3
b
a-b-c
list",
    );
}


#[test]
fn keywords_zh_builtins() {
    assert_out(
        "tests/keywords/zh-builtins.mq.md",
        "5
7",
    );
}

#[test]
fn bytecode_zh_builtins() {
    assert_out_backend(
        "tests/keywords/zh-builtins.mq.md",
        "bytecode",
        "5
7",
    );
}

#[test]
fn keywords_zh_bool() {
    assert_out(
        "tests/keywords/zh-bool.mq.md",
        "ok
none-falsy",
    );
}

#[test]
fn bytecode_zh_bool() {
    assert_out_backend(
        "tests/keywords/zh-bool.mq.md",
        "bytecode",
        "ok
none-falsy",
    );
}


#[test]
fn structure_import_lib() {
    assert_out(
        "tests/structure/import-lib.mq.md",
        "x
2
a",
    );
}

#[test]
fn bytecode_import_lib() {
    assert_out_backend(
        "tests/structure/import-lib.mq.md",
        "bytecode",
        "x
2
a",
    );
}


#[test]
fn error_undefined_var() {
    assert_err(
        "tests/errors/undefined-var.mq.md",
        "7:1",
        "undefined variable `missing`",
    );
}

#[test]
fn error_unknown_fn() {
    assert_err(
        "tests/errors/unknown-fn.mq.md",
        "7:1",
        "unknown function `no_such_fn`",
    );
}

#[test]
fn error_bad_arity() {
    assert_err(
        "tests/errors/bad-arity.mq.md",
        "7:1",
        "missing argument for parameter `x`",
    );
}

#[test]
fn error_syntax_bad_line() {
    assert_err(
        "tests/errors/syntax-bad-line.mq.md",
        "7:1",
        "unrecognized statement",
    );
}

#[test]
fn error_div_zero() {
    assert_err("tests/errors/div-zero.mq.md", "7:1", "division by zero");
}

#[test]
fn error_bad_int() {
    assert_err(
        "tests/errors/bad-int.mq.md",
        "7:1",
        "cannot convert to int",
    );
}

#[test]
fn error_bad_split_sep() {
    assert_err(
        "tests/errors/bad-split-sep.mq.md",
        "7:1",
        "split sep must not be empty",
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

fn assert_out_stdin(path: &str, backend: &str, stdin: &str, expect: &str) {
    let (code, stdout, stderr) =
        run_with_stdin(&["run", path, "--backend", backend], stdin);
    assert_eq!(code, 0, "{path} backend={backend} stderr={stderr}");
    assert_eq!(
        stdout.trim_end(),
        expect.trim_end(),
        "{path} backend={backend} stdout={stdout:?}"
    );
}

#[test]
fn keywords_input() {
    assert_out_stdin(
        "tests/keywords/input.mq.md",
        "tree",
        "Alice\n",
        "Name:Hello Alice!",
    );
}

#[test]
fn keywords_input_stdin_file() {
    let (code, stdout, stderr) = run(&[
        "run",
        "tests/keywords/input.mq.md",
        "--stdin-file",
        "tests/keywords/input-stdin.txt",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout.trim_end(), "Name:Hello Alice!");
}

#[test]
fn bytecode_input() {
    assert_out_stdin(
        "tests/keywords/input.mq.md",
        "bytecode",
        "Alice\n",
        "Name:Hello Alice!",
    );
}

#[test]
fn bytecode_hello() {
    assert_out_backend(
        "tests/structure/hello.mq.md",
        "bytecode",
        "Hello World!",
    );
}

#[test]
fn bytecode_branch() {
    assert_out_backend("tests/structure/branch.mq.md", "bytecode", "零");
}

#[test]
fn bytecode_loop() {
    assert_out_backend(
        "tests/structure/loop.mq.md",
        "bytecode",
        "3
2
1
今天吃苹果
今天吃梨",
    );
}

#[test]
fn bytecode_collection() {
    assert_out_backend(
        "tests/structure/collection.mq.md",
        "bytecode",
        "苹果
梨
桃",
    );
}

#[test]
fn bytecode_print_keyword() {
    assert_out_backend(
        "tests/keywords/print.mq.md",
        "bytecode",
        "named form\npositional",
    );
}

#[test]
fn bytecode_bool_logic() {
    assert_out_backend(
        "tests/keywords/bool-logic.mq.md",
        "bytecode",
        "ok\nnone-falsy",
    );
}

#[test]
fn bytecode_nested_call() {
    assert_out_backend(
        "tests/structure/nested-call.mq.md",
        "bytecode",
        "Hello World!",
    );
}

#[test]
fn bytecode_positional_call() {
    assert_out_backend(
        "tests/structure/positional-call.mq.md",
        "bytecode",
        "Hello Marqdo!\nHello World!",
    );
}

#[test]
fn bytecode_import() {
    assert_out_backend(
        "tests/structure/import/main.mq.md",
        "bytecode",
        "42
你好，Marqdo!",
    );
}


#[test]
fn bytecode_fn_end_hr() {
    assert_out_backend(
        "tests/structure/fn-end-hr.mq.md",
        "bytecode",
        "Hello World!",
    );
}

#[test]
fn bytecode_fn_end_empty_return() {
    assert_out_backend(
        "tests/structure/fn-end-empty-return.mq.md",
        "bytecode",
        "Hi Marqdo!",
    );
}

#[test]
fn bytecode_paragraph_comment() {
    assert_out_backend(
        "tests/structure/paragraph-comment.mq.md",
        "bytecode",
        "ok",
    );
}

#[test]
fn trace_eval_json_tree() {
    let (code, stdout, stderr) = run(&["run", "tests/structure/hello.mq.md", "--trace-eval"]);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout.trim_end(), "Hello World!");
    assert!(stderr.contains("\"event\":\"enter_fn\""), "{stderr}");
    assert!(stderr.contains("\"event\":\"stmt\""), "{stderr}");
    assert!(stderr.contains("\"span\":\"5:1\""), "{stderr}");
    assert!(stderr.contains("\"event\":\"leave_fn\""), "{stderr}");
}

#[test]
fn trace_eval_json_bytecode() {
    let (code, stdout, stderr) = run(&[
        "run",
        "tests/structure/hello.mq.md",
        "--backend",
        "bytecode",
        "--trace-eval",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout.trim_end(), "Hello World!");
    assert!(stderr.contains("\"event\":\"op\""), "{stderr}");
    assert!(stderr.contains("\"kind\":\"print\""), "{stderr}");
    assert!(stderr.contains("\"span\":\"5:1\""), "{stderr}");
}

#[test]
fn catalog_writes_yaml() {
    let dir = tempfile_dir("mq-cat");
    let (code, _, stderr) = run(&[
        "catalog",
        "tests",
        "-o",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let yaml = std::fs::read_to_string(dir.join("catalog.yaml")).unwrap();
    assert!(yaml.contains("type: Marqdo Catalog"));
    assert!(yaml.contains("structure/hello"));
    assert!(yaml.contains("utils.mq.md"), "{yaml}");
    assert!(
        yaml.contains("???????") || yaml.contains("imports:
      - utils.mq.md"),
        "{yaml}"
    );
}

#[test]
fn view_output_writes_html() {
    let dir = tempfile_dir("mq-site");
    let (code, _, stderr) = run(&[
        "view",
        "output",
        "tests/structure/hello.mq.md",
        "-o",
        dir.to_str().unwrap(),
        "--no-exec",
    ]);
    assert_eq!(code, 0, "{stderr}");
    let index = std::fs::read_to_string(dir.join("index.html")).unwrap();
    assert!(index.contains("marqdo"));
    assert!(!index.contains("fonts.googleapis.com"));
    assert!(index.contains("#ffffff") || index.contains("--surface: #ffffff"));
    assert!(index.contains("nav-toggle"));
    assert!(!index.contains("Run with input"), "static export should omit live stdin form");
    assert!(dir.join("pages").join("hello.mq.md.html").exists());
}

#[test]
fn lib_fs_read() {
    assert_out("tests/lib/fs-read.mq.md", "exists\nhello-lib");
}

#[test]
fn lib_fs_read_zh() {
    assert_out("tests/lib/文件-读.mq.md", "存在\nhello-lib");
}

#[test]
fn lib_time_format() {
    assert_out("tests/lib/time-format.mq.md", "1970-01-01\n0");
}

#[test]
fn lib_json_roundtrip() {
    assert_out(
        "tests/lib/json-roundtrip.mq.md",
        "map\n1\n{\"a\":1,\"b\":\"x\"}",
    );
}

#[test]
fn lib_sys_cwd() {
    assert_out("tests/lib/sys-cwd.mq.md", "cwd-ok");
}

#[test]
fn lib_net_encode() {
    assert_out("tests/lib/net-encode.mq.md", "a+b");
}

#[test]
fn bytecode_lib_json() {
    assert_out_backend(
        "tests/lib/json-roundtrip.mq.md",
        "bytecode",
        "map\n1\n{\"a\":1,\"b\":\"x\"}",
    );
}

fn tempfile_dir(prefix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

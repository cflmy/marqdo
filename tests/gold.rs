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
fn structure_object_handle() {
    assert_out("tests/structure/object-handle.mq.md", "counter\n3\n4");
}

#[test]
fn structure_inherit_parent_method() {
    assert_out(
        "tests/structure/inherit-parent-method.mq.md",
        "Loud
Hello, world!",
    );
}

#[test]
fn structure_inherit_override() {
    assert_out(
        "tests/structure/inherit-override.mq.md",
        "Hello, world!
HELLO, world!",
    );
}

#[test]
fn structure_inherit_explicit_super() {
    assert_out(
        "tests/structure/inherit-explicit-super.mq.md",
        "Child
child
Ada",
    );
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
fn structure_branch_consecutive() {
    assert_out(
        "tests/structure/branch-consecutive.mq.md",
        "A\nB\nelse-ok",
    );
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
fn structure_collection_map() {
    assert_out(
        "tests/structure/collection-map.mq.md",
        "水果
蔬菜
数码用品
苹果
黄瓜
鼠标",
    );
}

#[test]
fn structure_collection_map_list() {
    assert_out(
        "tests/structure/collection-map-list.mq.md",
        "红富士
[红富士, 嘎啦]
黄瓜条",
    );
}

#[test]
fn structure_collection_cell_expr() {
    assert_out(
        "tests/structure/collection-cell-expr.mq.md",
        "sk-live
gpt-4o-mini
https://api.openai.com/v1
/chat/completions
sk-live",
    );
}

#[test]
fn structure_bare_id_italic_bold() {
    assert_out(
        "tests/structure/bare-id-italic-bold.mq.md",
        "2
2
quoted
7
ok
hi",
    );
}

#[test]
fn structure_footnote_index() {
    assert_out(
        "tests/structure/footnote-index.mq.md",
        "苹果
梨
桃",
    );
}

#[test]
fn structure_footnote_map_digits() {
    assert_out(
        "tests/structure/footnote-map-digits.mq.md",
        "half
other",
    );
}

#[test]
fn structure_collection_records() {
    assert_out(
        "tests/structure/collection-records.mq.md",
        "{品名: 苹果, 数量: 2}
苹果
3
{品名: 苹果, 数量: 2}
{品名: 梨, 数量: 3}",
    );
}

#[test]
fn structure_collection_records_at() {
    assert_out(
        "tests/structure/collection-records-at.mq.md",
        "apple
3",
    );
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
fn structure_ns_path3() {
    assert_out(
        "tests/structure/ns/path3.mq.md",
        "ok
constructed",
    );
}

#[test]
fn structure_ns_use() {
    assert_out("tests/structure/ns/use.mq.md", "hi");
}

#[test]
fn bytecode_ns_path3() {
    assert_out_backend(
        "tests/structure/ns/path3.mq.md",
        "bytecode",
        "ok
constructed",
    );
}

#[test]
fn bytecode_ns_use() {
    assert_out_backend("tests/structure/ns/use.mq.md", "bytecode", "hi");
}

#[test]
fn error_ns_instance_method() {
    assert_err(
        "tests/errors/ns-instance-method.mq.md",
        "8:1",
        "instance method",
    );
}

#[test]
fn error_legacy_gt_import() {
    assert_err(
        "tests/errors/legacy-gt-import.mq.md",
        "3:1",
        "legacy frontmatter import",
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
fn structure_narrative_bold() {
    assert_out("tests/structure/narrative-bold.mq.md", "ok");
}

#[test]
fn structure_optional_param() {
    assert_out(
        "tests/structure/optional-param.mq.md",
        "Hi, Ada!\nHello, Bob!",
    );
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
fn error_footnote_zero() {
    assert_err(
        "tests/errors/footnote-zero.mq.md",
        "13:1",
        "`[^0]` is invalid",
    );
}

#[test]
fn error_footnote_oob() {
    assert_err(
        "tests/errors/footnote-oob.mq.md",
        "13:1",
        "out of range",
    );
}

#[test]
fn error_footnote_missing_key() {
    assert_err(
        "tests/errors/footnote-missing-key.mq.md",
        "13:1",
        "missing map key",
    );
}

#[test]
fn error_table_dup_header() {
    assert_err(
        "tests/errors/table-dup-header.mq.md",
        "9:1",
        "duplicate table header",
    );
}

#[test]
fn error_table_row_marker_only() {
    assert_err(
        "tests/errors/table-row-marker-only.mq.md",
        "9:1",
        "row-oriented table needs field columns",
    );
}

#[test]
fn error_inherit_cycle() {
    assert_err(
        "tests/errors/inherit-cycle.mq.md",
        "5:1",
        "cyclic inheritance",
    );
}

#[test]
fn error_inherit_unknown_base() {
    assert_err(
        "tests/errors/inherit-unknown-base.mq.md",
        "5:1",
        "unknown base type",
    );
}

#[test]
fn error_inherit_bad_rhs() {
    assert_err(
        "tests/errors/inherit-bad-rhs.mq.md",
        "5:1",
        "object inheritance requires",
    );
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
fn keywords_input_escape() {
    for backend in ["tree", "bytecode"] {
        assert_out_stdin(
            "tests/keywords/input-escape.mq.md",
            backend,
            "Ada\n",
            "Name:\nHello Ada!",
        );
    }
}

#[test]
fn keywords_quoted_string() {
    assert_out("tests/keywords/quoted-string.mq.md", "a\nb");
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
fn bytecode_inherit_parent_method() {
    assert_out_backend(
        "tests/structure/inherit-parent-method.mq.md",
        "bytecode",
        "Loud
Hello, world!",
    );
}

#[test]
fn bytecode_inherit_override() {
    assert_out_backend(
        "tests/structure/inherit-override.mq.md",
        "bytecode",
        "Hello, world!
HELLO, world!",
    );
}

#[test]
fn bytecode_branch() {
    assert_out_backend("tests/structure/branch.mq.md", "bytecode", "零");
}

#[test]
fn bytecode_branch_consecutive() {
    assert_out_backend(
        "tests/structure/branch-consecutive.mq.md",
        "bytecode",
        "A\nB\nelse-ok",
    );
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
fn bytecode_collection_map() {
    assert_out_backend(
        "tests/structure/collection-map.mq.md",
        "bytecode",
        "水果
蔬菜
数码用品
苹果
黄瓜
鼠标",
    );
}

#[test]
fn bytecode_collection_map_list() {
    assert_out_backend(
        "tests/structure/collection-map-list.mq.md",
        "bytecode",
        "红富士
[红富士, 嘎啦]
黄瓜条",
    );
}

#[test]
fn bytecode_footnote_index() {
    assert_out_backend(
        "tests/structure/footnote-index.mq.md",
        "bytecode",
        "苹果
梨
桃",
    );
}

#[test]
fn bytecode_collection_records() {
    assert_out_backend(
        "tests/structure/collection-records.mq.md",
        "bytecode",
        "{品名: 苹果, 数量: 2}
苹果
3
{品名: 苹果, 数量: 2}
{品名: 梨, 数量: 3}",
    );
}

#[test]
fn bytecode_collection_records_at() {
    assert_out_backend(
        "tests/structure/collection-records-at.mq.md",
        "bytecode",
        "apple
3",
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
    assert!(yaml.contains("concepts:"), "{yaml}");
    let index = std::fs::read_to_string(dir.join("index.md")).unwrap();
    assert!(index.contains("## Modules"), "{index}");
    assert!(index.contains("## Agent knowledge"), "{index}");
    // Import graph: structure/import/main depends on utils — clickable on module page.
    let import_page = std::fs::read_to_string(
        dir.join("modules")
            .join("structure__import__main.md"),
    )
    .unwrap();
    assert!(
        import_page.contains("](") && import_page.contains("utils"),
        "depends should link to utils module: {import_page}"
    );
}

#[test]
fn ext_mq_md_never_calls_host() {
    // Hard rule: ext/**/*.mq.md must not invoke host_* (use lib/* or plugin names).
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext");
    let mut offenders = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                walk(&p, out);
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.ends_with(".mq.md") {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&p) else {
                continue;
            };
            for (i, line) in src.lines().enumerate() {
                let t = line.trim_start();
                // Executable call lines / bindings that invoke host_*.
                if (t.starts_with('>') || t.starts_with('*')) && t.contains("host_") {
                    out.push(format!("{}:{}: {line}", p.display(), i + 1));
                }
            }
        }
    }
    walk(&root, &mut offenders);
    assert!(
        offenders.is_empty(),
        "ext must not call host_* (use lib/* or agent_* plugin names):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn catalog_includes_agent_kb_concepts() {
    let dir = tempfile_dir("mq-cat-kb");
    // Ensure a concept exists under tests/ext agent-kb (offline reuse test leaves one).
    let (code, _, stderr) = run(&[
        "run",
        "tests/ext/agent-kb-reuse.mq.md",
    ]);
    assert_eq!(code, 0, "seed kb via reuse test: {stderr}");

    let (code, _, stderr) = run(&[
        "catalog",
        "tests/ext",
        "-o",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let yaml = std::fs::read_to_string(dir.join("catalog.yaml")).unwrap();
    assert!(
        yaml.contains("Marqdo Task") || yaml.contains("Marqdo Agent Skill"),
        "expected agent-kb concepts in catalog: {yaml}"
    );
    assert!(
        dir.join("concepts").join("tasks").exists()
            || dir.join("concepts").join("skills").exists(),
        "concepts should be copied into catalog out dir"
    );
    let index = std::fs::read_to_string(dir.join("index.md")).unwrap();
    assert!(
        !index.contains("No `.marqdo/agent-kb` concepts"),
        "{index}"
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
    let page = std::fs::read_to_string(dir.join("pages").join("hello.mq.md.html")).unwrap();
    assert!(page.contains("katex.min.css"), "view layout should load KaTeX");
    assert!(
        !page.contains("name=\"stdin\""),
        "hello has no input; no preset form"
    );
    assert!(dir.join("pages").join("hello.mq.md.html").exists());
}

#[test]
fn view_input_static_awaits_stdin() {
    let dir = tempfile_dir("mq-input-view");
    let (code, _, stderr) = run(&[
        "view",
        "output",
        "tests/keywords/input.mq.md",
        "-o",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let page = std::fs::read_to_string(dir.join("pages").join("input.mq.md.html")).unwrap();
    assert!(
        page.contains("Fill preset input") || page.contains("status-pill pending"),
        "should defer execution, not fail"
    );
    assert!(
        !page.contains("input needs a line"),
        "static export should not surface input error before stdin"
    );
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
fn lib_table_collections() {
    assert_out(
        "tests/lib/table-collections.mq.md",
        "Bearer-x\nA\nb\nhi\nTrue\n3\n[Authorization, role, content]\n0",
    );
}

#[test]
fn lib_table_collections_zh() {
    assert_out("tests/lib/表-集合.mq.md", "Bearer-x\nA\n3");
}

#[test]
fn lib_sys_cwd() {
    assert_out("tests/lib/sys-cwd.mq.md", "cwd-ok");
}

#[test]
fn lib_sys_dotenv() {
    assert_out("tests/lib/sys-dotenv.mq.md", "loaded");
}

#[test]
fn lib_writeback_get() {
    assert_out("tests/lib/writeback-get.mq.md", "hello-writeback");
}

#[test]
fn lib_subtask_spawn() {
    assert_out("tests/lib/subtask-spawn.mq.md", "child_ok\ndone\nquiet-ok");
}

#[test]
fn lib_subtask_quiet_io() {
    assert_out("tests/lib/subtask-quiet-io.mq.md", "quiet-io-ok");
}

#[test]
fn lib_subtask_fn() {
    assert_out("tests/lib/subtask-fn.mq.md", "42");
}

#[test]
fn lib_subtask_foreign() {
    if !python_available() {
        eprintln!("skip lib_subtask_foreign: no python on PATH");
        return;
    }
    assert_out("tests/lib/subtask-foreign.mq.md", "15");
}

#[test]
fn lib_json_quote() {
    assert_out("tests/lib/json-quote.mq.md", "\"hi\"");
}

#[test]
fn lib_plugin_demo() {
    let status = Command::new(env!("CARGO"))
        .args(["build", "-p", "marqdo_plugin_demo"])
        .status()
        .expect("cargo build plugin demo");
    assert!(status.success(), "failed to build marqdo_plugin_demo");

    let lib_name = if cfg!(windows) {
        "demo.dll"
    } else if cfg!(target_os = "macos") {
        "libdemo.dylib"
    } else {
        "libdemo.so"
    };
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target")
        });
    let built = target_dir.join("debug").join(lib_name);
    assert!(
        built.is_file(),
        "missing plugin artifact {}",
        built.display()
    );
    // Must live under the program's fs_root (`tests/lib/`) for path sandbox.
    let plugin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("lib")
        .join(lib_name);
    std::fs::copy(&built, &plugin).expect("copy demo plugin into tests/lib");

    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/lib/plugin-demo.mq.md"])
        .env("MARQDO_TEST_PLUGIN", lib_name)
        .output()
        .expect("failed to run marqdo");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "plugin-demo stderr={stderr}");
    assert_eq!(stdout.trim_end(), "3\nok", "plugin-demo");
}

#[test]
fn ext_cli_add_list_remove_llm() {
    let tmp = tempfile_dir("marqdo-ext-cli");
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext");
    assert!(src.join("ai").join("llm.mq.md").is_file());

    let bin = env!("CARGO_BIN_EXE_marqdo");
    let status = Command::new(bin)
        .args(["ext", "add", "llm"])
        .env("MARQDO_EXT", &tmp)
        .env("MARQDO_EXT_SOURCE", &src)
        .env_remove("USERPROFILE")
        .env_remove("HOME")
        .status()
        .expect("ext add");
    assert!(status.success(), "ext add llm failed");
    assert!(tmp.join("ai").join("llm.mq.md").is_file());
    assert!(tmp.join("ai").join("大模型.mq.md").is_file());

    let out = Command::new(bin)
        .args(["ext", "list"])
        .env("MARQDO_EXT", &tmp)
        .output()
        .expect("ext list");
    assert_eq!(out.status.code().unwrap_or(1), 0);
    let list = String::from_utf8_lossy(&out.stdout);
    assert!(list.contains("llm") && list.contains("yes"), "list={list}");

    let mq = tmp.join("smoke.mq.md");
    std::fs::write(
        &mq,
        "---\nimport llm:ext/ai/llm.mq.md\n---\n\n# main\n\n> print text=ext-cli-ok\n",
    )
    .unwrap();
    let run = Command::new(bin)
        .args(["run", mq.to_str().unwrap()])
        .env("MARQDO_EXT", &tmp)
        .output()
        .expect("run smoke");
    assert_eq!(
        run.status.code().unwrap_or(1),
        0,
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim_end(),
        "ext-cli-ok"
    );

    let status = Command::new(bin)
        .args(["ext", "remove", "llm"])
        .env("MARQDO_EXT", &tmp)
        .status()
        .expect("ext remove");
    assert!(status.success());
    assert!(!tmp.join("ai").join("llm.mq.md").is_file());
}

#[test]
fn ext_cli_add_agent() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");

    let tmp = tempfile_dir("marqdo-ext-agent");
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext");
    let bin = env!("CARGO_BIN_EXE_marqdo");
    let status = Command::new(bin)
        .args(["ext", "add", "agent"])
        .env("MARQDO_EXT", &tmp)
        .env("MARQDO_EXT_SOURCE", &src)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("ext add agent");
    assert!(status.success(), "ext add agent failed");
    assert!(tmp.join("ai").join("agent.mq.md").is_file());
    assert!(tmp.join("ai").join("智能体.mq.md").is_file());
    assert!(
        tmp.join("native").join(if cfg!(windows) {
            "agent.dll"
        } else if cfg!(target_os = "macos") {
            "libagent.dylib"
        } else {
            "libagent.so"
        })
        .is_file()
            || tmp.join("agent.plugin").is_file(),
        "agent native plugin should be installed"
    );
}

#[test]
fn ext_cli_add_web() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");

    let tmp = tempfile_dir("marqdo-ext-web");
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext");
    let bin = env!("CARGO_BIN_EXE_marqdo");
    let status = Command::new(bin)
        .args(["ext", "add", "web"])
        .env("MARQDO_EXT", &tmp)
        .env("MARQDO_EXT_SOURCE", &src)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("ext add web");
    assert!(status.success(), "ext add web failed");
    assert!(tmp.join("web").join("web.mq.md").is_file());
    assert!(tmp.join("web").join("网页.mq.md").is_file());
    assert!(
        tmp.join("native").join(if cfg!(windows) {
            "web.dll"
        } else if cfg!(target_os = "macos") {
            "libweb.dylib"
        } else {
            "libweb.so"
        })
        .is_file()
            || tmp.join("web.plugin").is_file(),
        "web native plugin should be installed"
    );
}

#[test]
fn ext_web_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-smoke.mq.md",
        "render-ok
compose-nav-ok
compose-side-ok
styles-ok
compose-ok
db-ok",
    );
}

#[test]
fn ext_web_form_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-form-smoke.mq.md",
        "validate-ok
submit-ok
db-ok
render-ok",
    );
}

#[test]
fn ext_web_admin_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-admin-smoke.mq.md",
        "schema-ok
new-form-ok
insert-ok
get-ok
update-ok
edit-ok
required-ok",
    );
}

#[test]
fn ext_web_route_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-route-smoke.mq.md",
        "about-ok
docs-ok
render-ok",
    );
}

#[test]
fn ext_web_form_embed_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-form-embed-smoke.mq.md",
        "form-id-ok
render-ok",
    );
}

#[test]
fn ext_web_part_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-part-smoke.mq.md",
        "route-stamp-ok
home-part-ok
route-part-ok",
    );
}

#[test]
fn ext_web_select_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-select-smoke.mq.md",
        "map-where-ok
like-ok
all-ok",
    );
}

#[test]
fn ext_web_zh_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-zh-smoke.mq.md",
        "render-ok
compose-ok
form-ok
route-ok",
    );
}

#[test]
fn ext_web_static_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_web"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build web plugin");
    assert!(status.success(), "failed to build marqdo_plugin_web");
    assert_out(
        "tests/ext/web-static-smoke.mq.md",
        "dir-ok
mount-ok
custom-ok",
    );
}

#[test]
fn ext_quantum_bell_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out(
        "tests/ext/quantum-bell-smoke.mq.md",
        "p00-ok
p11-ok
ping-ok",
    );
}

#[test]
fn ext_quantum_gates_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out(
        "tests/ext/quantum-gates-smoke.mq.md",
        "z-sandwich-ok
ry-ok
run-ok
shots-ok",
    );
}

#[test]
fn ext_quantum_zh_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-zh-smoke.mq.md", "zh-ok");
}

#[test]
fn ext_quantum_steps_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    let path = "tests/ext/quantum-steps-smoke.mq.md";
    let (code, stdout, stderr) = run(&["run", path]);
    assert_eq!(code, 0, "{path} stderr={stderr}");
    assert!(
        stdout.contains("steps-ok"),
        "{path} missing steps-ok in {stdout}"
    );
    assert!(
        stdout.contains("draw-ok"),
        "{path} missing draw-ok in {stdout}"
    );
    let svg_path = "tests/ext/quantum-steps-draw.svg";
    let svg = std::fs::read_to_string(svg_path).unwrap_or_default();
    assert!(
        svg.contains("<svg") && svg.contains("q0"),
        "{path} bad svg at {svg_path}: {svg}"
    );
    let _ = std::fs::remove_file(svg_path);
}

#[test]
fn ext_quantum_draw_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-draw-smoke.mq.md", "probs-ok\nbloch-ok");
    for p in [
        "tests/ext/quantum-draw-probs.svg",
        "tests/ext/quantum-draw-bloch.svg",
    ] {
        let svg = std::fs::read_to_string(p).unwrap_or_default();
        assert!(svg.contains("<svg"), "expected svg at {p}");
        let _ = std::fs::remove_file(p);
    }
}

#[test]
fn ext_quantum_gate_matrix_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out(
        "tests/ext/quantum-gate-matrix-smoke.mq.md",
        "match-ok\nreject-ok",
    );
}

#[test]
fn ext_quantum_author_api_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out(
        "tests/ext/quantum-author-api-smoke.mq.md",
        "state-ok\ndraw-ok\nappend-ok",
    );
    let svg_path = "tests/ext/quantum-author-draw.svg";
    let svg = std::fs::read_to_string(svg_path).unwrap_or_default();
    assert!(
        svg.contains("<svg") && svg.contains("stroke-dasharray"),
        "expected barrier dashes in {svg_path}"
    );
    let _ = std::fs::remove_file(svg_path);
}

#[test]
fn ext_quantum_noise_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-noise-smoke.mq.md", "noise-ok");
}

#[test]
fn ext_quantum_gate_heatmap_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-gate-heatmap-smoke.mq.md", "heatmap-ok");
    let svg_path = "tests/ext/quantum-gate-heatmap.svg";
    let svg = std::fs::read_to_string(svg_path).unwrap_or_default();
    assert!(
        svg.contains("<svg") && svg.contains("H matrix"),
        "expected matrix heatmap in {svg_path}"
    );
    let _ = std::fs::remove_file(svg_path);
}

#[test]
fn ext_quantum_author_api_zh_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out(
        "tests/ext/quantum-author-api-zh-smoke.mq.md",
        "state-ok\ndraw-ok\nappend-ok",
    );
    let svg_path = "tests/ext/quantum-author-zh-draw.svg";
    let svg = std::fs::read_to_string(svg_path).unwrap_or_default();
    assert!(
        svg.contains("<svg") && svg.contains("stroke-dasharray"),
        "expected barrier dashes in {svg_path}"
    );
    let _ = std::fs::remove_file(svg_path);
}

#[test]
fn ext_quantum_noise_zh_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-noise-zh-smoke.mq.md", "noise-ok");
}

#[test]
fn ext_quantum_amp_damp_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-amp-damp-smoke.mq.md", "amp-ok");
}

#[test]
fn ext_quantum_amp_damp_zh_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-amp-damp-zh-smoke.mq.md", "amp-ok");
}

#[test]
fn ext_quantum_custom_gate_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-custom-gate-smoke.mq.md", "custom-ok");
}

#[test]
fn ext_quantum_custom_gate_zh_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_quantum"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build quantum plugin");
    assert!(status.success(), "failed to build marqdo_plugin_quantum");
    assert_out("tests/ext/quantum-custom-gate-zh-smoke.mq.md", "custom-ok");
}

#[test]
fn ext_agent_framework_smoke() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");

    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/agent-smoke.mq.md"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run agent-smoke");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "agent-smoke stderr={stderr}");
    assert_eq!(
        stdout.trim_end(),
        "skill-ok\ntools-ok\nsource-ok\nprotocol-ok\n获取时间\n获取时间\nok-time\n2\n0\nmain",
        "agent-smoke"
    );
}

#[test]
fn ext_fs_text_patch() {
    assert_out(
        "tests/ext/fs-text-patch.mq.md",
        "alpha BETA gamma\n2\nALPHA BETA GAMMA",
    );
}

#[test]
fn ext_agent_inspect_workbook() {
    assert_out(
        "tests/ext/agent-inspect-fixture.mq.md",
        "0\nok-slot\npending",
    );
}

#[test]
fn ext_agent_plan_confirm() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");

    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/agent-plan-confirm.mq.md"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run agent-plan-confirm");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "agent-plan-confirm stderr={stderr}");
    assert_eq!(
        stdout.trim_end(),
        "pending\nworkbook-ok\nskeleton-ok",
        "agent-plan-confirm"
    );
}

#[test]
fn ext_agent_plan_decision() {
    assert_out(
        "tests/ext/agent-plan-decision.mq.md",
        "DONE\nall good\nCONTINUE\nDONE\nRUN\nRUN\ncall\nlib_catalog\nread\nstderr\ndecision\ncatalog-ok\ndual-ok\nsolidify-ok\n1\nkeep new keep",
    );
}

#[test]
fn ext_agent_plan_observe() {
    assert_out(
        "tests/ext/agent-plan-observe.mq.md",
        "excerpt-stripped\nhas-value-ok\nread-source-ok",
    );
}

#[test]
fn ext_llm_complete_live() {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/llm-complete.mq.md"])
        .output()
        .expect("run llm-complete");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "llm-complete stderr={stderr}");
    let reply = stdout.trim_end().to_lowercase();
    assert!(
        reply.contains("pong"),
        "expected pong in reply, got {stdout:?}"
    );
}

#[test]
fn ext_llm_stream_live() {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/llm-stream-live.mq.md"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run llm-stream-live");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "llm-stream-live stderr={stderr}");
    let lines: Vec<&str> = stdout.trim_end().lines().collect();
    assert!(
        lines.len() >= 2,
        "llm-stream-live expected echo line + count + answer, got {stdout:?}"
    );
    let answer = lines.last().unwrap().to_lowercase();
    assert!(
        answer.contains("pong"),
        "expected pong in streamed answer, got {stdout:?}"
    );
    let n: usize = lines[lines.len() - 2]
        .parse()
        .unwrap_or(0);
    assert!(n >= 2, "expected at least delta+done events, n={n} stdout={stdout:?}");
}

#[test]
fn ext_agent_run_live() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");

    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/agent-run-live.mq.md"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run agent-run-live");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "agent-run-live stderr={stderr}");
    let lines: Vec<&str> = stdout.trim_end().lines().collect();
    assert!(lines.len() >= 3, "agent-run-live stdout={stdout}");
    assert!(
        lines[0].chars().any(|c| c.is_ascii_digit()),
        "expected date in reply, got {:?}",
        lines[0]
    );
    assert_eq!(lines[1], "2", "history after step");
    assert_eq!(lines[2], "0", "history after clear");
    // Caller may persist the step map via lib/writeback; agent itself does not write.
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ext/agent-run-live.mq.md"),
    )
    .expect("read agent-run-live after run");
    assert!(
        src.contains("\"status\"") || src.contains("marqdo-out"),
        "expected caller writeback of step result map"
    );
    assert!(
        !stdout.contains("marqdo-out"),
        "writeback must not be printed to stdout"
    );
}

#[test]
fn ext_agent_plan_live() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");

    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/agent-plan-live.mq.md"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run agent-plan-live");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "agent-plan-live stderr={stderr}");
    let lines: Vec<&str> = stdout.trim_end().lines().collect();
    assert!(
        lines.iter().any(|l| *l == "ok"),
        "agent-plan-live missing ok: stdout={stdout}"
    );
    assert!(
        lines.iter().any(|l| *l == "workbook-ok"),
        "agent-plan-live missing workbook-ok: stdout={stdout}"
    );
    assert!(
        lines.iter().filter(|l| **l == "hit").count() >= 1,
        "agent-plan-live expected OKF hit: stdout={stdout}"
    );
}

#[test]
fn ext_agent_kb_reuse() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");

    let goal = "Reply with exactly the word pong and nothing else.";
    let mut h: u64 = 5381;
    for b in goal.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u64::from(*b));
    }
    let sig = format!("{h:016x}")[..12].to_string();
    let slug = "reply-with-exactly-the-word-pong-and-nothing-els";
    // Default file-subtask quiet: no child print on parent stdout; answer via wait.value.
    let expect = format!("{sig}\n{slug}\npromoted\nstable\n{slug}\nlookup-ok\n0\npong\nstable-path");
    assert_out("tests/ext/agent-kb-reuse.mq.md", &expect);
}

#[test]
fn ext_agent_kb_alias() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");
    assert_out(
        "tests/ext/agent-kb-alias.mq.md",
        "promoted
alias-written
alias-ok
alias
same-slug
exact
trip-promoted
帮我规划明天的行程
canonical-ok
canonical
list-ok",
    );
}

#[test]
fn ext_agent_kb_near() {
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_agent"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("build agent plugin");
    assert!(status.success(), "failed to build marqdo_plugin_agent");
    assert_out(
        "tests/ext/agent-kb-near.mq.md",
        "promoted
near-ok
near
same-slug
off-miss
neg-miss
rank-ok
prompt-score-ok",
    );
}

#[test]
fn ext_agent_soft_match() {
    assert_out(
        "tests/ext/agent-soft-match.mq.md",
        "REUSE
trip-plan
NEW
REUSE
行程-a
prompt-ok",
    );
}

#[test]
fn ext_agent_lib_call() {
    assert_out(
        "tests/ext/agent-lib-call.mq.md",
        "call
lib.fs.exists
exists-ok
deny-ok
callable-ok",
    );
}

#[test]
fn lib_net_encode() {
    assert_out("tests/lib/net-encode.mq.md", "a+b");
}

#[test]
fn lib_net_openai_sse() {
    assert_out(
        "tests/lib/net-openai-sse.mq.md",
        "Hel
lo
Hello",
    );
}

#[test]
fn lib_net_openai_sse_reasoning() {
    assert_out(
        "tests/lib/net-openai-sse-reasoning.mq.md",
        "3
reasoning
think
delta
Hi
done
Hi",
    );
}

#[test]
fn ext_llm_stream_offline() {
    assert_out("tests/ext/llm-stream-offline.mq.md", "Hi!");
}

#[test]
fn ext_llm_ctor_offline() {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", "tests/ext/llm-ctor-offline.mq.md"])
        .env("OPENAI_API_KEY", "sk-test")
        .env_remove("OPENAI_MODEL")
        .env_remove("MARQDO_LLM_MODEL")
        .output()
        .expect("run llm-ctor-offline");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "llm-ctor-offline stderr={stderr}");
    assert_eq!(
        stdout.trim_end(),
        "gpt-4o-mini\n/chat/completions",
        "stdout={stdout}"
    );
}

#[test]
fn ext_agent_plan_stream_offline() {
    assert_out(
        "tests/ext/agent-plan-stream-offline.mq.md",
        "4
round
delta
Hi
decision
done
child
pending
1
done
await
2
decompose",
    );
}

#[test]
fn lib_math_formula() {
    assert_out(
        "tests/lib/math-formula.mq.md",
        "formula\n2*x\n[-1.4142135624, 1.4142135624]\n7",
    );
}

#[test]
fn lib_math_formula_chain() {
    assert_out(
        "tests/lib/math-formula-chain.mq.md",
        "formula\n3*2*x\n12",
    );
}

#[test]
fn lib_math_random() {
    assert_out("tests/lib/math-random.mq.md", "4\n7\n4");
}

#[test]
fn lib_math_zh_num() {
    assert_out("tests/lib/数学-数值.mq.md", "num\n0");
}

#[test]
fn lib_math_plot() {
    let path = "tests/lib/math-plot.mq.md";
    let (code, stdout, stderr) = run(&["run", path]);
    assert_eq!(code, 0, "{path} stderr={stderr}");
    assert!(
        stdout.lines().next() == Some("text"),
        "{path} stdout={stdout}"
    );
    assert!(
        stdout.contains("plot:"),
        "{path} missing plot: line in {stdout}"
    );
    let _ = std::fs::remove_file("tests/lib/math-plot-plot-1.svg");
}

fn python_available() -> bool {
    for cmd in ["python", "python3"] {
        if let Ok(out) = Command::new(cmd)
            .args(["-c", "print(1)"])
            .output()
        {
            if out.status.success() {
                return true;
            }
        }
    }
    false
}

#[test]
fn lib_foreign_python() {
    if !python_available() {
        eprintln!("skip lib_foreign_python: no python on PATH");
        return;
    }
    assert_out("tests/lib/foreign-python.mq.md", "hello-from-python");
}

#[test]
fn lib_foreign_run_lang() {
    if !python_available() {
        eprintln!("skip lib_foreign_run_lang: no python on PATH");
        return;
    }
    assert_out("tests/lib/foreign-run-lang.mq.md", "4");
}

#[test]
fn lib_foreign_zh() {
    if !python_available() {
        eprintln!("skip lib_foreign_zh: no python on PATH");
        return;
    }
    assert_out("tests/lib/外联-python.mq.md", "中文外联");
}

#[test]
fn bytecode_lib_math_formula() {
    assert_out_backend(
        "tests/lib/math-formula.mq.md",
        "bytecode",
        "formula\n2*x\n[-1.4142135624, 1.4142135624]\n7",
    );
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

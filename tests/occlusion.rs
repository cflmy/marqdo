//! `<TAB>` M2: occlusion syntax end-to-end (CLI + `MARQDO_MARKUP=occlusion`).
//!
//! The mode flag keeps `[键](集合)` (classic index) intact until M7 migrates the
//! corpus; these tests pin both sides of the switch.

use std::process::Command;

fn run(path: &str, markup: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_marqdo"));
    cmd.args(["run", path]);
    if let Some(m) = markup {
        cmd.env("MARQDO_MARKUP", m);
    }
    let out = cmd.output().expect("run marqdo");
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn occlusion_value_is_payload() {
    let (code, stdout, stderr) = run("tests/occlusion/value.mq.md", Some("occlusion"));
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "a=2\nb=42\nc=True");
}

#[test]
fn occlusion_off_keeps_classic_index() {
    let (code, stdout, stderr) = run("tests/occlusion/classic-index.mq.md", None);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "值");
}

#[test]
fn occlusion_mode_rejects_legacy_index_shape() {
    // In occlusion mode `[键](集合)` is read as an occlusion whose payload is the
    // collection — i.e. it returns the *whole* map, not the member. This is why
    // M7 must migrate before flipping the default; here we only pin that the
    // shape no longer means "index".
    let (code, stdout, stderr) = run("tests/occlusion/classic-index.mq.md", Some("occlusion"));
    assert_eq!(code, 0, "stderr={stderr}");
    assert_ne!(stdout.trim_end(), "值", "occlusion mode must not index");
}

// ---- M2.5: label diagnostics (`label.missing` / `label.empty_payload`) ----

fn check(path: &str, markup: &str) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["check", path])
        .env("MARQDO_MARKUP", markup)
        .output()
        .expect("run marqdo check");
    (
        out.status.code().unwrap_or(1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn label_missing_is_an_error() {
    let (code, text) = check("tests/occlusion/empty-label.mq.md", "occlusion");
    assert_eq!(code, 1, "text={text}");
    assert!(text.contains("遮蔽缺少描述"), "text={text}");
}

#[test]
fn label_empty_payload_is_a_warning() {
    let (code, text) = check("tests/occlusion/empty-payload.mq.md", "occlusion");
    assert_eq!(code, 0, "warnings must not fail the build: text={text}");
    assert!(text.contains("空载荷遮蔽"), "text={text}");
}

#[test]
fn label_ok_has_no_diagnostics() {
    let (code, text) = check("tests/occlusion/clean.mq.md", "occlusion");
    assert_eq!(code, 0, "text={text}");
    assert!(!text.contains("空载荷"), "text={text}");
    assert!(!text.contains("缺少描述"), "text={text}");
}

// ---- M4: L1 reference consistency (`label.ref_dangling`) ----

#[test]
fn label_ref_dangling_is_an_error() {
    let (code, text) = check("tests/occlusion/ref-dangling.mq.md", "occlusion");
    assert_eq!(code, 1, "text={text}");
    assert!(text.contains("描述引用了不存在的实体"), "text={text}");
}

#[test]
fn label_refs_that_resolve_are_fine() {
    let (code, text) = check("tests/occlusion/ref-ok.mq.md", "occlusion");
    assert_eq!(code, 0, "text={text}");
    assert!(!text.contains("不存在的实体"), "text={text}");
}

// ---- M9: two-tier label search (`labels` CLI + MLSP `labels.search`) ----

fn labels(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .arg("labels")
        .args(args)
        .env("MARQDO_MARKUP", "occlusion")
        .output()
        .expect("run marqdo labels");
    (
        out.status.code().unwrap_or(1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn labels_pending_uses_builtin_groups() {
    let (code, text) = labels(&["tests/occlusion/labels.mq.md", "--count"]);
    assert_eq!(code, 0, "text={text}");
    assert_eq!(text.trim(), "2", "expected 待开发 + 模拟: {text}");
}

#[test]
fn labels_grep_replaces_builtin() {
    let (code, text) = labels(&["tests/occlusion/labels.mq.md", "--grep", "联调", "--count"]);
    assert_eq!(code, 0, "text={text}");
    assert_eq!(text.trim(), "1", "text={text}");
}

#[test]
fn labels_grep_with_builtin_unions() {
    let (code, text) = labels(&[
        "tests/occlusion/labels.mq.md",
        "--grep",
        "联调",
        "--also-builtin",
        "--count",
    ]);
    assert_eq!(code, 0, "text={text}");
    assert_eq!(text.trim(), "3", "text={text}");
}

#[test]
fn mlsp_labels_search_returns_hits() {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .arg("mlsp")
        .env("MARQDO_MARKUP", "occlusion")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn mlsp");
    let req = r#"{"id":1,"method":"labels.search","params":{"path":"tests/occlusion/labels.mq.md","also_builtin":true}}"#;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(format!("{req}\n").as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("\"count\":2"), "text={text}");
    assert!(text.contains("客户留存率"), "text={text}");
}

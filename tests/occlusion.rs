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

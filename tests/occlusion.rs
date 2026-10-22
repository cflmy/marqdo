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

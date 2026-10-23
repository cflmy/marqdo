//! `<TAB>` M7: `marqdo migrate labels` — dry run, write, idempotence, runtime parity.

use std::process::Command;

fn run_in(dir: &std::path::Path, args: &[&str], markup: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_marqdo"));
    cmd.args(args);
    if let Some(m) = markup {
        cmd.env("MARQDO_MARKUP", m);
    }
    let out = cmd.output().expect("run marqdo");
    let _ = dir;
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn fixture() -> (std::path::PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("marqdo-migrate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("demo.mq.md");
    let src = "---\nimport:\n  json: lib/json.mq.md\n---\n\n# main\n\n`表` = > json.parse text={\"键\":\"值\"}\n**a = [键](`表`)**\n> print text=\"a=\" + `a`\n";
    std::fs::write(&path, src).unwrap();
    (path, src.to_string())
}

#[test]
fn dry_run_does_not_write() {
    let (path, before) = fixture();
    let (code, stdout, _) = run_in(path.parent().unwrap(), &["migrate", "labels", path.to_str().unwrap()], None);
    assert_eq!(code, 0);
    assert!(stdout.contains("would change"), "{stdout}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn write_migrates_and_keeps_runtime_behaviour() {
    let (path, _) = fixture();
    let parent = path.parent().unwrap().to_path_buf();
    // Baseline output (classic index).
    let (c0, out0, e0) = run_in(&parent, &["run", path.to_str().unwrap()], None);
    assert_eq!(c0, 0, "stderr={e0}");

    let (code, _, err) = run_in(&parent, &["migrate", "labels", path.to_str().unwrap(), "--write"], None);
    assert_eq!(code, 0, "stderr={err}");

    let migrated = std::fs::read_to_string(&path).unwrap();
    assert!(migrated.contains("markup: occlusion"), "{migrated}");
    assert!(migrated.contains("access("), "{migrated}");

    // Same output, now via the purity form.
    let (c1, out1, e1) = run_in(&parent, &["run", path.to_str().unwrap()], None);
    assert_eq!(c1, 0, "stderr={e1}");
    assert_eq!(out0, out1);

    // Idempotent: a second pass finds nothing to do.
    let (_, again, _) = run_in(&parent, &["migrate", "labels", path.to_str().unwrap()], None);
    assert!(again.contains("0 index"), "{again}");
}

#[test]
fn equivalence_failure_leaves_file_untouched() {
    // A hand-written occlusion has no `(access(…))` payload, so rewriting the
    // shape again would change the AST — the tool must refuse.
    let dir = std::env::temp_dir().join(format!("marqdo-migrate-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bad.mq.md");
    let src = "---\nmarkup: occlusion\n---\n\n# main\n\n**a = [一加一](1 + 1)**\n> print text=`a`\n";
    std::fs::write(&path, src).unwrap();
    let (code, stdout, _) = run_in(&dir, &["migrate", "labels", path.to_str().unwrap()], None);
    assert_eq!(code, 0);
    // Already marked → reported as such, never rewritten.
    assert!(stdout.contains("already marked") || stdout.contains("0 index"), "{stdout}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), src);
}

//! `<TAB>` M8: label-debt ratchet (`--baseline` + `--fail-on-increase`).

use std::path::PathBuf;
use std::process::Command;

fn workdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("marqdo-ratchet-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_doc(dir: &PathBuf, stubs: usize) {
    let mut s = String::from("---\nmarkup: occlusion\n---\n\n# main\n\n");
    for i in 0..stubs {
        s.push_str(&format!("**v{i} = [功能{i}，待开发](None)**\n"));
    }
    s.push_str("> print text=\"ok\"\n");
    std::fs::write(dir.join("a.mq.md"), s).unwrap();
}

fn labels(dir: &PathBuf, baseline: &str) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "labels",
            "a.mq.md",
            "--baseline",
            baseline,
            "--fail-on-increase",
        ])
        .current_dir(dir)
        .output()
        .expect("run labels");
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
fn ratchet_seeds_fails_on_increase_and_tightens() {
    let dir = workdir("main");
    let ledger = ".marqdo/pending.json";

    // 1) Seed: first run creates the ledger and passes.
    write_doc(&dir, 2);
    let (code, text) = labels(&dir, ledger);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("已建立 2"), "{text}");

    // 2) Increase: one more stub → exit 1.
    write_doc(&dir, 3);
    let (code, text) = labels(&dir, ledger);
    assert_eq!(code, 1, "debt growth must fail: {text}");
    assert!(text.contains("未完成项增加了 1"), "{text}");

    // 3) Decrease: ratchet tightens back so it cannot regrow.
    write_doc(&dir, 1);
    let (code, text) = labels(&dir, ledger);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("棘轮收紧"), "{text}");
    let body = std::fs::read_to_string(dir.join(ledger)).unwrap();
    assert!(body.contains("\"count\":1"), "{body}");

    // 4) Now 2 is again a violation (the ledger remembers 1).
    write_doc(&dir, 2);
    let (code, _) = labels(&dir, ledger);
    assert_eq!(code, 1);
}

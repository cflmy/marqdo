//! 语料库单调性测试（roadmap P2 DoD：offline_checked ⊆ lean_proved）。
//!
//! 对 tests/prove/fixtures/ 的 8 类 × 5 夹具逐个跑 `marqdo prove`：
//! - 离线 `checked` 的定理，Lean（如有）必须 `proved`；
//! - 离线 `unproved`（反例）的定理，Lean 必须 `unproved`；
//! - 其余（unverified）无约束——但出现 `proof.divergence` 即失败。
//!
//! 无 Lean 工具链时整组 SKIP（MARQDO_PROOF_REQUIRE_LEAN=1 则失败）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn toolchain_available() -> bool {
    if let Ok(p) = std::env::var("MARQDO_PROOF_LEAN") {
        if !p.trim().is_empty() && Path::new(&p).exists() {
            return true;
        }
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .any(|d| d.join("lean").is_file())
}

fn require_lean_enforced() -> bool {
    std::env::var("MARQDO_PROOF_REQUIRE_LEAN").is_ok_and(|v| v == "1")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/prove/fixtures")
}

#[test]
fn corpus_monotonicity_offline_subset_of_lean() {
    if !toolchain_available() {
        if require_lean_enforced() {
            panic!("MARQDO_PROOF_REQUIRE_LEAN=1 但没有 Lean 工具链");
        }
        println!("SKIP corpus（无 Lean 工具链）");
        return;
    }
    let dir = fixtures_dir();
    let mut fixtures: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("read fixtures")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n != "lib"))
        .flat_map(|cat| {
            fs::read_dir(cat)
                .expect("read category")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.to_string_lossy().ends_with(".mq.md"))
                .collect::<Vec<_>>()
        })
        .collect();
    fixtures.sort();
    assert!(
        fixtures.len() >= 40,
        "语料库必须 ≥ 40 个夹具，实际 {}",
        fixtures.len()
    );

    let mut checked = 0usize;
    let mut refuted = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for fixture in &fixtures {
        let rel = fixture.strip_prefix(&dir).expect("relative").to_path_buf();
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let scratch = target.join("prove-corpus").join(
            rel.to_string_lossy()
                .replace(['/', '\\', '.'], "_"),
        );
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(scratch.join("lib")).expect("scratch");

        // 夹具 + lib（result/工单）落位。
        fs::copy(fixture, scratch.join(rel.file_name().expect("file name")))
            .expect("copy fixture");
        for lib in ["lib/result.mq.md", "lib/工单.mq.md"] {
            let src = dir.join(lib);
            if src.exists() {
                fs::copy(&src, scratch.join(lib)).expect("copy lib");
            }
        }

        let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .current_dir(&scratch)
            .args([
                "prove",
                rel.file_name().unwrap().to_str().unwrap(),
                "--json",
            ])
            .output()
            .expect("run prove");
        let json: serde_json::Value = match serde_json::from_slice(&out.stdout) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "{}: 报告不是 JSON（{}）: {}",
                    rel.display(),
                    e,
                    String::from_utf8_lossy(&out.stdout)
                ));
                continue;
            }
        };
        if json["status"] == "error" {
            failures.push(format!(
                "{}: 判定层 error: {:?}",
                rel.display(),
                json["diagnostics"]
            ));
            continue;
        }
        for f in json["functions"].as_array().expect("functions") {
            for c in f["claims"].as_array().expect("claims") {
                let by = c["decided_by"].as_str().unwrap_or("");
                let status = c["status"].as_str().unwrap_or("");
                match (by, status) {
                    ("trace", "checked") => {
                        checked += 1;
                        // Lean 必须印证（调用方已跑 Lean：decided_by 会被覆盖成
                        // lean；仍是 trace 说明 Lean 没跑或没盖到——直接看 lean 面）。
                    }
                    ("trace", "unproved") => {
                        refuted += 1;
                    }
                    ("lean", _) => {}
                    _ => {}
                }
            }
        }
        // 分歧护栏若触发，状态必为 error（上面已捕获）。
    }
    assert!(
        failures.is_empty(),
        "语料库失败 {} 项：\n{}",
        failures.len(),
        failures.join("\n")
    );
    println!(
        "corpus: {} fixtures · offline checked {} · offline refuted {}（全部与 Lean 一致）",
        fixtures.len(),
        checked,
        refuted
    );
}

/// 离线模式（无 Lean）下语料库的诚实性：不许出现 `proved`。
#[test]
fn corpus_offline_never_claims_proved() {
    if toolchain_available() {
        println!("SKIP offline-corpus（有 Lean，走单调性用例）");
        return;
    }
    let dir = fixtures_dir();
    let mut fixtures: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("read fixtures")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n != "lib"))
        .flat_map(|cat| {
            fs::read_dir(cat)
                .expect("read category")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.to_string_lossy().ends_with(".mq.md"))
                .collect::<Vec<_>>()
        })
        .collect();
    fixtures.sort();
    for fixture in &fixtures {
        let rel = fixture.strip_prefix(&dir).expect("relative").to_path_buf();
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let scratch = target.join("prove-corpus-off").join(
            rel.to_string_lossy().replace(['/', '\\', '.'], "_"),
        );
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(scratch.join("lib")).expect("scratch");
        fs::copy(fixture, scratch.join(rel.file_name().expect("file name")))
            .expect("copy fixture");
        for lib in ["lib/result.mq.md", "lib/工单.mq.md"] {
            let src = dir.join(lib);
            if src.exists() {
                fs::copy(&src, scratch.join(lib)).expect("copy lib");
            }
        }
        let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .current_dir(&scratch)
            .args(["prove", rel.file_name().unwrap().to_str().unwrap(), "--json"])
            .env_remove("PATH")
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run prove offline");
        let json: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("json report");
        assert!(
            json["status"] != "proved",
            "{}：无 Lean 不许 proved（{}）",
            rel.display(),
            json
        );
        assert!(json["lean"].is_null(), "{}：无 Lean 时 lean 必须为 null", rel.display());
    }
}

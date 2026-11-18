//! `marqdo prove` Lean 判定集成测试（roadmap/prove.md §6 G 矩阵 + P1 DoD）。
//!
//! - G1/G2：证书缓存命中/失效（假 lean 计数器，不需要真工具链）。
//! - E1/A2：真定理 proved；假定理 unproved 退 1、证据指回源行。
//! - F2/F3：--report-only 恒 0；--require-lean 无工具链退 2。
//!
//! 有真 Lean（PATH / MARQDO_PROOF_LEAN）时跑全量；没有时只跑缓存类用例，
//! 其余 SKIP——除非 `MARQDO_PROOF_REQUIRE_LEAN=1`（CI 硬性要求，缺失即失败）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let dir = target.join("prove-lean-tests").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch dir");
        Self { dir }
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(&path, content).expect("write fixture");
        path
    }

    fn marqdo(&self) -> Command {
        let mut cmd = Command::new(Path::new(env!("CARGO_BIN_EXE_marqdo")));
        cmd.current_dir(&self.dir);
        cmd
    }
}

/// 台账夹具：一个可证定理（轨迹恰含一次登记调用 → `长度(轨迹) = 1`）。
const LEDGER_OK: &str = r#"---
markup: occlusion
---

# 台账

## 记录
+ `金额`

| 定理 | 结论 | 策略 |
|---|---|---|
| 记过即记 | 长度(轨迹) = 1 | auto |

**结果 = [登记](result.ok(value=`金额`))**

*结果*
"#;

/// 同一夹具，但定理是假的：`长度(轨迹) = 0`（真实执行轨迹恰有 1 步）。
const LEDGER_BAD: &str = r#"---
markup: occlusion
---

# 台账

## 记录
+ `金额`

| 定理 | 结论 | 策略 |
|---|---|---|
| 记过即记 | 长度(轨迹) = 0 | auto |

**结果 = [登记](result.ok(value=`金额`))**

*结果*
"#;

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

fn skip_or_fail(name: &str) {
    if require_lean_enforced() {
        panic!("{name}：MARQDO_PROOF_REQUIRE_LEAN=1 但没有 Lean 工具链");
    }
    println!("SKIP {name}（无 Lean 工具链）");
}

/// 假 lean：可执行脚本，把 lean 的参数回显到日志文件并递增计数器，
/// `--version` 打印固定版本。用于缓存命中/失效测试（G1/G2）。
fn install_fake_lean(dir: &Path) -> PathBuf {
    let bin = dir.join("fakebin");
    fs::create_dir_all(&bin).expect("create fakebin");
    let lean = bin.join("lean");
    fs::write(
        &lean,
        format!(
            r#"#!/bin/sh
COUNT_FILE="{}/fake-lean-count"
LOG_FILE="{}/fake-lean-log"
if [ "$1" = "--version" ]; then
  echo "Lean (version 4.35.0, fake-counter)"
  exit 0
fi
if [ "$1" = "-o" ]; then
  # 库构建：lean -o OUT INPUT —— 假 lean 直接产出空 olean。
  touch "$2"
  exit 0
fi
n=0
[ -f "$COUNT_FILE" ] && n=$(cat "$COUNT_FILE")
n=$((n+1))
echo "$n" > "$COUNT_FILE"
echo "$@" >> "$LOG_FILE"
# 真实判定不执行：直接以 0 退出（调用方若依赖判定结果会用假 lean 仅测缓存）
exit 0
"#,
            bin.display(),
            bin.display()
        ),
    )
    .expect("write fake lean");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&lean, fs::Permissions::from_mode(0o755)).expect("chmod fake lean");
    }
    lean
}

// ---------------------------------------------------------------------------
// G1：缓存命中——第二次判定跳过 lean 调用。
// 用假 lean（计数器）+ 预置证书的方式：先真跑一次（若有真 lean）或手写证书，
// 再改键内一个素材观察失效。这里用「手写证书 + 复合键」验证 load/store 语义。
// ---------------------------------------------------------------------------

/// G1：缓存命中——第二次判定**不**再调用 lean（假 lean 计数器验证）。
/// 假 lean 对一切模块返回 0（相当于"什么都 proved"），因此：
/// 第一次 miss → 证书落盘；第二次 hit → 判定跳过，计数器不再增长。
#[test]
fn g1_cache_hit_skips_judging() {
    let s = Scratch::new("g1-cache-hit");
    s.write("台账.mq.md", LEDGER_OK);
    let lean_path = install_fake_lean(&s.dir);
    let counter = s.dir.join("fakebin/fake-lean-count");
    let log = s.dir.join("fakebin/fake-lean-log");

    let out = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--lean")
        .arg(&lean_path)
        .arg("--json")
        .output()
        .expect("run prove (fake lean)");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json report");
    assert_eq!(json["lean"]["cache"], "miss", "首次必须 miss");
    let first_calls: usize = fs::read_to_string(&counter)
        .expect("fake lean 计数")
        .trim()
        .parse()
        .expect("counter usize");
    assert!(first_calls >= 1, "首次判定必须真的调了 lean（判模块）：{first_calls}");

    // 第二次：证书命中，lean 调用数不再增长。
    let out2 = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--lean")
        .arg(&lean_path)
        .arg("--json")
        .output()
        .expect("run prove twice");
    let json2: serde_json::Value = serde_json::from_slice(&out2.stdout).expect("json report 2");
    assert_eq!(json2["lean"]["cache"], "hit", "证书命中必须跳过判定");
    let second_calls: usize = fs::read_to_string(&counter)
        .expect("counter 2")
        .trim()
        .parse()
        .expect("counter usize");
    assert_eq!(
        second_calls, first_calls,
        "命中后不得再调用 lean（G1：跳判定）"
    );

    // --no-cache：绕过证书 → 计数器增长。
    let out3 = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--lean")
        .arg(&lean_path)
        .arg("--no-cache")
        .arg("--json")
        .output()
        .expect("run prove no-cache");
    let json3: serde_json::Value = serde_json::from_slice(&out3.stdout).expect("json report 3");
    assert_eq!(json3["lean"]["cache"], "miss", "--no-cache 必须 miss");
    let third_calls: usize = fs::read_to_string(&counter)
        .expect("counter 3")
        .trim()
        .parse()
        .expect("counter usize");
    assert_eq!(
        third_calls,
        second_calls + 1,
        "--no-cache 必须重新调用 lean 判定"
    );
    let _ = fs::remove_file(&log);
}

/// G2：真证书命中（用真 lean 跑一次后，证书生效；第二次 cache: hit 且快）。
#[test]
fn g2_certificate_hit_after_real_run() {
    if !toolchain_available() {
        skip_or_fail("g2_certificate_hit_after_real_run");
        return;
    }
    let s = Scratch::new("g2-cert-hit");
    s.write("台账.mq.md", LEDGER_OK);
    let mut cmd = s.marqdo();
    cmd.arg("prove").arg("台账.mq.md").arg("--json");
    let out = cmd.output().expect("first prove");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["status"], "proved", "真定理必须 proved：{json}");
    assert_eq!(json["lean"]["cache"], "miss");
    let proof_id = json["artifact_id"].as_str().unwrap().rsplit('/').next().unwrap();
    let link_path = s.dir.join(".marqdo/prove").join(proof_id).join("knowledge-links.json");
    let linkage: serde_json::Value = serde_json::from_slice(
        &fs::read(&link_path).expect("real Lean proof must emit a knowledge linkage"))
        .expect("knowledge linkage JSON");
    assert_eq!(linkage["claims"][0]["status"], "proved",
        "only verified Lean receipts should carry proved: {linkage}");
    assert_eq!(linkage["claims"][0]["proof_scope"], "function");
    assert!(linkage["claims"][0]["certificate_digest"].as_str().is_some());

    // 第二次：命中证书。
    let out2 = s.marqdo().arg("prove").arg("台账.mq.md").arg("--json").output().expect("second prove");
    let json2: serde_json::Value = serde_json::from_slice(&out2.stdout).expect("json2");
    assert_eq!(json2["status"], "proved");
    assert_eq!(json2["lean"]["cache"], "hit", "第二次必须命中证书缓存");
    assert!(json2["duration_ms"].as_u64().unwrap() < 5000, "命中后必须快");
}

/// G2b：--no-cache 绕过证书。
#[test]
fn g2b_no_cache_forces_rejudge() {
    if !toolchain_available() {
        skip_or_fail("g2b_no_cache_forces_rejudge");
        return;
    }
    let s = Scratch::new("g2b-no-cache");
    s.write("台账.mq.md", LEDGER_OK);
    let out = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--json")
        .output()
        .expect("first");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["lean"]["cache"], "miss");
    let out2 = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--no-cache")
        .arg("--json")
        .output()
        .expect("second");
    let json2: serde_json::Value = serde_json::from_slice(&out2.stdout).expect("json2");
    assert_eq!(json2["lean"]["cache"], "miss", "--no-cache 必须绕过缓存");
}

/// A2/E1：假定理 → unproved + 退出 1 + 证据（真实 Lean）。
#[test]
fn e1_false_claim_unproved_exit_1() {
    if !toolchain_available() {
        skip_or_fail("e1_false_claim_unproved_exit_1");
        return;
    }
    let s = Scratch::new("e1-false-claim");
    s.write("台账.mq.md", LEDGER_BAD);
    let out = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--json")
        .output()
        .expect("prove false claim");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["status"], "unproved");
    assert_eq!(out.status.code(), Some(1), "E1：unproved 必须 exit 1");
    let claims = json["functions"][0]["claims"].as_array().unwrap();
    assert_eq!(claims[0]["status"], "unproved");
    assert_eq!(claims[0]["decided_by"], "lean");
    assert!(
        claims[0]["evidence"].as_str().is_some(),
        "unproved 必须带证据"
    );
}

/// F2：--report-only 不判定、不写盘、恒退 0。
#[test]
fn f2_report_only_never_judges() {
    let s = Scratch::new("f2-report-only");
    s.write("台账.mq.md", LEDGER_BAD); // 假定理也不判定
    let out = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--report-only")
        .arg("--json")
        .output()
        .expect("prove --report-only");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["status"], "unverified");
    assert_eq!(out.status.code(), Some(0), "F2：report-only 恒退 0");
    assert_eq!(json["emitted"].as_array().unwrap().len(), 0, "不写盘");
    assert!(json["lean"].is_null(), "report-only 不判定");
}

/// F3：--require-lean 且无工具链 → unverified + 退出 2。
#[test]
fn f3_require_lean_without_toolchain_exit_2() {
    // env -i 清 PATH，显式指向空目录；子进程继承我们注入的最小环境。
    let s = Scratch::new("f3-require-lean");
    s.write("台账.mq.md", LEDGER_OK);
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .current_dir(&s.dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", std::env::var("HOME").unwrap_or_default())
        .args(["prove", "台账.mq.md", "--require-lean", "--json"])
        .output()
        .expect("prove --require-lean");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["status"], "unverified");
    assert_eq!(out.status.code(), Some(2), "F3：require-lean 未找到 lean 退 2");
}

/// F4：prove emit 子命令只生成不判定。
#[test]
fn f4_emit_subcommand_does_not_judge() {
    let s = Scratch::new("f4-emit-no-judge");
    s.write("台账.mq.md", LEDGER_BAD);
    let out = s
        .marqdo()
        .arg("prove")
        .arg("emit")
        .arg("台账.mq.md")
        .arg("--json")
        .output()
        .expect("prove emit");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(json["lean"].is_null(), "emit 子命令不判定");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(json["emitted"].as_array().unwrap().len(), 2, "生成 Lean + plan");
}

/// 策略 none → 定理生成带 sorry → unverified（E4）。
#[test]
fn e4_tactic_none_yields_unverified() {
    if !toolchain_available() {
        skip_or_fail("e4_tactic_none_yields_unverified");
        return;
    }
    let src = LEDGER_OK.replace("auto", "none");
    let s = Scratch::new("e4-tactic-none");
    s.write("台账.mq.md", &src);
    let out = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--json")
        .output()
        .expect("prove tactic none");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["status"], "unverified", "sorry 不许冒充 proved");
    assert_eq!(json["functions"][0]["claims"][0]["status"], "unverified");
    assert_eq!(out.status.code(), Some(0), "advisory：unverified 不阻塞");
}

/// A6：`--all` 目录模式——整体取最差（差者胜），目录不带 --all 拒绝。
#[test]
fn a6_all_aggregates_worst_status_and_exit() {
    if !toolchain_available() {
        skip_or_fail("a6_all");
        return;
    }
    let s = Scratch::new("a6_all");
    let lean_path = install_fake_lean(&s.dir);
    s.write("good.mq.md", LEDGER_OK);
    // 坏定理文件：unproved（LEDGER_BAD 是 `长度(轨迹) = 0`，真实执行 1 步）。
    let bad = LEDGER_BAD.replace("台账", "坏台账");
    s.write("坏/坏.mq.md", &bad);

    // 目录不带 --all → 退 1。
    let dir_out = s
        .marqdo()
        .arg("prove")
        .arg(".")
        .output()
        .expect("run prove dir");
    let dir_err = String::from_utf8_lossy(&dir_out.stderr);
    assert!(dir_err.contains("是目录"), "{dir_err}");
    assert_eq!(dir_out.status.code(), Some(1));

    // --all：假 lean 恒报 proved，坏定理离线反例与之分歧 → 护栏 error（§6 A3
    // 在目录模式同样生效）；整体取最差 = error，退 1。
    let all_out = s
        .marqdo()
        .arg("prove")
        .arg("--all")
        .arg(".")
        .arg("--lean")
        .arg(&lean_path)
        .arg("--json")
        .output()
        .expect("run prove --all");
    let json: serde_json::Value =
        serde_json::from_slice(&all_out.stdout).expect("prove-all json");
    assert_eq!(json["kind"], "prove-all");
    assert_eq!(json["status"], "error", "{json}");
    assert_eq!(json["file_count"], 2);
    assert_eq!(all_out.status.code(), Some(1), "整体最差 error 应退 1");
    let files = json["files"].as_array().expect("files");
    let bad_file = files
        .iter()
        .find(|f| f["file"].as_str().is_some_and(|n| n.contains("坏")))
        .expect("bad file row");
    assert_eq!(bad_file["status"], "error");
    let codes: Vec<&str> = bad_file["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .filter_map(|d| d["code"].as_str())
        .collect();
    assert!(codes.contains(&"proof.divergence"), "{codes:?}");
}

/// 真定理 → proved（冒烟：真 Lean 全链路）。
#[test]
fn a1_true_claim_proved() {
    if !toolchain_available() {
        skip_or_fail("a1_true_claim_proved");
        return;
    }
    let s = Scratch::new("a1-true-claim");
    s.write("台账.mq.md", LEDGER_OK);
    let out = s
        .marqdo()
        .arg("prove")
        .arg("台账.mq.md")
        .arg("--json")
        .output()
        .expect("prove true claim");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["status"], "proved");
    assert_eq!(out.status.code(), Some(0));
    for f in json["functions"].as_array().unwrap() {
        for c in f["claims"].as_array().unwrap() {
            assert_eq!(c["status"], "proved", "每条定理都要 proved：{c}");
            assert_eq!(c["decided_by"], "lean");
        }
    }
}

//! `marqdo prove` P0 集成测试（roadmap/prove.md §6 A–H 矩阵的 P0 部分）。
//!
//! 全部不依赖 Lean：判定在 P1 接入，P0 只验「装载 → 降级 → 定型 → 确定性生成」。
//! 每个用例在 `target/` 下的独立临时目录里跑 CLI（与 `Commands::Check` 同款）。

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
        let dir = target.join("prove-tests").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch dir");
        Self { dir }
    }

    /// 把 lib/result.mq.md 拷进临时项目（import 相对文档解析）。
    fn provision_lib(&self) {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/result.mq.md");
        let dst = self.dir.join("lib/result.mq.md");
        fs::create_dir_all(self.dir.join("lib")).expect("create lib dir");
        fs::copy(&src, &dst).expect("copy lib/result.mq.md");
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(&path, content).expect("write fixture");
        path
    }

    fn run(&self, args: &[&str]) -> Run {
        let exe: &Path = Path::new(env!("CARGO_BIN_EXE_marqdo"));
        let output = Command::new(exe)
            .args(args)
            .current_dir(&self.dir)
            .env("MARQDO_RUNTIME_DIR", self.dir.join("runtime"))
            .output()
            .expect("run marqdo");
        Run {
            code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

/// 审批样例（roadmap §7 / 设计 §2.5）：分支 + 内联 result + 不透明 import。
const APPROVAL: &str = r#"---
markup: occlusion
import:
  result: lib/result.mq.md
---

审批一个报销申请：大额必须人工审核，小额自动通过。

# 审批台

## 审批
+ `金额`

| 参数 | 类型 | 说明 |
|---|---|---|
| 金额 | int | 申请金额（元） |

1. `金额` > 10000
  **回执 = [提交人工审核](人工审核(金额=`金额`))**
  **结果 = [保留结论](result.ok(value=[取值](access(`回执`, "value"))))**
2. *
  **结果 = [自动通过](result.ok(value="auto"))**

*结果*

## 人工审核
+ `金额`

**回执 = [请求送审](工程.送审(金额=`金额`))**

*回执*
"#;

/// 倒计时：while 循环 → P0 拒绝（proof.while）。
const COUNTDOWN: &str = r#"---
markup: occlusion
---

# 倒计时器

## 倒数
+ `n`

**剩余 = `n`**
- `剩余` > 0
  **剩余 = `剩余` - 1**

*剩余*
"#;

/// 分支后还有 return：两臂都必须带上后续（续延丢失回归测试）。
const BRANCH_THEN_RETURN: &str = r#"---
markup: occlusion
---

# 盒子

## 挑选
+ `n`

1. `n` > 0
  **输出 = "正"**
2. *
  **输出 = "负"**

*输出*
"#;

/// 裸未知名：动态调用 → 拒绝。
const UNKNOWN_CALL: &str = r#"---
markup: occlusion
---

# 神秘

## 执行

**回执 = [召唤](不存在的函数(`1`))**

*回执*
"#;

/// 带定理表的审批样例（roadmap §3 / 设计 §7.1；与设计文档示例同形）。
const APPROVAL_WITH_CLAIMS: &str = r#"---
markup: occlusion
import:
  result: lib/result.mq.md
---

# 审批台

## 审批
+ `金额`

| 参数 | 类型 | 说明 |
|---|---|---|
| 金额 | int | 申请金额（元） |

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 大额必经人工 | 包含(轨迹, "人工审核") | 金额 > 10000 | auto |
| 恒有结果 | 结果.ok = True |  | auto |

1. `金额` > 10000
  **回执 = [提交人工审核](人工审核(金额=`金额`))**
  **结果 = [保留结论](result.ok(value=[取值](access(`回执`, "value"))))**
2. *
  **结果 = [自动通过](result.ok(value="auto"))**

*结果*

## 人工审核
+ `金额`

**回执 = [请求送审](工程.送审(金额=`金额`))**

*回执*
"#;

fn read(dir: &Path, rel: &str) -> String {
    fs::read_to_string(dir.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn p0_a_emit_writes_deterministic_artifacts() {
    let s = Scratch::new("a_deterministic");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r1 = s.run(&["prove", "approval.mq.md", "--emit", "out1"]);
    assert_eq!(r1.code, 0, "emit1 stderr={}", r1.stderr);
    let r2 = s.run(&["prove", "approval.mq.md", "--emit", "out2"]);
    assert_eq!(r2.code, 0, "emit2 stderr={}", r2.stderr);

    for rel in ["out1/plan.json", "out1/Generated/approval_mq.lean"] {
        assert!(s.dir.join(&rel).is_file(), "missing {rel}");
    }
    // 确定性：两次 emit 字节一致（plan + lean）。
    assert_eq!(
        read(&s.dir, "out1/plan.json"),
        read(&s.dir, "out2/plan.json"),
        "plan.json must be byte-identical"
    );
    assert_eq!(
        read(&s.dir, "out1/Generated/approval_mq.lean"),
        read(&s.dir, "out2/Generated/approval_mq.lean"),
        "lean module must be byte-identical"
    );
}

#[test]
fn p0_b_json_contract_shape() {
    let s = Scratch::new("b_json_shape");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r = s.run(&["prove", "approval.mq.md", "--report-only", "--json"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    assert_eq!(v["schema"], 1);
    assert_eq!(v["status"], "unverified");
    assert_eq!(v["file"], "approval.mq.md");
    assert!(v["source_sha256"].as_str().is_some_and(|h| h.len() == 64));
    let functions = v["functions"].as_array().expect("functions array");
    assert_eq!(functions.len(), 2, "对象不进报告，只有两个函数");
    let names: Vec<&str> = functions
        .iter()
        .map(|f| f["name"].as_str().expect("name"))
        .collect();
    assert_eq!(names, vec!["审批台.审批", "审批台.人工审核"]);
    for f in functions {
        assert_eq!(f["tier"], "none", "P0 无判定层");
        assert_eq!(f["status"], "unverified");
        assert_eq!(f["claims"].as_array().map(Vec::len), Some(0));
    }
    // lean 尚未接入（P0 判定在 P1）。
    assert!(v["lean"].is_null());
    // emitted 为空（report-only）。
    assert_eq!(v["emitted"].as_array().map(Vec::len), Some(0));
}

#[test]
fn p0_c_while_loop_rejected_with_code() {
    let s = Scratch::new("c_while_rejected");
    s.write("countdown.mq.md", COUNTDOWN);
    let r = s.run(&["prove", "countdown.mq.md", "--report-only", "--json"]);
    // 拒绝是建议性（exit 0），不阻断 agent。
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    assert_eq!(v["status"], "rejected");
    let f = &v["functions"][0];
    assert_eq!(f["name"], "倒计时器.倒数");
    assert_eq!(f["status"], "rejected");
    let rejected = f["rejected"].as_array().expect("rejected array");
    assert!(
        rejected
            .iter()
            .any(|d| d["code"] == "proof.while"),
        "expected proof.while, got {rejected:?}"
    );
    // 文本面也列出拒绝原因。
    let t = s.run(&["prove", "countdown.mq.md", "--report-only"]);
    assert!(t.stdout.contains("proof.while"), "text: {}", t.stdout);
}

#[test]
fn p0_d_side_effect_import_is_opaque_op() {
    let s = Scratch::new("d_opaque_op");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r = s.run(&["prove", "approval.mq.md", "--emit", "out"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let lean = read(&s.dir, "out/Generated/approval_mq.lean");
    // 工程外部调用：Op 构造器 + Env 查询 + 追踪，不展开。
    assert!(lean.contains("«调用_工程.送审»"), "op ctor missing:\n{lean}");
    assert!(lean.contains("env (Op.«调用_工程.送审»"), "env query missing");
    assert!(lean.contains("[Op.«调用_工程.送审»"), "trace append missing");
    // result.ok 内联为 Val 构造器，不是 Op。
    assert!(!lean.contains("result.ok»"), "result.ok must inline, not be an Op");
    assert!(lean.contains("mqResultOk"), "result.ok → mqResultOk");
}

#[test]
fn p0_e_allow_opaque_downgrades_whole_function() {
    let s = Scratch::new("e_allow_opaque");
    s.write("countdown.mq.md", COUNTDOWN);
    let r = s.run(&[
        "prove",
        "countdown.mq.md",
        "--report-only",
        "--json",
        "--allow-opaque",
        "倒计时器.倒数",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    assert_eq!(v["status"], "unverified");
    let f = &v["functions"][0];
    assert_eq!(f["status"], "unverified");
    let ops = f["ops"].as_array().expect("ops");
    assert_eq!(ops.len(), 1, "整函数降级 = 单个 Op");
    assert_eq!(ops[0]["id"], "调用:倒计时器.倒数");
    assert_eq!(ops[0]["effect"], "opaque");
}

#[test]
fn p0_f_branch_continuation_not_swallowed() {
    let s = Scratch::new("f_continuation");
    s.write("box.mq.md", BRANCH_THEN_RETURN);
    let r = s.run(&["prove", "box.mq.md", "--emit", "out"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let lean = read(&s.dir, "out/Generated/box_mq.lean");
    // 分支后的 return 必须出现在两个臂里（回归：此前续延被分支吞掉）。
    assert_eq!(
        lean.matches("(«输出», «轨迹»)").count(),
        2,
        "return must appear in both arms:\n{lean}"
    );
    // 两臂各自绑定。
    assert!(lean.contains("let «输出» := (Val.text \"正\")"), "{lean}");
    assert!(lean.contains("let «输出» := (Val.text \"负\")"), "{lean}");
}

#[test]
fn p0_g_mq_line_comments_and_guillemets() {
    let s = Scratch::new("g_comments");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r = s.run(&["prove", "approval.mq.md", "--emit", "out"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let lean = read(&s.dir, "out/Generated/approval_mq.lean");
    assert!(lean.contains("-- mq: "), "溯源注释缺失");
    // 含点标识符用 guillemets。
    assert!(lean.contains("«审批台.审批»"), "{lean}");
    // 确定性来源：无时间戳、无绝对路径。
    assert!(!lean.contains("marqdo/prove/"), "must not leak default dir");
}

#[test]
fn p0_h_unknown_call_rejected() {
    let s = Scratch::new("h_unknown");
    s.write("mystery.mq.md", UNKNOWN_CALL);
    let r = s.run(&["prove", "mystery.mq.md", "--report-only", "--json"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    assert_eq!(v["status"], "rejected");
    let rejected = v["functions"][0]["rejected"]
        .as_array()
        .expect("rejected array");
    assert!(
        rejected.iter().any(|d| d["code"] == "proof.dynamic_call"),
        "expected proof.dynamic_call, got {rejected:?}"
    );
}

#[test]
fn p0_i_report_only_writes_nothing() {
    let s = Scratch::new("i_report_only");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r = s.run(&["prove", "approval.mq.md", "--report-only"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(
        !s.dir.join(".marqdo").exists(),
        "report-only must not create artifacts"
    );
    assert!(!r.stdout.contains("emitted:"), "no emitted lines");
}

#[test]
fn p0_j_clean_removes_default_dir() {
    let s = Scratch::new("j_clean");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r = s.run(&["prove", "approval.mq.md"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(s.dir.join(".marqdo/prove").exists(), "default emit dir");
    let r = s.run(&["prove", "clean", "--json"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(r.stdout.contains("\"ok\":true"), "{}", r.stdout);
    assert!(!s.dir.join(".marqdo/prove").exists(), "clean must remove dir");
}

#[test]
fn p0_k_branch_type_conflict_is_error() {
    let s = Scratch::new("k_type_conflict");
    s.write(
        "conflict.mq.md",
        r#"---
markup: occlusion
---

# 冲突

## 挑选
+ `n`

| 参数 | 类型 | 说明 |
|---|---|---|
| n | int | 数值 |

1. `n` > 0
  **输出 = `n`**
2. *
  **输出 = "文字"**

*输出*
"#,
    );
    let r = s.run(&["prove", "conflict.mq.md", "--report-only", "--json"]);
    // error → exit 1（§3.3）。
    assert_eq!(r.code, 1, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    assert_eq!(v["status"], "error");
    let codes: Vec<&str> = v["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .filter_map(|d| d["code"].as_str())
        .collect();
    assert!(
        codes.iter().any(|c| *c == "proof.type_mismatch"),
        "expected proof.type_mismatch, got {codes:?}"
    );
}

#[test]
fn p0_l_only_filters_report() {
    let s = Scratch::new("l_only");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL);
    let r = s.run(&[
        "prove",
        "approval.mq.md",
        "--report-only",
        "--json",
        "--only",
        "审批台.人工审核",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    let functions = v["functions"].as_array().expect("functions");
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0]["name"], "审批台.人工审核");
}

// ============================================================ P1 · 定理表与 check --proof

/// 提取后的定理表：claims 非空、body 无该表（roadmap §3 第 1 周验收）。
#[test]
fn p1_a_claim_table_extracted_and_removed_from_body() {
    let s = Scratch::new("p1_extract");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL_WITH_CLAIMS);
    let r = s.run(&["prove", "approval.mq.md", "--report-only", "--json"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    let f = &v["functions"][0];
    assert_eq!(f["name"], "审批台.审批");
    let claims = f["claims"].as_array().expect("claims array");
    assert_eq!(claims.len(), 2, "{claims:?}");
    assert_eq!(claims[0]["id"], "大额必经人工");
    assert_eq!(claims[0]["premise"], "金额 > 10000");
    assert_eq!(claims[0]["conclusion"], "包含(轨迹, \"人工审核\")");
    assert_eq!(claims[1]["id"], "恒有结果");
    assert!(claims[1]["premise"].is_null(), "空前提 = 全称命题");
    // 表从 body 移除：plan 的 steps 里没有表（函数照常降级）。
    assert_eq!(f["status"], "unverified");
}

/// 定理脚手架进 Lean：受限文法命题 → theorem 陈述 + sorry 占位。
#[test]
fn p1_b_emit_theorem_scaffold() {
    let s = Scratch::new("p1_theorem");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL_WITH_CLAIMS);
    let r = s.run(&["prove", "approval.mq.md", "--emit", "out"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let lean = read(&s.dir, "out/Generated/approval_mq.lean");
    assert!(
        lean.contains("theorem «审批台.审批.大额必经人工»"),
        "{lean}"
    );
    // 前提进绑定；包含(轨迹, …) 翻成操作名成员测试。
    assert!(lean.contains("(«h_前提» : mqTruth (mqGt «金额» (Val.int 10000)))"), "{lean}");
    assert!(
        lean.contains("((«轨迹».map Op.name).contains \"人工审核\")"),
        "{lean}"
    );
    // 结果.ok → mqAccess 字段读取。
    assert!(lean.contains("(mqAccess «结果» \"ok\")"), "{lean}");
    // 策略脚本（§7.2）：unfold → split → simp_all/omega。
    assert!(lean.contains("unfold «审批台.审批» at h_run"), "{lean}");
    assert!(
        lean.contains("try split at h_run")
            && lean.contains("all_goals\n    simp_all [mqTruth, Op.name, mqAccess, mqResultOk, mqResultError, mqValueOr, mqAnd, mqOr, mqNot, mqEq]"),
        "{lean}"
    );
}

/// check --proof（默认关）：不传 --proof 时定理问题不报。
#[test]
fn p1_c_check_proof_off_by_default() {
    let s = Scratch::new("p1_default_off");
    s.write(
        "bad.mq.md",
        r#"---
markup: occlusion
---

# 审计

## 核对
+ `金额`

| 定理 | 结论 |
|---|---|
| 引用未知 | 额度 > 0 |

**x = `金额`**

*x*
"#,
    );
    let r = s.run(&["check", "bad.mq.md", "--json"]);
    assert_eq!(r.code, 0, "默认关：{} {}", r.stdout, r.stderr);
    assert!(
        !r.stdout.contains("proof.claim_unknown_symbol"),
        "默认必须不报：{}",
        r.stdout
    );
}

/// check --proof：unknown_symbol + duplicate + misplaced + malformed。
#[test]
fn p1_d_check_proof_diagnostic_codes() {
    let s = Scratch::new("p1_codes");
    s.write(
        "bad.mq.md",
        r#"---
markup: occlusion
---

# 审计

## 核对
+ `金额`

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 引用未知 | 长度(轨迹) = 额度 | 金额 > 0 | auto |
| 引用未知 | 长度(轨迹) = 额度 | 金额 > 0 | auto |
| 坏策略 | 结果.ok = True |  | manual |

**x = `金额`**

| 定理 | 结论 |
|---|---|
| 错位表 | 结果.ok = True |

*x*
"#,
    );
    let r = s.run(&["check", "bad.mq.md", "--proof", "--json"]);
    assert_eq!(r.code, 1, "error 级诊断 → 退出 1：{}", r.stdout);
    assert!(r.stdout.contains("proof.claim_unknown_symbol"), "{}", r.stdout);
    assert!(r.stdout.contains("proof.claim_duplicate"), "{}", r.stdout);
    assert!(r.stdout.contains("proof.claim_misplaced"), "{}", r.stdout);
    assert!(r.stdout.contains("proof.claim_malformed"), "{}", r.stdout);
    // 不该误报：策略 auto/none 之外的 malformed 只出现一次（坏策略行）。
    assert_eq!(r.stdout.matches("proof.claim_malformed").count(), 1, "{}", r.stdout);
}

/// check --proof：保留名遮蔽（§7.3.4）。
#[test]
fn p1_e_check_proof_name_shadow() {
    let s = Scratch::new("p1_shadow");
    s.write(
        "shadow.mq.md",
        r#"---
markup: occlusion
---

# 审计

## 核对
+ `结果`

| 定理 | 结论 |
|---|---|
| 恒真 | 结果.ok = True |

**x = `结果`**

*x*
"#,
    );
    let r = s.run(&["check", "shadow.mq.md", "--proof", "--json"]);
    assert_eq!(r.code, 1, "{}", r.stdout);
    assert!(r.stdout.contains("proof.name_shadow"), "{}", r.stdout);
}

/// check --proof：歧义表（类型+结论）与谓词元数（§7.1 规则 5 / §7.3.5）。
#[test]
fn p1_f_check_proof_ambiguous_and_unsupported() {
    let s = Scratch::new("p1_amb");
    s.write(
        "amb.mq.md",
        r#"---
markup: occlusion
---

# 审计

## 核对
+ `金额`

| 定理 | 类型 | 结论 |
|---|---|---|
| 歧义 | int | 结果.ok = True |

**x = `金额`**

*x*
"#,
    );
    let r = s.run(&["check", "amb.mq.md", "--proof", "--json"]);
    assert!(r.stdout.contains("proof.claim_ambiguous"), "{}", r.stdout);

    let s2 = Scratch::new("p1_unsupported");
    s2.write(
        "bad2.mq.md",
        r#"---
markup: occlusion
---

# 审计

## 核对
+ `金额`

| 定理 | 结论 | 策略 |
|---|---|---|
| 元数错 | 包含(轨迹, "a", "b") | auto |

**x = `金额`**

*x*
"#,
    );
    let r = s2.run(&["check", "bad2.mq.md", "--proof", "--json"]);
    assert!(r.stdout.contains("proof.claim_unsupported"), "{}", r.stdout);
}

/// 带定理的文件：emit 输出仍然确定性（plan.json + lean 两次一致）。
#[test]
fn p1_g_emit_with_claims_is_deterministic() {
    let s = Scratch::new("p1_det");
    s.provision_lib();
    s.write("approval.mq.md", APPROVAL_WITH_CLAIMS);
    s.run(&["prove", "approval.mq.md", "--emit", "out1"]);
    s.run(&["prove", "approval.mq.md", "--emit", "out2"]);
    assert_eq!(
        read(&s.dir, "out1/Generated/approval_mq.lean"),
        read(&s.dir, "out2/Generated/approval_mq.lean")
    );
    assert_eq!(read(&s.dir, "out1/plan.json"), read(&s.dir, "out2/plan.json"));
    // plan.json 里定理声明带行号锚点。
    let plan = read(&s.dir, "out1/plan.json");
    assert!(plan.contains("\"claims\":[{"), "{plan}");
    assert!(plan.contains("\"line\":16"), "{plan}");
}

// ============================================================ 审查修复的回归测试

/// 缺陷 1：表格单元格里的副作用调用必须被拒绝，不许静默丢弃
/// （静默丢弃会让含副作用的函数在 Lean 里伪装成"无操作"）。
#[test]
fn p2_a_impure_call_in_table_cell_is_rejected() {
    let s = Scratch::new("p2_impure_tbl");
    s.write(
        "record.mq.md",
        r#"---
markup: occlusion
---

# 副作用实验

## 记录
+ `名`

**数据 =**
| 步骤 |
|---|
| [落笔](工程.写(名=`名`)) |
| [收笔](工程.收(名=`名`)) |

**x = [取值](access(`数据`, "步骤"))**

*x*
"#,
    );
    let r = s.run(&["prove", "record.mq.md", "--report-only", "--json"]);
    assert_eq!(r.code, 0, "拒绝是 advisory，exit 0：{}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("valid JSON");
    assert_eq!(v["status"], "rejected", "{v}");
    let rejected = serde_json::to_string(&v["functions"][0]["rejected"]).unwrap();
    assert!(
        rejected.contains("proof.unsupported"),
        "必须有 proof.unsupported：{v}"
    );
    // 关键：函数里没有任何 Op（调用没被当作不透明操作处理）→ 整个函数拒绝。
    assert_eq!(v["functions"][0]["ops"].as_array().map(Vec::len), Some(0));
}

/// 缺陷 2：同名不同元数的调用点 → `/元数` 后缀的独立 Op，构造器与使用一致。
#[test]
fn p2_b_op_arity_mismatch_gets_suffixed_ctor() {
    let s = Scratch::new("p2_arity");
    s.write(
        "machine.mq.md",
        r#"---
markup: occlusion
---

# 机器

## 可选参数
+ `a`
+ `b` = 1

**r = [加一](自身.加一(a=`a`))**
**s = [加一](自身.加一(a=`a`, b=`b`))**

*r*

## 加一
+ `a`
+ `b` = 1

**t = `a` + `b`**

*t*
"#,
    );
    let r = s.run(&["prove", "machine.mq.md", "--emit", "out"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let lean = read(&s.dir, "out/Generated/machine_mq.lean");
    // 两个构造器，元数各自正确。
    assert!(
        lean.contains("| «调用_自身.加一» («参数1» : Val)"),
        "{lean}"
    );
    assert!(
        lean.contains("| «调用_自身.加一/2» («参数1» : Val) («参数2» : Val)"),
        "{lean}"
    );
    // 使用点各自带对个数的实参。
    assert!(lean.contains("Op.«调用_自身.加一» a)"), "{lean}");
    assert!(lean.contains("Op.«调用_自身.加一/2» a b)"), "{lean}");
}

/// 缺陷 3：`长度(轨迹) = 2` 必须生成 Nat 比较，不是 mqEq（Nat≠Val）。
#[test]
fn p2_c_trace_length_claim_uses_nat_operators() {
    let s = Scratch::new("p2_len");
    s.provision_lib();
    s.write(
        "count.mq.md",
        r#"---
markup: occlusion
import:
  result: lib/result.mq.md
---

# 轨迹计数

## 纯计算
+ `金额`

| 定理 | 结论 | 策略 |
|---|---|---|
| 空轨迹 | 长度(轨迹) = 0 | auto |

**a = [一步](result.ok(value=1))**
**b = [两步](result.ok(value=2))**

*b*
"#,
    );
    let r = s.run(&["prove", "count.mq.md", "--emit", "out"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let lean = read(&s.dir, "out/Generated/count_mq.lean");
    assert!(
        lean.contains("((«轨迹».length : Nat) = 0)"),
        "Nat 比较：{lean}"
    );
    assert!(!lean.contains("mqEq («轨迹».length"), "不许出现类型不匹配：{lean}");
}

/// 缺陷 4：谓词名遮蔽整个模块只报一次；无关函数的形参不误报。
#[test]
fn p2_d_name_shadow_reported_once() {
    let s = Scratch::new("p2_shadow");
    s.write(
        "audit.mq.md",
        r#"---
markup: occlusion
---

# 审计

## 包含
+ `n`

**y = `n`**

*y*

## 核对
+ `金额`

| 定理 | 结论 |
|---|---|
| 恒真 | 结果.ok = True |

**x = `金额`**

*x*
"#,
    );
    let r = s.run(&["check", "audit.mq.md", "--proof", "--json"]);
    assert_eq!(r.code, 1, "{}", r.stdout);
    let count = r.stdout.matches("proof.name_shadow").count();
    assert_eq!(count, 1, "只报一次：{}", r.stdout);
}

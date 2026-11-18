//! T3.3 golden：**20 例易锚定错误**的修复循环（护栏机械执行 + 金修复可复现）。
//!
//! 每例走完整循环：
//! 诊断（含 `span`/`doc_anchor`）→ 推导靶点范围 → 金修复（源 diff 自动推导行编辑，
//! 顺便机器验证「易锚定」= 金编辑全落范围内）→ `mlsp repair_apply` 应用 → 复验通过。
//!
//! 护栏负例内嵌每例：越界编辑（第 1 行 `# main`，永不在靶点内）必拒
//! （`mlsp.repair_out_of_scope` + `abstain`），源零字节不改。
//! 模型实验见 `tests/repair_experiment.rs`（同一批 20 例）。

mod common;

use common::*;
use marqdo::repair::{apply_edits, Edit};
use serde_json::json;

#[test]
fn golden_repair_loop_all_20() {
    let cases = cases();
    assert_eq!(cases.len(), 20, "golden 必须正好 20 例");

    for case in &cases {
        // 1. 诊断（broken 必须给出期望错误码）
        let code = case.code();
        let diags = diagnose(case.verify, &case.broken, &case.name);
        assert!(
            diags.iter().any(|d| d["code"] == code),
            "[{}] 期望诊断 {code}，实得 {diags:?}",
            case.name
        );
        let ranges: Vec<(u32, u32)> = diags.iter().flat_map(ranges_of).collect();
        assert!(!ranges.is_empty(), "[{}] 诊断必须带可锚定范围: {diags:?}", case.name);

        // 2. 金修复 → 行编辑（diff 自动推导）；「易锚定」= 全部编辑落在靶点内
        let edits = derive_edits(&case.broken, &case.gold);
        assert!(!edits.is_empty(), "[{}] 金修复必须产生编辑", case.name);
        let applied = apply_edits(&case.broken, &ranges, &edits).unwrap_or_else(|oos| {
            panic!("[{}] 金修复越界（不是易锚定错误）: 越界 {:?}，范围 {:?}", case.name, oos.lines, oos.ranges)
        });
        assert_eq!(applied, case.gold, "[{}] 行编辑结果应与金修复逐字节一致", case.name);

        // 3. 经 MLSP 面应用（护栏所在）
        let resp = apply_via_mlsp(&case.broken, &ranges, &edits);
        assert_eq!(resp["ok"], true, "[{}] repair_apply 应接受金修复: {resp}", case.name);
        assert_eq!(resp["result"]["source"], case.gold, "[{}]", case.name);

        // 4. 复验（干净 = 空诊断）
        let after = diagnose(case.verify, &case.gold, &case.name);
        assert!(after.is_empty(), "[{}] 修复后必须零诊断: {after:?}", case.name);

        // 5. 护栏负例：越界编辑（第 1 行 `# main`）必拒 + 源零字节不改
        let bad = vec![Edit::Replace { line: 1, text: "# 被篡改".to_string() }];
        let resp = apply_via_mlsp(&case.broken, &ranges, &bad);
        assert_eq!(resp["ok"], false, "[{}] 越界编辑必须被拒: {resp}", case.name);
        assert_eq!(resp["error"]["code"], "mlsp.repair_out_of_scope", "[{}]", case.name);
        assert_eq!(resp["error"]["on_violation"], "abstain", "[{}]", case.name);
    }
}

#[test]
fn repair_apply_refuses_any_out_of_scope_line() {
    // 独立护栏例：部分越界 = 全部拒绝（全有或全无），源零字节不改。
    let src = "# main\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | number | |\n| name | text | 是 |\n\n*1*\n";
    let ranges = [(3u32, 6u32)]; // 只有字段契约表
    let edits = vec![
        Edit::Replace { line: 5, text: "| id | text | |".to_string() }, // 表内 ✓
        Edit::Delete { line: 1 },                                      // 越界 ✗
    ];
    let resp = apply_via_mlsp(src, &ranges, &edits);
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["error"]["code"], "mlsp.repair_out_of_scope");
    assert!(resp["error"]["lines"].as_array().unwrap().contains(&json!(1)));

    // 范围内编辑照常可用（含表尾追加）
    let edits = vec![
        Edit::Replace { line: 5, text: "| id | text | |".to_string() },
        Edit::InsertBefore { line: 7, text: "| email | text | 是 |".to_string() }, // end+1 追加 ✓
    ];
    let resp = apply_via_mlsp(src, &ranges, &edits);
    assert_eq!(resp["ok"], true, "{resp}");
    let out = resp["result"]["source"].as_str().unwrap();
    assert!(out.contains("| id | text | |"));
    assert!(out.contains("| email | text | 是 |"));
}

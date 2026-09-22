//! T3.3 golden：**20 例易锚定错误**的修复循环（护栏机械执行 + 金修复可复现）。
//!
//! 每例走完整循环：
//! 诊断（含 `span`/`doc_anchor`）→ 推导靶点范围 → 金修复（源 diff 自动推导行编辑，
//! 顺便机器验证「易锚定」= 金编辑全落范围内）→ `mlsp repair_apply` 应用 → 复验通过。
//!
//! 护栏负例内嵌每例：越界编辑（第 1 行 `# main`，永不在靶点内）必拒
//! （`mlsp.repair_out_of_scope` + `abstain`），源零字节不改。
//! 模型实验（≥80% 修复率）只需把 `derive_edits` 的金修复换成模型输出的编辑。

use marqdo::diagnostics::Diagnostic;
use marqdo::mlsp::handle_line;
use marqdo::repair::{apply_edits, Edit};
use marqdo::{run_file, RunOptions};
use serde_json::{json, Value};

fn mcall(req: Value) -> Value {
    serde_json::from_str(&handle_line(&req.to_string())).expect("MLSP 响应必须是合法 JSON")
}

#[derive(Clone, Copy)]
enum Verify {
    /// 运行期契约错（四边界）：修复后 run 必须成功。
    Run(&'static str),
    /// 静态错（check / parse）：修复后 validate 零诊断。
    Static(&'static str),
}

struct Case {
    name: String,
    broken: String,
    gold: String,
    verify: Verify,
}

// ---------- 诊断 ----------

fn static_diags(src: &str) -> Vec<Value> {
    let resp = mcall(json!({"id": 1, "method": "validate", "params": {"source": src}}));
    if resp["ok"] == false {
        return vec![resp["error"].clone()];
    }
    resp["result"]["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn temp_path(idx: usize) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("marqdo_repair_gold_{idx}.mq.md"))
}

fn run_diag(src: &str, idx: usize) -> Value {
    let p = temp_path(idx);
    std::fs::write(&p, src).unwrap();
    let err = run_file(&p, &RunOptions::default())
        .expect_err("broken 源必须报契约错");
    match Diagnostic::find(&err) {
        Some(d) => d.to_json(),
        None => json!({"code": "?", "message": format!("{err:#}")}),
    }
}

fn run_ok(src: &str, idx: usize) {
    let p = temp_path(idx);
    std::fs::write(&p, src).unwrap();
    run_file(&p, &RunOptions::default()).expect("修复后必须运行成功");
    let _ = std::fs::remove_file(&p);
}

/// 诊断 JSON → 靶点范围（span 行 + doc_anchor 的 L 范围）。
fn ranges_of(v: &Value) -> Vec<(u32, u32)> {
    let mut rs = Vec::new();
    if let Some(l) = v["span"]["line"].as_u64() {
        rs.push((l as u32, l as u32));
    }
    if let Some(a) = v["doc_anchor"].as_str() {
        if let Some(rest) = a.split('#').nth(1) {
            if let Some(rest) = rest.strip_prefix('L') {
                if let Some((s, e)) = rest.split_once("-L") {
                    if let (Ok(s), Ok(e)) = (s.parse(), e.parse()) {
                        rs.push((s, e));
                    }
                }
            }
        }
    }
    rs
}

// ---------- 金修复 → 行编辑（前缀/后缀 diff） ----------

fn derive_edits(broken: &str, gold: &str) -> Vec<Edit> {
    let b: Vec<&str> = broken.lines().collect();
    let g: Vec<&str> = gold.lines().collect();
    let mut p = 0usize;
    while p < b.len() && p < g.len() && b[p] == g[p] {
        p += 1;
    }
    let mut s = 0usize;
    while s < b.len() - p
        && s < g.len() - p
        && b[b.len() - 1 - s] == g[g.len() - 1 - s]
    {
        s += 1;
    }
    let mut edits = Vec::new();
    for l in (p + 1)..=(b.len() - s) {
        edits.push(Edit::Delete { line: l as u32 });
    }
    let at = (b.len() - s + 1) as u32;
    for text in &g[p..(g.len() - s)] {
        edits.push(Edit::InsertBefore {
            line: at,
            text: (*text).to_string(),
        });
    }
    edits
}

fn apply_via_mlsp(src: &str, ranges: &[(u32, u32)], edits: &[Edit]) -> Value {
    let resp = mcall(json!({
        "id": 2,
        "method": "repair_apply",
        "params": {
            "source": src,
            "ranges": ranges.iter().map(|(s, e)| json!([s, e])).collect::<Vec<_>>(),
            "edits": edits.iter().map(|e| match e {
                Edit::Replace { line, text } => json!({"op": "replace", "line": line, "text": text}),
                Edit::Delete { line } => json!({"op": "delete", "line": line}),
                Edit::InsertBefore { line, text } => json!({"op": "insert", "line": line, "text": text}),
            }).collect::<Vec<_>>(),
        }
    }));
    resp
}

// ---------- 20 例（10 骨架 × 2 变体） ----------

fn cases() -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    let push = |out: &mut Vec<Case>, name: &str, broken: String, gold: String, verify: Verify| {
        out.push(Case {
            name: name.to_string(),
            broken,
            gold,
            verify,
        });
    };

    // 1–2 调用实参：数字形参传了文本（金修复改调用行 = span 侧）
    for (v, lit, n) in [("a", "七", 7), ("b", "八", 8)] {
        let broken = format!(
            "# main\n\n> 加一 \"{lit}\"\n\n## 加一\n\n对于输入变量`n`,期望是数字。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 输入值 |\n\n\
             | 返回 | 类型 | 说明 |\n|------|------|------|\n|  | number | 结果 |\n\n*n+1*\n"
        );
        let gold = broken.replace(&format!("> 加一 \"{lit}\""), &format!("> 加一 {n}"));
        push(&mut out, &format!("arg-num-{v}"), broken, gold, Verify::Run("contract.arg_mismatch"));
    }

    // 3–4 调用实参：文本形参传了裸数（金修复加引号 = span 侧）
    for (v, n) in [("a", 7), ("b", 8)] {
        let broken = format!(
            "# main\n\n> 转换 {n}\n\n## 转换\n\n对于输入变量`s`,期望是文本。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| s | text | 输入 |\n\n\
             | 返回 | 类型 | 说明 |\n|------|------|------|\n|  | text | 结果 |\n\n*s*\n"
        );
        let gold = broken.replace(&format!("> 转换 {n}"), &format!("> 转换 \"{n}\""));
        push(&mut out, &format!("arg-text-{v}"), broken, gold, Verify::Run("contract.arg_mismatch"));
    }

    // 5–6 返回契约：返回文本却声明 number（金修复改契约行 = anchor 侧）
    for (v, lit) in [("a", "七"), ("b", "你好")] {
        let broken = format!(
            "# main\n\n**结果 = > 转换 \"{lit}\"**\n*结果*\n\n## 转换\n\n对于输入变量`s`,期望是文本。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| s | text | 输入 |\n\n\
             | 返回 | 类型 | 说明 |\n|------|------|------|\n|  | number | 结果 |\n\n*s*\n"
        );
        let gold = broken.replace("|  | number | 结果 |", "|  | text | 结果 |");
        push(&mut out, &format!("ret-type-{v}"), broken, gold, Verify::Run("contract.return_mismatch"));
    }

    // 7–8 字段类型：数据与字段契约不符（金修复改契约行 = anchor 侧）
    {
        let broken = "# main\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | number | |\n| name | text | 是 |\n\n\
             `用户` =\n| id | name |\n|------|------|\n| abc | 小明 |\n\n*`用户`[^name]*\n"
            .to_string();
        let gold = broken.replace("| id | number | |", "| id | text | |");
        push(&mut out, "field-type-a", broken, gold, Verify::Run("contract.field_mismatch"));

        let broken = "# main\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | number | |\n| name | number | 是 |\n\n\
             `用户` =\n| id | name |\n|------|------|\n| 1 | 小明 |\n\n*`用户`[^name]*\n"
            .to_string();
        let gold = broken.replace("| name | number | 是 |", "| name | text | 是 |");
        push(&mut out, "field-type-b", broken, gold, Verify::Run("contract.field_mismatch"));
    }

    // 9–10 缺字段：数据没有契约要求的非空字段（金修复标可空 = anchor 侧）
    {
        let broken = "# main\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | number | |\n| name | text | |\n\n\
             `用户` =\n| id | qq |\n|------|------|\n| 1 | 5 |\n\n*`用户`[^id]*\n"
            .to_string();
        let gold = broken.replace("| name | text | |", "| name | text | 是 |");
        push(&mut out, "field-nullable-a", broken, gold, Verify::Run("contract.field_mismatch"));

        let broken = "# main\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | number | |\n| name | text | |\n\n\
             `用户` =\n| name | qq |\n|------|------|\n| 小明 | 5 |\n\n*`用户`[^name]*\n"
            .to_string();
        let gold = broken.replace("| id | number | |", "| id | number | 是 |");
        push(&mut out, "field-nullable-b", broken, gold, Verify::Run("contract.field_mismatch"));
    }

    // 11–12 未知键：访问不在契约里的字段（金修复改索引键 = span 侧）
    {
        let broken = "# main\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | number | |\n| name | text | 是 |\n\n\
             `用户` =\n| id | name |\n|------|------|\n| 1 | 小明 |\n\n*`用户`[^email]*\n"
            .to_string();
        let gold = broken.replace("[^email]", "[^name]");
        let broken_b = broken.replace("[^email]", "[^phone]");
        let gold_b = broken_b.replace("[^phone]", "[^id]");
        push(&mut out, "key-fix-a", broken, gold, Verify::Run("contract.unknown_key"));
        push(&mut out, "key-fix-b", broken_b, gold_b, Verify::Run("contract.unknown_key"));
    }

    // 13–14 类型名拼错（金修复改正拼写 = anchor 侧）
    for (v, typo, fix) in [("a", "nubmer", "number"), ("b", "txt", "text")] {
        let broken = format!(
            "# main\n\n## 加一\n\n对于输入变量`n`,执行加一。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| n | {typo} | 输入值 |\n\n*n+1*\n"
        );
        let gold = broken.replace(typo, fix);
        push(&mut out, &format!("type-typo-{v}"), broken, gold, Verify::Static("contract.unknown_type"));
    }

    // 15–16 契约行名字错位（金修复改行名 = anchor 侧）
    for (v, wrong) in [("a", "nn"), ("b", "nm")] {
        let broken = format!(
            "# main\n\n## 加一\n\n对于输入变量`n`,执行加一。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| {wrong} | number | 输入值 |\n\n*n+1*\n"
        );
        let gold = broken.replace(&format!("| {wrong} | number | 输入值 |"), "| n | number | 输入值 |");
        push(&mut out, &format!("drift-rename-{v}"), broken, gold, Verify::Static("contract.param_drift"));
    }

    // 17–18 新升参未被契约覆盖（金修复补契约行 = anchor 侧，插入表尾）
    for (v, extra) in [("a", "m"), ("b", "k")] {
        let broken = format!(
            "# main\n\n## 加一\n\n对于输入变量`n`,临时量`{extra}`。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 输入值 |\n\n*n + {extra}*\n"
        );
        let gold = broken.replace(
            "| n | number | 输入值 |",
            &format!("| n | number | 输入值 |\n| {extra} | number | 临时 |"),
        );
        push(&mut out, &format!("drift-cover-{v}"), broken, gold, Verify::Static("contract.param_drift"));
    }

    // 19–20 两份契约打架（金修复删第二张表 = anchor 侧，纯删除）
    {
        let broken = "# main\n\n## 加一\n\n对于输入变量`n`,执行加一。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 输入值 |\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 重复 |\n\n*n+1*\n"
            .to_string();
        let gold = broken.replace(
            "\n| 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 重复 |",
            "",
        );
        push(&mut out, "dup-delete-a", broken, gold, Verify::Static("contract.duplicate"));

        let broken = "# main\n\n## 转换\n\n对于输入变量`s`,期望是文本。\n\n\
             | 参数 | 类型 | 说明 |\n|------|------|------|\n| s | text | 输入 |\n\n\
             | 返回 | 类型 | 说明 |\n|------|------|------|\n|  | text | 结果 |\n\n\
             | 返回 | 类型 | 说明 |\n|------|------|------|\n|  | text | 同上 |\n\n*s*\n"
            .to_string();
        let gold = broken.replace(
            "\n| 返回 | 类型 | 说明 |\n|------|------|------|\n|  | text | 同上 |",
            "",
        );
        push(&mut out, "dup-delete-b", broken, gold, Verify::Static("contract.duplicate"));
    }

    out
}

// ---------- 完整修复循环 ----------

#[test]
fn golden_repair_loop_all_20() {
    let cases = cases();
    assert_eq!(cases.len(), 20, "golden 必须正好 20 例");

    for (idx, case) in cases.iter().enumerate() {
        // 1. 诊断（broken 必须给出期望错误码）
        let (code, diags) = match case.verify {
            Verify::Run(code) => (code, vec![run_diag(&case.broken, idx)]),
            Verify::Static(code) => (code, static_diags(&case.broken)),
        };
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

        // 4. 复验
        match case.verify {
            Verify::Run(_) => run_ok(&case.gold, idx),
            Verify::Static(_) => {
                let after = static_diags(&case.gold);
                assert!(after.is_empty(), "[{}] 修复后必须零诊断: {after:?}", case.name);
            }
        }

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

//! 修复循环测试共享件：MLSP 调用、统一诊断、金修复行编辑推导、golden 20 例。
//!
//! 被 `tests/repair_loop.rs`（确定性护栏/golden）与 `tests/repair_experiment.rs`
//! （模型在环实验）共用——同一批 20 例、同一套诊断与应用路径。
#![allow(dead_code)]

use marqdo::diagnostics::Diagnostic;
use marqdo::mlsp::handle_line;
use marqdo::repair::Edit;
use marqdo::{run_file, RunOptions};
use serde_json::{json, Value};

/// 调一次 MLSP（进程内）。
pub fn mcall(req: Value) -> Value {
    serde_json::from_str(&handle_line(&req.to_string())).expect("MLSP 响应必须是合法 JSON")
}

/// 验证方式：运行期契约错走 run；静态错（check / parse）走 validate。
#[derive(Clone, Copy)]
pub enum Verify {
    Run(&'static str),
    Static(&'static str),
}

pub struct Case {
    pub name: String,
    pub broken: String,
    pub gold: String,
    pub verify: Verify,
}

impl Case {
    pub fn code(&self) -> &'static str {
        match self.verify {
            Verify::Run(c) | Verify::Static(c) => c,
        }
    }
}

pub fn static_diags(src: &str) -> Vec<Value> {
    let resp = mcall(json!({"id": 1, "method": "validate", "params": {"source": src}}));
    if resp["ok"] == false {
        return vec![resp["error"].clone()];
    }
    resp["result"]["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

pub fn temp_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("marqdo_repair_{}_{}.mq.md", std::process::id(), tag))
}

/// 统一诊断入口：**干净 ⇒ 空数组**（Run 成功 / validate 零诊断）。
pub fn diagnose(verify: Verify, src: &str, tag: &str) -> Vec<Value> {
    match verify {
        Verify::Run(_) => {
            let p = temp_path(tag);
            std::fs::write(&p, src).unwrap();
            match run_file(&p, &RunOptions::default()) {
                Ok(_) => vec![],
                Err(e) => vec![match Diagnostic::find(&e) {
                    Some(d) => d.to_json(),
                    None => json!({"code": "?", "message": format!("{e:#}")}),
                }],
            }
        }
        Verify::Static(_) => static_diags(src),
    }
}

pub fn run_ok(src: &str, tag: &str) {
    let p = temp_path(tag);
    std::fs::write(&p, src).unwrap();
    run_file(&p, &RunOptions::default()).expect("修复后必须运行成功");
    let _ = std::fs::remove_file(&p);
}

/// 诊断 JSON → 靶点范围（span 行 + doc_anchor 的 L 范围）。
pub fn ranges_of(v: &Value) -> Vec<(u32, u32)> {
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

/// 金修复 → 行编辑（前缀/后缀 diff 自动推导）。
pub fn derive_edits(broken: &str, gold: &str) -> Vec<Edit> {
    let b: Vec<&str> = broken.lines().collect();
    let g: Vec<&str> = gold.lines().collect();
    let mut p = 0usize;
    while p < b.len() && p < g.len() && b[p] == g[p] {
        p += 1;
    }
    let mut s = 0usize;
    while s < b.len() - p && s < g.len() - p && b[b.len() - 1 - s] == g[g.len() - 1 - s] {
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

/// 经 `mlsp repair_apply` 应用（护栏所在）。
pub fn apply_via_mlsp(src: &str, ranges: &[(u32, u32)], edits: &[Edit]) -> Value {
    mcall(json!({
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
    }))
}

/// golden 20 例（10 骨架 × 2 变体）。
pub fn cases() -> Vec<Case> {
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

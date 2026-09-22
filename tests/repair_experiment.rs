//! 模型在环实验 · three-problems.md §5.2(a)：**锚点修复**（带 `doc_anchor` 的诊断 vs 去锚诊断）。
//!
//! 本实验同时是新功能面（`validate` / `repair_targets` / `repair_apply` 护栏）的**真实消费者**：
//!
//! ```text
//! 诊断 → repair_targets（靶点 + max_attempts=2 + abstain）→ 模型生成行编辑
//!      → repair_apply（越界必拒）→ 复验（run / validate）→（≤2 轮）
//! ```
//!
//! 运行（需要 `.env` 或 `OPENAI_API_KEY`，默认 `llm.cflmy.cn/v1` + 模型 `cflmy`）：
//!
//! ```bash
//! cargo test --test repair_experiment -- --ignored --nocapture
//! MARQDO_EXP_LIMIT=3 cargo test --test repair_experiment -- --ignored --nocapture  # 冒烟
//! ```
//!
//! 报告写入 `doc/roadmap/repair-experiment-<日期>.md`。

mod common;

use common::*;
use marqdo::repair::Edit;
use serde_json::{json, Value};
use std::io::Write;
use std::sync::Mutex;

// ---------- LLM 接入（OpenAI 兼容；curl 免依赖） ----------

struct Llm {
    key: String,
    base: String,
    model: String,
}

fn llm_cfg() -> Llm {
    fn env_of(keys: &[&str]) -> Option<String> {
        keys.iter()
            .find_map(|k| std::env::var(k).ok())
            .filter(|s| !s.is_empty())
    }
    let mut key = env_of(&["OPENAI_API_KEY", "MARQDO_LLM_API_KEY", "USTC_LLM_API_KEY"]);
    let mut base = env_of(&["OPENAI_BASE_URL", "MARQDO_LLM_BASE_URL"]);
    let mut model = env_of(&["OPENAI_MODEL", "MARQDO_LLM_MODEL"]);
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if let Ok(txt) = std::fs::read_to_string(&env_path) {
        for line in txt.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let v = v.trim().trim_matches('"').to_string();
                match k.trim() {
                    "OPENAI_API_KEY" | "MARQDO_LLM_API_KEY" => key = key.or(Some(v)),
                    "OPENAI_BASE_URL" | "MARQDO_LLM_BASE_URL" => base = base.or(Some(v)),
                    "OPENAI_MODEL" | "MARQDO_LLM_MODEL" => model = model.or(Some(v)),
                    _ => {}
                }
            }
        }
    }
    Llm {
        key: key.expect("未找到 API 密钥：请设置 OPENAI_API_KEY 或仓库根 .env"),
        base: base.unwrap_or_else(|| "https://llm.cflmy.cn/v1".to_string()),
        model: model.unwrap_or_else(|| "cflmy".to_string()),
    }
}

/// 调用一次 LLM；空体/非 JSON/瞬时错误自动重试 3 次（容错重试），仍失败 ⇒ Err（记为基础设施失败）。
fn chat(llm: &Llm, system: &str, user: &str) -> Result<String, String> {
    let req = json!({
        "model": llm.model,
        "temperature": 0,
        "max_tokens": 8000,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ]
    });
    let url = format!("{}/chat/completions", llm.base.trim_end_matches('/'));
    let auth = format!("Authorization: Bearer {}", llm.key);
    let mut last = String::new();
    for _ in 0..5 {
        let mut child = std::process::Command::new("curl")
            .args(["-sS", "-m", "120", "--retry", "2", "--retry-all-errors", "--retry-delay", "2", &url, "-H", &auth, "-H", "Content-Type: application/json", "--data-binary", "@-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("启动 curl");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(req.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        match serde_json::from_str::<Value>(&text) {
            Ok(v) if v["error"].is_object() => {
                last = format!("LLM 报错: {}", v["error"]);
            }
            Ok(v) => {
                if let Some(c) = v["choices"][0]["message"]["content"].as_str() {
                    return Ok(c.to_string());
                }
                last = format!("响应缺 content: {}", truncate(&text, 120));
            }
            Err(_) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                last = format!("非 JSON / 空体: {} {}", truncate(&text, 80), truncate(&err, 80));
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    Err(last)
}

// ---------- 提示词与解析 ----------

const SYSTEM: &str = "你是 Marqdo 语言（.mq.md，Markdown 标记即语法）的有界修复器。\n\
叙述是文档；`**…**` 是代码语句；`*…*` 是返回；`>` 是调用；GFM 表格是数据或契约表（如 参数/返回/字段 契约）。\n\
你会收到一份带行号的源码与结构化诊断。请只输出一个 JSON 数组（不要 markdown 围栏、不要任何解释），\n\
元素形如 {\"op\":\"replace\",\"line\":N,\"text\":\"整行新内容\"} / {\"op\":\"delete\",\"line\":N} / {\"op\":\"insert\",\"line\":N,\"text\":\"新行\"}\n\
（insert 表示在第 N 行之前插入）。触碰的行号必须落在给定范围内（insert 的 N 允许取范围末端+1）。\n\
若判断无法在约束内修复，输出 []。尽量最小改动。";

fn prompt(src: &str, ranges: &[(u32, u32)], diags: &[Value], feedback: &str) -> String {
    let numbered: String = src
        .lines()
        .enumerate()
        .map(|(i, l)| format!("{:6}|{}\n", i + 1, l))
        .collect();
    let ranges_txt = ranges
        .iter()
        .map(|(a, b)| format!("[{a},{b}]"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut s = format!(
        "请修复下面这份 Marqdo 程序的错误。\n\n\
         允许触碰的行范围（1 起，含端点；insert 可取范围末端+1）：{ranges_txt}\n\n\
         结构化诊断（JSON）：\n{}\n\n\
         源码（带行号）：\n{numbered}",
        serde_json::to_string_pretty(diags).unwrap()
    );
    if !feedback.is_empty() {
        s.push_str(&format!(
            "\n上一次尝试未通过验证：{feedback}\n请换一种修法（仍受行范围约束）。\n"
        ));
    }
    s
}

/// 从模型输出提取行编辑（容错：数组 / 单对象 / NDJSON 多对象流 / 字符串化元素 /
/// 字符串型行号 / ```围栏 都收）；解析失败 ⇒ 空 = 弃权。
fn parse_edits(reply: &str) -> Vec<Edit> {
    let s = reply.trim();
    let s = s
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    // 1) 数组形优先（元素可能是字符串化对象）；2) 回退到全串扫描平衡对象（覆盖 NDJSON 流）
    let mut values: Vec<Value> = Vec::new();
    if let (Some(a), Some(b)) = (s.find('['), s.rfind(']')) {
        if b > a {
            if let Ok(list) = serde_json::from_str::<Vec<Value>>(&s[a..=b]) {
                values = list;
            }
        }
    }
    if values.is_empty() {
        values = scan_objects(s);
    }
    values
        .iter()
        .filter_map(|e| {
            // 元素也可能是字符串化的 JSON 对象（模型防转义时常见）
            let obj: Value = if e.is_string() {
                serde_json::from_str(e.as_str().unwrap_or("")).unwrap_or(Value::Null)
            } else {
                e.clone()
            };
            let line = match obj.get("line")? {
                Value::Number(n) => n.as_u64()? as u32,
                Value::String(s) => s.trim().parse::<u32>().ok()?,
                _ => return None,
            };
            let text = obj.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
            Some(match obj.get("op")?.as_str()? {
                "replace" => Edit::Replace { line, text },
                "delete" => Edit::Delete { line },
                "insert" => Edit::InsertBefore { line, text },
                _ => return None,
            })
        })
        .collect()
}

/// 扫描文本中所有**平衡的** JSON 对象（字符串感知）；非法片段跳过。
fn scan_objects(s: &str) -> Vec<Value> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'{' {
            let (mut depth, mut in_str, mut esc) = (0i32, false, false);
            let mut j = i;
            let mut closed = false;
            while j < b.len() {
                let c = b[j] as char;
                if esc {
                    esc = false;
                } else if c == '\\' && in_str {
                    esc = true;
                } else if c == '"' {
                    in_str = !in_str;
                } else if !in_str {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            closed = true;
                            break;
                        }
                    }
                }
                j += 1;
            }
            if closed {
                let frag = &s[i..=j];
                let v = serde_json::from_str::<Value>(frag)
                    .or_else(|_| serde_json::from_str::<Value>(&unquote_line_nums(frag)));
                if let Ok(v) = v {
                    out.push(v);
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// 确定性修补模型的常见 JSON 笔误：`"line":3` 数字后多余引号（`"line":3"` → `"line":3`）。
/// 通用 JSON 容错，与 Marqdo 语法无关；只动 `"line"` 后的数字位。
fn unquote_line_nums(frag: &str) -> String {
    let cs: Vec<char> = frag.chars().collect();
    let mut out = String::with_capacity(frag.len());
    let mut i = 0;
    while i < cs.len() {
        if cs[i..].starts_with(&['"', 'l', 'i', 'n', 'e', '"'][..]) {
            out.extend(['"', 'l', 'i', 'n', 'e', '"']);
            i += 6;
            while i < cs.len() && (cs[i] == ':' || cs[i].is_whitespace()) {
                out.push(cs[i]);
                i += 1;
            }
            let mut digits = 0;
            while i < cs.len() && cs[i].is_ascii_digit() {
                out.push(cs[i]);
                digits += 1;
                i += 1;
            }
            if digits > 0 && i < cs.len() && cs[i] == '"' {
                i += 1; // 剥掉数字后多余的引号
            }
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

/// 消融臂（plain）：把文档上下文从诊断里剥掉——只留 code/severity/message/span。
fn strip_doc_context(diags: &[Value]) -> Vec<Value> {
    diags
        .iter()
        .map(|d| {
            let mut o = d.clone();
            if let Some(obj) = o.as_object_mut() {
                for k in ["doc_anchor", "doc_quote", "suggestion", "contract_ref"] {
                    obj.remove(k);
                }
            }
            o
        })
        .collect()
}

/// 真实消费 `mlsp repair_targets`：诊断 → 靶点范围。
fn target_ranges(diag: &Value) -> Vec<(u32, u32)> {
    let resp = mcall(json!({"id": 9, "method": "repair_targets", "params": {
        "code": diag["code"], "doc_anchor": diag["doc_anchor"], "span": diag["span"], "suggestion": diag["suggestion"]
    }}));
    assert_eq!(resp["ok"], true, "repair_targets 必须可用: {resp}");
    assert_eq!(resp["result"]["repair"]["on_violation"], "abstain");
    let mut rs: Vec<(u32, u32)> = resp["result"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(ranges_of)
        .collect();
    rs.sort_unstable();
    rs.dedup();
    rs
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}

// ---------- 单例实验 ----------

#[derive(Debug)]
struct Outcome {
    case: String,
    arm: &'static str,
    fixed: bool,
    one_shot: bool,
    attempts: u32,
    rejections: u32,
    abstained: bool,
    note: String,
}

fn run_case(llm: &Llm, case: &Case, arm: &'static str) -> Outcome {
    let tag = format!("{}_{}", arm, case.name);
    let mut src = case.broken.clone();
    let mut attempts = 0u32;
    let mut rejections = 0u32;
    let mut abstained = false;
    let mut fixed = false;
    let mut one_shot = false;
    let mut feedback = String::new();
    let mut note = String::new();

    let mut infra = 0u32;
    while attempts < 2 {
        let diags = diagnose(case.verify, &src, &tag);
        let Some(diag) = diags.first().cloned() else {
            fixed = true;
            one_shot = true;
            break;
        };
        let ranges = target_ranges(&diag);
        assert!(!ranges.is_empty(), "[{}] 诊断必须可锚定: {diag}", case.name);

        let shown = if arm == "anchor" {
            diags.clone()
        } else {
            strip_doc_context(&diags)
        };
        let reply = match chat(llm, SYSTEM, &prompt(&src, &ranges, &shown, &feedback)) {
            Ok(r) => r,
            Err(e) => {
                // 基础设施抖动不计入修复尝试
                infra += 1;
                if infra >= 2 {
                    note = format!("LLM 调用失败（重试后）: {e}");
                    break;
                }
                continue;
            }
        };
        attempts += 1;
        let edits = parse_edits(&reply);
        if edits.is_empty() {
            abstained = true;
            note = format!("模型弃权/输出不可解析: {}", truncate(&reply, 100));
            break;
        }

        let resp = apply_via_mlsp(&src, &ranges, &edits);
        if resp["ok"] == false {
            // 护栏：越界必拒 + abstain（源零字节不改 —— apply 全有或全无）
            rejections += 1;
            assert_eq!(resp["error"]["code"], "mlsp.repair_out_of_scope", "{resp}");
            assert_eq!(resp["error"]["on_violation"], "abstain", "{resp}");
            note = format!("越界被拒: {}", truncate(&resp["error"]["message"].to_string(), 100));
            feedback = note.clone();
            continue;
        }

        let new_src = resp["result"]["source"].as_str().unwrap().to_string();
        let after = diagnose(case.verify, &new_src, &tag);
        if after.is_empty() {
            fixed = true;
            one_shot = attempts == 1;
            break;
        }
        src = new_src;
        feedback = format!("修复后仍有诊断: {}", truncate(&after[0].to_string(), 240));
        note = feedback.clone();
    }

    Outcome {
        case: case.name.clone(),
        arm,
        fixed,
        one_shot,
        attempts,
        rejections,
        abstained,
        note,
    }
}

// ---------- 实验主入口 ----------

#[test]
#[ignore = "模型在环实验：需要 .env / OPENAI_API_KEY 与网络（cflmy @ llm.cflmy.cn）"]
fn exp_a_anchored_repair_with_model() {
    let llm = llm_cfg();
    let mut cs = cases();
    if let Ok(n) = std::env::var("MARQDO_EXP_LIMIT") {
        if let Ok(n) = n.parse::<usize>() {
            cs.truncate(n);
        }
    }

    let outcomes: Mutex<Vec<Outcome>> = Mutex::new(Vec::new());
    for arm in ["anchor", "plain"] {
        let queue: Mutex<Vec<usize>> = Mutex::new((0..cs.len()).collect());
        std::thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|| loop {
                    let i = queue.lock().unwrap().pop();
                    let Some(i) = i else { break };
                    let o = run_case(&llm, &cs[i], arm);
                    eprintln!(
                        "[{arm}] {:<18} {} attempts={} rejects={} {}",
                        o.case,
                        if o.fixed { if o.one_shot { "FIXED@1" } else { "FIXED@2" } } else { "FAILED " },
                        o.attempts,
                        o.rejections,
                        o.note
                    );
                    outcomes.lock().unwrap().push(o);
                });
            }
        });
    }

    // ---- 汇总 ----
    let outs = outcomes.into_inner().unwrap();
    let mut report = String::new();
    report.push_str(&format!(
        "# 修复实验（§5.2(a) 锚点消融）—— {}\n\n\
         | | |\n|---|---|\n| 模型 | `{}` @ `{}` |\n| 样本 | golden 20 例（易锚定错误） |\n\
         | 协议 | `validate` → `repair_targets` → 模型行编辑 → `repair_apply`（越界必拒/abstain）→ 复验；≤2 轮 |\n\
         | 对照 | `anchor` = 诊断带 `doc_anchor`/`doc_quote`/`suggestion`；`plain` = 全部剥掉 |\n\n",
        chrono_date(), llm.model, llm.base
    ));

    let arm_stats = |arm: &str| -> String {
        let rows: Vec<&Outcome> = outs.iter().filter(|o| o.arm == arm).collect();
        let n = rows.len().max(1) as f64;
        let one = rows.iter().filter(|o| o.one_shot).count();
        let fin = rows.iter().filter(|o| o.fixed).count();
        let att: f64 = rows.iter().map(|o| o.attempts as f64).sum::<f64>() / n;
        let rej = rows.iter().map(|o| o.rejections).sum::<u32>();
        let abs = rows.iter().filter(|o| o.abstained).count();
        format!(
            "| {arm} | {one}/{rows_n}（{one_pct:.0}%） | {fin}/{rows_n}（{fin_pct:.0}%） | {att:.2} | {rej} | {abs} |\n",
            rows_n = rows.len(),
            one_pct = 100.0 * one as f64 / n,
            fin_pct = 100.0 * fin as f64 / n,
        )
    };
    report.push_str("## 汇总（设计目标：一轮修复率 ≥ 80%）\n\n| 臂 | 一轮修复率 | 最终修复率(≤2轮) | 平均尝试 | 越界拒（护栏） | 弃权 |\n|---|---|---|---|---|---|\n");
    let stat_a = arm_stats("anchor");
    let stat_p = arm_stats("plain");
    report.push_str(&stat_a);
    report.push_str(&stat_p);

    report.push_str("\n## 逐例\n\n| 例 | 臂 | 结果 | 轮次 | 越界拒 | 备注 |\n|---|---|---|---|---|---|\n");
    let mut sorted: Vec<&Outcome> = outs.iter().collect();
    sorted.sort_by(|a, b| a.case.cmp(&b.case).then(a.arm.cmp(b.arm)));
    for o in &sorted {
        report.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            o.case,
            o.arm,
            if o.fixed { if o.one_shot { "一轮修复" } else { "二轮修复" } } else { "未修复" },
            o.attempts,
            o.rejections,
            o.note.replace('|', "\\|")
        ));
    }
    report.push_str("\n## 护栏核对\n\n- 每次越界编辑都以 `mlsp.repair_out_of_scope` + `abstain` 被拒，且 `repair_apply` 全有或全无（源零字节不改）——由 `tests/repair_loop.rs` 的确定性断言覆盖；本实验计数仅作记录。\n");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("doc/roadmap")
        .join(format!("repair-experiment-{}-{}.md", chrono_date(), llm.model));
    std::fs::write(&path, &report).unwrap();
    eprintln!("\n{report}");
    eprintln!("报告已写入 {}", path.display());

    // ---- 功能正确性断言（实验 = 真实消费者） ----
    let anchor_fixed = outs.iter().filter(|o| o.arm == "anchor" && o.fixed).count();
    assert!(
        anchor_fixed >= 1,
        "修复回路必须至少闭环一例（否则功能链路有问题）：\n{:#?}",
        outs
    );
}

/// 无外部 chrono 依赖的日期（UTC）。
fn chrono_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // 2026-09-22 ≈ 1790000000s 附近的简易换算不必精确到历法；直接用运行日环境变量兜底
    if let Ok(d) = std::env::var("MARQDO_EXP_DATE") {
        return d;
    }
    // 简化：从 unix 秒推 YYYY-MM-DD（1970-01-01 起算，忽略闰秒）
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant 的 days→civil 算法（公历）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

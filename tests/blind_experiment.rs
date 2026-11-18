//! P4 实验 (b) 盲测——**查询式（Q）vs 全文档背诵（D）**
//! （[perf-validation.md](../../doc/roadmap/perf-validation.md) §2；three-problems.md §3.7 最终判据）。
//!
//! 唯一变量 = 语法知识获取方式：Q 臂只给硬规则 + 契约表形状 + MLSP 查询通道（≤3 次）；
//! D 臂给硬规则 + 契约表形状 + 完整 markup 参考（`blind/docs-bundle.md`，背诵式全语法表）。
//! 两臂任务 / 提示词模板 / 修复回路（≤2 次）/ 模型完全相同。
//!
//! 判据：Q 功能通过率不劣于 D；首轮语法错误率 Q ≤ D；token Q ≤ 70% of D；修复轮次不升。
//! 执行：`OPENAI_MODEL=… cargo test --test blind_experiment -- --ignored --nocapture`。

use std::io::Write;

use marqdo::mlsp::handle_line;
use serde_json::{json, Value};

// ---------- 材料（冻结；只读实验材料） ----------

const DOCS_BUNDLE: &str = include_str!("blind/docs-bundle.md");
const T_CLAIMS: &str = include_str!("blind/tasks/claims_reserve_settle.md");
const T_MILESTONE: &str = include_str!("blind/tasks/contract_milestone_pay.md");
const T_VENDOR: &str = include_str!("blind/tasks/vendor_po_gate.md");
const T_GCD: &str = include_str!("blind/tasks/qb_gcd.md");
const T_KADANE: &str = include_str!("blind/tasks/qb_max_sublist_sum.md");
const T_FACTORS: &str = include_str!("blind/tasks/qb_get_factors.md");

struct Task {
    id: &'static str,
    desc: &'static str,
    oracle: &'static str,
}

fn tasks() -> Vec<Task> {
    vec![
        Task { id: "claims_reserve_settle", desc: T_CLAIMS, oracle: "28000" },
        Task { id: "contract_milestone_pay", desc: T_MILESTONE, oracle: "37354" },
        Task { id: "vendor_po_gate", desc: T_VENDOR, oracle: "814" },
        Task { id: "qb_gcd", desc: T_GCD, oracle: "6\n21\n5\n1" },
        Task { id: "qb_max_sublist_sum", desc: T_KADANE, oracle: "6\n6\n-1" },
        Task { id: "qb_get_factors", desc: T_FACTORS, oracle: "2*2*3\n13\n2*2*3*5" },
    ]
}

// ---------- 提示词（受控：硬规则与契约形状两臂同文） ----------

const HARD_RULES: &str = "\
## Hard rules (do not violate)\n\
1. **File suffix** must be `.mq.md`.\n\
2. **`**…**` = code** (assign / call). **`*…*` = return value**. Output is **not** either marker — use `> print text=…` / `**print text=\"…\"**` / `> 打印 内容=…`.\n\
3. **Prose:** unmarked narrative may contain `` `名` `` (declare/ref), inline `**code**`, and `*return*`. Soft Markdown emphasis that is not code-shaped is ignored. Dead `` `名` `` (never read) is OK and does not become a param.\n\
4. **Do not invent keywords** `if` / `else` / `while` / `for` / `def` / `return`. Control flow is Markdown: **`1.` `2.` … branches**, **`-` loops**, arm `N. *` = else, `#` = **object/type**, `##`+ = **function/method**. Params: prefer prose `` `名` `` / `` `名`=默认 `` (inferred); `` + `名` `` still accepted.\n\
5. **Identifiers:** `` `名` `` in prose; inside `*…*` / `**…**`, bare ids are **variables**. **Text literals must be quoted** in bold/italic: `**print text=\"hi\"**`. Standalone `>` calls: bare words = text; vars need ticks: `` > str `n` ``.\n\
6. **Structure lines** (`#` `>` `+` `-` `|` `1.`) need not be wrapped in bold.\n\
7. **Paths:** In bare expressions `/` is division. In call args / defaults / table cells, unspaced `a/b` and quoted paths are text.\n\
8. Prefer ending side-effect-only bodies with `*None*` / `*无*` / whole-line `**` / `****` when the next lines must belong to an outer function. Lone `---` / `***` are Markdown thematic breaks (skipped everywhere), not function end.\n\
9. **`ext/**` never calls `host_*`.**\n\
10. **Code-as-documentation:** prefer GFM tables for maps/lists; prefer named helpers over `json.set` / `json.append` chains.\n\
11. **Table element access has two spellings** — `[键](集合)` is the recommended one (e.g. `[name](`用户`)`, `[1](`xs`)`); footnote form `` `集合`[^键] `` is legacy-compat only.\n\
12. **Bracket-marked calls:** function name may be marked with `[函数名]`; bare words before `[` are boolean modifiers. `]` immediately followed by `(` is **index** `[key](coll)`, not a call.\n";

const CONTRACT_SHAPES: &str = "\
## 渐进式契约表（可选校验元数据）\n\
函数契约 = `##` 函数体内**首个可执行行之前**的未绑定文档表，两种列头：\n\
\n\
| 参数 | 类型 | 说明 |\n\
|------|------|------|\n\
| `n` | number | 输入值 |\n\
\n\
| 返回 | 类型 | 说明 |\n\
|------|------|------|\n\
|  | number | 返回说明 |\n\
\n\
集合契约 = `` `名` = `` 绑定**之前**的 `| 字段 | 类型 | 可空 |` 表。\n\
对象契约 = `# 对象` 体内首个成员之前的 `| 字段 | 类型 |` 表。\n\
规则：**被绑定的表是数据，永不视为契约**；类型词表 = `text` / `number` / `bool` / `list` / `map` / `any` / 对象名；\n\
契约表一经出现即为完整声明（写全参数行，否则报 param_drift 警告）。\n";

const QUERY_PROTOCOL: &str = "\
## On-demand syntax queries（你的唯一语法来源）\n\
你**没有** Marqdo 语法参考。对任何不确定的语法细节，先查询再写码：单独输出一行\n\
`QUERY: <关键字>`（如 `QUERY: 返回`、`QUERY: 表`、`QUERY: 调用`），我会回给你对应构造卡片（语义/戒律/样例原文）。\n\
最多 3 次查询。**不要凭 Markdown/Python 习惯猜语法。**\n";

fn system_prompt(arm: &str) -> String {
    let intro = "你是 Marqdo 语言（`.mq.md`：Markdown 标记即语法）的程序作者。写完整可运行程序（含 `# main`）。\n\n";
    let mut s = String::from(intro);
    s.push_str(HARD_RULES);
    s.push('\n');
    s.push_str(CONTRACT_SHAPES);
    s.push('\n');
    if arm == "docs" {
        s.push_str("## Markup reference (v0.3, complete)\n\n");
        s.push_str(DOCS_BUNDLE);
    } else {
        s.push_str(QUERY_PROTOCOL);
    }
    s
}

// ---------- LLM 接入（OpenAI 兼容；curl 免依赖；带 usage 计量） ----------

struct Llm {
    key: String,
    base: String,
    model: String,
}

struct Reply {
    text: String,
    ptok: u32,
    ctok: u32,
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

/// 多轮对话一次；空体/非 JSON/瞬时错误自动重试（容错重试），仍失败 ⇒ Err（基础设施失败）。
fn chat(llm: &Llm, msgs: &[(String, String)]) -> Result<Reply, String> {
    let messages: Vec<Value> = msgs
        .iter()
        .map(|(r, c)| json!({"role": r, "content": c}))
        .collect();
    let url = format!("{}/chat/completions", llm.base.trim_end_matches('/'));
    let auth = format!("Authorization: Bearer {}", llm.key);
    let mut last = String::new();
    let mut max_tokens = 12000u64; // reasoning 模型先思考后作答；截断时自适应加倍
    for _ in 0..12 {
        let req = json!({
            "model": llm.model,
            "temperature": 0,
            "max_tokens": max_tokens,
            "messages": messages
        });
        let mut child = std::process::Command::new("curl")
            .args(["-sS", "-m", "180", "--retry", "2", "--retry-all-errors", "--retry-delay", "2", &url, "-H", &auth, "-H", "Content-Type: application/json", "--data-binary", "@-"])
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
                let reply = v["choices"][0]["message"]["content"].as_str().unwrap_or("");
                if !reply.is_empty() {
                    return Ok(Reply {
                        text: reply.to_string(),
                        ptok: v["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                        ctok: v["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    });
                }
                if v["choices"][0]["finish_reason"] == "length" && max_tokens < 32000 {
                    max_tokens *= 2; // reasoning 吃满额度 ⇒ 加倍后重试
                    last = format!("截断（finish=length，content 空），max_tokens 加倍至 {max_tokens}");
                } else {
                    last = format!("响应缺 content: {}", truncate(&text, 120));
                }
            }
            Err(_) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                last = format!("非 JSON / 空体: {} {}", truncate(&text, 80), truncate(&err, 80));
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
    Err(last)
}

fn truncate(s: &str, n: usize) -> String {
    let t: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{t}…")
    } else {
        t
    }
}

// ---------- 程序提取 / 查询 / 执行 ----------

/// 从回复中提取程序源：优先含 `# main` 的围栏，其次最长围栏，再次裸文含 `# main`。
fn extract_program(reply: &str) -> Option<String> {
    let mut best: Option<String> = None;
    let mut rest = reply;
    while let Some(a) = rest.find("```") {
        let after = &rest[a + 3..];
        let after = after.strip_prefix("markdown").unwrap_or(after);
        let after = after.strip_prefix('\n').unwrap_or(after);
        if let Some(b) = after.find("```") {
            let block = &after[..b];
            if block.contains("# main") {
                return Some(dedent(block));
            }
            if best.as_ref().map(|s: &String| s.len() < block.len()).unwrap_or(true) {
                best = Some(block.to_string());
            }
            rest = &after[b + 3..];
        } else {
            break;
        }
    }
    if let Some(b) = best {
        return Some(dedent(&b));
    }
    if reply.contains("# main") {
        return Some(reply.trim().to_string());
    }
    None
}

fn dedent(block: &str) -> String {
    let lines: Vec<&str> = block.lines().collect();
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let out: String = lines
        .iter()
        .map(|l| l.get(indent..).unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n", out.trim_end())
}

/// QUERY 行 → 查询关键字列表。
fn extract_queries(reply: &str) -> Vec<String> {
    reply
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            let rest = t.strip_prefix("QUERY:").or_else(|| t.strip_prefix("QUERY："))?;
            let q = rest.trim();
            if q.is_empty() {
                None
            } else {
                Some(q.to_string())
            }
        })
        .collect()
}

fn mlp_syntax(query: &str) -> String {
    handle_line(&json!({"id": 1, "method": "syntax", "params": {"query": query}}).to_string())
}

fn validate_codes(src: &str) -> (bool, Vec<String>, String) {
    let resp: Value =
        serde_json::from_str(&handle_line(&json!({"id": 1, "method": "validate", "params": {"source": src}}).to_string()))
            .expect("MLSP 响应必须是合法 JSON");
    if resp["ok"] != true {
        let code = resp["error"]["code"].as_str().unwrap_or("?").to_string();
        return (false, vec![code], resp.to_string());
    }
    let diags = resp["result"]["diagnostics"].as_array().cloned().unwrap_or_default();
    let codes: Vec<String> = diags
        .iter()
        .filter_map(|d| d["code"].as_str().map(String::from))
        .collect();
    (true, codes, diags.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("\n"))
}

fn run_src(task_id: &str, arm: &str, src: &str) -> (i32, String, String) {
    let p = std::env::temp_dir().join(format!(
        "marqdo_blind_{}_{}_{}.mq.md",
        std::process::id(),
        arm,
        task_id
    ));
    std::fs::write(&p, src).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", p.to_str().unwrap()])
        .output()
        .expect("运行 marqdo");
    let _ = std::fs::remove_file(&p);
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

// ---------- 单例执行 ----------

struct Outcome {
    task: &'static str,
    arm: &'static str,
    result: String, // pass / load-fail / run-fail / wrong-output / infra
    rounds: u32,    // 通过前的修复次数（首轮通过 = 0）
    first_load_fail: bool,
    first_contract_diag: bool,
    ptok: u32,
    ctok: u32,
    queries: u32,
    note: String,
}

fn run_case(llm: &Llm, task: &Task, arm: &'static str) -> Outcome {
    let mut msgs: Vec<(String, String)> = vec![
        ("system".into(), system_prompt(arm)),
        ("user".into(), format!("{}\n\n开始写程序。", task.desc)),
    ];
    let (mut ptok, mut ctok) = (0u32, 0u32);
    let mut queries = 0u32;

    // 查询阶段（Q 臂协议；D 臂模型不被教导 QUERY，自然不会发）
    let mut text = String::new();
    let mut infra: Option<String> = None;
    for _ in 0..6 {
        match chat(llm, &msgs) {
            Ok(r) => {
                ptok += r.ptok;
                ctok += r.ctok;
                text = r.text;
            }
            Err(e) => {
                infra = Some(e);
                break;
            }
        }
        if extract_program(&text).is_some() {
            break; // 已带程序（即便附带 QUERY 行也直接用）
        }
        let qs = extract_queries(&text);
        if qs.is_empty() {
            break; // 无程序无查询 ⇒ 交给修复回路要程序
        }
        let mut answer = String::new();
        for q in qs.iter().take(3usize.saturating_sub(queries as usize)) {
            answer.push_str(&format!("QUERY {} → {}\n", q, mlp_syntax(q)));
            queries += 1;
        }
        if answer.is_empty() {
            answer.push_str("查询额度已用完（最多 3 次）。不要再查询，直接写完整程序。\n");
        }
        msgs.push(("assistant".into(), text.clone()));
        msgs.push((
            "user".into(),
            format!("{answer}\n继续；完成后把完整程序放在**单个** ```markdown 围栏里。"),
        ));
    }
    if let Some(e) = infra {
        return Outcome {
            task: task.id,
            arm,
            result: "infra".into(),
            rounds: 0,
            first_load_fail: false,
            first_contract_diag: false,
            ptok,
            ctok,
            queries,
            note: truncate(&e, 140),
        };
    }

    let mut out = Outcome {
        task: task.id,
        arm,
        result: "infra".into(),
        rounds: 0,
        first_load_fail: false,
        first_contract_diag: false,
        ptok,
        ctok,
        queries,
        note: String::new(),
    };

    for attempt in 0..3u32 {
        let src_opt = extract_program(&text);
        let (load_ok, codes, diag_txt) = match &src_opt {
            Some(src) => validate_codes(src),
            None => (false, vec!["harness.no_program".to_string()], String::new()),
        };
        let contract_diag = codes.iter().any(|c| c.starts_with("contract."));
        if attempt == 0 {
            out.first_load_fail = src_opt.is_none() || !load_ok;
            out.first_contract_diag = contract_diag;
        }
        let (code, stdout, stderr) = match &src_opt {
            Some(src) => run_src(task.id, arm, src),
            None => (
                1,
                String::new(),
                "回复中没有 ```markdown 围栏程序（或不含 # main）。".to_string(),
            ),
        };
        if code == 0 && stdout.trim_end() == task.oracle.trim() {
            out.result = "pass".into();
            out.rounds = attempt;
            out.note = format!("queries={queries}");
            break;
        }
        out.result = if src_opt.is_none() || !load_ok {
            "load-fail".into()
        } else if code != 0 {
            "run-fail".into()
        } else {
            "wrong-output".into()
        };
        out.note = if src_opt.is_none() {
            format!("回复中无程序 | 开头: {}", truncate(&text.replace('\n', " "), 160))
        } else {
            truncate(&format!("{diag_txt} | stderr={stderr}"), 300)
        };
        if attempt == 2 {
            break;
        }
        msgs.push(("assistant".into(), text.clone()));
        msgs.push((
            "user".into(),
            format!(
                "程序未通过。\nvalidate 诊断：\n{diag_txt}\n\n运行 stderr：\n{}\n\n运行 stdout：\n{}\n\n请修复问题，把**完整**程序放在**单个** ```markdown 围栏里（围栏内是纯 Marqdo 源码，含 `# main`）。",
                truncate(&stderr, 500),
                truncate(&stdout, 200)
            ),
        ));
        match chat(llm, &msgs) {
            Ok(r) => {
                ptok += r.ptok;
                ctok += r.ctok;
                text = r.text;
            }
            Err(e) => {
                out.note = format!("infra: {}", truncate(&e, 100));
                break;
            }
        }
    }
    out.ptok = ptok;
    out.ctok = ctok;
    out
}

// ---------- 实验入口 ----------

#[test]
#[ignore]
fn exp_b_blind_query_vs_docs() {
    let llm = llm_cfg();
    let limit: usize = std::env::var("MARQDO_EXP_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);
    let mut outs: Vec<Outcome> = Vec::new();
    for arm in ["query", "docs"] {
        for t in tasks().iter().take(limit) {
            let o = run_case(&llm, t, arm);
            eprintln!(
                "[{}] {:<24} {:<12} rounds={} tok=({},{}) q={} | {}",
                o.arm, o.task, o.result, o.rounds, o.ptok, o.ctok, o.queries, o.note
            );
            outs.push(o);
        }
    }

    // 汇总（按判据口径：infra 例从比率中剔除并单列）
    let mut md = String::new();
    md.push_str(&format!(
        "# 盲测实验（§5.2(b) 查询式 vs 全文档背诵）—— {}\n\n| | |\n|---|---|\n| 模型 | `{}` @ `{}` |\n| 样本 | 6 任务（业务 3 + QuixBugs 改写 3），oracle = stdout 精确匹配 |\n| 协议 | 系统提示词 = 硬规则 12 条 + 契约表三形状（两臂同文）；**唯一变量 = 语法来源**：`query` 臂 MLSP 查询通道（≤3 次）vs `docs` 臂完整 markup 参考（背诵） |\n| 修复 | validate 诊断 + stderr 回喂，≤2 次修复；temperature=0，max_tokens=8000 |\n\n",
        chrono_date(),
        llm.model,
        llm.base
    ));

    md.push_str("## 汇总（判据：Q 通过率不劣于 D；首轮语法错误率 Q≤D；token Q≤70%%D；轮次不升）\n\n");
    md.push_str("| 臂 | 通过(一轮) | 通过(最终≤3) | 首轮语法错误 | 首轮契约诊断 | 平均修复轮(过例) | token 合计(prompt+completion) | 弃权/基础设施 |\n");
    md.push_str("|---|---|---|---|---|---|---|---|\n");
    for arm in ["query", "docs"] {
        let os: Vec<&Outcome> = outs.iter().filter(|o| o.arm == arm).collect();
        let n = os.len();
        let infra = os.iter().filter(|o| o.result == "infra").count();
        let denom = n - infra;
        let pass1 = os.iter().filter(|o| o.result == "pass" && o.rounds == 0).count();
        let passf = os.iter().filter(|o| o.result == "pass").count();
        let lf0 = os.iter().filter(|o| o.first_load_fail).count();
        let cd0 = os.iter().filter(|o| o.first_contract_diag).count();
        let rounds: f64 = {
            let p: Vec<u32> = os.iter().filter(|o| o.result == "pass").map(|o| o.rounds).collect();
            if p.is_empty() {
                0.0
            } else {
                p.iter().sum::<u32>() as f64 / p.len() as f64
            }
        };
        let ptok: u32 = os.iter().map(|o| o.ptok).sum();
        let ctok: u32 = os.iter().map(|o| o.ctok).sum();
        md.push_str(&format!(
            "| {arm} | {pass1}/{denom} | {passf}/{denom} | {lf0}/{denom} | {cd0}/{denom} | {rounds:.2} | {}（{}+{}） | {infra} |\n",
            ptok + ctok,
            ptok,
            ctok
        ));
    }

    md.push_str("\n## 逐例\n\n| 例 | 臂 | 结果 | 修复轮 | 首轮语法错 | 首轮契约诊断 | token(p+c) | 查询 | 备注 |\n|---|---|---|---|---|---|---|---|---|\n");
    for o in &outs {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {}+{} | {} | {} |\n",
            o.task,
            o.arm,
            o.result,
            o.rounds,
            if o.first_load_fail { "是" } else { "否" },
            if o.first_contract_diag { "是" } else { "否" },
            o.ptok,
            o.ctok,
            o.queries,
            o.note.replace('|', "\\|").replace('\n', " ")
        ));
    }
    md.push_str("\n报告由 `tests/blind_experiment.rs` 生成；材料冻结于 `tests/blind/`（任务书 + docs-bundle），判据见 `doc/roadmap/perf-validation.md` §2。\n");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("doc/roadmap")
        .join(format!("blind-experiment-{}-{}.md", chrono_date(), llm.model));
    std::fs::write(&path, md).unwrap();
    eprintln!("报告已写入 {}", path.display());
}

fn chrono_date() -> String {
    if let Ok(d) = std::env::var("MARQDO_EXP_DATE") {
        return d;
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (y, m, d) = civil_from_days((secs / 86400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

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

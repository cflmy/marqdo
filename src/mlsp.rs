//! Marqdo Language Server for AI（MLSP）—— AI 的可查询工具面（T3.1/T3.2，T2.6 并入 `schema`）。
//!
//! 目标（[three-problems.md](../../doc/roadmap/three-problems.md) §3）：**AI 不必背 Marqdo
//! 语法细节**——把「记语法」换成「查工具」。语法唯一事实来源是 `parse::CORE_CONSTRUCTS`
//! （与 `doc/design/core-surface.md` 由 CI 守卫同步）；本模块只加**检索别名**，不定义语法。
//!
//! 行分隔 JSON 协议：每行一个请求 `{"id":…,"method":…,"params":{…}}`，
//! 每行一个响应 `{"id":…,"ok":true,"result":{…}}` / `{"id":…,"ok":false,"error":{…}}`。
//!
//! | method | 作用 |
//! |--------|------|
//! | `locate` | 语法/符号定位：构造卡片，或定义位置 + 契约 |
//! | `syntax` | 语法问答：关键字/别名 → 构造卡片 |
//! | `validate` | 程序校验：结构化诊断数组（parse + 契约 check） |
//! | `repair_targets` | 修复靶点：错误 → **有界**修复目标（越界必拒，护栏） |
//! | `repair_apply` | 应用行编辑（全有或全无；越界 ⇒ `mlsp.repair_out_of_scope` abstain） |
//! | `schema` | 契约查询（T2.6）：单元名 → 形参/返回/字段契约 |

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::ast::{Function, Module};
use crate::check::check_module;
use crate::contract::{Contract, Ty};
use crate::diagnostics::Diagnostic;
use crate::load;
use crate::parse::CORE_CONSTRUCTS;

/// 行分隔 JSON：一行请求 → 一行响应（永不 panic；错误也是结构化 JSON）。
pub fn handle_line(line: &str) -> String {
    let trimmed = line.trim();
    let id = match serde_json::from_str::<Value>(trimmed) {
        Ok(req) => req.get("id").cloned().unwrap_or(Value::Null),
        Err(e) => {
            return json!({
                "id": null,
                "ok": false,
                "error": {"code": "mlsp.bad_request", "message": format!("请求不是合法 JSON: {e}")}
            })
            .to_string();
        }
    };
    let req: Value = serde_json::from_str(trimmed).expect("already parsed");
    match dispatch(&req) {
        Ok(result) => json!({"id": id, "ok": true, "result": result}).to_string(),
        Err(error) => json!({"id": id, "ok": false, "error": error}).to_string(),
    }
}

fn dispatch(req: &Value) -> Result<Value, Value> {
    let params = req.get("params").cloned().unwrap_or(json!({}));
    match req.get("method").and_then(|m| m.as_str()) {
        Some("locate") => locate(&params),
        Some("syntax") => syntax(&params),
        Some("validate") => validate(&params),
        Some("repair_targets") => repair_targets(&params),
        Some("repair_apply") => repair_apply(&params),
        Some("schema") => schema(&params),
        other => Err(err(
            "mlsp.unknown_method",
            format!("未知方法 {other:?}；可用：locate / syntax / validate / repair_targets / repair_apply / schema"),
        )),
    }
}

fn err(code: &str, message: impl Into<String>) -> Value {
    json!({"code": code, "message": message.into()})
}

// ---------- 构造卡片（语法唯一事实来源 = CORE_CONSTRUCTS） ----------

/// 检索别名（中/英关键字）——只做匹配，不定义语法。
const ALIASES: &[(&str, &[&str])] = &[
    ("narrative", &["叙述", "文字", "说明", "注释", "narrative", "prose"]),
    ("blank", &["空行", "blank"]),
    ("heading-unit", &["标题", "单元", "函数", "对象", "heading", "定义"]),
    ("call-line", &["调用", "call"]),
    ("branch-list", &["分支", "条件", "判断", "branch", "if"]),
    ("loop-list", &["循环", "遍历", "loop", "for", "while"]),
    ("param", &["参数", "形参", "param"]),
    ("table", &["表", "表格", "table", "集合", "数据"]),
    ("fence", &["代码块", "围栏", "fence", "公式", "code"]),
    ("hr", &["分隔线", "水平线", "hr"]),
    ("writeback", &["回写", "输出块", "writeback"]),
    ("ident", &["变量", "标识符", "ident", "名字"]),
    ("bold-code", &["粗体", "赋值", "语句", "bold", "执行"]),
    ("italic-return", &["返回", "return", "斜体"]),
    ("empty-return", &["空返回", "none", "结束", "副作用"]),
    ("link-index", &["取元", "索引", "index", "键", "行"]),
    ("bracket-call", &["修饰", "括号", "bracket", "开关参数"]),
    ("footnote-index", &["脚注", "footnote", "取列", "列"]),
    ("soft-emphasis", &["强调", "emphasis", "软强调"]),
];

/// core-surface.md 全文（与 `tests/core_surface.rs` 守卫同源）——卡片机械携带
/// 语义/戒律/样例**原文**，AI 一次查询即可自持，不必回读文档（three-problems.md §3.2「单构造戒律」）。
const SURFACE_MD: &str = include_str!("../doc/design/core-surface.md");

/// 机械提取 core-surface.md 清单行：`| \`id\` | 外形 | 语义 | 戒律 | 样例 |` →
///（语义, 戒律, 样例, 行原文）；只读不改，卡片内容与文档永不漂移。
fn surface_row(id: &str) -> Option<(String, String, String, String)> {
    for line in SURFACE_MD.lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 5 {
            continue;
        }
        let rid = cells[0].trim_matches('`');
        if rid == id && !rid.is_empty() && !rid.contains(' ') && !rid.starts_with('-') {
            return Some((
                cells[2].to_string(),
                cells[3].to_string(),
                cells[4].to_string(),
                t.to_string(),
            ));
        }
    }
    None
}

fn card(id: &str) -> Value {
    let form = CORE_CONSTRUCTS
        .iter()
        .find(|(cid, _)| *cid == id)
        .map(|(_, f)| *f)
        .unwrap_or("?");
    let mut v = json!({
        "id": id,
        "form": form,
        "manual": "doc/design/core-surface.md",
    });
    if let Some((semantics, rule, sample, quote)) = surface_row(id) {
        v["semantics"] = semantics.into();
        v["rule"] = rule.into();
        v["sample"] = sample.into();
        v["doc_quote"] = quote.into();
    }
    v
}

fn match_constructs(query: &str) -> Vec<Value> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return ALIASES.iter().map(|(id, _)| card(id)).collect();
    }
    ALIASES
        .iter()
        .filter(|(_, keys)| {
            keys.iter()
                .any(|k| k.to_lowercase().contains(&q) || q.contains(&k.to_lowercase()))
        })
        .map(|(id, _)| card(id))
        .collect()
}

fn syntax(params: &Value) -> Result<Value, Value> {
    let query = params.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let cards = match_constructs(query);
    if cards.is_empty() {
        return Err(err(
            "mlsp.no_match",
            format!("没有匹配「{query}」的语法构造；换关键字（如 返回/表/调用）或省略 query 列出全部"),
        ));
    }
    Ok(json!({"cards": cards}))
}

fn locate(params: &Value) -> Result<Value, Value> {
    if params.get("path").is_some() || params.get("source").is_some() {
        let (module, path) = module_from_params(params)?;
        let symbol = params
            .get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| err("mlsp.bad_request", "locate 定位符号需要 params.symbol"))?;
        let fun = find_fn(&module, symbol)
            .ok_or_else(|| err("mlsp.unknown_unit", format!("没有名为 `{symbol}` 的单元")))?;
        return Ok(json!({
            "unit": fun.name,
            "level": fun.level,
            "span": span_json(path.as_deref(), fun.span),
            "params": fun.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
            "contract": fun.contract.as_ref().map(|c| contract_json(c, path.as_deref())),
            "manual": "doc/design/core-surface.md",
        }));
    }
    Ok(json!({"cards": match_constructs(
        params.get("query").and_then(|q| q.as_str()).unwrap_or("")
    )}))
}

fn validate(params: &Value) -> Result<Value, Value> {
    let (module, path) = module_from_params(params)?;
    let mut diags = check_module(&module, path.as_deref());
    diags.sort_by_key(|d| (d.span.line, d.span.col));
    let jsons: Vec<Value> = diags.iter().map(|d| d.to_json()).collect();
    let ok = !diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostics::Severity::Error));
    Ok(json!({"ok": ok, "diagnostics": jsons}))
}

/// 修复靶点（T3.3 护栏）：锚点局部修复，**越界必拒**（禁止整文件重写/无关重构）。
fn repair_targets(params: &Value) -> Result<Value, Value> {
    let code = params.get("code").and_then(|c| c.as_str()).unwrap_or("");
    let anchor = params.get("doc_anchor").and_then(|a| a.as_str());
    let suggestion = params.get("suggestion").and_then(|s| s.as_str());
    if code.is_empty() {
        return Err(err("mlsp.bad_request", "repair_targets 需要 params.code"));
    }
    Ok(json!({
        "targets": [{
            "code": code,
            "doc_anchor": anchor,
            "span": params.get("span").cloned().unwrap_or(Value::Null),
        }],
        "repair": {
            "strategy": "anchored-local",
            "bounded": true,
            "max_attempts": 2,
            "scope": "doc_anchor 所锚定的契约表/语句行",
            "suggestion": suggestion,
            "on_violation": "abstain",
            "note": "只许改动锚定范围内的行；越界改动必须拒绝并报 abstain"
        }
    }))
}

/// 应用有界修复（T3.3）：行编辑全有或全无；越界 ⇒ `mlsp.repair_out_of_scope`（abstain），源零字节不改。
fn repair_apply(params: &Value) -> Result<Value, Value> {
    let source = source_text(params)?;
    let ranges = params
        .get("ranges")
        .and_then(|r| r.as_array())
        .ok_or_else(|| err("mlsp.bad_request", "repair_apply 需要 params.ranges（[[起,止],…]）"))?
        .iter()
        .map(|pair| {
            let a = pair.as_array()?;
            Some((a.first()?.as_u64()? as u32, a.get(1)?.as_u64()? as u32))
        })
        .collect::<Option<Vec<(u32, u32)>>>()
        .ok_or_else(|| err("mlsp.bad_request", "ranges 必须是 [[起,止],…]"))?;

    let mut edits = Vec::new();
    let edit_list = params
        .get("edits")
        .and_then(|e| e.as_array())
        .ok_or_else(|| err("mlsp.bad_request", "repair_apply 需要 params.edits"))?;
    for e in edit_list {
        let line = e
            .get("line")
            .and_then(|l| l.as_u64())
            .ok_or_else(|| err("mlsp.bad_request", "edit 缺 line"))? as u32;
        let text = e.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
        edits.push(match e.get("op").and_then(|o| o.as_str()) {
            Some("replace") => crate::repair::Edit::Replace { line, text },
            Some("delete") => crate::repair::Edit::Delete { line },
            Some("insert") => crate::repair::Edit::InsertBefore { line, text },
            other => {
                return Err(err(
                    "mlsp.bad_request",
                    format!("未知编辑 op {other:?}（replace / delete / insert）"),
                ))
            }
        });
    }

    match crate::repair::apply_edits(&source, &ranges, &edits) {
        Ok(new_source) => {
            if params.get("write").and_then(|w| w.as_bool()).unwrap_or(false) {
                let path = params
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| err("mlsp.bad_request", "write=true 需要 params.path"))?;
                std::fs::write(path, &new_source)
                    .map_err(|e| err("mlsp.io_error", format!("写回 {path} 失败: {e}")))?;
            }
            Ok(json!({"source": new_source, "edits_applied": edits.len()}))
        }
        Err(oos) => Err(json!({
            "code": "mlsp.repair_out_of_scope",
            "message": format!(
                "修复越界（abstain）：触及行 {:?} 不在靶点范围 {:?} 内；已拒绝全部编辑，源未改动",
                oos.lines, oos.ranges
            ),
            "lines": oos.lines,
            "ranges": oos.ranges,
            "on_violation": "abstain",
        })),
    }
}

/// 取待修源码（source 直给 / path 读取）。
fn source_text(params: &Value) -> Result<String, Value> {
    if let Some(s) = params.get("source").and_then(|s| s.as_str()) {
        return Ok(s.to_string());
    }
    let path = params
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| err("mlsp.bad_request", "需要 params.source 或 params.path"))?;
    std::fs::read_to_string(path)
        .map_err(|e| err("mlsp.io_error", format!("读取 {path} 失败: {e}")))
}

/// 契约查询（T2.6）：单元名 → 形参/返回/字段契约。
fn schema(params: &Value) -> Result<Value, Value> {
    let (module, path) = module_from_params(params)?;
    let unit = params
        .get("unit")
        .and_then(|u| u.as_str())
        .ok_or_else(|| err("mlsp.bad_request", "schema 需要 params.unit"))?;
    let fun = find_fn(&module, unit)
        .ok_or_else(|| err("mlsp.unknown_unit", format!("没有名为 `{unit}` 的单元")))?;
    let mut vars = serde_json::Map::new();
    for (name, c) in &fun.var_contracts {
        vars.insert(name.clone(), contract_json(c, path.as_deref()));
    }
    Ok(json!({
        "unit": fun.name,
        "contract": fun.contract.as_ref().map(|c| contract_json(c, path.as_deref())),
        "vars": Value::Object(vars),
        "params_inferred": fun.params.iter().map(|p| json!({
            "name": p.name, "inferred": p.inferred
        })).collect::<Vec<_>>(),
    }))
}

// ---------- 公共小工具 ----------

fn module_from_params(params: &Value) -> Result<(Module, Option<PathBuf>), Value> {
    if let Some(src) = params.get("source").and_then(|s| s.as_str()) {
        let module = load::load_module_from_source(src)
            .map_err(|e| diagnostic_err("mlsp.parse_error", &e))?;
        return Ok((module, None));
    }
    let path = params
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| err("mlsp.bad_request", "需要 params.path 或 params.source"))?;
    let path = PathBuf::from(path);
    let module = load::load_module(&path).map_err(|e| diagnostic_err("mlsp.parse_error", &e))?;
    Ok((module, Some(path)))
}

/// 错误链里有结构化诊断就原样透出（AI/MLSP 面不许拍平）。
fn diagnostic_err(code: &str, e: &anyhow::Error) -> Value {
    match Diagnostic::find(e) {
        Some(d) => d.to_json(),
        None => err(code, format!("{e:#}")),
    }
}

fn find_fn<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    fn walk<'a>(funs: &'a [Function], name: &str) -> Option<&'a Function> {
        funs.iter().find(|f| f.name == name).or_else(|| {
            funs.iter()
                .find_map(|f| walk(&f.children, name))
        })
    }
    walk(&module.functions, name)
}

fn span_json(path: Option<&Path>, span: crate::diagnostics::Span) -> Value {
    json!({
        "file": path.map(|p| p.display().to_string()),
        "line": span.line,
        "col": span.col,
    })
}

fn ty_json(ty: &Ty) -> Value {
    json!(ty.name())
}

fn contract_json(c: &Contract, path: Option<&Path>) -> Value {
    json!({
        "params": c.params.iter().map(|p| json!({
            "name": p.name, "type": ty_json(&p.ty), "desc": p.desc
        })).collect::<Vec<_>>(),
        "returns": c.returns.as_ref().map(|r| json!({
            "type": ty_json(&r.ty), "desc": r.desc
        })),
        "fields": c.fields.iter().map(|f| json!({
            "name": f.name, "type": ty_json(&f.ty), "nullable": f.nullable, "desc": f.desc
        })).collect::<Vec<_>>(),
        "doc_anchor": c.anchor(path),
    })
}

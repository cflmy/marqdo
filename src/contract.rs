//! Progressive contracts (渐进式契约 / 标注类型) —— schema 长在表格/说明段落里。
//!
//! 动态默认、按需校验、零契约文件零回归；TS/TS 的全程序静态类型被明确**拒绝**。
//! 契约唯一来源是**未绑定文档表**（绑定表是数据，永不视为契约）：
//!
//! | 形状（含 `类型` 列） | 允许位置 | 附着对象 |
//! |----------------------|----------|----------|
//! | `参数`/`返回` 表 | 函数体首个可执行行之前 | 函数（形参/返回契约） |
//! | `字段` 表 | 紧邻 `` `名` = `` 绑定之前 | 变量（集合契约） |
//! | `字段` 表 | 对象（`#`）体首成员之前 | 对象类型（字段契约） |
//!
//! 提取在**升参推断之前**执行（见 `parse`），契约表的单元不会污染推断。
//! 校验只发生在四个运行时边界（调用实参 / 字段访问 / 表绑定 / 返回）与
//! `marqdo check` 静态互查；契约表本身永不执行、永不进入值空间。
//! 锚点诊断见 [`Contract::anchor`] / [`Contract::quote_text`]（T2.4）。

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::ast::Expr;
use crate::diagnostics::{ContractRef, Diagnostic, Severity, Span};
use crate::value::Value;

/// 契约类型词汇（`doc/design/core-surface.md` 的文档层扩展，不进语言核心）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// 任意值（类型列留空）。
    Any,
    Text,
    /// `int` / `num`（`Value::Int` / `Value::Num` / 符号公式）。
    Number,
    Bool,
    List,
    Map,
    /// 对象类型名（`#` 对象；`check` 阶段校验存在性，防 `nubmer` 拼错）。
    Object(String),
}

impl Ty {
    /// 解析类型列文本（大小写不敏感；未知名 → `Object(name)`，由 check 验证）。
    pub fn parse(raw: &str) -> Ty {
        let name = raw.trim().trim_matches('`').to_ascii_lowercase();
        match name.as_str() {
            "" | "any" | "任意" => Ty::Any,
            "text" | "str" | "string" | "文本" | "字串" | "字符串" => Ty::Text,
            "number" | "int" | "float" | "num" | "numeric" | "数字" | "数值" => Ty::Number,
            "bool" | "boolean" | "布尔" => Ty::Bool,
            "list" | "列表" | "表" => Ty::List,
            "map" | "dict" | "object" | "字典" | "映射" => Ty::Map,
            _ => Ty::Object(raw.trim().trim_matches('`').to_string()),
        }
    }

    /// 契约类型名（诊断消息显示用；与 `type` 内置的运行时名对照）。
    pub fn name(&self) -> String {
        match self {
            Ty::Any => "any".into(),
            Ty::Text => "text".into(),
            Ty::Number => "number".into(),
            Ty::Bool => "bool".into(),
            Ty::List => "list".into(),
            Ty::Map => "map".into(),
            Ty::Object(n) => n.clone(),
        }
    }

    /// 值是否满足契约类型（对象类型看 `_type` 实例标签）。
    pub fn matches_value(&self, v: &Value) -> bool {
        match self {
            Ty::Any => true,
            Ty::Text => matches!(v, Value::Text(_)),
            // 符号公式（$$…$$）是数值表达式，按 number 收。
            Ty::Number => matches!(v, Value::Int(_) | Value::Num(_) | Value::Formula(_)),
            Ty::Bool => matches!(v, Value::Bool(_)),
            Ty::List => matches!(v, Value::List(_)),
            Ty::Map => matches!(v, Value::Map(_)),
            Ty::Object(n) => instance_type_name(v).as_deref() == Some(n.as_str()),
        }
    }

    /// 列语义（几何即类型）：标量按本类型，或**元素全匹配**的列向量。
    pub fn matches_column(&self, v: &Value) -> bool {
        if self.matches_value(v) {
            return true;
        }
        match v {
            Value::List(items) => items.iter().all(|it| self.matches_value(it)),
            Value::Map(pairs) => pairs
                .iter()
                .all(|(_, it)| match it {
                    // map-of-lists 的列向量
                    Value::List(xs) => xs.iter().all(|x| self.matches_value(x)),
                    scalar => self.matches_value(scalar),
                })
                && !pairs.is_empty(),
            _ => false,
        }
    }
}

/// 运行时 `type` 名（诊断消息里的「实际」侧）。
pub fn type_of(v: &Value) -> &'static str {
    match v {
        Value::None => "none",
        Value::Bool(_) => "bool",
        Value::Int(_) => "int",
        Value::Num(_) => "num",
        Value::Text(_) => "text",
        Value::Secret(_) => "secret",
        Value::List(_) => "list",
        Value::Map(_) => "map",
        Value::Formula(_) => "formula",
        Value::Code(_) => "code",
    }
}

fn instance_type_name(v: &Value) -> Option<String> {
    let Value::Map(pairs) = v else { return None };
    pairs
        .iter()
        .find(|(k, _)| k == "_type")
        .and_then(|(_, v)| match v {
            Value::Text(t) => Some(t.clone()),
            _ => None,
        })
}

#[derive(Debug, Clone)]
pub struct ParamContract {
    pub name: String,
    pub ty: Ty,
    pub desc: String,
}

#[derive(Debug, Clone)]
pub struct ReturnContract {
    pub ty: Ty,
    pub desc: String,
}

#[derive(Debug, Clone)]
pub struct FieldContract {
    pub name: String,
    pub ty: Ty,
    pub nullable: bool,
    pub desc: String,
}

/// 一份契约（一个契约表块；同单元多表由 [`merge_contract`] 合并）。
#[derive(Debug, Clone)]
pub struct Contract {
    pub params: Vec<ParamContract>,
    pub returns: Option<ReturnContract>,
    pub fields: Vec<FieldContract>,
    pub start_line: u32,
    pub end_line: u32,
    /// 契约表原文行（诊断 `doc_quote`）。
    pub quote: Vec<String>,
}

impl Contract {
    /// 文档锚点（T2.4）：`file.mq.md#L4-L7`。
    pub fn anchor(&self, path: Option<&Path>) -> String {
        match path {
            Some(p) => format!("{}#L{}-L{}", p.display(), self.start_line, self.end_line),
            None => format!("#L{}-L{}", self.start_line, self.end_line),
        }
    }

    pub fn quote_text(&self) -> String {
        self.quote.join("\n")
    }

    /// 契约声明摘要（诊断 `contract_ref.declared`）。
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if !self.params.is_empty() {
            let list = self
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.ty.name()))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("参数[{list}]"));
        }
        if let Some(r) = &self.returns {
            parts.push(format!("返回 {}", r.ty.name()));
        }
        if !self.fields.is_empty() {
            let list = self
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name, f.ty.name()))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("字段[{list}]"));
        }
        parts.join("; ")
    }

    /// 装配带契约上下文的诊断（code + suggestion + doc_anchor + doc_quote + contract_ref）。
    pub fn diagnostic(
        &self,
        code: &str,
        unit: &str,
        message: String,
        suggestion: String,
        path: Option<&Path>,
        span: Span,
    ) -> Diagnostic {
        Diagnostic::new(path, span, message)
            .with_code(code)
            .with_severity(Severity::Error)
            .with_suggestion(suggestion)
            .with_doc_anchor(self.anchor(path))
            .with_doc_quote(self.quote_text())
            .with_contract_ref(ContractRef {
                unit: unit.to_string(),
                kind: "contract".to_string(),
                name: unit.to_string(),
                declared: self.describe(),
            })
    }
}

/// 把第二个契约表合并进第一个（多表块；重复行 → Err 防两份契约打架）。
pub fn merge_contract(dst: &mut Contract, src: Contract) -> std::result::Result<(), String> {
    for p in src.params {
        if dst.params.iter().any(|q| q.name == p.name) {
            return Err(format!("契约参数 `{}` 重复（防两份契约打架）", p.name));
        }
        dst.params.push(p);
    }
    if let Some(r) = src.returns {
        if dst.returns.is_some() {
            return Err("返回契约重复（防两份契约打架）".to_string());
        }
        dst.returns = Some(r);
    }
    for f in src.fields {
        if dst.fields.iter().any(|g| g.name == f.name) {
            return Err(format!("契约字段 `{}` 重复（防两份契约打架）", f.name));
        }
        dst.fields.push(f);
    }
    dst.start_line = dst.start_line.min(src.start_line);
    dst.end_line = dst.end_line.max(src.end_line);
    for q in src.quote {
        if !dst.quote.contains(&q) {
            dst.quote.push(q);
        }
    }
    Ok(())
}

// ---------- 契约表识别与解析 ----------

const PARAM_HEADERS: &[&str] = &["参数", "形参", "param", "params", "arg", "args"];
const RETURN_HEADERS: &[&str] = &["返回", "返回值", "return", "returns", "result"];
const FIELD_HEADERS: &[&str] = &["字段", "列", "键", "field", "fields", "key", "keys", "column", "columns"];
const TYPE_HEADERS: &[&str] = &["类型", "type"];
const DESC_HEADERS: &[&str] = &["说明", "描述", "备注", "注释", "desc", "description"];
const NULLABLE_HEADERS: &[&str] = &["可空", "nullable", "optional"];

fn norm_header(h: &str) -> String {
    h.trim().trim_matches('`').to_ascii_lowercase()
}

fn header_is(h: &str, vocab: &[&str]) -> bool {
    let n = norm_header(h);
    vocab.iter().any(|v| *v == n)
}

fn find_col(headers: &[String], vocab: &[&str]) -> Option<usize> {
    headers.iter().position(|h| header_is(h, vocab))
}

/// 单元格 → 文本（text / `` `ident` `` / 数字）；嵌套表 → None（那是数据表）。
fn cell_to_text(e: &Expr) -> Option<String> {
    match e {
        Expr::Literal(crate::ast::Literal::Text(s)) => Some(s.clone()),
        Expr::Var(name) => Some(name.clone()),
        Expr::Literal(crate::ast::Literal::Int(i)) => Some(i.to_string()),
        Expr::Literal(crate::ast::Literal::Num(n)) => Some(n.to_string()),
        Expr::Literal(crate::ast::Literal::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

/// 列值 → 行文本（标量 = 单行；列向量 = 多行；其它 = None）。
fn col_rows(e: &Expr) -> Option<Vec<String>> {
    match e {
        Expr::List(items) => items.iter().map(cell_to_text).collect(),
        scalar => Some(vec![cell_to_text(scalar)?]),
    }
}

fn is_nullable(raw: &str) -> bool {
    matches!(
        norm_header(raw).as_str(),
        "是" | "true" | "yes" | "y" | "可空" | "✓"
    )
}

/// 未绑定文档表（`Expr::Map`）→ 契约块。
///
/// - `None`：非契约形状（普通数据/说明表，原样保留）。
/// - `Some(Err)`：契约形状但畸形（缺参数名、列不齐、多行返回表…）——必产诊断。
/// - `Some(Ok)`：合法契约（`参数`/`返回`/`字段` 三形状之一，须含 `类型` 列）。
pub fn parse_contract_table(
    pairs: &[(String, Expr)],
    start_line: u32,
    end_line: u32,
    quote: Vec<String>,
) -> Option<Result<Contract>> {
    let headers: Vec<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
    let kind = if find_col(&headers, PARAM_HEADERS).is_some() {
        1
    } else if find_col(&headers, RETURN_HEADERS).is_some() {
        2
    } else if find_col(&headers, FIELD_HEADERS).is_some() {
        3
    } else {
        return None;
    };
    // 契约表必须有 `类型` 列（否则是普通说明表——如 `| 参数 | 说明 |`）。
    let Some(type_col) = find_col(&headers, TYPE_HEADERS) else {
        return None;
    };

    let meta = || (start_line, end_line, quote.clone());
    Some(match kind {
        1 => parse_params_table(pairs, &headers, type_col, meta()),
        2 => parse_return_table(pairs, &headers, type_col, meta()),
        _ => parse_fields_table(pairs, &headers, type_col, meta()),
    })
}

fn contract(
    meta: (u32, u32, Vec<String>),
) -> Contract {
    Contract {
        params: Vec::new(),
        returns: None,
        fields: Vec::new(),
        start_line: meta.0,
        end_line: meta.1,
        quote: meta.2,
    }
}

fn malformed(meta: (u32, u32, Vec<String>), span_line: u32, msg: String) -> Result<Contract> {
    let (start, end, quote) = meta;
    let c = Contract {
        params: Vec::new(),
        returns: None,
        fields: Vec::new(),
        start_line: start,
        end_line: end,
        quote,
    };
    Err(c
        .diagnostic(
            "contract.malformed",
            "(契约表)",
            msg,
            "按 `参数|类型|说明` / `返回|类型|说明` / `字段|类型|可空|说明` 形状修正契约表".into(),
            None,
            Span { line: span_line, col: 1 },
        )
        .into())
}

fn parse_params_table(
    pairs: &[(String, Expr)],
    headers: &[String],
    type_col: usize,
    meta: (u32, u32, Vec<String>),
) -> Result<Contract> {
    let start_line = meta.0;
    let name_col = find_col(headers, PARAM_HEADERS).unwrap();
    let desc_col = find_col(headers, DESC_HEADERS);
    let names = col_rows(&pairs[name_col].1).unwrap_or_default();
    let types = col_rows(&pairs[type_col].1).unwrap_or_default();
    let descs = desc_col
        .and_then(|c| col_rows(&pairs[c].1))
        .unwrap_or_default();
    if names.len() != types.len() || (!descs.is_empty() && descs.len() != names.len()) {
        return malformed(meta, start_line, "契约表列不齐（参数/类型/说明 行数不一致）".into());
    }
    let mut c = contract(meta.clone());
    for (i, name) in names.iter().enumerate() {
        let name = name.trim();
        if name.is_empty() {
            return malformed(meta, start_line, format!("契约表第 {} 行缺参数名", i + 1));
        }
        c.params.push(ParamContract {
            name: name.to_string(),
            ty: Ty::parse(&types[i]),
            desc: descs.get(i).cloned().unwrap_or_default(),
        });
    }
    Ok(c)
}

fn parse_return_table(
    pairs: &[(String, Expr)],
    headers: &[String],
    type_col: usize,
    meta: (u32, u32, Vec<String>),
) -> Result<Contract> {
    let start_line = meta.0;
    let desc_col = find_col(headers, DESC_HEADERS);
    let types = col_rows(&pairs[type_col].1).unwrap_or_default();
    let descs = desc_col
        .and_then(|c| col_rows(&pairs[c].1))
        .unwrap_or_default();
    if types.len() != 1 {
        return malformed(meta, start_line, "返回契约只能一行（`返回|类型|说明`）".into());
    }
    let mut c = contract(meta.clone());
    c.returns = Some(ReturnContract {
        ty: Ty::parse(&types[0]),
        desc: descs.first().cloned().unwrap_or_default(),
    });
    Ok(c)
}

fn parse_fields_table(
    pairs: &[(String, Expr)],
    headers: &[String],
    type_col: usize,
    meta: (u32, u32, Vec<String>),
) -> Result<Contract> {
    let start_line = meta.0;
    let name_col = find_col(headers, FIELD_HEADERS).unwrap();
    let desc_col = find_col(headers, DESC_HEADERS);
    let null_col = find_col(headers, NULLABLE_HEADERS);
    let names = col_rows(&pairs[name_col].1).unwrap_or_default();
    let types = col_rows(&pairs[type_col].1).unwrap_or_default();
    if names.len() != types.len() {
        return malformed(meta, start_line, "契约表列不齐（字段/类型 行数不一致）".into());
    }
    let mut c = contract(meta.clone());
    for (i, name) in names.iter().enumerate() {
        let name = name.trim();
        if name.is_empty() {
            return malformed(meta, start_line, format!("契约表第 {} 行缺字段名", i + 1));
        }
        c.fields.push(FieldContract {
            name: name.to_string(),
            ty: Ty::parse(&types[i]),
            nullable: null_col
                .and_then(|c| col_rows(&pairs[c].1))
                .and_then(|rows| rows.get(i).cloned())
                .map(|s| is_nullable(&s))
                .unwrap_or(false),
            desc: desc_col
                .and_then(|c| col_rows(&pairs[c].1))
                .and_then(|rows| rows.get(i).cloned())
                .unwrap_or_default(),
        });
    }
    Ok(c)
}

// ---------- 运行时边界校验（T2.2，四个边界） ----------

/// 边界 1：调用实参 vs 形参契约。
pub fn call_arg_diag(
    c: &Contract,
    unit: &str,
    bound: &HashMap<String, Value>,
    path: Option<&Path>,
    span: Span,
) -> Option<Diagnostic> {
    for p in &c.params {
        let Some(v) = bound.get(&p.name) else { continue };
        if v == &Value::None {
            continue; // 显式 None = 未填，交给缺参/默认值语义
        }
        if !p.ty.matches_value(v) {
            return Some(c.diagnostic(
                "contract.arg_mismatch",
                unit,
                format!(
                    "参数 `{}` 期望 {}，实际 {}（契约校验）",
                    p.name,
                    p.ty.name(),
                    type_of(v)
                ),
                format!(
                    "把实参改为 {}，或修正契约表 `参数/类型` 列（锚点 {}）",
                    p.ty.name(),
                    c.anchor(path)
                ),
                path,
                span,
            ));
        }
    }
    None
}

/// 边界 2：返回值 vs 返回契约。
pub fn return_diag(
    c: &Contract,
    unit: &str,
    v: &Value,
    path: Option<&Path>,
    span: Span,
) -> Option<Diagnostic> {
    let r = c.returns.as_ref()?;
    if !r.ty.matches_value(v) {
        return Some(c.diagnostic(
            "contract.return_mismatch",
            unit,
            format!(
                "返回值期望 {}，实际 {}（契约校验）",
                r.ty.name(),
                type_of(v)
            ),
            format!(
                "修正返回值，或调整契约表 `返回|类型`（锚点 {}）",
                c.anchor(path)
            ),
            path,
            span,
        ));
    }
    None
}

/// 边界 3：表绑定 vs 字段契约（集合契约；几何即类型——标量或列向量皆可）。
pub fn fields_diag(
    c: &Contract,
    unit: &str,
    v: &Value,
    path: Option<&Path>,
    span: Span,
) -> Option<Diagnostic> {
    if c.fields.is_empty() {
        return None;
    }
    let Value::Map(pairs) = v else {
        return Some(c.diagnostic(
            "contract.field_mismatch",
            unit,
            format!("绑定值期望表（map），实际 {}（契约校验）", type_of(v)),
            "绑定一个表值，或删除该字段契约表".into(),
            path,
            span,
        ));
    };
    for f in &c.fields {
        match pairs.iter().find(|(k, _)| k == &f.name) {
            Some((_, val)) => {
                if !f.ty.matches_column(val) {
                    return Some(c.diagnostic(
                        "contract.field_mismatch",
                        unit,
                        format!(
                            "字段 `{}` 期望 {}，实际 {}（契约校验）",
                            f.name,
                            f.ty.name(),
                            type_of(val)
                        ),
                        format!("修正数据，或调整契约表 `字段/类型`（锚点 {}）", c.anchor(path)),
                        path,
                        span,
                    ));
                }
            }
            None if f.nullable => {}
            None => {
                return Some(c.diagnostic(
                    "contract.field_mismatch",
                    unit,
                    format!("缺少字段 `{}`（契约要求，不可空）", f.name),
                    format!("补上字段 `{}`，或在契约表 `可空` 列标 `是`（锚点 {}）", f.name, c.anchor(path)),
                    path,
                    span,
                ));
            }
        }
    }
    None
}

/// 边界 4：字段访问 `[键](集合)` vs 字段契约（键存在性 + 取值类型）。
pub fn index_diag(
    c: &Contract,
    unit: &str,
    key: &str,
    v: &Value,
    path: Option<&Path>,
    span: Span,
) -> Option<Diagnostic> {
    let Some(f) = c.fields.iter().find(|f| f.name == key) else {
        return Some(c.diagnostic(
            "contract.unknown_key",
            unit,
            format!("字段 `{key}` 不在契约中（契约校验）"),
            format!(
                "使用契约内的字段（{}），或在契约表补一行",
                c.fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" / ")
            ),
            path,
            span,
        ));
    };
    if v != &Value::None && !f.ty.matches_column(v) {
        return Some(c.diagnostic(
            "contract.field_mismatch",
            unit,
            format!(
                "字段 `{key}` 期望 {}，实际 {}（契约校验）",
                f.ty.name(),
                type_of(v)
            ),
            format!("修正数据，或调整契约表 `字段/类型`（锚点 {}）", c.anchor(path)),
            path,
            span,
        ));
    }
    None
}

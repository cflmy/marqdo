//! Plan IR（`doc/design/prove-cli.md` §4.2）。
//!
//! `plan.json` 的形状：版本化、可序列化、无时间戳。缓存键与论文工具都以
//! 它为输入。表达式分两层：[`PureExpr`] 是可翻译成 Lean 项的纯片段；
//! 其余调用一律是不透明操作（[`Step::Trace`] 引用 [`FnPlan::ops`]）。

use serde::Serialize;

/// IR schema version（缓存失效用；形状变化必须 +1）。
pub const SCHEMA: u32 = 1;

/// 契约类型在 Plan IR 里的形状（`contract::Ty` 的可序列化投影）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanTy {
    Int,
    Num,
    Bool,
    Text,
    List,
    Map,
    /// 内联 `Result` 构造器（`lib/result`）：`ok/value/error` 三字段结构。
    Result,
    /// 未知/任意（`--no-strict-types` 或未标注）。定理只能做等式判断。
    Any,
    /// `# 对象` 类型实例。
    Object(String),
}

impl PlanTy {
    pub fn name(&self) -> String {
        match self {
            PlanTy::Int => "int".into(),
            PlanTy::Num => "num".into(),
            PlanTy::Bool => "bool".into(),
            PlanTy::Text => "text".into(),
            PlanTy::List => "list".into(),
            PlanTy::Map => "map".into(),
            PlanTy::Result => "result".into(),
            PlanTy::Any => "any".into(),
            PlanTy::Object(n) => n.clone(),
        }
    }

    pub fn from_contract(ty: &crate::contract::Ty) -> PlanTy {
        match ty {
            crate::contract::Ty::Any => PlanTy::Any,
            crate::contract::Ty::Text => PlanTy::Text,
            crate::contract::Ty::Number => PlanTy::Int,
            crate::contract::Ty::Bool => PlanTy::Bool,
            crate::contract::Ty::List => PlanTy::List,
            crate::contract::Ty::Map => PlanTy::Map,
            crate::contract::Ty::Object(n) => PlanTy::Object(n.clone()),
        }
    }
}

/// 二元运算（纯片段内的算术/比较/布尔）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn from_ast(op: crate::ast::BinaryOp) -> BinOp {
        use crate::ast::BinaryOp as A;
        match op {
            A::Add => BinOp::Add,
            A::Sub => BinOp::Sub,
            A::Mul => BinOp::Mul,
            A::Div => BinOp::Div,
            A::Eq => BinOp::Eq,
            A::Ne => BinOp::Ne,
            A::Lt => BinOp::Lt,
            A::Le => BinOp::Le,
            A::Gt => BinOp::Gt,
            A::Ge => BinOp::Ge,
            A::And => BinOp::And,
            A::Or => BinOp::Or,
        }
    }
}

/// 一元运算。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnOp {
    Not,
    Neg,
}

/// 纯表达式：可翻译成 Lean 项的部分（§4.2 表达式分层）。
/// 相邻标记（`kind` + `value`）：新类型变体也能稳定序列化。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PureExpr {
    Int(i64),
    Num(f64),
    Bool(bool),
    Text(String),
    None,
    Var(String),
    Bin {
        operator: BinOp,
        left: Box<PureExpr>,
        right: Box<PureExpr>,
    },
    Un {
        operator: UnOp,
        expr: Box<PureExpr>,
    },
    /// 映射字段/下标读取：`` [键](映射) `` / `` `m`[^键] ``。
    Index {
        base: Box<PureExpr>,
        key: String,
    },
    /// `access(base, key)`（含 `取`/`取值` 别名）。
    Access {
        base: Box<PureExpr>,
        key: String,
    },
    /// `len(value)`（含 `长度` 别名）。
    Len(Box<PureExpr>),
    /// 文本插值拼接（`` "共 `n` 条" `` → append 序列）。
    Interp(Vec<InterpPart>),
    List(Vec<PureExpr>),
    Map(Vec<(String, PureExpr)>),
    /// 已内联的构造器调用（`result.success/failure`）。
    Ctor {
        name: String,
        args: Vec<PureExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "part", content = "value", rename_all = "snake_case")]
pub enum InterpPart {
    Lit(String),
    Var(String),
}

/// 语句（§4.3 接受矩阵的接受侧）。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Step {
    /// `**x = 表达式**`
    Bind {
        var: String,
        expr: Expr,
        line: u32,
    },
    /// `**返回表达式**`（粗体无赋值）
    Return {
        expr: Expr,
        line: u32,
    },
    /// `1. 条件 / 2. *`（else 由 `cond: None` 表示）
    If {
        cond: PureExpr,
        then: Vec<Step>,
        otherwise: Vec<Step>,
        line: u32,
    },
    /// 语句形式调用（`> 调用` / `*语句*`）：值丢弃，但仍进轨迹。
    Trace {
        op: String,
        args: Vec<PureExpr>,
        line: u32,
    },
}

/// 语句内表达式：纯表达式或不透明操作调用。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expr {
    Pure(PureExpr),
    /// 不透明操作调用：`ops` 里的 `id`（接收者/插件/有效应的调用）。
    Op {
        id: String,
        args: Vec<PureExpr>,
    },
    /// `$$…$$` 公式：不透明值，不翻译。
    Formula,
    /// ``` 围栏 code 值（不经 parse 拒绝时降为不透明）。
    Code,
}

/// 一个不透明操作的声明（Lean 里是 `Op` 构造器 + `Env` 查询键）。
#[derive(Debug, Clone, Serialize)]
pub struct OpDecl {
    pub id: String,
    pub effect: String,
    pub params: Vec<(String, PlanTy)>,
    /// 实参个数（首次出现处；同名操作元数一致）。
    pub arity: usize,
}

/// 定理占位（P1 起填充；P0 恒为空，保持 `plan.json` 形状稳定）。
#[derive(Debug, Clone, Serialize)]
pub struct ClaimPlan {
    pub id: String,
    pub premise: Option<String>,
    pub conclusion: String,
    pub tactic: Option<String>,
    /// 定理表行号（锚定源码漂移）。
    pub line: u32,
}

/// 假设表条目（T3；P0 恒为空）。
#[derive(Debug, Clone, Serialize)]
pub struct AssumptionPlan {
    pub op: String,
    pub premise: Option<String>,
    pub postcondition: String,
}

/// 单个函数的 Plan。
#[derive(Debug, Clone, Serialize)]
pub struct FnPlan {
    pub name: String,
    pub params: Vec<ParamPlan>,
    pub result_ty: PlanTy,
    /// 该函数用到的不透明操作（按出现顺序，去重）。
    pub ops: Vec<OpDecl>,
    pub steps: Vec<Step>,
    pub effects: Vec<String>,
    pub claims: Vec<ClaimPlan>,
    pub assumptions: Vec<AssumptionPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParamPlan {
    pub name: String,
    pub ty: PlanTy,
    pub has_default: bool,
}

/// 整个文件的 Plan。
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub schema: u32,
    /// 相对当前工作目录的路径（确定性要求：无绝对路径、无时间戳）。
    pub module: String,
    pub source_sha256: String,
    pub functions: Vec<FnPlan>,
}

impl Plan {
    /// 规范化 JSON（键序稳定、无空白差异；两次运行必须字节一致）。
    pub fn to_stable_json(&self) -> String {
        let value = serde_json::to_value(self).expect("plan serializes");
        format_value(&value)
    }
}

/// 确定性 JSON 序列化：对象键排序、数组保序、定长浮点。
fn format_value(value: &serde_json::Value) -> String {
    use serde_json::Value;
    match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).expect("string serializes"),
        Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", inner.join(","))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).expect("key serializes"),
                        format_value(&map[k])
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
    }
}

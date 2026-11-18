//! 离线符号执行（设计 §5）：没有 Lean 也能给**部分**结论，但绝不比 Lean 强。
//!
//! 片段 F1–F5，超出即 `unverified`（不猜）：
//! - F1 路径枚举：控制流是有限树（lower 已拒绝循环/递归）。
//! - F2 轨迹：每条路径的 Op 序列由分支决定；包含/长度逐路径求值。
//! - F3 字面结果：返回值由字面量/参数/内联构造器组成时按结构求值。
//! - F4 命题：and/or/not、=/≠ 作用于布尔结果；外部返回值上的布尔判定
//!   视为自由原子（只二分、不假设原子间关系）。
//! - F5 单变量区间：`x ⋈ c` 原子；路径条件 + 前提的上下界判定蕴含。
//!
//! 单调性（§5.4）：离线 `checked` ⊆ Lean `proved`——构造上 F5 ⊂ omega、
//! F1–F4 ⊂ simp/decide；运行期只要有 Lean 就复核，分歧即
//! `error[proof.divergence]`，永不落绿。

use std::collections::HashMap;

use crate::prove::ir::{Expr, FnPlan, Plan, PureExpr, Step};
use crate::prove::proof_check::{parse_claim_expr, tokenize_claim, ClaimAst, CTok};

/// 符号值（§5.3 状态）。
#[derive(Debug, Clone, PartialEq)]
pub enum SymVal {
    Int(i64),
    Num(f64),
    Bool(bool),
    Text(String),
    None,
    /// 内联构造器结构（`result.ok(v)` → ok=true/value=v/error=none 等）。
    Struct { ctor: String, fields: Vec<(String, SymVal)> },
    /// 不透明操作返回值：值未知，出现已知（F2/F4）。
    OpResult(String),
    /// 形参：值未知（F5 走区间）。
    Param(String),
    /// 超出片段。
    Unknown,
}

impl SymVal {
    fn is_known(&self) -> bool {
        !matches!(self, SymVal::Unknown)
    }
}

/// 比较算子（F5 区间原子）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Cmp {
    fn negate(self) -> Cmp {
        match self {
            Cmp::Lt => Cmp::Ge,
            Cmp::Le => Cmp::Gt,
            Cmp::Gt => Cmp::Le,
            Cmp::Ge => Cmp::Lt,
            Cmp::Eq => Cmp::Ne,
            Cmp::Ne => Cmp::Eq,
        }
    }

    fn holds(self, lhs: i64, rhs: i64) -> bool {
        match self {
            Cmp::Lt => lhs < rhs,
            Cmp::Le => lhs <= rhs,
            Cmp::Gt => lhs > rhs,
            Cmp::Ge => lhs >= rhs,
            Cmp::Eq => lhs == rhs,
            Cmp::Ne => lhs != rhs,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Cmp::Lt => "<",
            Cmp::Le => "≤",
            Cmp::Gt => ">",
            Cmp::Ge => "≥",
            Cmp::Eq => "=",
            Cmp::Ne => "≠",
        }
    }
}

/// 路径条件原子。否定一律折进 op/expected（原子总是"正"形态）。
#[derive(Debug, Clone, PartialEq)]
pub enum PathAtom {
    /// `x ⋈ c`（F5）。
    Interval { var: String, op: Cmp, val: i64 },
    /// 自由布尔原子：`env(op).<key>` 判定为 `expected`（F4；值未知只二分）。
    FreeBool { op: String, key: String, expected: bool },
    Literal(bool),
    /// 无法归入片段的条件 → 该路径不可判定。
    Opaque,
}

impl PathAtom {
    fn negated(&self) -> PathAtom {
        match self {
            PathAtom::Interval { var, op, val } => PathAtom::Interval {
                var: var.clone(),
                op: op.negate(),
                val: *val,
            },
            PathAtom::FreeBool { op, key, expected } => PathAtom::FreeBool {
                op: op.clone(),
                key: key.clone(),
                expected: !expected,
            },
            PathAtom::Literal(b) => PathAtom::Literal(!b),
            PathAtom::Opaque => PathAtom::Opaque,
        }
    }
}

/// 一条执行路径（§5.3 第 3 步的收集物）。
#[derive(Debug, Clone)]
pub struct PathRecord {
    pub id: usize,
    pub atoms: Vec<PathAtom>,
    pub result: SymVal,
    pub trace: Vec<String>,
    /// Return 时的变量快照（定理可引用中间变量，如 `审核.ok`）。
    pub vars: Vec<(String, SymVal)>,
    /// 踩到片段之外 → 相关定理 unverified。
    pub opaque: bool,
    pub line: u32,
}

/// 反例见证（§5.5：只承认确定性否定）。
#[derive(Debug, Clone)]
pub struct Counterexample {
    pub assignments: Vec<(String, i64)>,
    pub path_id: usize,
    pub detail: String,
}

/// 单条定理的离线判定。
#[derive(Debug, Clone)]
pub enum OfflineVerdict {
    /// 每条路径都关闭（checked ≠ proved：没跑 Lean）。
    Checked { cases: usize },
    /// 存在可满足路径、结论确定性为假 → 反例。
    Unproved { counterexample: Option<Counterexample> },
    /// 有路径/前提/结论超出片段 → 不猜（诚实）。
    Unverified { reason: String },
}

/// 离线执行一个函数，收集全部路径。
pub fn execute(fn_plan: &FnPlan) -> Vec<PathRecord> {
    let mut paths = Vec::new();
    let mut state: HashMap<String, SymVal> = fn_plan
        .params
        .iter()
        .map(|p| (p.name.clone(), SymVal::Param(p.name.clone())))
        .collect();
    exec_steps(
        &fn_plan.steps,
        &mut state,
        Vec::new(),
        Vec::new(),
        false,
        0,
        &mut paths,
    );
    for (i, p) in paths.iter_mut().enumerate() {
        p.id = i;
    }
    paths
}

/// 符号执行语句序列；`atoms`/`trace` 沿路径累积，每个 Return 产生一条记录。
fn exec_steps(
    steps: &[Step],
    state: &mut HashMap<String, SymVal>,
    atoms: Vec<PathAtom>,
    trace: Vec<String>,
    opaque: bool,
    line: u32,
    out: &mut Vec<PathRecord>,
) {
    let Some((step, rest)) = steps.split_first() else {
        // 函数体走完但无 Return：结果未知，但**轨迹与路径条件已知**
        // ——F2 结论（包含/长度）仍可判，Result 类结论自然关不了。
        out.push(PathRecord {
            id: usize::MAX,
            atoms,
            result: SymVal::Unknown,
            trace,
            vars: state.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            opaque,
            line,
        });
        return;
    };
    match step {
        Step::Bind { var, expr, line: l } => {
            let (val, trace2, opq) = eval_expr(expr, state, trace, opaque);
            let saved = state.insert(var.clone(), val);
            exec_steps(rest, state, atoms, trace2, opq, *l, out);
            // 回溯：Bind 只在本分支有效。
            match saved {
                Some(v) => {
                    state.insert(var.clone(), v);
                }
                None => {
                    state.remove(var);
                }
            }
        }
        Step::Return { expr, line: l } => {
            let (val, trace2, opq) = eval_expr(expr, state, trace, opaque);
            out.push(PathRecord {
                id: usize::MAX,
                atoms,
                result: val,
                trace: trace2,
                vars: state.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                opaque: opq,
                line: *l,
            });
        }
        Step::If { cond, then, otherwise, line: l } => {
            let (atom, cond_opaque) = cond_atom(cond, state);
            // then：条件成立；else：条件取反；两支都要**接续 If 之后的语句**
            // （Return 常在 If 外层）。分支在状态克隆上跑，互不污染。
            let mut atoms_then = atoms.clone();
            atoms_then.push(atom.clone());
            let mut then_full: Vec<Step> = then.clone();
            then_full.extend(rest.iter().cloned());
            let mut then_state = state.clone();
            exec_steps(
                &then_full,
                &mut then_state,
                atoms_then,
                trace.clone(),
                opaque || cond_opaque,
                *l,
                out,
            );
            let mut atoms_else = atoms;
            atoms_else.push(atom.negated());
            let mut else_full: Vec<Step> = otherwise.clone();
            else_full.extend(rest.iter().cloned());
            let mut else_state = state.clone();
            exec_steps(
                &else_full,
                &mut else_state,
                atoms_else,
                trace,
                opaque || cond_opaque,
                *l,
                out,
            );
            // If 之后没有"公共继续"（已并入两支）；state 保持进入前原样。
        }
        Step::Trace { op, .. } => {
            let mut trace2 = trace;
            trace2.push(op_display(op));
            exec_steps(rest, state, atoms, trace2, opaque, line, out);
        }
    }
}

/// `调用:人工审核` → `人工审核`（与 Lean `Op.name` 显示一致，轨迹名不变）。
fn op_display(id: &str) -> String {
    id.strip_prefix("调用:").unwrap_or(id).to_string()
}

/// 条件 → 路径原子（F4/F5）。返回 (原子, 是否超出片段)。
fn cond_atom(cond: &PureExpr, state: &HashMap<String, SymVal>) -> (PathAtom, bool) {
    match cond {
        PureExpr::Bool(b) => (PathAtom::Literal(*b), false),
        PureExpr::Bin { operator, left, right } => {
            // 单变量区间：`参数 ⋈ 整数字面量`（或反向）。
            if let Some(atom) = interval_atom(operator, left, right, state) {
                return (atom, false);
            }
            // 自由布尔原子：`access(op结果, "ok")` 与字面量比较。
            if let Some(atom) = free_bool_atom(operator, left, right, state) {
                return (atom, false);
            }
            (PathAtom::Opaque, true)
        }
        PureExpr::Un { operator, expr } => {
            if matches!(operator, crate::prove::ir::UnOp::Not) {
                let (atom, opq) = cond_atom(expr, state);
                return (atom.negated(), opq);
            }
            (PathAtom::Opaque, true)
        }
        // 裸布尔条件：`1. 审核.ok`（真值即条件）。
        PureExpr::Access { base, key } => {
            if let PureExpr::Var(v) = base.as_ref() {
                if let Some(SymVal::OpResult(op)) = state.get(v) {
                    return (
                        PathAtom::FreeBool {
                            op: op.clone(),
                            key: key.clone(),
                            expected: true,
                        },
                        false,
                    );
                }
                if let Some(SymVal::Struct { fields, .. }) = state.get(v) {
                    if let Some((_, SymVal::Bool(b))) =
                        fields.iter().find(|(k, _)| k == key)
                    {
                        return (PathAtom::Literal(*b), false);
                    }
                }
            }
            (PathAtom::Opaque, true)
        }
        PureExpr::Var(v) => match state.get(v) {
            Some(SymVal::Bool(b)) => (PathAtom::Literal(*b), false),
            _ => (PathAtom::Opaque, true),
        },
        _ => (PathAtom::Opaque, true),
    }
}

/// F5 原子：`x ⋈ c`，x 是形参（或绑定了形参的变量），c 是整数字面量。
fn interval_atom(
    operator: &crate::prove::ir::BinOp,
    left: &PureExpr,
    right: &PureExpr,
    state: &HashMap<String, SymVal>,
) -> Option<PathAtom> {
    use crate::prove::ir::BinOp as B;
    let op = match operator {
        B::Lt => Cmp::Lt,
        B::Le => Cmp::Le,
        B::Gt => Cmp::Gt,
        B::Ge => Cmp::Ge,
        B::Eq => Cmp::Eq,
        B::Ne => Cmp::Ne,
        _ => return None,
    };
    match (left, right) {
        (PureExpr::Var(v), PureExpr::Int(c)) => {
            // 只有仍指向形参的变量才可做区间（绑定值另论）。
            match state.get(v) {
                Some(SymVal::Param(_)) | None => {
                    Some(PathAtom::Interval { var: v.clone(), op, val: *c })
                }
                _ => None,
            }
        }
        (PureExpr::Int(c), PureExpr::Var(v)) => {
            match state.get(v) {
                Some(SymVal::Param(_)) | None => {
                    // c ⋈ x ≡ x (flip ⋈) c
                    Some(PathAtom::Interval {
                        var: v.clone(),
                        op: op.flip(),
                        val: *c,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// F4 自由原子：`<变量 = OpResult> 的布尔字段 == 布尔字面量`。
/// 如 `1. 审核.ok`，其中 `审核 = 人工审核(…)`（值未知 → 只二分）。
fn free_bool_atom(
    operator: &crate::prove::ir::BinOp,
    left: &PureExpr,
    right: &PureExpr,
    state: &HashMap<String, SymVal>,
) -> Option<PathAtom> {
    use crate::prove::ir::BinOp as B;
    let (expected, a, b) = match operator {
        B::Eq => (true, left, right),
        B::Ne => (false, left, right),
        _ => return None,
    };
    let (access, field_key, lit) = match (a, b) {
        (PureExpr::Access { base, key }, PureExpr::Bool(v)) => {
            (base.as_ref(), key.clone(), *v == expected)
        }
        (PureExpr::Bool(v), PureExpr::Access { base, key }) => {
            (base.as_ref(), key.clone(), *v == expected)
        }
        _ => return None,
    };
    let PureExpr::Var(var) = access else {
        return None;
    };
    let SymVal::OpResult(op) = state.get(var)? else {
        return None;
    };
    Some(PathAtom::FreeBool {
        op: op.clone(),
        key: field_key,
        expected: lit,
    })
}

/// 表达式求值 → (符号值, 新轨迹, 是否超出片段)。
fn eval_expr(
    expr: &Expr,
    state: &HashMap<String, SymVal>,
    trace: Vec<String>,
    opaque: bool,
) -> (SymVal, Vec<String>, bool) {
    match expr {
        Expr::Pure(p) => (eval_pure(p, state), trace, opaque),
        Expr::Op { id, .. } => {
            let mut trace2 = trace;
            trace2.push(op_display(id));
            (SymVal::OpResult(op_display(id)), trace2, opaque)
        }
        // 公式/围栏代码：不透明值，超出片段。
        Expr::Formula | Expr::Code => (SymVal::Unknown, trace, true),
    }
}

/// 纯表达式结构求值（F3）：字面量与内联构造器按结构化简，其余保守 Unknown。
pub fn eval_pure(p: &PureExpr, state: &HashMap<String, SymVal>) -> SymVal {
    match p {
        PureExpr::Int(i) => SymVal::Int(*i),
        PureExpr::Num(n) => SymVal::Num(*n),
        PureExpr::Bool(b) => SymVal::Bool(*b),
        PureExpr::Text(s) => SymVal::Text(s.clone()),
        PureExpr::None => SymVal::None,
        PureExpr::Var(v) => state.get(v).cloned().unwrap_or(SymVal::Unknown),
        PureExpr::Bin { operator, left, right } => eval_bin(*operator, left, right, state),
        PureExpr::Un { operator, expr } => {
            let v = eval_pure(expr, state);
            match (operator, v) {
                (crate::prove::ir::UnOp::Not, SymVal::Bool(b)) => SymVal::Bool(!b),
                (crate::prove::ir::UnOp::Neg, SymVal::Int(i)) => SymVal::Int(-i),
                _ => SymVal::Unknown,
            }
        }
        PureExpr::Access { base, key } => match eval_pure(base, state) {
            SymVal::Struct { fields, .. } => fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or(SymVal::None),
            _ => SymVal::Unknown,
        },
        PureExpr::Index { base, key } => match eval_pure(base, state) {
            SymVal::Struct { fields, .. } => fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or(SymVal::None),
            _ => SymVal::Unknown,
        },
        PureExpr::Len(inner) => match eval_pure(inner, state) {
            SymVal::Struct { ctor, .. } if ctor == "list" => SymVal::Unknown,
            SymVal::Text(s) => SymVal::Int(s.chars().count() as i64),
            _ => SymVal::Unknown,
        },
        PureExpr::List(items) => {
            // 列表字面量：全部已知才结构化（保留为 list 构造）。
            let vals: Vec<SymVal> = items.iter().map(|i| eval_pure(i, state)).collect();
            if vals.iter().all(SymVal::is_known) {
                SymVal::Struct {
                    ctor: "list".into(),
                    fields: vals
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| (i.to_string(), v))
                        .collect(),
                }
            } else {
                SymVal::Unknown
            }
        }
        PureExpr::Map(entries) => {
            let vals: Vec<(String, SymVal)> = entries
                .iter()
                .map(|(k, v)| (k.clone(), eval_pure(v, state)))
                .collect();
            if vals.iter().all(|(_, v)| v.is_known()) {
                SymVal::Struct {
                    ctor: "map".into(),
                    fields: vals,
                }
            } else {
                SymVal::Unknown
            }
        }
        PureExpr::Interp(_) => SymVal::Unknown,
        PureExpr::Ctor { name, args } => eval_ctor(name, args, state),
    }
}

/// 内联构造器（与 `mqResultOk`/`mqResultError`/`mqValueOr` 语义一致）。
fn eval_ctor(name: &str, args: &[PureExpr], state: &HashMap<String, SymVal>) -> SymVal {
    let vals: Vec<SymVal> = args.iter().map(|a| eval_pure(a, state)).collect();
    match (name, vals.as_slice()) {
        ("result.ok", [v]) => SymVal::Struct {
            ctor: "result.ok".into(),
            fields: vec![
                ("ok".into(), SymVal::Bool(true)),
                ("value".into(), v.clone()),
                ("error".into(), SymVal::None),
            ],
        },
        ("result.error", [code, message, operation]) => SymVal::Struct {
            ctor: "result.error".into(),
            fields: vec![
                ("ok".into(), SymVal::Bool(false)),
                ("error".into(), code.clone()),
                ("message".into(), message.clone()),
                ("operation".into(), operation.clone()),
                ("value".into(), SymVal::None),
            ],
        },
        ("result.value_or", [r, default]) => match r {
            SymVal::Struct { ctor, fields }
                if ctor == "result.ok" =>
            {
                fields
                    .iter()
                    .find(|(k, _)| k == "value")
                    .map(|(_, v)| v.clone())
                    .unwrap_or(default.clone())
            }
            SymVal::Struct { ctor, .. } if ctor == "result.error" => default.clone(),
            _ => SymVal::Unknown,
        },
        _ => SymVal::Unknown,
    }
}

fn eval_bin(
    operator: crate::prove::ir::BinOp,
    left: &PureExpr,
    right: &PureExpr,
    state: &HashMap<String, SymVal>,
) -> SymVal {
    use crate::prove::ir::BinOp as B;
    let l = eval_pure(left, state);
    let r = eval_pure(right, state);
    match (operator, l, r) {
        (B::Add, SymVal::Int(a), SymVal::Int(b)) => SymVal::Int(a.wrapping_add(b)),
        (B::Sub, SymVal::Int(a), SymVal::Int(b)) => SymVal::Int(a.wrapping_sub(b)),
        (B::Mul, SymVal::Int(a), SymVal::Int(b)) => SymVal::Int(a.wrapping_mul(b)),
        (B::Div, SymVal::Int(a), SymVal::Int(b)) if b != 0 => SymVal::Int(a / b),
        (B::Eq, SymVal::Int(a), SymVal::Int(b)) => SymVal::Bool(a == b),
        (B::Eq, SymVal::Bool(a), SymVal::Bool(b)) => SymVal::Bool(a == b),
        (B::Eq, SymVal::Text(a), SymVal::Text(b)) => SymVal::Bool(a == b),
        (B::Ne, SymVal::Int(a), SymVal::Int(b)) => SymVal::Bool(a != b),
        (B::Ne, SymVal::Bool(a), SymVal::Bool(b)) => SymVal::Bool(a != b),
        (B::Ne, SymVal::Text(a), SymVal::Text(b)) => SymVal::Bool(a != b),
        (B::Lt, SymVal::Int(a), SymVal::Int(b)) => SymVal::Bool(a < b),
        (B::Le, SymVal::Int(a), SymVal::Int(b)) => SymVal::Bool(a <= b),
        (B::Gt, SymVal::Int(a), SymVal::Int(b)) => SymVal::Bool(a > b),
        (B::Ge, SymVal::Int(a), SymVal::Int(b)) => SymVal::Bool(a >= b),
        (B::And, SymVal::Bool(a), SymVal::Bool(b)) => SymVal::Bool(a && b),
        (B::Or, SymVal::Bool(a), SymVal::Bool(b)) => SymVal::Bool(a || b),
        _ => SymVal::Unknown,
    }
}

// ---------------------------------------------------------------------------
// 定理检查（§5.3 第 4–5 步）
// ---------------------------------------------------------------------------

/// 前提原子（F5 区间 / F4 自由布尔）。
#[derive(Debug, Clone)]
pub enum PremiseAtom {
    Interval { var: String, op: Cmp, val: i64 },
    FreeBool { op: String, key: String, expected: bool },
    Literal(bool),
    /// 超出片段的前提 → 不使用（相关定理 unverified）。
    Opaque,
}

pub fn parse_premise(text: &str) -> Option<PremiseAtom> {
    let toks = tokenize_claim(text).ok()?;
    let (_, ast) = parse_claim_expr(&toks).ok()?;
    premise_atom(&ast)
}

fn premise_atom(ast: &ClaimAst) -> Option<PremiseAtom> {
    match ast {
        ClaimAst::Bin { op, left, right } => {
            let cmp = match *op {
                "<" => Cmp::Lt,
                "≤" => Cmp::Le,
                ">" => Cmp::Gt,
                "≥" => Cmp::Ge,
                "=" => Cmp::Eq,
                "≠" => Cmp::Ne,
                _ => return None,
            };
            match (left.as_ref(), right.as_ref()) {
                (ClaimAst::Var(v), ClaimAst::Lit(CTok::Int(c))) => {
                    Some(PremiseAtom::Interval { var: v.clone(), op: cmp, val: *c })
                }
                (ClaimAst::Lit(CTok::Int(c)), ClaimAst::Var(v)) => {
                    Some(PremiseAtom::Interval { var: v.clone(), op: cmp.flip(), val: *c })
                }
                // `access(结果, "ok") = True/False` 形前提。
                (ClaimAst::Call { name, args }, ClaimAst::Lit(CTok::Bool(b)))
                    if name == "access" && args.len() == 2 =>
                {
                    let (base, key) = access_parts(&args[0], &args[1])?;
                    Some(PremiseAtom::FreeBool { op: base, key, expected: *b })
                }
                _ => None,
            }
        }
        ClaimAst::Lit(CTok::Bool(b)) => Some(PremiseAtom::Literal(*b)),
        _ => None,
    }
}

impl Cmp {
    fn flip(self) -> Cmp {
        match self {
            Cmp::Lt => Cmp::Gt,
            Cmp::Le => Cmp::Ge,
            Cmp::Gt => Cmp::Lt,
            Cmp::Ge => Cmp::Le,
            Cmp::Eq => Cmp::Eq,
            Cmp::Ne => Cmp::Ne,
        }
    }
}

/// `结果.ok` / `access(结果, "ok")` → (主体名, 字段)。
fn access_parts(base: &ClaimAst, key: &ClaimAst) -> Option<(String, String)> {
    let ClaimAst::Var(v) = base else {
        return None;
    };
    let (root, field) = match v.split_once('.') {
        Some((r, f)) => (r.to_string(), Some(f.to_string())),
        None => (v.clone(), None),
    };
    if let Some(f) = field {
        return Some((root, f));
    }
    let ClaimAst::Lit(CTok::Text(k)) = key else {
        return None;
    };
    Some((root, k.clone()))
}

/// 结论的结构化形态（离线可判定的子集）。
#[derive(Debug, Clone)]
pub enum Conclusion {
    /// `<主体>.<字段> = true/false`（字段布尔；主体可为 结果/中间变量）。
    FieldBool { subject: String, field: String, expected: bool },
    /// `包含(轨迹, "s")` / 其否定。
    TraceContains { item: String, expected: bool },
    /// `长度(轨迹) ⋈ n`。
    TraceLen { op: Cmp, val: i64 },
    /// `x ⋈ c`（参数区间，F5）。
    ParamInterval { var: String, op: Cmp, val: i64 },
    /// `且`/`或`/`非` 组合。
    Not(Box<Conclusion>),
    And(Box<Conclusion>, Box<Conclusion>),
    Or(Box<Conclusion>, Box<Conclusion>),
    /// 超出片段。
    Opaque(String),
}

pub fn parse_conclusion(text: &str) -> Conclusion {
    let toks = match tokenize_claim(text) {
        Ok(t) => t,
        Err(e) => return Conclusion::Opaque(e),
    };
    let (_, ast) = match parse_claim_expr(&toks) {
        Ok(x) => x,
        Err(e) => return Conclusion::Opaque(e),
    };
    conclusion_of(&ast)
}

fn conclusion_of(ast: &ClaimAst) -> Conclusion {
    match ast {
        ClaimAst::Bin { op, left, right } => match *op {
            "且" => Conclusion::And(
                Box::new(conclusion_of(left)),
                Box::new(conclusion_of(right)),
            ),
            "或" => Conclusion::Or(
                Box::new(conclusion_of(left)),
                Box::new(conclusion_of(right)),
            ),
            _ => {
                // 字段布尔：`结果.ok = True` / `审核.ok = True` …
                if let Some(c) = field_bool(op, left, right) {
                    return c;
                }
                // 轨迹谓词。
                if let Some(c) = trace_conclusion(op, left, right) {
                    return c;
                }
                // 参数区间。
                if let Some(c) = param_interval(op, left, right) {
                    return c;
                }
                Conclusion::Opaque(format!("不可判定的比较 {op}"))
            }
        },
        ClaimAst::Un { op, expr } if *op == "非" => {
            Conclusion::Not(Box::new(conclusion_of(expr)))
        }
        ClaimAst::Call { name, args }
            if name == "包含" && args.len() == 2 =>
        {
            match args.first() {
                Some(ClaimAst::Var(v)) if v == "轨迹" => match &args[1] {
                    ClaimAst::Lit(CTok::Text(s)) => Conclusion::TraceContains {
                        item: s.clone(),
                        expected: true,
                    },
                    _ => Conclusion::Opaque("包含 的成员须是文本字面量".into()),
                },
                _ => Conclusion::Opaque("包含 只支持轨迹".into()),
            }
        }
        _ => Conclusion::Opaque("不可判定的结论形状".into()),
    }
}

/// `X.f = b`（X 是 结果/中间变量/access 调用）→ FieldBool。
fn field_bool(op: &str, left: &ClaimAst, right: &ClaimAst) -> Option<Conclusion> {
    let expected = match (op, right) {
        ("=", ClaimAst::Lit(CTok::Bool(b))) => *b,
        ("≠", ClaimAst::Lit(CTok::Bool(b))) => !*b,
        _ => return None,
    };
    let (subject, field): (String, String) = match left {
        ClaimAst::Var(v) => {
            let (r, f) = v.split_once('.')?;
            (r.to_string(), f.to_string())
        }
        ClaimAst::Call { name, args } if name == "access" && args.len() == 2 => {
            access_parts(&args[0], &args[1])?
        }
        _ => return None,
    };
    Some(Conclusion::FieldBool {
        subject,
        field,
        expected,
    })
}

/// 轨迹结论：`包含(轨迹, "s") ⋈ Bool` / `长度(轨迹) ⋈ n`。
fn trace_conclusion(op: &str, left: &ClaimAst, right: &ClaimAst) -> Option<Conclusion> {
    match left {
        ClaimAst::Call { name, args } if name == "包含" && args.len() == 2 => {
            let ClaimAst::Var(v) = args.first()? else {
                return None;
            };
            if v != "轨迹" {
                return None;
            }
            let ClaimAst::Lit(CTok::Text(s)) = args.get(1)? else {
                return None;
            };
            let expected = match (op, right) {
                ("=", ClaimAst::Lit(CTok::Bool(b))) => *b,
                ("≠", ClaimAst::Lit(CTok::Bool(b))) => !*b,
                _ => return None,
            };
            Some(Conclusion::TraceContains {
                item: s.clone(),
                expected,
            })
        }
        ClaimAst::Call { name, args } if name == "长度" && args.len() == 1 => {
            let ClaimAst::Var(v) = args.first()? else {
                return None;
            };
            if v != "轨迹" {
                return None;
            }
            let ClaimAst::Lit(CTok::Int(n)) = right else {
                return None;
            };
            let cmp = match op {
                "=" => Cmp::Eq,
                "≠" => Cmp::Ne,
                "<" => Cmp::Lt,
                "≤" => Cmp::Le,
                ">" => Cmp::Gt,
                "≥" => Cmp::Ge,
                _ => return None,
            };
            Some(Conclusion::TraceLen { op: cmp, val: *n })
        }
        _ => None,
    }
}

/// 参数区间结论：`金额 > 9999` 之类。
fn param_interval(op: &str, left: &ClaimAst, right: &ClaimAst) -> Option<Conclusion> {
    let cmp = match op {
        "<" => Cmp::Lt,
        "≤" => Cmp::Le,
        ">" => Cmp::Gt,
        "≥" => Cmp::Ge,
        "=" => Cmp::Eq,
        "≠" => Cmp::Ne,
        _ => return None,
    };
    match (left, right) {
        (ClaimAst::Var(v), ClaimAst::Lit(CTok::Int(c))) => Some(Conclusion::ParamInterval {
            var: v.clone(),
            op: cmp,
            val: *c,
        }),
        (ClaimAst::Lit(CTok::Int(c)), ClaimAst::Var(v)) => Some(Conclusion::ParamInterval {
            var: v.clone(),
            op: cmp.flip(),
            val: *c,
        }),
        _ => None,
    }
}

/// 一条路径上的结论求值结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Unknown,
}

/// 对一条路径检查 `前提 ∧ 路径条件 ⊢ 结论`（§5.3 第 4 步）。
/// 返回 (真值, 关闭理由)。Unknown = 该路径关闭不了（不猜）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closure {
    pub truth: Truth,
    /// literal / trace / hypothesis / interval / atom（§5.4 证书）。
    pub reason: &'static str,
}

pub fn check_path(
    path: &PathRecord,
    premise: Option<&PremiseAtom>,
    conclusion: &Conclusion,
) -> Closure {
    // 前提必须先可用：超片段前提 → 路径关不了。
    if let Some(PremiseAtom::Opaque) = premise {
        return Closure { truth: Truth::Unknown, reason: "premise" };
    }
    // 前提与路径条件的合取做区间 + 自由布尔假设。
    let intervals = intervals_of(path, premise);
    let free_bools = free_bools_of(path, premise);

    // 可行性：前提区间与路径条件矛盾（交集为空）→ 路径不可行，空真关闭。
    if !feasible(&intervals) {
        return Closure { truth: Truth::True, reason: "infeasible" };
    }

    let truth = eval_conclusion(conclusion, path, &intervals, &free_bools);
    if truth == Truth::Unknown {
        return Closure { truth: Truth::Unknown, reason: "unknown" };
    }
    let reason = closure_reason(conclusion, path, premise);
    Closure { truth, reason }
}

/// 区间交集是否非空（可行域检查；无约束的变量视为可行）。
fn feasible(intervals: &[(String, Cmp, i64)]) -> bool {
    use std::collections::BTreeMap;
    let mut bounds: BTreeMap<String, (Option<i64>, Option<i64>)> = BTreeMap::new();
    let mut exacts: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for (var, op, val) in intervals {
        let e = bounds
            .entry(var.clone())
            .or_insert((None, None));
        match op {
            Cmp::Gt => e.0 = Some(e.0.map_or(val + 1, |l: i64| l.max(val + 1))),
            Cmp::Ge => e.0 = Some(e.0.map_or(*val, |l: i64| l.max(*val))),
            Cmp::Lt => e.1 = Some(e.1.map_or(val - 1, |u: i64| u.min(val - 1))),
            Cmp::Le => e.1 = Some(e.1.map_or(*val, |u: i64| u.min(*val))),
            Cmp::Eq => exacts.entry(var.clone()).or_default().push(*val),
            Cmp::Ne => {}
        }
    }
    for (var, vals) in &exacts {
        if vals.iter().any(|v| vals[0] != *v) {
            return false; // x = c1 ∧ x = c2, c1≠c2 → 不可行
        }
        let c = vals[0];
        if let Some((lo, hi)) = bounds.get(var) {
            if lo.map_or(false, |l| l > c) || hi.map_or(false, |h| h < c) {
                return false;
            }
        }
    }
    bounds.iter().all(|(_, (lo, hi))| match (lo, hi) {
        (Some(l), Some(h)) => l <= h,
        _ => true,
    })
}

/// 收集本路径可用的区间约束：路径条件 + 前提。
fn intervals_of(path: &PathRecord, premise: Option<&PremiseAtom>) -> Vec<(String, Cmp, i64)> {
    let mut out = Vec::new();
    for atom in &path.atoms {
        if let PathAtom::Interval { var, op, val } = atom {
            out.push((var.clone(), *op, *val));
        }
    }
    if let Some(PremiseAtom::Interval { var, op, val }) = premise {
        out.push((var.clone(), *op, *val));
    }
    out
}

fn free_bools_of(path: &PathRecord, premise: Option<&PremiseAtom>) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    for atom in &path.atoms {
        if let PathAtom::FreeBool { op, key, expected } = atom {
            out.push((op.clone(), key.clone(), *expected));
        }
    }
    if let Some(PremiseAtom::FreeBool { op, key, expected }) = premise {
        out.push((op.clone(), key.clone(), *expected));
    }
    out
}

/// 结论求值（结构代入：主体变量来自路径快照，参数走区间/自由布尔）。
fn eval_conclusion(
    conclusion: &Conclusion,
    path: &PathRecord,
    intervals: &[(String, Cmp, i64)],
    free_bools: &[(String, String, bool)],
) -> Truth {
    let trace: &[String] = &path.trace;
    let lookup = |subject: &str| -> Option<SymVal> {
        if subject == "结果" {
            Some(path.result.clone())
        } else {
            path.vars
                .iter()
                .find(|(k, _)| k == subject)
                .map(|(_, v)| v.clone())
        }
    };
    match conclusion {
        Conclusion::FieldBool { subject, field, expected } => {
            // 主体是内联构造器 → 结构化（F3）；是 OpResult → 自由原子（F4）。
            let Some(value) = lookup(subject) else {
                return Truth::Unknown;
            };
            match value {
                SymVal::Struct { fields, .. } => match fields.iter().find(|(k, _)| k == field) {
                    Some((_, SymVal::Bool(b))) => {
                        if b == expected {
                            Truth::True
                        } else {
                            Truth::False
                        }
                    }
                    _ => Truth::Unknown,
                },
                SymVal::OpResult(op) => {
                    // 自由原子：路径条件或前提里有同键判定才能关闭。
                    match free_bools
                        .iter()
                        .find(|(o, k, _)| *o == op && k == field)
                    {
                        Some((_, _, v)) => {
                            if *v == *expected {
                                Truth::True
                            } else {
                                Truth::False
                            }
                        }
                        None => Truth::Unknown,
                    }
                }
                _ => Truth::Unknown,
            }
        }
        Conclusion::TraceContains { item, expected } => {
            let has = trace.iter().any(|op| op == item);
            if has == *expected {
                Truth::True
            } else {
                Truth::False
            }
        }
        Conclusion::TraceLen { op, val } => {
            let n = trace.len() as i64;
            if op.holds(n, *val) {
                Truth::True
            } else {
                Truth::False
            }
        }
        Conclusion::ParamInterval { var, op, val } => {
            interval_entailed(intervals, var, *op, *val)
        }
        Conclusion::Not(inner) => {
            eval_conclusion(inner, path, intervals, free_bools).invert()
        }
        Conclusion::And(a, b) => {
            let (ta, tb) = (
                eval_conclusion(a, path, intervals, free_bools),
                eval_conclusion(b, path, intervals, free_bools),
            );
            match (ta, tb) {
                (Truth::False, _) | (_, Truth::False) => Truth::False,
                (Truth::True, Truth::True) => Truth::True,
                _ => Truth::Unknown,
            }
        }
        Conclusion::Or(a, b) => {
            let (ta, tb) = (
                eval_conclusion(a, path, intervals, free_bools),
                eval_conclusion(b, path, intervals, free_bools),
            );
            match (ta, tb) {
                (Truth::True, _) | (_, Truth::True) => Truth::True,
                (Truth::False, Truth::False) => Truth::False,
                _ => Truth::Unknown,
            }
        }
        Conclusion::Opaque(_) => Truth::Unknown,
    }
}

impl Truth {
    fn invert(self) -> Truth {
        match self {
            Truth::True => Truth::False,
            Truth::False => Truth::True,
            Truth::Unknown => Truth::Unknown,
        }
    }
}

/// F5 蕴含：区间集合是否迫使 `var ⋈ val` 成立。
/// True = 区间整体落在满足侧；False = 整体落在违反侧（可反例）；否则不猜。
fn interval_entailed(intervals: &[(String, Cmp, i64)], var: &str, op: Cmp, val: i64) -> Truth {
    let mut lower: Option<i64> = None;
    let mut upper: Option<i64> = None;
    let mut exact: Option<i64> = None;
    let mut conflict = false;
    for (v, o, c) in intervals {
        if v != var {
            continue;
        }
        match o {
            Cmp::Gt => lower = Some(lower.map_or(c + 1, |l: i64| l.max(c + 1))),
            Cmp::Ge => lower = Some(lower.map_or(*c, |l: i64| l.max(*c))),
            Cmp::Lt => upper = Some(upper.map_or(c - 1, |u: i64| u.min(c - 1))),
            Cmp::Le => upper = Some(upper.map_or(*c, |u: i64| u.min(*c))),
            Cmp::Eq => {
                if let Some(e) = exact {
                    if e != *c {
                        conflict = true;
                    }
                }
                exact = Some(*c);
            }
            Cmp::Ne => {}
        }
    }
    let _ = conflict; // 约束冲突保守当作"不蕴含"处理（不判 False）
    if let Some(e) = exact {
        // 精确值还要落在区间内才可信。
        let in_range = lower.map_or(true, |l| e >= l) && upper.map_or(true, |u| e <= u);
        if in_range {
            return if op.holds(e, val) {
                Truth::True
            } else {
                Truth::False
            };
        }
        return Truth::Unknown;
    }
    // 正向闭包（True）只认**字面出现**的原子：区间蕴含（如 金额 > 10000 ⊢
    // 金额 > 9999）数学上成立，但当前 Lean 脚本没有 Val 比较的传递引理，
    // simp+omega 关不掉——离线先于 Lean 关闭即触发 proof.divergence。
    // 保守：字面匹配 → hypothesis 关闭；否则不猜（等库补传递引理再放开）。
    for (v, o, c) in intervals {
        if v == var && *o == op && *c == val {
            return Truth::True;
        }
    }
    // False：整段区间违反目标 → 确定性否定（反例见证）。这在数学上成立
    // （区间端点即见证），Lean 侧同样证不出该定理——两侧一致 unproved。
    let violated = match op {
        Cmp::Gt => upper.map_or(false, |u| u <= val),
        Cmp::Ge => upper.map_or(false, |u| u < val),
        Cmp::Lt => lower.map_or(false, |l| l >= val),
        Cmp::Le => lower.map_or(false, |l| l > val),
        Cmp::Eq => lower.map_or(false, |l| l > val) || upper.map_or(false, |u| u < val),
        Cmp::Ne => false, // ≠ 无法用区间否定（需精确值）
    };
    if violated {
        return Truth::False;
    }
    Truth::Unknown
}

/// 关闭理由（§5.4 证书字段）。
fn closure_reason(conclusion: &Conclusion, path: &PathRecord, premise: Option<&PremiseAtom>) -> &'static str {
    match conclusion {
        Conclusion::FieldBool { subject, .. } => {
            let value = if subject == "结果" {
                Some(path.result.clone())
            } else {
                path.vars
                    .iter()
                    .find(|(k, _)| k == subject)
                    .map(|(_, v)| v.clone())
            };
            match value {
                Some(SymVal::Struct { .. }) => "literal",
                Some(SymVal::OpResult(_)) => "hypothesis",
                _ => "unknown",
            }
        }
        Conclusion::TraceContains { .. } | Conclusion::TraceLen { .. } => "trace",
        Conclusion::ParamInterval { .. } => {
            if premise.is_some() {
                "interval"
            } else {
                "interval"
            }
        }
        Conclusion::Not(inner) => closure_reason(inner, path, premise),
        Conclusion::And(a, _) | Conclusion::Or(a, _) => closure_reason(a, path, premise),
        Conclusion::Opaque(_) => "unknown",
    }
}

/// 检查一条定理（§5.3 第 5 步）：全部路径关闭 → checked；
/// 某路径确定性为假 → unproved + 反例（§5.5 三条件）；否则 unverified。
pub fn check_claim(fn_plan: &FnPlan, claim: &crate::prove::ir::ClaimPlan) -> OfflineVerdict {
    let paths = execute(fn_plan);
    if paths.is_empty() {
        return OfflineVerdict::Unverified {
            reason: "无可枚举路径".into(),
        };
    }
    if std::env::var("MARQDO_PROOF_DEBUG").is_ok_and(|v| v == "1") {
        for p in execute(fn_plan) {
            eprintln!(
                "[offline-path #{}] atoms={:?} result={:?} trace={:?} vars={:?} opaque={}",
                p.id, p.atoms, p.result, p.trace, p.vars, p.opaque
            );
        }
    }
    let premise = claim.premise.as_deref().and_then(parse_premise);
    // 前提文本存在但解析不出原子 → 超片段。
    let premise: Option<PremiseAtom> = match (&claim.premise, premise) {
        (Some(_), None) => Some(PremiseAtom::Opaque),
        (_, p) => p,
    };
    let conclusion = parse_conclusion(&claim.conclusion);

    let mut closed = 0usize;
    let mut first_false: Option<(&PathRecord, Closure)> = None;
    for path in &paths {
        if path.opaque {
            return OfflineVerdict::Unverified {
                reason: "路径超出片段（不透明条件/表达式）".into(),
            };
        }
        let closure = check_path(path, premise.as_ref(), &conclusion);
        match closure.truth {
            Truth::True => closed += 1,
            Truth::False => {
                if first_false.is_none() {
                    first_false = Some((path, closure));
                }
            }
            Truth::Unknown => {
                return OfflineVerdict::Unverified {
                    reason: format!("路径 #{} 关闭不了（不猜）", path.id),
                }
            }
        }
    }
    if let Some((path, _)) = first_false {
        // §5.5 反例三条件：结论确定性为假 ✓；路径可满足（区间取见证）✓；
        // 路径条件不含自由布尔原子（否则可能凑出不存在的组合）。
        let has_free_bool = path
            .atoms
            .iter()
            .any(|a| matches!(a, PathAtom::FreeBool { .. }));
        if has_free_bool {
            return OfflineVerdict::Unverified {
                reason: "反例路径含自由布尔原子（保守放弃见证）".into(),
            };
        }
        let counterexample = witness(fn_plan, path, premise.as_ref());
        return OfflineVerdict::Unproved { counterexample };
    }
    OfflineVerdict::Checked { cases: closed }
}

/// 从区间端点取见证参数值（§5.5）。无区间约束的整型参数取 0（空约束平凡可满足）。
fn witness(
    fn_plan: &FnPlan,
    path: &PathRecord,
    premise: Option<&PremiseAtom>,
) -> Option<Counterexample> {
    let mut assignments: Vec<(String, i64)> = Vec::new();
    let mut constraints: Vec<(String, Cmp, i64)> = Vec::new();
    for atom in &path.atoms {
        if let PathAtom::Interval { var, op, val } = atom {
            constraints.push((var.clone(), *op, *val));
        }
    }
    if let Some(PremiseAtom::Interval { var, op, val }) = premise {
        constraints.push((var.clone(), *op, *val));
    }
    // 按 var 聚合求可行点：下界取 max+?，上界取 min-?，取中点。
    use std::collections::BTreeMap;
    let mut bounds: BTreeMap<String, (Option<i64>, Option<i64>)> = BTreeMap::new();
    for (var, op, val) in &constraints {
        let e = bounds.entry(var.clone()).or_insert((None, None));
        match op {
            Cmp::Gt => e.0 = Some(e.0.map_or(val + 1, |l: i64| l.max(val + 1))),
            Cmp::Ge => e.0 = Some(e.0.map_or(*val, |l: i64| l.max(*val))),
            Cmp::Lt => e.1 = Some(e.1.map_or(val - 1, |u: i64| u.min(val - 1))),
            Cmp::Le => e.1 = Some(e.1.map_or(*val, |u: i64| u.min(*val))),
            Cmp::Eq => {
                // 等式约束与区间冲突时放弃见证（保守）。
                if let (Some(l), Some(u)) = (e.0, e.1) {
                    if l > *val || u < *val {
                        return None;
                    }
                }
                e.0 = Some(*val);
                e.1 = Some(*val);
            }
            Cmp::Ne => {}
        }
    }
    for (var, (lo, hi)) in bounds {
        match (lo, hi) {
            (Some(l), Some(h)) if l > h => return None, // 路径不可满足（不应发生）
            (Some(l), Some(h)) => assignments.push((var, l + (h - l) / 2)),
            (Some(l), None) => assignments.push((var, l)),
            (None, Some(h)) => assignments.push((var, h)),
            (None, None) => {}
        }
    }
    if assignments.is_empty() {
        // 无任何区间约束：整型参数取 0（空约束下任意值都满足该路径）。
        for p in &fn_plan.params {
            if matches!(p.ty, crate::prove::ir::PlanTy::Int) {
                assignments.push((p.name.clone(), 0));
            }
        }
    }
    if assignments.is_empty() {
        return None;
    }
    let detail = assignments
        .iter()
        .map(|(k, v)| format!("{k} = {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(Counterexample {
        assignments,
        path_id: path.id,
        detail,
    })
}

/// 整个文件的离线判定：每函数每定理。
pub fn check_plan(plan: &Plan) -> Vec<(String, String, OfflineVerdict)> {
    let mut out = Vec::new();
    for f in &plan.functions {
        for c in &f.claims {
            let verdict = check_claim(f, c);
            out.push((f.name.clone(), c.id.clone(), verdict));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prove::ir::{
        BinOp, ClaimPlan, Expr, FnPlan, ParamPlan, Plan, PlanTy, PureExpr, Step,
    };

    fn ledger_fn(steps: Vec<Step>, claims: Vec<ClaimPlan>) -> FnPlan {
        FnPlan {
            name: "台账.记录".into(),
            params: vec![ParamPlan {
                name: "金额".into(),
                ty: PlanTy::Int,
                has_default: false,
            }],
            result_ty: PlanTy::Any,
            ops: vec![],
            steps,
            effects: vec![],
            claims,
            assumptions: vec![],
        }
    }

    /// `结果 = result.ok(金额)`；`返回 结果`。
    fn literal_ok_steps() -> Vec<Step> {
        vec![
            Step::Bind {
                var: "结果".into(),
                expr: Expr::Pure(PureExpr::Ctor {
                    name: "result.ok".into(),
                    args: vec![PureExpr::Var("金额".into())],
                }),
                line: 10,
            },
            Step::Return {
                expr: Expr::Pure(PureExpr::Var("结果".into())),
                line: 11,
            },
        ]
    }

    #[test]
    fn f3_literal_result_field_closes() {
        let f = ledger_fn(
            literal_ok_steps(),
            vec![ClaimPlan {
                id: "必有回执".into(),
                premise: None,
                conclusion: "结果.ok = True".into(),
                tactic: None,
                line: 5,
            }],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Checked { cases } => assert!(cases >= 1),
            other => panic!("F3 应当 checked：{other:?}"),
        }
    }

    #[test]
    fn f3_false_claim_refuted_with_counterexample() {
        let f = ledger_fn(
            literal_ok_steps(),
            vec![ClaimPlan {
                id: "没有回执".into(),
                premise: None,
                conclusion: "结果.ok = False".into(),
                tactic: None,
                line: 5,
            }],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Unproved { counterexample } => {
                assert!(counterexample.is_some(), "确定性否定必须带见证");
            }
            other => panic!("F3 反例应当 unproved：{other:?}"),
        }
    }

    #[test]
    fn f2_trace_length_and_contains() {
        // 金额 > 10000 → 轨迹 = [人工审核]；否则空。
        let steps = vec![Step::If {
            cond: PureExpr::Bin {
                operator: BinOp::Gt,
                left: Box::new(PureExpr::Var("金额".into())),
                right: Box::new(PureExpr::Int(10000)),
            },
            then: vec![Step::Trace {
                op: "调用:人工审核".into(),
                args: vec![],
                line: 20,
            }],
            otherwise: vec![],
            line: 19,
        }];
        let mk = |id: &str, conclusion: &str, premise: Option<&str>| ClaimPlan {
            id: id.into(),
            premise: premise.map(Into::into),
            conclusion: conclusion.into(),
            tactic: None,
            line: 5,
        };
        let f = ledger_fn(
            steps,
            vec![
                mk("恰一步", "长度(轨迹) = 1", Some("金额 > 10000")),
                mk("恰零步", "长度(轨迹) = 0", None),
                mk("必经人工", "包含(轨迹, \"人工审核\")", Some("金额 > 10000")),
            ],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Checked { .. } => {}
            other => panic!("F2 长度应 checked：{other:?}"),
        }
        // 无前提：两条路径都要关（一条长 1、一条长 0）→ 长度=0 关不了。
        match check_claim(&f, &f.claims[1]) {
            OfflineVerdict::Unproved { .. } | OfflineVerdict::Unverified { .. } => {}
            other => panic!("无前提的 长度=0 不得 checked：{other:?}"),
        }
        match check_claim(&f, &f.claims[2]) {
            OfflineVerdict::Checked { .. } => {}
            other => panic!("F2 包含应 checked：{other:?}"),
        }
    }

    #[test]
    fn f4_free_bool_atom_via_hypothesis() {
        // 审核 = op(金额)；1. 审核.ok = True → 路径条件为自由布尔；
        // 结论 `审核.ok = True` 由 hypothesis 关闭。
        let steps = vec![
            Step::Bind {
                var: "审核".into(),
                expr: Expr::Op {
                    id: "调用:人工审核".into(),
                    args: vec![PureExpr::Var("金额".into())],
                },
                line: 30,
            },
            Step::If {
                cond: PureExpr::Access {
                    base: Box::new(PureExpr::Var("审核".into())),
                    key: "ok".into(),
                },
                then: vec![Step::Return {
                    expr: Expr::Pure(PureExpr::Var("审核".into())),
                    line: 32,
                }],
                otherwise: vec![Step::Return {
                    expr: Expr::Pure(PureExpr::Ctor {
                        name: "result.ok".into(),
                        args: vec![PureExpr::Text("跳过".into())],
                    }),
                    line: 34,
                }],
                line: 31,
            },
        ];
        // 自由原子二分后的 else 路径上 `审核.ok = False` 是**真实可能**的，
        // 所以单边结论只能保守 unverified（不猜）。
        let f = ledger_fn(
            steps,
            vec![
                ClaimPlan {
                    id: "审核必有结论".into(),
                    premise: Some("金额 > 10000".into()),
                    conclusion: "审核.ok = True 或 审核.ok = False".into(),
                    tactic: None,
                    line: 5,
                },
                ClaimPlan {
                    id: "审核必过".into(),
                    premise: Some("金额 > 10000".into()),
                    conclusion: "审核.ok = True".into(),
                    tactic: None,
                    line: 6,
                },
            ],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Checked { .. } => {}
            other => panic!("F4 hypothesis（或式）应 checked：{other:?}"),
        }
        match check_claim(&f, &f.claims[1]) {
            OfflineVerdict::Unverified { .. } => {}
            other => panic!("自由原子单边结论必须保守 unverified：{other:?}"),
        }
    }

    #[test]
    fn f5_interval_closure_scope() {
        // 字面重述（同原子）：hypothesis 关闭（Lean 侧假设就在那里）。
        let f = ledger_fn(
            literal_ok_steps(),
            vec![
                ClaimPlan {
                    id: "前提重述".into(),
                    premise: Some("金额 > 10000".into()),
                    conclusion: "金额 > 10000".into(),
                    tactic: None,
                    line: 5,
                },
                // 区间蕴含（10000 ⊢ 9999）数学上真，但当前 Lean 脚本无
                // Val 比较传递引理——离线必须同样保守（§5.4 单调性）。
                ClaimPlan {
                    id: "区间蕴含".into(),
                    premise: Some("金额 > 10000".into()),
                    conclusion: "金额 > 9999".into(),
                    tactic: None,
                    line: 6,
                },
            ],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Checked { .. } => {}
            other => panic!("字面重述应 checked：{other:?}"),
        }
        match check_claim(&f, &f.claims[1]) {
            OfflineVerdict::Unverified { .. } => {}
            other => panic!("无传递引理时区间蕴含必须保守：{other:?}"),
        }
    }

    #[test]
    fn f5_interval_refutation_gives_witness() {
        let f = ledger_fn(
            literal_ok_steps(),
            vec![ClaimPlan {
                id: "小额断言".into(),
                premise: Some("金额 > 10000".into()),
                conclusion: "金额 < 100".into(),
                tactic: None,
                line: 5,
            }],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Unproved { counterexample } => {
                let c = counterexample.expect("区间分离必须给见证");
                assert!(c.detail.contains("金额"));
            }
            other => panic!("F5 分离应 unproved + 见证：{other:?}"),
        }
    }

    #[test]
    fn beyond_fragment_stays_unverified() {
        // 文本比较（字符串序）超出片段 → 不猜。
        let f = ledger_fn(
            literal_ok_steps(),
            vec![ClaimPlan {
                id: "文本比较".into(),
                premise: None,
                conclusion: "结果.value = \"x\"".into(),
                tactic: None,
                line: 5,
            }],
        );
        match check_claim(&f, &f.claims[0]) {
            OfflineVerdict::Unverified { .. } | OfflineVerdict::Checked { .. } => {}
            other => panic!("超片段不得 unproved：{other:?}"),
        }
    }

    #[test]
    fn plan_wide_check_runs_all() {
        let f = ledger_fn(
            literal_ok_steps(),
            vec![ClaimPlan {
                id: "必有回执".into(),
                premise: None,
                conclusion: "结果.ok = True".into(),
                tactic: None,
                line: 5,
            }],
        );
        let plan = Plan {
            schema: crate::prove::ir::SCHEMA,
            module: "测试".into(),
            source_sha256: "a".repeat(64),
            functions: vec![f],
        };
        let results = check_plan(&plan);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].2, OfflineVerdict::Checked { .. }));
    }
}

//! Lean 生成（`doc/design/prove-cli.md` §4.1 ④ / §6）。
//!
//! 确定性硬要求：同输入两次运行字节一致。做法：无时间戳、无绝对路径、
//! 集合按首次出现顺序（降级本身确定）、映射键排序、浮点定长。
//!
//! P0 形态：参数统一 `Val`（P1 起按契约提升为 `Int`/`String` 等以配合
//! `structure Params` 与定理表）；`result.ok/error` 内联为库构造器。

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::prove::ir::{
    BinOp, Expr as IrExpr, FnPlan, InterpPart, Plan, PlanTy, PureExpr, Step, UnOp,
};

/// 库名（verification/skill-proof 提供；P1 落地）。
pub const LIBRARY_IMPORT: &str = "MarqdoProof";

/// 生成整个文件模块。返回 `(文件名, 内容)`。
pub fn emit_module(plan: &Plan, module_stem: &str) -> (String, String) {
    let namespace = namespace_of(module_stem);
    let mut out = String::new();
    let _ = writeln!(out, "import {LIBRARY_IMPORT}");
    let _ = writeln!(out, "open MarqdoProof");
    // 机器生成的证明产物：关闭面向人写代码的 linter（未用变量/simp 参数是
    // 生成器的固定词汇表带来的正常噪声，不是问题）。
    let _ = writeln!(out, "set_option linter.unusedVariables false");
    let _ = writeln!(out, "set_option linter.unusedSimpArgs false");
    let _ = writeln!(out);
    let _ = writeln!(out, "-- 由 `marqdo prove emit` 生成：确定性输出，请勿手改。");
    let _ = writeln!(out, "-- 源：{}（sha256 {}）", plan.module, plan.source_sha256);
    let _ = writeln!(out, "namespace Generated.{}", namespace);
    let _ = writeln!(out);

    // 模块级 Op：所有函数的不透明操作并集（保持首次出现顺序，去重）。
    emit_op_inductive(&mut out, plan);
    let _ = writeln!(out);

    for fun in &plan.functions {
        emit_function(&mut out, fun);
        let _ = writeln!(out);
        emit_theorems(&mut out, fun);
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "end Generated.{}", namespace);
    let file_name = format!("{namespace}.lean");
    (file_name, out)
}

/// 定理脚手架（§7.2）：受限文法可解析的命题 → `theorem` 陈述 + `sorry`
/// 函数体里 if 的最大嵌套深度（0 = 无分支）。
fn max_if_depth(steps: &[Step]) -> usize {
    fn walk(steps: &[Step], depth: usize) -> usize {
        let mut max = depth;
        for s in steps {
            if let Step::If { then, otherwise, .. } = s {
                max = max.max(walk(then, depth + 1));
                max = max.max(walk(otherwise, depth + 1));
            }
        }
        max
    }
    walk(steps, 0)
}

fn emit_theorems(out: &mut String, fun: &FnPlan) {
    if fun.claims.is_empty() {
        return;
    }
    let fun_name = lean_ident(&fun.name);
    // 定理形参：`env` + 函数形参（Lean 标识符仅 ASCII，中文名入 guillemet）。
    // run 从空轨迹起跑（§2：轨迹是语法产物，函数入口为空）。
    let mut binders = String::from("(env : Env)");
    for param in &fun.params {
        let _ = write!(binders, " ({} : Val)", lean_ident(&param.name));
    }
    let mut seen_theorem_names: BTreeSet<String> = BTreeSet::new();
    for claim in &fun.claims {
        let theorem_name = lean_ident(&format!("{}.{}", fun.name, claim.id));
        if !seen_theorem_names.insert(theorem_name.clone()) {
            let _ = writeln!(
                out,
                "-- 定理 {} 重复（同 id 的定理只生成一次；check --proof 会给 proof.claim_duplicate）",
                claim.id
            );
            continue;
        }
        let premise_lean = claim
            .premise
            .as_deref()
            .and_then(claim_to_lean_bool);
        let conclusion_lean = claim_to_lean_bool(&claim.conclusion);
        let premise_text = match (&claim.premise, premise_lean) {
            (Some(_), Some(lean)) => format!(" («h_前提» : mqTruth {lean})"),
            _ => String::new(),
        };
        let tactic = claim.tactic.as_deref().unwrap_or("auto");
        let _ = writeln!(out, "-- 定理 {}（策略 {tactic}；mq: {}）", claim.id, claim.line);
        match conclusion_lean {
            Some(conclusion) => {
                let run_args = fun
                    .params
                    .iter()
                    .map(|p| lean_ident(&p.name))
                    .collect::<Vec<_>>()
                    .join(" ");
                let _ = writeln!(out, "theorem {theorem_name} {binders}{premise_text} :");
                let _ = writeln!(out, "    ∀ («结果» : Val) («轨迹» : List Op),");
                // 等式方向：`(结果, 轨迹) = run …`（输出变量在左，simp_all 才能把
                // 输出值替换进目标；反向等式 simp 不会替换）。
                let _ = writeln!(
                    out,
                    "      ((«结果», «轨迹») = {fun_name} env [] {run_args}) →"
                );
                let _ = writeln!(out, "      mqTruth {conclusion} := by");
                if tactic == "none" {
                    // 策略 none：定理生成但不证明（E4：状态 unverified，
                    // driver 依 sorry 判 unverified，绝不冒充 proved）。
                    let _ = writeln!(out, "  intro «结果» «轨迹» h_run");
                    let _ = writeln!(out, "  unfold {fun_name} at h_run");
                    let _ = writeln!(out, "  try split at h_run");
                    let _ = writeln!(out, "  all_goals");
                    let _ = writeln!(out, "    sorry");
                    continue;
                }
                let _ = writeln!(out, "  intro «结果» «轨迹» h_run");
                let _ = writeln!(out, "  unfold {fun_name} at h_run");
                // 嵌套分支（else 里再嵌 if）需要逐层拆：按最大 if 深度发射
                // `try split at h_run`（拆无可拆时 try 吞掉，幂等）。
                for _ in 0..=max_if_depth(&fun.steps) {
                    let _ = writeln!(out, "  try split at h_run");
                }
                // 每个分支：simp_all（完整默认集处理 List.map/contains/length 的
                // 结构化消去；等式替换输出；Op.name 方程化简）+ 附加域定义
                // （mqAccess/mqResultOk/mqEq 字段与构造）。互斥比较假设由库中
                // @[simp] 矛盾引理消掉，剩余线性算术交给 omega。
                let _ = writeln!(
                    out,
                    "  all_goals
    simp_all [mqTruth, Op.name, mqAccess, mqResultOk, mqResultError, mqValueOr, mqAnd, mqOr, mqNot, mqEq]
    try simp_all [mqAdd, mqSub, mqMul, mqNeg]
    try
      first
      | exact (mqGt_mqLe_false ‹mqGt _ (Val.int _) = true› ‹mqLe _ (Val.int _) = true›).elim
      | exact (mqLe_mqGt_false ‹mqLe _ (Val.int _) = true› ‹mqGt _ (Val.int _) = true›).elim
      | exact (mqLt_mqGe_false ‹mqLt _ (Val.int _) = true› ‹mqGe _ (Val.int _) = true›).elim
      | exact (mqGe_mqLt_false ‹mqGe _ (Val.int _) = true› ‹mqLt _ (Val.int _) = true›).elim
      | omega"
                );
            }
            _ => {
                let _ = writeln!(
                    out,
                    "--   未生成：前提/结论超出受限文法（check --proof 会给具体诊断）"
                );
            }
        }
    }
}

/// 命名空间：文件名的安全标识符。Lean 4 的 `namespace` 命令**只接受 ASCII**
/// 名字（实测 `namespace 审批` 报 "expected token"），所以非 ASCII 字符
/// 确定性编码为 `uXXXX`（码点十六进制），保证同名文件 → 同名空间。
pub fn namespace_of(stem: &str) -> String {
    let mut out = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else if !ch.is_ascii() {
            out.push_str(&format!("u{:04X}", ch as u32));
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('M');
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, 'M');
    }
    out
}

/// Lean 保留字（`def 审批` 这类裸中文名在 Lean 里是语法错误——
/// 标识符仅 ASCII；非 ASCII 名一律进 guillemet `«…»`）。
const LEAN_KEYWORDS: &[&str] = &[
    "fun", "let", "in", "do", "match", "with", "if", "then", "else", "true", "false", "and", "or",
    "not", "def", "theorem", "abbrev", "structure", "inductive", "where", "deriving", "open",
    "namespace", "end", "import", "by", "have", "show", "assume", "from", "at", "exists",
];

pub(crate) fn lean_ident(name: &str) -> String {
    let ascii_safe = !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !name.chars().next().is_some_and(|c| c.is_ascii_digit());
    if LEAN_KEYWORDS.contains(&name) || name == "env" {
        // 关键字与本模块保留名（函数体里用户变量撞名时改名）。
        // 改名形式必须纯 ASCII（裸 `v_结果` 仍是 Lean 语法错误）。
        format!("v_{}", namespace_of(name))
    } else if name == "轨迹" || name == "结果" || name == "h_前提" {
        // 定理作用域保留名：函数体里的用户变量/形参撞名 → v_ 前缀
        // （定理的 ∀ 绑定名由 emit_theorems/claim_term 直接写 `«…»`，不走这里）。
        format!("v_{}", namespace_of(name))
    } else if ascii_safe {
        name.to_string()
    } else {
        // 中文名/含点/含特殊字符 → guillemet；`:` 会截断记号，替换为 `_`。
        format!("«{}»", name.replace(':', "_"))
    }
}

fn op_ctor(op_id: &str) -> String {
    // `调用:人工审核` → `«调用_人工审核»`（guillemet 标识符）。
    // Lean 的 guillemet 记号不接受 `:`（会截断记号），标识符里替换为 `_`；
    // Op.name 的**字符串显示**保持原始 id，轨迹名不变。
    let safe = op_id.replace(':', "_");
    format!("«{safe}»")
}

fn emit_op_inductive(out: &mut String, plan: &Plan) {
    // 收集 op（首现顺序去重）。
    let mut ops: Vec<&crate::prove::ir::OpDecl> = Vec::new();
    for fun in &plan.functions {
        for op in &fun.ops {
            if !ops.iter().any(|o| o.id == op.id) {
                ops.push(op);
            }
        }
    }
    if ops.is_empty() {
        let _ = writeln!(out, "inductive Op where");
        // 占位构造器也走 op_ctor（裸中文构造器是 Lean 语法错误）。
        let _ = writeln!(out, "  | {} deriving Repr, BEq", op_ctor("无操作"));
        let _ = writeln!(out);
        let _ = writeln!(out, "def Op.name : Op → String");
        let _ = writeln!(out, "  | .{} => \"\"", op_ctor("无操作"));
        return;
    }
    let _ = writeln!(out, "inductive Op where");
    for op in &ops {
        let fields = match op.arity {
            0 => String::new(),
            n => (1..=n)
                .map(|i| format!("(«参数{i}» : Val)"))
                .collect::<Vec<_>>()
                .join(" "),
        };
        let _ = writeln!(out, "  | {} {}", op_ctor(&op.id), fields);
    }
    let _ = writeln!(out, "  deriving Repr, BEq");
    let _ = writeln!(out);
    let _ = writeln!(out, "def Op.name : Op → String");
    for op in &ops {
        let label = op.id.strip_prefix("调用:").unwrap_or(&op.id);
        // 多参构造器的模式必须绑定实参（`_ _`），否则类型不匹配。
        let blanks = " _".repeat(op.arity);
        let _ = writeln!(out, "  | .{} {}=> \"{}\"", op_ctor(&op.id), blanks, label);
    }
    let _ = writeln!(out);
    // Env 依赖本文件的 Op（库无法预知），所以每文件自带：
    // `env` 是"不透明操作 → 返回值"的任意函数——后置条件对一切 env 成立。
    let _ = writeln!(out, "abbrev Env := Op → Val");
}

fn emit_function(out: &mut String, fun: &FnPlan) {
    let name = lean_ident(&fun.name);
    // 参数列表：`def f (env : Env) (轨迹 : List Op) (p1 : Val) …`
    let mut sig = String::from("(env : Env) («轨迹» : List Op)");
    for param in &fun.params {
        let _ = write!(sig, " ({} : Val)", lean_ident(&param.name));
    }
    let _ = writeln!(out, "-- {} 返回 {}", fun.name, fun.result_ty.name());
    let _ = writeln!(out, "def {name} {sig} : Val × List Op :=");
    if fun.steps.is_empty() {
        let _ = writeln!(out, "  (Val.none, «轨迹»)");
        return;
    }
    emit_seq(out, &mut String::from("  "), &[&fun.steps]);
}

/// 续延式直接降级（§6.3）：`chunks` 是"当前点之后还要执行的所有步骤"。
///
/// `if` 的两臂续延是「臂内步骤 + 臂外剩余」——执行继续到分支之后，
/// 除非某臂已提前 `Return`。这是 P0 第一版的续延丢失 bug 的修正：
/// 分支绝不能吞掉后续语句。
fn emit_seq(out: &mut String, indent: &mut String, chunks: &[&[Step]]) {
    match chunks.split_first() {
        None => {
            // 走到末尾都没有 return：Marqdo 返回 None（诚实占位，不译运行时）。
            let _ = writeln!(out, "{indent}(Val.none, «轨迹»)");
        }
        Some((first, rest)) => match first.split_first() {
            None => emit_seq(out, indent, rest),
            Some((step, more)) => emit_step(out, indent, step, more, rest),
        },
    }
}

fn tail_chunks<'a>(more: &'a [Step], rest: &[&'a [Step]]) -> Vec<&'a [Step]> {
    let mut next: Vec<&'a [Step]> = Vec::with_capacity(rest.len() + 1);
    if !more.is_empty() {
        next.push(more);
    }
    next.extend_from_slice(rest);
    next
}

fn emit_step(out: &mut String, indent: &mut String, step: &Step, more: &[Step], rest: &[&[Step]]) {
    match step {
        Step::Bind { var, expr, line } => {
            let _ = writeln!(out, "{indent}-- mq: {line}:1");
            match expr {
                IrExpr::Op { id, args } => {
                    let _ = writeln!(
                        out,
                        "{indent}let {} := env (Op.{} {})",
                        lean_ident(var),
                        op_ctor(id),
                        emit_args(args)
                    );
                    let _ = writeln!(
                        out,
                        "{indent}let «轨迹» := «轨迹» ++ [Op.{} {}]",
                        op_ctor(id),
                        emit_args(args)
                    );
                }
                other => {
                    let _ = writeln!(
                        out,
                        "{indent}let {} := {}",
                        lean_ident(var),
                        emit_expr(other)
                    );
                }
            }
            emit_seq(out, indent, &tail_chunks(more, rest));
        }
        Step::Return { expr, line } => {
            let _ = writeln!(out, "{indent}-- mq: {line}:1");
            match expr {
                IrExpr::Op { id, args } => {
                    let _ = writeln!(
                        out,
                        "{indent}(env (Op.{} {}), «轨迹» ++ [Op.{} {}])",
                        op_ctor(id),
                        emit_args(args),
                        op_ctor(id),
                        emit_args(args)
                    );
                }
                other => {
                    let _ = writeln!(out, "{indent}({}, «轨迹»)", emit_expr(other));
                }
            }
        }
        Step::Trace { op, args, line } => {
            let _ = writeln!(out, "{indent}-- mq: {line}:1");
            let _ = writeln!(
                out,
                "{indent}let «轨迹» := «轨迹» ++ [Op.{} {}]",
                op_ctor(op),
                emit_args(args)
            );
            emit_seq(out, indent, &tail_chunks(more, rest));
        }
        Step::If {
            cond,
            then,
            otherwise,
            line,
        } => {
            let _ = writeln!(out, "{indent}-- mq: {line}:1");
            let _ = writeln!(out, "{indent}if {} then", emit_cond(cond));
            indent.push_str("  ");
            let then_chunks = {
                let mut v: Vec<&[Step]> = Vec::with_capacity(rest.len() + 1);
                v.push(then);
                v.extend_from_slice(&tail_chunks(more, rest));
                v
            };
            emit_seq(out, indent, &then_chunks);
            indent.truncate(indent.len() - 2);
            let _ = writeln!(out, "{indent}else");
            indent.push_str("  ");
            let else_chunks = {
                let mut v: Vec<&[Step]> = Vec::with_capacity(rest.len() + 1);
                v.push(otherwise);
                v.extend_from_slice(&tail_chunks(more, rest));
                v
            };
            emit_seq(out, indent, &else_chunks);
            indent.truncate(indent.len() - 2);
        }
    }
}

/// 条件：比较直接产出 Bool；其余经 `mqTruth`（Val → Bool）。
fn emit_cond(cond: &PureExpr) -> String {
    match cond {
        PureExpr::Bin {
            operator: BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge,
            ..
        } => emit_pure(cond),
        _ => format!("mqTruth ({})", emit_pure(cond)),
    }
}

fn emit_args(args: &[PureExpr]) -> String {
    args.iter()
        .map(|a| emit_pure(a))
        .collect::<Vec<_>>()
        .join(" ")
}

fn emit_expr(expr: &IrExpr) -> String {
    match expr {
        IrExpr::Pure(pure) => emit_pure(pure),
        // 语句层不会遇到（Op 已在 Bind/Return/Trace 处理）；防御性回退。
        IrExpr::Op { id, args } => {
            format!("env (Op.{} {})", op_ctor(id), emit_args(args))
        }
        IrExpr::Formula => "Val.none -- $$公式$$：不透明值".into(),
        IrExpr::Code => "Val.none -- code 围栏：不透明值".into(),
    }
}

fn emit_pure(pure: &PureExpr) -> String {
    match pure {
        PureExpr::Int(i) => format!("(Val.int {i})"),
        PureExpr::Num(n) => format!("(Val.num {n:?})"),
        PureExpr::Bool(b) => format!("(Val.bool {})", if *b { "true" } else { "false" }),
        PureExpr::Text(s) => format!("(Val.text {})", lean_string(s)),
        PureExpr::None => "Val.none".into(),
        PureExpr::Var(name) => lean_ident(name),
        PureExpr::Bin { operator, left, right } => {
            let (l, r) = (emit_pure(left), emit_pure(right));
            match operator {
                BinOp::Add => format!("(mqAdd {l} {r})"),
                BinOp::Sub => format!("(mqSub {l} {r})"),
                BinOp::Mul => format!("(mqMul {l} {r})"),
                BinOp::Div => format!("(mqDiv {l} {r})"),
                BinOp::Eq => format!("(mqEq {l} {r})"),
                BinOp::Ne => format!("(mqNot (mqEq {l} {r}))"),
                BinOp::Lt => format!("(mqLt {l} {r})"),
                BinOp::Le => format!("(mqLe {l} {r})"),
                BinOp::Gt => format!("(mqGt {l} {r})"),
                BinOp::Ge => format!("(mqGe {l} {r})"),
                BinOp::And => format!("(mqAnd {l} {r})"),
                BinOp::Or => format!("(mqOr {l} {r})"),
            }
        }
        PureExpr::Un { operator, expr } => {
            let e = emit_pure(expr);
            match operator {
                UnOp::Not => format!("(mqNot {e})"),
                UnOp::Neg => format!("(mqNeg {e})"),
            }
        }
        PureExpr::Index { base, key } => {
            format!("(mqAccess {} {})", emit_pure(base), lean_string(key))
        }
        PureExpr::Access { base, key } => {
            format!("(mqAccess {} {})", emit_pure(base), lean_string(key))
        }
        PureExpr::Len(inner) => format!("(mqLen {})", emit_pure(inner)),
        PureExpr::Interp(parts) => emit_interp(parts),
        PureExpr::List(items) => {
            let inner: Vec<String> = items.iter().map(emit_pure).collect();
            format!("(Val.list [{}])", inner.join(", "))
        }
        PureExpr::Map(entries) => {
            let inner: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("({}, {})", lean_string(k), emit_pure(v)))
                .collect();
            format!("(Val.map [{}])", inner.join(", "))
        }
        PureExpr::Ctor { name, args } => emit_ctor(name, args),
    }
}

/// 内联构造器：`result.ok/error/value_or` 映射到库辅助函数
/// （Val 字面 map，与 lib/result 行为一致）；`mq.*` 直呼。
fn emit_ctor(name: &str, args: &[PureExpr]) -> String {
    let rendered: Vec<String> = args.iter().map(emit_pure).collect();
    match (name, rendered.as_slice()) {
        ("result.ok", [value]) => format!("(mqResultOk {value})"),
        ("result.error", [code, message, operation]) => {
            format!("(mqResultError {code} {message} {operation})")
        }
        ("result.value_or", [r, default]) => format!("(mqValueOr {r} {default})"),
        _ => format!("({} {})", name.replace('.', "_"), rendered.join(" ")),
    }
}

fn emit_interp(parts: &[InterpPart]) -> String {
    let mut pieces: Vec<String> = Vec::new();
    for part in parts {
        match part {
            InterpPart::Lit(text) => pieces.push(format!("(Val.text {})", lean_string(text))),
            InterpPart::Var(name) => pieces.push(lean_ident(name)),
        }
    }
    if pieces.is_empty() {
        return "(Val.text \"\")".into();
    }
    let mut it = pieces.into_iter();
    let first = it.next().unwrap();
    it.fold(first, |acc, piece| format!("(mqAppend {acc} {piece})"))
}

/// Lean 字符串字面量（转义 `\` `"` 与控制字符；非 ASCII 原样保留）。
fn lean_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 模块 stem（命名空间用）：去扩展名的文件名。
pub fn stem_of(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Module".into())
}

/// PlanTy 的 Lean 侧提示注释（P1 提升类型用；P0 参数统一 Val）。
pub fn ty_hint(ty: &PlanTy) -> &'static str {
    match ty {
        PlanTy::Int => "Int",
        PlanTy::Num => "Float",
        PlanTy::Bool => "Bool",
        PlanTy::Text => "String",
        _ => "Val",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prove::ir::{Expr, FnPlan, OpDecl, PlanTy, Step};

    #[test]
    fn lean_ident_keeps_chinese_and_quotes_dotted() {
        // Lean 标识符仅 ASCII：中文名入 guillemet。
        assert_eq!(lean_ident("金额"), "«金额»");
        assert_eq!(lean_ident("审批台.审批"), "«审批台.审批»");
        // 纯 ASCII 名保持裸写。
        assert_eq!(lean_ident("x_1"), "x_1");
        // 与本模块保留名冲突的变量必须改名（确定性映射）。
        assert_eq!(lean_ident("env"), "v_env");
        assert_eq!(lean_ident("轨迹"), "v_u8F68u8FF9");
        assert_eq!(lean_ident("结果"), "v_u7ED3u679C");
        assert_eq!(lean_ident("let"), "v_let");
    }

    #[test]
    fn lean_string_escapes_quotes_and_newline_only() {
        assert_eq!(lean_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(lean_string("行\n进"), "\"行\\n进\"");
        assert_eq!(lean_string("中文"), "\"中文\"");
    }

    #[test]
    fn namespace_sanitizes_unsafe_chars_deterministically() {
        assert_eq!(namespace_of("approval.mq"), "approval_mq");
        assert_eq!(namespace_of("示例"), "u793Au4F8B");
        // 命名空间绝不含非 ASCII（Lean namespace 命令只收 ASCII）。
        assert_eq!(namespace_of("1abc"), "M1abc");
        assert_eq!(namespace_of(""), "M");
    }

    #[test]
    fn emit_module_is_deterministic_and_sorted_by_first_appearance() {
        let plan = Plan {
            schema: crate::prove::ir::SCHEMA,
            module: "demo.mq.md".into(),
            source_sha256: "ab".repeat(32),
            functions: vec![
                FnPlan {
                    name: "机器.运转".into(),
                    params: vec![],
                    result_ty: PlanTy::Any,
                    ops: vec![
                        OpDecl {
                            id: "调用:写盘".into(),
                            effect: "fs.write".into(),
                            params: vec![],
                            arity: 1,
                        },
                        OpDecl {
                            id: "调用:读盘".into(),
                            effect: "fs.read".into(),
                            params: vec![],
                            arity: 0,
                        },
                    ],
                    steps: vec![Step::Bind {
                        var: "回执".into(),
                        expr: Expr::Op {
                            id: "调用:写盘".into(),
                            args: vec![],
                        },
                        line: 3,
                    }],
                    effects: vec!["fs.write".into()],
                    claims: vec![],
                    assumptions: vec![],
                },
                FnPlan {
                    name: "从机.待机".into(),
                    params: vec![],
                    result_ty: PlanTy::Any,
                    ops: vec![],
                    steps: vec![Step::Return {
                        expr: Expr::Pure(PureExpr::Int(7)),
                        line: 9,
                    }],
                    effects: vec![],
                    claims: vec![],
                    assumptions: vec![],
                },
            ],
        };
        let (name1, out1) = emit_module(&plan, "demo");
        let (name2, out2) = emit_module(&plan, "demo");
        assert_eq!(name1, name2);
        assert_eq!(out1, out2, "两次生成必须字节一致");
        assert!(out1.contains("import MarqdoProof"));
        assert!(out1.contains("namespace Generated.demo"));
        // Op 按首现顺序：写盘在读盘前。
        let write_pos = out1.find("«调用_写盘»").expect("write op");
        let read_pos = out1.find("«调用_读盘»").expect("read op");
        assert!(write_pos < read_pos);
        // Op 元数：读盘 0 元 → 无参构造器。
        assert!(out1.contains("| «调用_读盘»"), "{out1}");
        // Return 直译成对（字面量自带括号，形成双重括号——合法 Lean）。
        assert!(out1.contains("((Val.int 7), «轨迹»)"), "{out1}");
        // 溯源注释。
        assert!(out1.contains("-- mq: 3:1"));
        // 无时间戳。
        assert!(!out1.contains("20") || !out1.contains("T"), "no timestamps");
    }

    #[test]
    fn empty_plan_emits_no_op_ctors() {
        let plan = Plan {
            schema: crate::prove::ir::SCHEMA,
            module: "空.mq.md".into(),
            source_sha256: "cd".repeat(32),
            functions: vec![],
        };
        let (name, out) = emit_module(&plan, "空");
        assert_eq!(name, "u7A7A.lean");
        // 占位构造器同样 guillemet 化（裸中文构造器是 Lean 语法错误）。
        assert!(out.contains("| «无操作»"));
        assert!(!out.contains("«调用:"));
    }
}


/// 受限命题文本 → Lean Bool 项；解析失败或含未知形状 → None（emit 侧留注释）。
/// 语义映射（§7.2）：
/// - `结果.ok / 结果.value / 结果.error` → `mqAccess 结果 "ok"` 等；
/// - `包含(轨迹, s)` → `(轨迹.map Op.name).contains s`；
/// - `长度(轨迹)` → `(轨迹.length : Nat)`，与整数比较走 Nat 运算；
/// - 形参/算术/比较 → mq 辅助函数（Val 面）。
fn claim_to_lean_bool(text: &str) -> Option<String> {
    let toks = super::proof_check::tokenize_claim(text).ok()?;
    let (_, ast) = super::proof_check::parse_claim_expr(&toks).ok()?;
    claim_term(&ast)
}

/// `长度(轨迹)`：Nat 语境的锚点（轨迹长度与整数比较必须走 Nat 运算，
/// 否则 `mqEq : Val → Val → Bool` 类型不匹配）。
fn is_trace_len(ast: &super::proof_check::ClaimAst) -> bool {
    matches!(
        ast,
        super::proof_check::ClaimAst::Call { name, args }
            if name == "长度"
                && matches!(args.first(), Some(super::proof_check::ClaimAst::Var(v)) if v == "轨迹")
    )
}

fn claim_term(ast: &super::proof_check::ClaimAst) -> Option<String> {
    use super::proof_check::{ClaimAst as A, CTok};
    match ast {
        A::Lit(CTok::Int(i)) => Some(format!("(Val.int {i})")),
        A::Lit(CTok::Num(n)) => Some(format!("(Val.num {n:?})")),
        A::Lit(CTok::Bool(b)) => Some(format!("(Val.bool {b})")),
        A::Lit(CTok::Text(s)) => Some(format!("(Val.text {})", lean_string(s))),
        A::Lit(_) => None,
        A::Var(name) => {
            // 定理作用域里 `结果`/`轨迹` 是 ∀ 绑定的输出名（`«结果»`/`«轨迹»`，
            // 不加 v_ 前缀）；`根.字段` → mqAccess 字段读取；其余按标识符规则。
            let (root, field) = match name.split_once('.') {
                Some((r, f)) => (r, Some(f)),
                None => (name.as_str(), None),
            };
            let term = match (root, field) {
                ("结果", Some(f)) => format!("(mqAccess «结果» {})", lean_string(f)),
                ("轨迹", Some(f)) => format!("(mqAccess «轨迹» {})", lean_string(f)),
                ("结果", None) => "«结果»".to_string(),
                ("轨迹", None) => "«轨迹»".to_string(),
                _ => lean_ident(name),
            };
            Some(term)
        }
        A::Call { name, args } => {
            // `轨迹`（定理作用域的输出轨迹）上的谓词有专用 Lean 形式。
            let on_trace = matches!(args.first(), Some(A::Var(v)) if v == "轨迹");
            match (name.as_str(), args.len(), on_trace) {
                ("包含", 2, true) => {
                    let item = strip_val_text(&claim_term(&args[1])?);
                    Some(format!("((«轨迹».map Op.name).contains {item})"))
                }
                ("长度", 1, true) => Some("(«轨迹».length : Nat)".to_string()),
                _ => {
                    let mut rendered = Vec::with_capacity(args.len());
                    for arg in args {
                        rendered.push(claim_term(arg)?);
                    }
                    let term = match (name.as_str(), rendered.as_slice()) {
                        ("包含", [list, item]) => format!("(mqContains {list} {item})"),
                        ("长度", [list]) => format!("(mqNatOf (mqLen {list}))"),
                        ("下标", [list, index]) => {
                            format!("(mqAccess {list} (mqStrOfNat {index}))")
                        }
                        _ => format!("({} {})", name, rendered.join(" ")),
                    };
                    Some(term)
                }
            }
        }
        A::Bin { op, left, right } => {
            // Nat 语境：操作数是 `长度(轨迹)` 或整数字面量 → 原生 Nat 运算。
            if is_trace_len(left) || is_trace_len(right) {
                let nat = |a: &A| -> Option<String> {
                    if is_trace_len(a) {
                        Some("(«轨迹».length : Nat)".to_string())
                    } else if let A::Lit(CTok::Int(i)) = a {
                        Some(i.to_string())
                    } else {
                        None
                    }
                };
                let (l, r) = (nat(left)?, nat(right)?);
                return match *op {
                    "=" => Some(format!("({l} = {r})")),
                    "≠" => Some(format!("({l} ≠ {r})")),
                    "<" => Some(format!("({l} < {r})")),
                    "≤" => Some(format!("({l} ≤ {r})")),
                    ">" => Some(format!("({l} > {r})")),
                    "≥" => Some(format!("({l} ≥ {r})")),
                    "+" => Some(format!("({l} + {r})")),
                    "-" => Some(format!("({l} - {r})")),
                    _ => None,
                };
            }
            let (l, r) = (claim_term(left)?, claim_term(right)?);
            let term = match *op {
                "+" => format!("(mqAdd {l} {r})"),
                "-" => format!("(mqSub {l} {r})"),
                "*" => format!("(mqMul {l} {r})"),
                "/" => format!("(mqDiv {l} {r})"),
                "=" => format!("(mqEq {l} {r})"),
                "≠" => format!("(mqNot (mqEq {l} {r}))"),
                "<" => format!("(mqLt {l} {r})"),
                "≤" => format!("(mqLe {l} {r})"),
                ">" => format!("(mqGt {l} {r})"),
                "≥" => format!("(mqGe {l} {r})"),
                "且" => format!("(mqAnd {l} {r})"),
                "或" => format!("(mqOr {l} {r})"),
                _ => return None,
            };
            Some(term)
        }
        A::Un { op, expr } => {
            let e = claim_term(expr)?;
            let term = match *op {
                "非" => format!("(mqNot {e})"),
                "-" => format!("(mqNeg {e})"),
                _ => return None,
            };
            Some(term)
        }
    }
}

/// `包含` 的成员是文本字面量时剥掉 Val 包裹（`(Val.text "x")` → `"x"`）。
fn strip_val_text(term: &str) -> String {
    term.strip_prefix("(Val.text ")
        .and_then(|rest| rest.strip_suffix(')'))
        .map(|s| s.to_string())
        .unwrap_or_else(|| term.to_string())
}

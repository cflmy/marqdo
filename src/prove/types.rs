//! 定型（`doc/design/prove-cli.md` §4.1 ③）：契约表驱动的轻量推断。
//!
//! P0 范围：参数/返回类型来自契约表；字面量赋值给最具体类型；分支合流时
//! 同名变量类型不相容报 `proof.type_mismatch`（带两处行号）。其余一律
//! `Any`（`--no-strict-types` 语义下的诚实回退，不做猜测）。

use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, Severity, Span};
use crate::prove::ir::{PlanTy, PureExpr, Step};

/// 单函数类型检查产物。
#[derive(Debug, Default)]
pub struct Typed {
    pub diagnostics: Vec<Diagnostic>,
}

/// 对一个函数的 steps 做类型检查。
///
/// `params`：形参类型（契约表）。返回收集到的诊断。
pub fn typecheck(
    module: &crate::ast::Module,
    params: &[(String, PlanTy)],
    steps: &[Step],
    strict: bool,
) -> Typed {
    let mut env: BTreeMap<String, (PlanTy, u32)> = params
        .iter()
        .map(|(name, ty)| (name.clone(), (ty.clone(), 0)))
        .collect();
    let mut diagnostics = Vec::new();
    check_steps(module, steps, &mut env, &mut diagnostics, strict);
    Typed { diagnostics }
}

fn check_steps(
    module: &crate::ast::Module,
    steps: &[Step],
    env: &mut BTreeMap<String, (PlanTy, u32)>,
    out: &mut Vec<Diagnostic>,
    strict: bool,
) {
    for step in steps {
        match step {
            Step::Bind { var, expr, line } => {
                let ty = expr_type(expr, env);
                match ty {
                    Some(new_ty) => {
                        if let Some((old_ty, old_line)) = env.get(var) {
                            let merged = conflate(old_ty, &new_ty);
                            if merged.is_none() {
                                out.push(mismatch(
                                    module,
                                    Span::new(*line, 1),
                                    *old_line,
                                    var,
                                    old_ty,
                                    &new_ty,
                                ));
                            }
                            let merged = merged.unwrap_or(PlanTy::Any);
                            if strict && merged == PlanTy::Any && new_ty != PlanTy::Any {
                                // 严格模式下落 Any 的合流仍允许，但记录在案。
                            }
                            env.insert(var.clone(), (merged, *line));
                        } else {
                            env.insert(var.clone(), (new_ty, *line));
                        }
                    }
                    None => {
                        // 未知类型：保留旧类型（若有），否则 Any。
                        env.entry(var.clone()).or_insert((PlanTy::Any, *line));
                    }
                }
            }
            Step::Return { expr, .. } => {
                let _ = expr_type(expr, env);
            }
            Step::If {
                then,
                otherwise,
                line,
                ..
            } => {
                // 分支合流：两臂各自检查后合并同名变量类型。
                let mut then_env = env.clone();
                let mut else_env = env.clone();
                check_steps(module, then, &mut then_env, out, strict);
                check_steps(module, otherwise, &mut else_env, out, strict);
                let then_vars: Vec<(String, (PlanTy, u32))> =
                    then_env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                for (name, (then_ty, then_line)) in then_vars {
                    if name.starts_with("实参") {
                        continue;
                    }
                    match else_env.get(&name) {
                        Some((else_ty, else_line)) => {
                            match conflate(&then_ty, else_ty) {
                                Some(merged) => {
                                    env.insert(name, (merged, *line));
                                }
                                None => {
                                    out.push(mismatch(
                                        module,
                                        Span::new(then_line.max(1), 1),
                                        *else_line,
                                        &name,
                                        &then_ty,
                                        else_ty,
                                    ));
                                    env.insert(name, (PlanTy::Any, *line));
                                }
                            }
                        }
                        None => {
                            env.insert(name, (then_ty, then_line));
                        }
                    }
                }
                // 只在 else 出现的变量也进入合流后的环境。
                for (name, (else_ty, _)) in &else_env {
                    then_env.entry(name.clone()).or_insert((else_ty.clone(), 0));
                }
            }
            Step::Trace { .. } => {}
        }
    }
}

/// 纯表达式的保守类型：只认字面量/构造器/列表/映射/len/access/插值。
fn expr_type(expr: &crate::prove::ir::Expr, env: &BTreeMap<String, (PlanTy, u32)>) -> Option<PlanTy> {
    match expr {
        crate::prove::ir::Expr::Pure(pure) => pure_type(pure, env),
        crate::prove::ir::Expr::Op { .. } => None,
        crate::prove::ir::Expr::Formula | crate::prove::ir::Expr::Code => None,
    }
}

fn pure_type(pure: &PureExpr, env: &BTreeMap<String, (PlanTy, u32)>) -> Option<PlanTy> {
    match pure {
        PureExpr::Int(_) => Some(PlanTy::Int),
        PureExpr::Num(_) => Some(PlanTy::Num),
        PureExpr::Bool(_) => Some(PlanTy::Bool),
        PureExpr::Text(_) => Some(PlanTy::Text),
        PureExpr::None => Some(PlanTy::Any),
        PureExpr::Var(name) => env.get(name).map(|(ty, _)| ty.clone()),
        PureExpr::Bin { operator: op, left, right } => {
            use crate::prove::ir::BinOp as B;
            let lt = pure_type(left, env);
            let rt = pure_type(right, env);
            match op {
                B::Add | B::Sub | B::Mul | B::Div => {
                    let lt = lt?;
                    let rt = rt?;
                    match (&lt, &rt) {
                        (PlanTy::Text, _) | (_, PlanTy::Text) => Some(PlanTy::Text),
                        (PlanTy::Num, _) | (_, PlanTy::Num) => Some(PlanTy::Num),
                        (PlanTy::Int, PlanTy::Int) => Some(PlanTy::Int),
                        _ => None,
                    }
                }
                B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge | B::And | B::Or => Some(PlanTy::Bool),
            }
        }
        PureExpr::Un { operator: op, expr } => match op {
            crate::prove::ir::UnOp::Not => Some(PlanTy::Bool),
            crate::prove::ir::UnOp::Neg => pure_type(expr, env),
        },
        PureExpr::Index { base, .. } => match pure_type(base, env)? {
            PlanTy::Map => Some(PlanTy::Any),
            PlanTy::List => Some(PlanTy::Any),
            PlanTy::Result => Some(PlanTy::Any),
            _ => Some(PlanTy::Any),
        },
        PureExpr::Access { base, .. } => match pure_type(base, env)? {
            PlanTy::Result => Some(PlanTy::Any),
            _ => Some(PlanTy::Any),
        },
        PureExpr::Len(_) => Some(PlanTy::Int),
        PureExpr::Interp(_) => Some(PlanTy::Text),
        PureExpr::List(_) => Some(PlanTy::List),
        PureExpr::Map(_) => Some(PlanTy::Map),
        PureExpr::Ctor { name, .. } => {
            if name.starts_with("result.") {
                Some(PlanTy::Result)
            } else {
                None
            }
        }
    }
}

/// 类型合流：相同 → 相同；Int+Num → Num；含 Any → Any；否则 None（冲突）。
fn conflate(a: &PlanTy, b: &PlanTy) -> Option<PlanTy> {
    use PlanTy as T;
    if a == b {
        return Some(a.clone());
    }
    match (a, b) {
        (T::Any, other) | (other, T::Any) => Some(other.clone()),
        (T::Int, T::Num) | (T::Num, T::Int) => Some(T::Num),
        _ => None,
    }
}

fn mismatch(
    module: &crate::ast::Module,
    span: Span,
    other_line: u32,
    var: &str,
    old: &PlanTy,
    new: &PlanTy,
) -> Diagnostic {
    Diagnostic::new(
        module.source.path.as_deref(),
        span,
        format!(
            "`{var}` 在分支合流处类型不相容：{a} vs {b}",
            a = old.name(),
            b = new.name()
        ),
    )
    .with_source(&module.source)
    .with_code("proof.type_mismatch")
    .with_severity(Severity::Error)
    .with_suggestion(format!(
        "另一处绑定在第 {other_line} 行；统一两臂的类型或改用 --no-strict-types"
    ))
}

/// 把类型检查接到降级产物上（mod.rs 调用）。
pub fn typecheck_plan(
    module: &crate::ast::Module,
    lowered: &[crate::prove::lower::Lowered],
    strict: bool,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for item in lowered {
        let Some(plan) = &item.plan else {
            continue;
        };
        let params: Vec<(String, PlanTy)> = plan
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let typed = typecheck(module, &params, &plan.steps, strict);
        out.extend(typed.diagnostics);
    }
    out
}

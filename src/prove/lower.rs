//! 降级：AST → Plan IR（`doc/design/prove-cli.md` §4.1 ②）。
//!
//! 三条硬边界（设计 §2.2）——`while`/递归/动态派发（函数值、无法解析的裸名）
//! **拒绝，不降级**；静态可拼写的未知调用（方法、`工程.成员` 一类插件路径）
//! 一律不透明化为 [`ir::Expr::Op`]（诚实：行为完全未知，效应照 sema 标注）。
//! 本模块不执行任何代码，只读 AST。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;

use crate::ast::{self, Arg, BinaryOp, CallExpr, Expr, Function, Module, Stmt};
use crate::diagnostics::{Diagnostic, Severity, Span};
use crate::prove::ir::{
    AssumptionPlan, BinOp, ClaimPlan, Expr as IrExpr, FnPlan, InterpPart, OpDecl, ParamPlan, Plan, PlanTy,
    PureExpr, Step, UnOp, SCHEMA,
};
use crate::sema::bind::{call_display, resolve_for_exec};
use crate::host::HostFn;
use crate::sema::ir::EffectSet;
use crate::sema::resolution::Target;

/// 单个函数的降级产物（plan 或拒绝原因）。
#[derive(Debug)]
pub struct Lowered {
    pub display: String,
    /// 对象体（`# 类型`）不是可证单元：不证也不算拒绝。
    pub is_object: bool,
    pub plan: Option<FnPlan>,
    /// 全部拒绝原因（`proof.while` / `proof.recursion` / `proof.dynamic_call` / …）。
    pub rejected: Vec<Diagnostic>,
}

/// 降级上下文：模块 + 效应表（来自 sema；缺失时效应保守记 `unknown`）。
pub struct Lowerer<'m> {
    module: &'m Module,
    /// `(file, display_name) → effects`（sema 闭包 artifact 的源位置键）。
    effects: BTreeMap<(String, String), EffectSet>,
    allow_opaque: BTreeSet<String>,
}

impl<'m> Lowerer<'m> {
    pub fn new(
        module: &'m Module,
        effects: BTreeMap<(String, String), EffectSet>,
        _strict_types: bool,
        allow_opaque: &[String],
    ) -> Self {
        Self {
            module,
            effects,
            allow_opaque: allow_opaque.iter().cloned().collect(),
        }
    }

    /// 降级全部函数（含子函数），按源码顺序。
    pub fn lower_module(&self) -> Vec<Lowered> {
        let mut out = Vec::new();
        self.lower_functions(self.module.functions.as_slice(), "", &mut out);
        out
    }

    fn lower_functions(&self, funs: &[Function], prefix: &str, out: &mut Vec<Lowered>) {
        for fun in funs {
            let display = if prefix.is_empty() {
                fun.name.clone()
            } else {
                format!("{prefix}.{}", fun.name)
            };
            out.push(self.lower_function(&display, fun));
            self.lower_functions(&fun.children, &display, out);
        }
    }

    fn lower_function(&self, display: &str, fun: &Function) -> Lowered {
        let mut rejected = Vec::new();
        // 对象体（`# 类型`）不是可证单元：P0 只证函数/方法。
        if fun.is_object() {
            return Lowered {
                display: display.to_string(),
                is_object: true,
                plan: None,
                rejected,
            };
        }
        // `--allow-opaque <FN>`：整体降级为单个不透明操作，优先于子集验收
        // （作者显式声明"行为未知"，因此 while/递归也不再是拒绝理由）。
        if self.allow_opaque.contains(display) {
            return Lowered {
                display: display.to_string(),
                is_object: false,
                plan: Some(self.opaque_whole(display, fun)),
                rejected,
            };
        }
        // 子集验收先于一切：拒绝的函数不生成 plan。
        self.check_subset(fun, &mut rejected);
        if !rejected.is_empty() {
            return Lowered {
                display: display.to_string(),
                is_object: false,
                plan: None,
                rejected,
            };
        }

        let mut body = Body::default();
        for stmt in &fun.body {
            self.lower_stmt(fun, stmt, &mut body);
        }
        if !body.rejected.is_empty() {
            return Lowered {
                display: display.to_string(),
                is_object: false,
                plan: None,
                rejected: body.rejected,
            };
        }
        let plan = FnPlan {
            name: display.to_string(),
            params: fun
                .params
                .iter()
                .map(|p| ParamPlan {
                    name: p.name.clone(),
                    ty: self.param_ty(fun, &p.name),
                    has_default: p.default.is_some(),
                })
                .collect(),
            result_ty: self.result_ty(fun),
            ops: body.ops,
            steps: body.steps,
            effects: body
                .op_effects
                .iter()
                .filter(|e| e.as_str() != "pure")
                .cloned()
                .collect(),
            claims: fun
                .claims
                .iter()
                .map(|c| ClaimPlan {
                    id: c.id.clone(),
                    premise: c.premise.clone(),
                    conclusion: c.conclusion.clone(),
                    tactic: c.tactic.clone(),
                    line: c.line,
                })
                .collect(),
            assumptions: fun.assumptions.iter().map(|a|AssumptionPlan{
                op:a.op.clone(), premise:a.premise.clone(),
                postcondition:a.postcondition.clone()
            }).collect(),
        };
        Lowered {
            display: display.to_string(),
            is_object: false,
            plan: Some(plan),
            rejected,
        }
    }

    fn opaque_whole(&self, display: &str, fun: &Function) -> FnPlan {
        // 行为完全未知：效应标签一律 `opaque`（§4.5，比猜测诚实）。
        let effect = "opaque".to_string();
        let mut ops = Vec::new();
        let id = format!("调用:{display}");
        ops.push(OpDecl {
            id: id.clone(),
            effect,
            params: Vec::new(),
            arity: fun.params.len(),
        });
        FnPlan {
            name: display.to_string(),
            params: fun
                .params
                .iter()
                .map(|p| ParamPlan {
                    name: p.name.clone(),
                    ty: PlanTy::Any,
                    has_default: p.default.is_some(),
                })
                .collect(),
            result_ty: PlanTy::Any,
            ops,
            steps: vec![Step::Trace {
                op: id,
                args: fun
                    .params
                    .iter()
                    .map(|p| PureExpr::Var(p.name.clone()))
                    .collect(),
                line: fun.span.line,
            }],
            effects: vec!["opaque".into()],
            claims: Vec::new(),
            assumptions: Vec::new(),
        }
    }

    // ---------------------------------------------------------------- 子集验收

    /// 子集验收（§4.3 拒绝侧）。收集全部原因，不止第一条。
    fn check_subset(&self, fun: &Function, rejected: &mut Vec<Diagnostic>) {
        for stmt in &fun.body {
            self.check_stmt(stmt, fun, rejected);
        }
        // 递归：本函数显示名出现在自身可静态解析的调用闭包里。
        let display = self.display_of(fun);
        if self.recurses(fun, &display, &mut BTreeSet::new()) {
            rejected.push(reject(
                self.module,
                fun.span,
                "proof.recursion",
                format!("`{display}` 递归调用不在可证子集（需要归纳证明，超出 P0–P3 范围）"),
            ));
        }
    }

    fn check_stmt(&self, stmt: &Stmt, fun: &Function, rejected: &mut Vec<Diagnostic>) {
        match stmt {
            Stmt::While { span, .. } => rejected.push(reject(
                self.module,
                *span,
                "proof.while",
                format!(
                    "`-` 循环不在可证子集：`{}` 被跳过（无循环 ⇒ 返回路径有限）",
                    fun.name
                ),
            )),
            Stmt::ForEach { span, .. } => rejected.push(reject(
                self.module,
                *span,
                "proof.foreach_pending",
                "foreach 在 P3 才进入可证子集（需要列表有限性证据）",
            )),
            Stmt::Assign { value, span, .. } => self.check_expr(value, fun, *span, rejected),
            Stmt::Return { value, span } => self.check_expr(value, fun, *span, rejected),
            Stmt::Expr { value, span } => self.check_expr(value, fun, *span, rejected),
            Stmt::Call { call, span } => self.check_call(call, fun, *span, rejected),
            Stmt::Branch { arms, .. } => {
                for arm in arms {
                    if let Some(cond) = &arm.condition {
                        // 条件的 span 在解析期未单独记录，用函数级锚点替代。
                        self.check_expr(cond, fun, fun.span, rejected);
                    }
                    for s in &arm.body {
                        self.check_stmt(s, fun, rejected);
                    }
                }
            }
        }
    }

    fn check_expr(&self, expr: &Expr, fun: &Function, span: Span, rejected: &mut Vec<Diagnostic>) {
        match expr {
            Expr::Call(call) => self.check_call(call, fun, span, rejected),
            Expr::Occluded { payload, .. } => {
                if let Some(payload) = payload {
                    self.check_expr(payload, fun, span, rejected);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr(left, fun, span, rejected);
                self.check_expr(right, fun, span, rejected);
            }
            Expr::Unary { expr, .. } => self.check_expr(expr, fun, span, rejected),
            Expr::List(items) => {
                for item in items {
                    self.check_expr(item, fun, span, rejected);
                }
            }
            Expr::Map(entries) => {
                for (_, value) in entries {
                    self.check_expr(value, fun, span, rejected);
                }
            }
            Expr::Index { base, .. } => self.check_expr(base, fun, span, rejected),
            Expr::Interp(parts) => {
                for part in parts {
                    if let ast::InterpPart::Index { .. } = part {
                        rejected.push(reject(
                            self.module,
                            span,
                            "proof.unsupported",
                            "插值中的下标读取不在 P0 纯片段",
                        ));
                    }
                }
            }
            Expr::Var(_) | Expr::Literal(_) | Expr::Formula(_) | Expr::Code(_) => {}
        }
    }

    fn check_call(&self, call: &CallExpr, fun: &Function, span: Span, rejected: &mut Vec<Diagnostic>) {
        for arg in &call.args {
            let value = match arg {
                Arg::Positional(value) | Arg::Named { value, .. } => value,
            };
            self.check_expr(value, fun, span, rejected);
        }
        match resolve_for_exec(self.module, fun, call) {
            Target::Local(target) => {
                if target.is_object() {
                    rejected.push(reject(
                        self.module,
                        span,
                        "proof.dynamic_call",
                        format!("对象构造 `{}` 不在 P0 纯片段", call_display(call)),
                    ));
                }
            }
            Target::Import { .. } | Target::Builtin(_) | Target::Host(_) => {}
            Target::Dynamic { reason, name } => {
                use crate::sema::bind::DynamicReason as R;
                match reason {
                    R::CallFn | R::Plugin => rejected.push(reject(
                        self.module,
                        span,
                        "proof.dynamic_call",
                        format!("`{name}`：函数值/未知裸名调用不在可证子集"),
                    )),
                    // 方法调用（`` `recv`.m ``）与两段静态路径（`工程.成员`）都是
                    // 静态可拼写的调用点 → 不透明操作，不拒绝（§4.3）。
                    R::MethodReceiver => {}
                }
            }
            Target::Unresolved { message, .. } => {
                rejected.push(reject(
                    self.module,
                    span,
                    "proof.dynamic_call",
                    format!("`{}` 无法静态解析：{message}", call_display(call)),
                ));
            }
        }
    }

    fn display_of(&self, fun: &Function) -> String {
        fn walk<'a>(
            funs: &'a [Function],
            prefix: &str,
            target: &Function,
            out: &mut Option<String>,
        ) {
            if out.is_some() {
                return;
            }
            for f in funs {
                let display = if prefix.is_empty() {
                    f.name.clone()
                } else {
                    format!("{prefix}.{}", f.name)
                };
                if std::ptr::eq(f, target) {
                    *out = Some(display.clone());
                    return;
                }
                walk(&f.children, &display, target, out);
            }
        }
        let mut found = None;
        walk(&self.module.functions, "", fun, &mut found);
        found.unwrap_or_else(|| fun.name.clone())
    }

    /// 递归检测：从 `fun` 出发的可静态解析局部调用是否回到 `display`。
    fn recurses(&self, fun: &Function, display: &str, visiting: &mut BTreeSet<String>) -> bool {
        if !visiting.insert(display.to_string()) {
            return true;
        }
        let mut found = false;
        let calls = self.all_calls(fun);
        for (call, callee_fun) in calls {
            if let Target::Local(target) = resolve_for_exec(self.module, callee_fun, call) {
                let target_display = self.display_of(target);
                if target_display == display || self.recurses(target, &target_display, visiting) {
                    found = true;
                    break;
                }
            }
        }
        visiting.remove(display);
        found
    }

    /// 函数体里每个调用点及其**所在函数**（解析作用域）。
    fn all_calls<'a>(&'a self, fun: &'a Function) -> Vec<(&'a CallExpr, &'a Function)> {
        let mut out = Vec::new();
        for stmt in &fun.body {
            self.collect_stmt_calls(stmt, fun, &mut out);
        }
        out
    }

    fn collect_stmt_calls<'a>(
        &'a self,
        stmt: &'a Stmt,
        owner: &'a Function,
        out: &mut Vec<(&'a CallExpr, &'a Function)>,
    ) {
        match stmt {
            Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::Expr { value, .. } => {
                for call in collect_calls(value) {
                    out.push((call, owner));
                }
            }
            Stmt::Call { call, .. } => out.push((call, owner)),
            Stmt::Branch { arms, .. } => {
                for arm in arms {
                    if let Some(cond) = &arm.condition {
                        for call in collect_calls(cond) {
                            out.push((call, owner));
                        }
                    }
                    for s in &arm.body {
                        self.collect_stmt_calls(s, owner, out);
                    }
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------- 语句降级

    fn lower_stmt(&self, fun: &Function, stmt: &Stmt, body: &mut Body) {
        match stmt {
            Stmt::Assign { name, value, span, .. } => {
                let expr = self.lower_expr(value, fun, span.line, body);
                body.steps.push(Step::Bind {
                    var: name.clone(),
                    expr,
                    line: span.line,
                });
            }
            Stmt::Return { value, span } => {
                let expr = self.lower_expr(value, fun, span.line, body);
                body.steps.push(Step::Return {
                    expr,
                    line: span.line,
                });
            }
            Stmt::Call { call, span } => {
                // 语句形式调用：值丢弃，但有效应的调用必须留下轨迹。
                if let IrExpr::Op { id, args } = self.lower_call_expr(call, fun, span.line, body) {
                    body.steps.push(Step::Trace {
                        op: id,
                        args,
                        line: span.line,
                    });
                }
            }
            Stmt::Expr { value, span } => {
                if let IrExpr::Op { id, args } = self.lower_expr(value, fun, span.line, body) {
                    body.steps.push(Step::Trace { op: id, args, line: span.line });
                }
            }
            Stmt::Branch { arms, span } => {
                let step = self.lower_branch(arms, span.line, fun, body);
                body.steps.extend(step);
            }
            Stmt::While { span, .. } => body.rejected.push(reject(
                self.module,
                *span,
                "proof.while",
                "循环不在可证子集",
            )),
            Stmt::ForEach { span, .. } => body.rejected.push(reject(
                self.module,
                *span,
                "proof.foreach_pending",
                "foreach 在 P3 才进入可证子集",
            )),
        }
    }

    /// 多臂分支 → 嵌套 if/else（保持源顺序；`1. 条件 / 2. *` 是两臂常态）。
    fn lower_branch(
        &self,
        arms: &[ast::BranchArm],
        line: u32,
        fun: &Function,
        body: &mut Body,
    ) -> Vec<Step> {
        let Some(first) = arms.first() else {
            return Vec::new();
        };
        let Some(cond) = &first.condition else {
            // 首臂即 else（无条件的 `1. *`）：平铺为主体。
            let before = body.steps.len();
            for s in &first.body {
                self.lower_stmt(fun, s, body);
            }
            return body.steps.split_off(before);
        };
        let cond_ir = self.lower_pure_required(cond, fun, line, body);
        let then = {
            let mut sub = Body {
                inline_depth: body.inline_depth,
                ..Body::default()
            };
            for s in &first.body {
                self.lower_stmt(fun, s, &mut sub);
            }
            let steps = std::mem::take(&mut sub.steps);
            body.absorb(sub);
            steps
        };
        let otherwise = if arms.len() > 1 {
            self.lower_branch(&arms[1..], line, fun, body)
        } else {
            Vec::new()
        };
        vec![Step::If {
            cond: cond_ir,
            then,
            otherwise,
            line,
        }]
    }

    // ---------------------------------------------------------------- 表达式降级

    fn lower_expr(&self, expr: &Expr, fun: &Function, line: u32, body: &mut Body) -> IrExpr {
        match expr {
            Expr::Call(call) => self.lower_call_expr(call, fun, line, body),
            Expr::Occluded { payload, .. } => match payload {
                Some(payload) => self.lower_expr(payload, fun, line, body),
                None => IrExpr::Pure(PureExpr::None),
            },
            other => match self.lower_pure(other, fun, body) {
                Some(pure) => IrExpr::Pure(pure),
                None => match other {
                    // `$$…$$` 公式与 code 围栏：不透明值（设计 §4.3，不翻译）。
                    Expr::Formula(_) => IrExpr::Formula,
                    Expr::Code(_) => IrExpr::Code,
                    // 动态键读取 `` [`k`](m) ``：不透明读操作。
                    Expr::Index { label: ast::IndexKey::Var(name), .. } => {
                        let id = body.register_op(
                            &format!("读取:`{name}`"),
                            "pure".to_string(),
                            0,
                        );
                        IrExpr::Op { id, args: Vec::new() }
                    }
                    _ => {
                        // 诚实边界（审查修复）：字面量容器（列表/映射/插值）里的调用
                        // 不能变成纯片段的一部分，也不许静默丢弃——静默丢弃会让
                        // 含副作用调用的函数伪装成"无操作"。拒绝并提示改写。
                        if !collect_calls(other).is_empty() {
                            body.rejected.push(reject(
                                self.module,
                                Span::new(line, 1),
                                "proof.unsupported",
                                "列表/映射字面量里的调用不在 P0 纯片段：先把调用绑定到变量（`x` = [调用](…)`），再放进字面量",
                            ));
                        }
                        IrExpr::Pure(PureExpr::None)
                    }
                },
            },
        }
    }

    /// 调用 → 内联构造器 / 纯内建 / 不透明 Op。
    fn lower_call_expr(&self, call: &CallExpr, fun: &Function, line: u32, body: &mut Body) -> IrExpr {
        let mut args: Vec<PureExpr> = Vec::with_capacity(call.args.len());
        for arg in call.args.iter() {
            let value = match arg {
                Arg::Positional(value) | Arg::Named { value, .. } => value,
            };
            args.push(self.lower_arg(value, fun, line, body));
        }
        match resolve_for_exec(self.module, fun, call) {
            Target::Builtin(name) => lower_builtin(name, args, body),
            Target::Host(host) => {
                if host_is_pure(host) {
                    // Access 特形（字段读取）或 mq.* 构造器。
                    return lower_host_pure(host, args);
                }
                let id = body.register_op(&call_display(call), effect_label_of_host(host), args.len());
                IrExpr::Op { id, args }
            }
            Target::Local(target) => {
                let display = self.display_of(target);
                // 自递归（防御；子集验收已拒绝）或超深内联 → 不透明。
                if std::ptr::eq(target, fun) || body.inline_depth >= 4 {
                    let id = body.register_op(&call_display(call), self.effect_of("self", &display), args.len());
                    return IrExpr::Op { id, args };
                }
                if self.is_inline_candidate(target) {
                    body.inline_depth += 1;
                    let lowered = self.inline_body(target, &args, body);
                    body.inline_depth -= 1;
                    lowered
                } else {
                    let id = body.register_op(&call_display(call), self.effect_of("self", &display), args.len());
                    IrExpr::Op { id, args }
                }
            }
            Target::Import { bind, lib, fun: target, .. } => {
                // Source-level verification of a tightly restricted imported
                // pure leaf: one return expression, only literal/parameter
                // arithmetic, exact positional args. This is NOT abstract
                // contract reuse; Lean verifies the actual substituted body.
                if body.inline_depth < 4
                    && imported_pure_leaf(target)
                    && call.args.len()==target.params.len()
                    && call.args.iter().all(|a|matches!(a,Arg::Positional(_)))
                {
                    body.inline_depth+=1;
                    let lowered=self.inline_body(target,&args,body);
                    body.inline_depth-=1;
                    return lowered;
                }
                // lib/result 的构造器内联为结构（§4.5；真实库名是 ok/error/value_or）。
                if bind == "result" && matches!(target.name.as_str(), "ok" | "error" | "value_or") {
                    return IrExpr::Pure(PureExpr::Ctor {
                        name: format!("result.{}", target.name),
                        args,
                    });
                }
                let display = if call.path.is_some() {
                    call_display(call)
                } else {
                    format!("{bind}.{}", target.name)
                };
                let effect = self.effect_of(&source_file(lib), &display);
                let effect = if effect == "unknown" {
                    imported_effect_label(lib, target)
                } else {
                    effect
                };
                let id = body.register_op(&display, effect, args.len());
                IrExpr::Op { id, args }
            }
            // 方法调用 / 两段静态路径：不透明（§4.3），接收者不展开。
            Target::Dynamic { name, .. } => {
                let id = body.register_op(&name, "unknown".to_string(), args.len());
                IrExpr::Op { id, args }
            }
            Target::Unresolved { message, .. } => {
                body.rejected.push(reject(
                    self.module,
                    Span::new(0, 0),
                    "proof.dynamic_call",
                    format!("`{}` 无法静态解析：{message}", call_display(call)),
                ));
                IrExpr::Pure(PureExpr::None)
            }
        }
    }

    /// 实参降级：纯表达式直接用；不透明子调用先绑定临时变量再引用。
    /// 临时名全局递增（`实参N`），避免内联展开时遮蔽调用者的同名变量。
    fn lower_arg(&self, value: &Expr, fun: &Function, line: u32, body: &mut Body) -> PureExpr {
        if let Some(pure) = self.lower_pure(value, fun, body) {
            return pure;
        }
        match self.lower_expr(value, fun, line, body) {
            // 嵌套纯调用（lower_pure 因解析上下文失败的回退路径）。
            IrExpr::Pure(pure) => pure,
            IrExpr::Op { id, args } => {
                body.temps_count += 1;
                let temp = format!("实参{}", body.temps_count);
                body.steps.push(Step::Bind {
                    var: temp.clone(),
                    expr: IrExpr::Op { id, args },
                    line: 0,
                });
                PureExpr::Var(temp)
            }
            _ => PureExpr::None,
        }
    }

    /// §4.5 内联判据（P0 保守版）：无分支的直线函数 + sema 效应 pure + 非递归。
    /// 含分支的被调在 P0 一律不透明化（符号执行落地后在 P2 放宽）。
    fn is_inline_candidate(&self, target: &Function) -> bool {
        if target.is_object() || has_branching(&target.body) {
            return false;
        }
        let stmt_count = count_stmts(&target.body);
        if stmt_count == 0 || stmt_count > 24 {
            return false;
        }
        let display = self.display_of(target);
        self.effect_of("self", &display) == "pure"
    }

    /// 内联直线函数体：形参 → 实参替换，语句平铺；返回值即调用值。
    /// op 声明按 id 去重共享；步骤先替换形参再并入主体。
    fn inline_body(&self, target: &Function, args: &[PureExpr], body: &mut Body) -> IrExpr {
        let mut result = IrExpr::Pure(PureExpr::None);
        let mut sub = Body {
            inline_depth: body.inline_depth,
            temps_count: body.temps_count,
            ..Body::default()
        };
        for stmt in &target.body {
            match stmt {
                Stmt::Return { value, .. } => {
                    result = self.lower_expr(value, target, crate::ast::stmt_line(stmt), &mut sub);
                }
                _ => self.lower_stmt(target, stmt, &mut sub),
            }
        }
        let params: Vec<String> = target.params.iter().map(|p| p.name.clone()).collect();
        for step in sub.steps {
            body.steps.push(substitute_step(&step, &params, args));
        }
        for op in sub.ops {
            if body.op_ids.insert(op.id.clone()) {
                body.op_effects.insert(op.effect.clone());
                body.ops.push(op);
            }
        }
        body.op_effects.extend(sub.op_effects);
        body.rejected.extend(sub.rejected);
        body.temps_count = body.temps_count.max(sub.temps_count);
        substitute_expr(&result, &params, args)
    }

    fn effect_of(&self, file: &str, display: &str) -> String {
        self.effects
            .get(&(file.to_string(), display.to_string()))
            .map(effect_label)
            .unwrap_or_else(|| "unknown".into())
    }

    fn param_ty(&self, fun: &Function, name: &str) -> PlanTy {
        if let Some(contract) = &fun.contract {
            if let Some(param) = contract.params.iter().find(|p| p.name == name) {
                return PlanTy::from_contract(&param.ty);
            }
        }
        PlanTy::Any
    }

    fn result_ty(&self, fun: &Function) -> PlanTy {
        if let Some(contract) = &fun.contract {
            if let Some(ret) = &contract.returns {
                return PlanTy::from_contract(&ret.ty);
            }
        }
        PlanTy::Any
    }

    // ---------------------------------------------------------------- 纯片段

    /// 纯表达式降级：能翻译返回 Some；不能返回 None（调用方决定不透明化）。
    fn lower_pure(&self, expr: &Expr, fun: &Function, body: &mut Body) -> Option<PureExpr> {
        match expr {
            Expr::Literal(ast::Literal::None) => Some(PureExpr::None),
            Expr::Literal(ast::Literal::Bool(b)) => Some(PureExpr::Bool(*b)),
            Expr::Literal(ast::Literal::Int(i)) => Some(PureExpr::Int(*i)),
            Expr::Literal(ast::Literal::Num(n)) => Some(PureExpr::Num(*n)),
            Expr::Literal(ast::Literal::Text(s)) => Some(PureExpr::Text(s.clone())),
            Expr::Var(name) => Some(PureExpr::Var(name.clone())),
            Expr::Binary { op, left, right } => Some(PureExpr::Bin {
                operator: bin_op(*op),
                left: Box::new(self.lower_pure(left, fun, body)?),
                right: Box::new(self.lower_pure(right, fun, body)?),
            }),
            Expr::Unary { op, expr } => Some(PureExpr::Un {
                operator: match op {
                    ast::UnaryOp::Not => UnOp::Not,
                    ast::UnaryOp::Neg => UnOp::Neg,
                },
                expr: Box::new(self.lower_pure(expr, fun, body)?),
            }),
            Expr::List(items) => Some(PureExpr::List(
                items
                    .iter()
                    .map(|item| self.lower_pure(item, fun, body))
                    .collect::<Option<Vec<_>>>()?,
            )),
            Expr::Map(entries) => Some(PureExpr::Map(
                entries
                    .iter()
                    .map(|(k, v)| Some((k.clone(), self.lower_pure(v, fun, body)?)))
                    .collect::<Option<Vec<_>>>()?,
            )),
            Expr::Index { base, label } => {
                let key = match label {
                    ast::IndexKey::Lit(key) => key.clone(),
                    ast::IndexKey::Var(_) => return None, // 动态键 → 不透明读
                };
                Some(PureExpr::Index {
                    base: Box::new(self.lower_pure(base, fun, body)?),
                    key,
                })
            }
            Expr::Interp(parts) => Some(PureExpr::Interp(
                parts
                    .iter()
                    .map(|part| match part {
                        ast::InterpPart::Lit(text) => Some(InterpPart::Lit(text.clone())),
                        ast::InterpPart::Var(name) => Some(InterpPart::Var(name.clone())),
                        ast::InterpPart::Index { .. } => None,
                    })
                    .collect::<Option<Vec<_>>>()?,
            )),
            Expr::Call(call) => {
                let args: Option<Vec<PureExpr>> = call
                    .args
                    .iter()
                    .map(|arg| match arg {
                        Arg::Positional(value) | Arg::Named { value, .. } => {
                            self.lower_pure(value, fun, body)
                        }
                    })
                    .collect();
                let args = args?;
                match resolve_for_exec(self.module, fun, call) {
                    Target::Builtin(name) if builtin_is_pure(name) => {
                        Some(match lower_builtin_pure(name, args) {
                            IrExpr::Pure(pure) => pure,
                            _ => return None,
                        })
                    }
                    Target::Host(host) if host_is_pure(host) => {
                        Some(match lower_host_pure(host, args) {
                            IrExpr::Pure(pure) => pure,
                            _ => return None,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// 分支条件必须是纯表达式（§4.3）；不是就报 unsupported。
    fn lower_pure_required(&self, expr: &Expr, fun: &Function, line: u32, body: &mut Body) -> PureExpr {
        match self.lower_pure(expr, fun, body) {
            Some(pure) => pure,
            None => {
                body.rejected.push(reject(
                    self.module,
                    Span::new(line, 1),
                    "proof.unsupported",
                    "分支条件不在纯片段（只接受字面量/变量/比较/布尔）",
                ));
                PureExpr::None
            }
        }
    }
}

// ---------------------------------------------------------------- 内建与效应

/// 纯内建（可进纯片段）。
fn builtin_is_pure(name: &str) -> bool {
    matches!(
        name,
        "len" | "str" | "int" | "type" | "trim" | "split" | "join" | "at"
    )
}

/// `print`/`input` 有效应，按不透明操作处理；其余内建进纯片段。
fn lower_builtin(name: &str, args: Vec<PureExpr>, body: &mut Body) -> IrExpr {
    match name {
        "print" => {
            let id = body.register_op("打印", "output.write", 1);
            IrExpr::Op { id, args }
        }
        "input" => {
            let id = body.register_op("输入", "user.input", 1);
            IrExpr::Op { id, args }
        }
        _ => lower_builtin_pure(name, args),
    }
}

fn lower_builtin_pure(name: &str, args: Vec<PureExpr>) -> IrExpr {
    match (name, args.as_slice()) {
        ("len", [only]) => IrExpr::Pure(PureExpr::Len(Box::new(only.clone()))),
        ("at", [base, key]) => IrExpr::Pure(PureExpr::Ctor {
            name: "mq.at".into(),
            args: vec![base.clone(), key.clone()],
        }),
        _ => IrExpr::Pure(PureExpr::Ctor {
            name: format!("mq.{name}"),
            args,
        }),
    }
}

/// 纯 Host 函数进纯片段；`access`/`取`/`取值` 是字段读取的标准形。
fn lower_host_pure(host: HostFn, args: Vec<PureExpr>) -> IrExpr {
    if matches!(host, HostFn::Access) {
        if let [base, PureExpr::Text(key)] = args.as_slice() {
            return IrExpr::Pure(PureExpr::Access {
                base: Box::new(base.clone()),
                key: key.clone(),
            });
        }
    }
    IrExpr::Pure(PureExpr::Ctor {
        name: format!("mq.{}", host.name()),
        args,
    })
}

fn host_is_pure(host: HostFn) -> bool {
    host.effects() == ["pure"]
}

fn effect_label_of_host(host: HostFn) -> String {
    let effects = host.effects();
    if effects == ["pure"] {
        "pure".into()
    } else {
        effects
            .first()
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| "unknown".into())
    }
}

/// Imported source transparency is enabled only for a pure expression
/// fragment that cannot inspect module globals or call external operations.
fn imported_pure_leaf(fun:&Function)->bool {
    fn closed(expr:&Expr,params:&std::collections::BTreeSet<&str>)->bool {
        match expr {
            Expr::Literal(_)=>true,
            Expr::Var(v)=>params.contains(v.as_str()),
            Expr::Occluded {payload:Some(inner),..}=>closed(inner,params),
            Expr::Unary {expr,..}=>closed(expr,params),
            Expr::Binary {left,right,..}=>closed(left,params)&&closed(right,params),
            _=>false,
        }
    }
    if fun.is_object() || fun.body.len()!=1{return false}
    let params:std::collections::BTreeSet<&str>=fun.params.iter()
        .map(|p|p.name.as_str()).collect();
    match &fun.body[0]{
        Stmt::Return {value,..}=>closed(value,&params),
        _=>false,
    }
}

/// 无 sema 效应表时的库函数效应回退（与 sema 的 imported_effects 同思路的保守版）。
fn imported_effect_label(lib: &Module, fun: &Function) -> String {
    let mut visiting = std::collections::HashSet::new();
    imported_effect_impl(lib, fun, &mut visiting)
}

fn imported_effect_impl(
    module: &Module,
    fun: &Function,
    visiting: &mut std::collections::HashSet<(Option<std::path::PathBuf>, u32)>,
) -> String {
    let identity = (module.source.path.clone(), fun.span.line);
    if !visiting.insert(identity) {
        return "pure".into();
    }
    let mut effect = "pure".to_string();
    for stmt in &fun.body {
        let calls: Vec<&CallExpr> = match stmt {
            Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::Expr { value, .. } => {
                collect_calls(value)
            }
            Stmt::Call { call, .. } => vec![call],
            _ => vec![],
        };
        for call in calls {
            match resolve_for_exec(module, fun, call) {
                Target::Builtin(name) => {
                    if !builtin_is_pure(name) {
                        effect = if name == "print" { "output.write" } else { "user.input" }.into();
                    }
                }
                Target::Host(host) => {
                    let label = effect_label_of_host(host);
                    if label != "pure" {
                        effect = label;
                    }
                }
                Target::Local(target) => {
                    let label = imported_effect_impl(module, target, visiting);
                    if label != "pure" {
                        effect = label;
                    }
                }
                Target::Import { lib: inner, fun: target, .. } => {
                    let label = imported_effect_impl(inner, target, visiting);
                    if label != "pure" {
                        effect = label;
                    }
                }
                _ => effect = "unknown".into(),
            }
        }
    }
    effect
}

fn effect_label(effects: &EffectSet) -> String {
    if effects.is_pure() {
        "pure".into()
    } else {
        effects
            .values()
            .into_iter()
            .find(|e| e.as_str() != "pure" && e.as_str() != "unknown")
            .unwrap_or_else(|| {
                if effects.contains("unknown") {
                    "unknown".into()
                } else {
                    "pure".into()
                }
            })
    }
}

fn source_file(module: &Module) -> String {
    module
        .source
        .path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn bin_op(op: BinaryOp) -> BinOp {
    use BinaryOp as A;
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

fn collect_calls(expr: &Expr) -> Vec<&CallExpr> {
    let mut out = Vec::new();
    fn go<'a>(expr: &'a Expr, out: &mut Vec<&'a CallExpr>) {
        match expr {
            Expr::Call(call) => {
                out.push(call);
                for arg in &call.args {
                    match arg {
                        Arg::Positional(value) | Arg::Named { value, .. } => go(value, out),
                    }
                }
            }
            Expr::Binary { left, right, .. } => {
                go(left, out);
                go(right, out);
            }
            Expr::Unary { expr, .. } => go(expr, out),
            Expr::List(items) => items.iter().for_each(|e| go(e, out)),
            Expr::Map(entries) => entries.iter().for_each(|(_, v)| go(v, out)),
            Expr::Index { base, .. } => go(base, out),
            Expr::Occluded { payload, .. } => {
                if let Some(p) = payload {
                    go(p, out)
                }
            }
            _ => {}
        }
    }
    go(expr, &mut out);
    out
}

fn has_branching(stmts: &[Stmt]) -> bool {
    fn go(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Branch { arms, .. } => {
                arms.iter().any(|arm| arm.body.iter().any(go))
            }
            Stmt::While { body, .. } | Stmt::ForEach { body, .. } => body.iter().any(go),
            _ => false,
        }
    }
    stmts.iter().any(go)
}

fn count_stmts(stmts: &[Stmt]) -> usize {
    fn go(stmt: &Stmt) -> usize {
        1 + match stmt {
            Stmt::Branch { arms, .. } => arms
                .iter()
                .map(|arm| arm.body.iter().map(go).sum::<usize>())
                .sum(),
            _ => 0,
        }
    }
    stmts.iter().map(go).sum()
}

fn reject(module: &Module, span: Span, code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(module.source.path.as_deref(), span, message)
        .with_source(&module.source)
        .with_code(code)
        .with_severity(Severity::Warning)
}

// ---------------------------------------------------------------- 实参替换

fn substitute_step(step: &Step, params: &[String], args: &[PureExpr]) -> Step {
    match step {
        Step::Bind { var, expr, line } => Step::Bind {
            var: var.clone(),
            expr: substitute_expr(expr, params, args),
            line: *line,
        },
        Step::Return { expr, line } => Step::Return {
            expr: substitute_expr(expr, params, args),
            line: *line,
        },
        Step::If {
            cond,
            then,
            otherwise,
            line,
        } => Step::If {
            cond: substitute_pure(cond, params, args),
            then: then.iter().map(|s| substitute_step(s, params, args)).collect(),
            otherwise: otherwise
                .iter()
                .map(|s| substitute_step(s, params, args))
                .collect(),
            line: *line,
        },
        Step::Trace { op, args: op_args, line } => Step::Trace {
            op: op.clone(),
            args: op_args
                .iter()
                .map(|a| substitute_pure(a, params, args))
                .collect(),
            line: *line,
        },
    }
}

fn substitute_expr(expr: &IrExpr, params: &[String], args: &[PureExpr]) -> IrExpr {
    match expr {
        IrExpr::Pure(pure) => IrExpr::Pure(substitute_pure(pure, params, args)),
        IrExpr::Op { id, args: op_args } => IrExpr::Op {
            id: id.clone(),
            args: op_args
                .iter()
                .map(|a| substitute_pure(a, params, args))
                .collect(),
        },
        other => other.clone(),
    }
}

fn substitute_pure(pure: &PureExpr, params: &[String], args: &[PureExpr]) -> PureExpr {
    match pure {
        PureExpr::Var(name) => params
            .iter()
            .position(|p| p == name)
            .and_then(|i| args.get(i))
            .cloned()
            .unwrap_or_else(|| pure.clone()),
        PureExpr::Bin { operator, left, right } => PureExpr::Bin {
            operator: *operator,
            left: Box::new(substitute_pure(left, params, args)),
            right: Box::new(substitute_pure(right, params, args)),
        },
        PureExpr::Un { operator, expr } => PureExpr::Un {
            operator: *operator,
            expr: Box::new(substitute_pure(expr, params, args)),
        },
        PureExpr::Index { base, key } => PureExpr::Index {
            base: Box::new(substitute_pure(base, params, args)),
            key: key.clone(),
        },
        PureExpr::Access { base, key } => PureExpr::Access {
            base: Box::new(substitute_pure(base, params, args)),
            key: key.clone(),
        },
        PureExpr::Len(inner) => PureExpr::Len(Box::new(substitute_pure(inner, params, args))),
        PureExpr::List(items) => PureExpr::List(
            items
                .iter()
                .map(|i| substitute_pure(i, params, args))
                .collect(),
        ),
        PureExpr::Map(entries) => PureExpr::Map(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), substitute_pure(v, params, args)))
                .collect(),
        ),
        PureExpr::Ctor { name, args: ctor_args } => PureExpr::Ctor {
            name: name.clone(),
            args: ctor_args
                .iter()
                .map(|a| substitute_pure(a, params, args))
                .collect(),
        },
        other => other.clone(),
    }
}

// ---------------------------------------------------------------- 降级状态

#[derive(Default)]
struct Body {
    steps: Vec<Step>,
    ops: Vec<OpDecl>,
    op_ids: BTreeSet<String>,
    op_effects: BTreeSet<String>,
    rejected: Vec<Diagnostic>,
    /// 临时实参变量计数（`实参N`，全局递增防遮蔽）。
    temps_count: usize,
    inline_depth: usize,
}

impl Body {
    fn register_op(&mut self, display: &str, effect: impl Into<String>, arity: usize) -> String {
        let base = format!("调用:{display}");
        // 同名不同元数的调用点（可选参数等）→ 加 `/元数` 后缀：构造器声明与
        // 每个使用点的实参个数保持一致，否则生成的 Lean 无法编译。
        let id = match self.ops.iter().find(|o| o.id == base) {
            Some(o) if o.arity == arity => base,
            Some(_) => format!("{base}/{arity}"),
            None => base,
        };
        if self.op_ids.insert(id.clone()) {
            let effect = effect.into();
            self.op_effects.insert(effect.clone());
            self.ops.push(OpDecl {
                id: id.clone(),
                effect,
                params: Vec::new(),
                arity,
            });
        }
        id
    }

    /// 子体降级后并回主状态（ops / 效应 / 拒绝 / 临时变量；steps 由调用方取走）。
    fn absorb(&mut self, sub: Body) {
        for op in sub.ops {
            if self.op_ids.insert(op.id.clone()) {
                self.op_effects.insert(op.effect.clone());
                self.ops.push(op);
            }
        }
        self.op_effects.extend(sub.op_effects);
        self.rejected.extend(sub.rejected);
        self.temps_count = self.temps_count.max(sub.temps_count);
    }
}

// ---------------------------------------------------------------- 入口

/// 从模块生成 Plan（确定性输出；`module_display` 为相对路径）。
pub fn build_plan(
    module: &Module,
    source_sha256: &str,
    module_display: &str,
    effects: BTreeMap<(String, String), EffectSet>,
    allow_opaque: &[String],
) -> Result<(Plan, Vec<Lowered>)> {
    let lowerer = Lowerer::new(module, effects, false, allow_opaque);
    let lowered = lowerer.lower_module();
    let plan = Plan {
        schema: SCHEMA,
        module: module_display.to_string(),
        source_sha256: source_sha256.to_string(),
        functions: lowered.iter().filter_map(|l| l.plan.clone()).collect(),
    };
    Ok((plan, lowered))
}

/// 效应表构建：sema 闭包 artifact → `(file, display) → effects`。
/// sema 编译失败时返回空表（效应保守记 `unknown`，绝不臆造 pure）。
pub fn effect_map(path: &Path) -> BTreeMap<(String, String), EffectSet> {
    let mut map = BTreeMap::new();
    if let Ok(compilation) = crate::sema::ir::compile_file_closure(path) {
        for artifact in &compilation.semantics.artifacts {
            map.insert(
                (artifact.source.file.clone(), artifact.display_name.clone()),
                artifact.effects.clone(),
            );
        }
    }
    map
}

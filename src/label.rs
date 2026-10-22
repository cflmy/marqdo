//! `<TAB>` M2.5/M4: occlusion-label collection and diagnostics.
//!
//! A label is **inert prose**: it never runs, never binds, never types. This
//! module only *reads* it — to report structural problems (`label.missing`,
//! `label.empty_payload`) and, later, to check staleness (`label.ref_dangling`).
//!
//! Design: [occlusion-description-and-search.md](../../doc/design/occlusion-description-and-search.md)

use crate::ast::{Arg, BranchArm, Expr, Function, Module, Stmt};
use crate::diagnostics::{Diagnostic, Severity, Span};

/// One `[描述](实现)` site found in the module.
#[derive(Debug, Clone)]
pub struct OcclusionSite {
    /// Raw label text (already trimmed by the parser).
    pub label: String,
    /// `false` when written as `[描述]()` — the empty-payload stub.
    pub has_payload: bool,
    /// 1-based source line of the enclosing statement.
    pub line: u32,
    /// Enclosing function/object name (`# main` → `main`).
    pub unit: String,
}

/// Collect every occlusion in `module` (declaration order).
pub fn collect(module: &Module) -> Vec<OcclusionSite> {
    let mut out = Vec::new();
    for fun in &module.functions {
        walk_fun(fun, &mut out);
    }
    out
}

fn walk_fun(fun: &Function, out: &mut Vec<OcclusionSite>) {
    for stmt in &fun.body {
        walk_stmt(stmt, &fun.name, out);
    }
    for child in &fun.children {
        walk_fun(child, out);
    }
}

fn walk_stmt(stmt: &Stmt, unit: &str, out: &mut Vec<OcclusionSite>) {
    match stmt {
        Stmt::Assign { value, span, .. }
        | Stmt::Return { value, span }
        | Stmt::Expr { value, span } => walk_expr(value, span.line, unit, out),
        Stmt::Call { call, span } => {
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => {
                        walk_expr(e, span.line, unit, out)
                    }
                }
            }
        }
        Stmt::Branch { arms, span } => {
            for arm in arms {
                walk_arm(arm, span.line, unit, out);
            }
        }
        Stmt::While { condition, body, span } => {
            walk_expr(condition, span.line, unit, out);
            for s in body {
                walk_stmt(s, unit, out);
            }
        }
        Stmt::ForEach { body, span, .. } => {
            for s in body {
                walk_stmt(s, unit, out);
            }
            let _ = span;
        }
    }
}

fn walk_arm(arm: &BranchArm, line: u32, unit: &str, out: &mut Vec<OcclusionSite>) {
    // `condition: Option<Expr>` for the `*` else-arm.
    if let Some(cond) = &arm.condition {
        walk_expr(cond, line, unit, out);
    }
    for s in &arm.body {
        walk_stmt(s, unit, out);
    }
}

fn walk_expr(expr: &Expr, line: u32, unit: &str, out: &mut Vec<OcclusionSite>) {
    match expr {
        Expr::Occluded { label, payload } => {
            out.push(OcclusionSite {
                label: label.clone(),
                has_payload: payload.is_some(),
                line,
                unit: unit.to_string(),
            });
            if let Some(inner) = payload {
                walk_expr(inner, line, unit, out);
            }
        }
        Expr::Unary { expr, .. } => walk_expr(expr, line, unit, out),
        Expr::Binary { left, right, .. } => {
            walk_expr(left, line, unit, out);
            walk_expr(right, line, unit, out);
        }
        Expr::Call(call) => {
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => walk_expr(e, line, unit, out),
                }
            }
        }
        Expr::List(items) => {
            for e in items {
                walk_expr(e, line, unit, out);
            }
        }
        Expr::Map(pairs) => {
            for (_, e) in pairs {
                walk_expr(e, line, unit, out);
            }
        }
        Expr::Index { base, .. } => walk_expr(base, line, unit, out),
        // Interpolation carries only variable *names* (strings), never nested
        // expressions, so there is nothing to walk into.
        Expr::Interp(_) => {}
        Expr::Literal(_) | Expr::Var(_) | Expr::Formula(_) | Expr::Code(_) => {}
    }
}

/// Diagnostics for label structure. `<TAB>` M2.5
///
/// - empty label → `label.missing` (**error**): you wrote a shape without prose
/// - empty payload → `label.empty_payload` (**warning**): say what the stub returns
pub fn check(module: &Module, path: Option<&std::path::Path>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for site in collect(module) {
        let span = Span::new(site.line, 1);
        if site.label.is_empty() {
            out.push(
                Diagnostic::new(
                    path,
                    span,
                    format!(
                        "遮蔽缺少描述：`[]({})` —— 给这段实现写一句人话，或改用不带遮蔽的写法",
                        if site.has_payload { "…" } else { "" }
                    ),
                )
                .with_code("label.missing")
                .with_severity(Severity::Error)
                .with_suggestion("写成 `[这段实现做什么](实现)`；描述是自由文本，不参与执行"),
            );
            continue;
        }
        if !site.has_payload {
            out.push(
                Diagnostic::new(
                    path,
                    span,
                    format!("空载荷遮蔽：`[{}]()` —— 这里没有可执行内容", site.label),
                )
                .with_code("label.empty_payload")
                .with_severity(Severity::Warning)
                .with_doc_quote(site.label.clone())
                .with_suggestion("写清桩的返回值（如 `(None)` 或 `(0)`），或补上真实实现"),
            );
        }
    }
    out
}

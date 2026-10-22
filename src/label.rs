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
    /// Identifiers the payload actually mentions (for L1 ref checks).
    pub payload_idents: std::collections::HashSet<String>,
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
            let mut idents = std::collections::HashSet::new();
            if let Some(inner) = payload {
                payload_idents(inner, &mut idents);
            }
            out.push(OcclusionSite {
                label: label.clone(),
                has_payload: payload.is_some(),
                line,
                unit: unit.to_string(),
                payload_idents: idents,
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

/// Backticked entity references inside a label: `` 按 `税率` 计算 `` → `["税率"]`.
///
/// Only explicit backticks count as claims — plain prose never is. `<TAB>` M4
pub fn refs(label: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = label;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        let name = after[..close].trim();
        if !name.is_empty() {
            out.push(name.to_string());
        }
        rest = &after[close + 1..];
    }
    out
}

/// Every identifier the payload actually mentions (variables, callees, path parts).
fn payload_idents(expr: &Expr, out: &mut std::collections::HashSet<String>) {
    match expr {
        Expr::Var(n) => {
            out.insert(n.clone());
        }
        Expr::Occluded { payload, .. } => {
            if let Some(inner) = payload {
                payload_idents(inner, out);
            }
        }
        Expr::Unary { expr, .. } => payload_idents(expr, out),
        Expr::Binary { left, right, .. } => {
            payload_idents(left, out);
            payload_idents(right, out);
        }
        Expr::Call(call) => {
            out.insert(call.callee.clone());
            if let Some(p) = &call.path {
                for seg in p {
                    out.insert(seg.clone());
                }
            }
            if let Some(r) = &call.receiver {
                out.insert(r.clone());
            }
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => payload_idents(e, out),
                }
            }
        }
        Expr::List(items) => {
            for e in items {
                payload_idents(e, out);
            }
        }
        Expr::Map(pairs) => {
            for (_, e) in pairs {
                payload_idents(e, out);
            }
        }
        Expr::Index { base, label } => {
            payload_idents(base, out);
            if let crate::ast::IndexKey::Var(n) = label {
                out.insert(n.clone());
            }
        }
        Expr::Interp(parts) => {
            for p in parts {
                if let crate::ast::InterpPart::Var(n) = p {
                    out.insert(n.clone());
                }
            }
        }
        Expr::Literal(_) | Expr::Formula(_) | Expr::Code(_) => {}
    }
}

/// Names that exist in the document scope: functions/objects (recursive),
/// import aliases, frontmatter keys. Builtins are checked separately.
fn module_symbols(module: &Module) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    fn walk_fun(f: &Function, out: &mut std::collections::HashSet<String>) {
        out.insert(f.name.clone());
        for p in &f.params {
            out.insert(p.name.clone());
        }
        for c in &f.children {
            walk_fun(c, out);
        }
    }
    for f in &module.functions {
        walk_fun(f, &mut out);
    }
    for imp in &module.imports {
        out.insert(imp.bind.clone());
    }
    for u in &module.uses {
        out.insert(u.bind.clone());
    }
    for (k, _) in &module.metadata {
        out.insert(k.clone());
    }
    out
}

/// Diagnostics for label structure. `<TAB>` M2.5
///
/// - empty label → `label.missing` (**error**): you wrote a shape without prose
/// - empty payload → `label.empty_payload` (**warning**): say what the stub returns
pub fn check(module: &Module, path: Option<&std::path::Path>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let symbols = module_symbols(module);
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

        // `<TAB>` M4 (L1): every backticked ref in the label must resolve —
        // either the payload mentions it, or the document defines it, or it is
        // a builtin. This is the deterministic half of staleness detection.
        for r in refs(&site.label) {
            if site.payload_idents.contains(&r)
                || symbols.contains(&r)
                || crate::host::HostFn::from_name(&r).is_some()
            {
                continue;
            }
            out.push(
                Diagnostic::new(
                    path,
                    span,
                    format!(
                        "描述引用了不存在的实体：`{r}` —— 本段实现没有用到它，模块里也没有定义"
                    ),
                )
                .with_code("label.ref_dangling")
                .with_severity(Severity::Error)
                .with_doc_quote(site.label.clone())
                .with_suggestion(format!(
                    "检查是否改名/删除；若只是措辞，把 `{r}` 的反引号去掉或改写描述"
                )),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::refs;

    #[test]
    fn refs_extracts_backticked_names() {
        assert_eq!(refs("按 `税率` 计算"), vec!["税率".to_string()]);
    }

    #[test]
    fn refs_multiple_and_skips_prose() {
        assert_eq!(
            refs("用 `a` 和 `b`，但 plain words 不算"),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn refs_none_and_unmatched_backtick() {
        assert!(refs("纯口语描述").is_empty());
        // Unmatched trailing backtick: ignore, do not panic.
        assert!(refs("半截 `名字").is_empty());
        // Empty ticks are not refs.
        assert!(refs("空 `` 引号").is_empty());
    }
}

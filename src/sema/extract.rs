//! Restricted pure-expression extract (S3).
//!
//! Only Literal / Var / Unary / Binary / List / Map / Index payloads may be
//! lifted. Calls, host, and plugins are refused so effect counts stay zero.

use std::collections::BTreeSet;

use crate::ast::Expr;
use crate::parse::expr::parse_expr_prefer_var;

#[derive(Debug, Clone)]
pub struct ExtractPlan {
    pub name: String,
    pub params: Vec<String>,
    pub expression: String,
    pub heading: String,
    pub call: String,
}

#[derive(Debug, Clone)]
pub struct ExtractError {
    pub code: &'static str,
    pub message: String,
}

/// Build an extract plan from a pure expression string (no file I/O).
pub fn plan_pure_extract(name: &str, expression: &str) -> Result<ExtractPlan, ExtractError> {
    let name = name.trim();
    if name.is_empty()
        || name.contains(|c: char| c.is_whitespace() || c == '.' || c == '/')
    {
        return Err(ExtractError {
            code: "extract.bad_name",
            message: "提取目标名必须是单个标识符".into(),
        });
    }
    let expr = parse_expr_prefer_var(expression).map_err(|e| ExtractError {
        code: "extract.parse",
        message: format!("无法解析表达式: {e}"),
    })?;
    if !is_pure_payload(&expr) {
        return Err(ExtractError {
            code: "extract.not_pure",
            message: "仅允许字面量/变量/算术/列表等纯载荷；拒绝调用与宿主效应".into(),
        });
    }
    let params: Vec<String> = free_vars(&expr).into_iter().collect();
    let call = if params.is_empty() {
        format!("{name}()")
    } else {
        let args = params
            .iter()
            .map(|p| format!("{p}=`{p}`"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{name}({args})")
    };
    let heading = if params.is_empty() {
        format!("## {name}\n\n*{expression}*\n")
    } else {
        let decls = params
            .iter()
            .map(|p| format!("`{p}`"))
            .collect::<Vec<_>>()
            .join("、");
        format!("## {name}\n\n参数：{decls}\n\n*{expression}*\n")
    };
    Ok(ExtractPlan {
        name: name.into(),
        params,
        expression: expression.trim().into(),
        heading,
        call,
    })
}

fn is_pure_payload(expr: &Expr) -> bool {
    match expr {
        Expr::Literal(_) | Expr::Var(_) => true,
        Expr::Unary { expr, .. } => is_pure_payload(expr),
        Expr::Binary { left, right, .. } => is_pure_payload(left) && is_pure_payload(right),
        Expr::List(items) => items.iter().all(is_pure_payload),
        Expr::Map(items) => items.iter().all(|(_, v)| is_pure_payload(v)),
        Expr::Index { base, .. } => is_pure_payload(base),
        Expr::Occluded { payload, .. } => payload
            .as_ref()
            .map(|p| is_pure_payload(p))
            .unwrap_or(true),
        Expr::Call(_) | Expr::Interp(_) | Expr::Formula(_) | Expr::Code(_) => false,
    }
}

fn free_vars(expr: &Expr) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    walk(expr, &mut out);
    out
}

fn walk(expr: &Expr, out: &mut BTreeSet<String>) {
    match expr {
        Expr::Var(name) => {
            out.insert(name.clone());
        }
        Expr::Unary { expr, .. } => walk(expr, out),
        Expr::Binary { left, right, .. } => {
            walk(left, out);
            walk(right, out);
        }
        Expr::List(items) => {
            for item in items {
                walk(item, out);
            }
        }
        Expr::Map(items) => {
            for (_, item) in items {
                walk(item, out);
            }
        }
        Expr::Index { base, .. } => walk(base, out),
        Expr::Occluded { payload, .. } => {
            if let Some(p) = payload {
                walk(p, out);
            }
        }
        Expr::Literal(_)
        | Expr::Call(_)
        | Expr::Interp(_)
        | Expr::Formula(_)
        | Expr::Code(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_product() {
        let plan = plan_pure_extract("商品小计", "数量 * 单价").unwrap();
        assert_eq!(plan.params, vec!["单价".to_string(), "数量".to_string()]);
        assert!(plan.heading.contains("## 商品小计"));
        assert!(plan.call.contains("商品小计("));
    }

    #[test]
    fn rejects_call() {
        let err = plan_pure_extract("x", "save()").unwrap_err();
        assert_eq!(err.code, "extract.not_pure");
    }
}

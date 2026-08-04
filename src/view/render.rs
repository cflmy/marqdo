//! Render Marqdo AST as HTML fragments for `view`.
//!
//! Comments are not in the AST; they are interleaved from classified source lines.
//! Expressions are shown in surface syntax, never Rust `Debug`.

use crate::ast::{
    Arg, BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Stmt, UnaryOp,
};
use crate::lex::{classify_source, ClassifiedLine, LineKind};
use crate::view::html::escape;

pub struct FileViewModel {
    #[allow(dead_code)]
    pub rel_path: String,
    pub source: String,
    pub structure_html: String,
    pub stdout: String,
    pub stderr: String,
    pub ok: bool,
}

pub fn render_module_structure(module: &Module, source: &str) -> String {
    let lines = classify_source(source);
    let mut out = String::new();

    if !module.imports.is_empty() {
        out.push_str("<div class=\"card\"><span class=\"badge\">import</span>");
        for imp in &module.imports {
            out.push_str(&format!("<span class=\"chip\">{}</span>", escape(imp)));
        }
        out.push_str("</div>");
    }

    let body_start = skip_frontmatter_end(&lines).unwrap_or(1);
    let mut cursor = body_start;

    if module.functions.is_empty() {
        out.push_str(&emit_comments(&lines, cursor, u32::MAX));
        if out.is_empty() {
            out.push_str("<p class=\"meta\">(empty module)</p>");
        }
        return out;
    }

    // Leading comments before first function
    let first = module.functions[0].span.line;
    out.push_str(&emit_comments(&lines, cursor, first));
    cursor = first;

    for (i, fun) in module.functions.iter().enumerate() {
        if i > 0 {
            out.push_str(&emit_comments(&lines, cursor, fun.span.line));
        }
        let (html, next) = render_fun(fun, &lines, 0);
        out.push_str(&html);
        cursor = next;
    }

    // Trailing comments after last function tree
    out.push_str(&emit_comments(&lines, cursor, u32::MAX));
    out
}

/// Line after closing frontmatter `---`, or 1 if none.
fn skip_frontmatter_end(lines: &[ClassifiedLine]) -> Option<u32> {
    let first_code = lines.iter().find(|l| l.kind == LineKind::Code)?;
    if first_code.text.trim() != "---" {
        return Some(1);
    }
    for l in lines.iter().skip_while(|l| l.line_no <= first_code.line_no) {
        if l.kind == LineKind::Code && l.text.trim() == "---" {
            return Some(l.line_no + 1);
        }
    }
    Some(1)
}

fn emit_comments(lines: &[ClassifiedLine], from_line: u32, before_line: u32) -> String {
    let mut s = String::new();
    for l in lines {
        if l.line_no < from_line {
            continue;
        }
        if l.line_no >= before_line {
            break;
        }
        if l.kind == LineKind::Comment {
            let text = l.text.trim();
            if text.is_empty() {
                continue;
            }
            s.push_str(&format!(
                "<div class=\"card comment-card\"><span class=\"badge\">comment</span><span class=\"comment-text\">{}</span></div>",
                escape(text)
            ));
        }
    }
    s
}

fn render_fun(fun: &Function, lines: &[ClassifiedLine], depth: usize) -> (String, u32) {
    let mut s = String::new();
    let nest = if depth > 0 { " nested" } else { "" };
    s.push_str(&format!("<div class=\"card fun-card{nest}\">"));
    s.push_str(&format!(
        "<div><span class=\"badge\">fn · h{}</span><strong>{}</strong></div>",
        fun.level,
        escape(&fun.name)
    ));
    if !fun.params.is_empty() {
        s.push_str("<div class=\"params\">");
        for p in &fun.params {
            s.push_str(&format!("<span class=\"chip\">{}</span>", escape(p)));
        }
        s.push_str("</div>");
    }

    let mut cursor = fun.span.line + 1;

    // Comments / gap before first body stmt or child
    let first_body = fun.body.first().map(stmt_start);
    let first_child = fun.children.first().map(|c| c.span.line);
    let first_content = match (first_body, first_child) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    if let Some(fc) = first_content {
        s.push_str(&emit_comments(lines, cursor, fc));
        cursor = fc;
    }

    // Interleave body stmts and nested functions by source line
    let mut bi = 0usize;
    let mut ci = 0usize;
    while bi < fun.body.len() || ci < fun.children.len() {
        let stmt_line = fun.body.get(bi).map(stmt_start);
        let child_line = fun.children.get(ci).map(|c| c.span.line);
        let take_stmt = match (stmt_line, child_line) {
            (Some(sl), Some(cl)) => sl <= cl,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        if take_stmt {
            let stmt = &fun.body[bi];
            let start = stmt_start(stmt);
            s.push_str(&emit_comments(lines, cursor, start));
            let (html, end) = render_stmt(stmt, lines);
            s.push_str(&html);
            cursor = end;
            bi += 1;
        } else {
            let child = &fun.children[ci];
            s.push_str(&emit_comments(lines, cursor, child.span.line));
            let (html, end) = render_fun(child, lines, depth + 1);
            s.push_str(&format!("<div class=\"nested\">{html}</div>"));
            cursor = end;
            ci += 1;
        }
    }

    s.push_str("</div>");
    (s, cursor)
}

fn stmt_start(stmt: &Stmt) -> u32 {
    match stmt {
        Stmt::Assign { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Branch { span, .. }
        | Stmt::While { span, .. }
        | Stmt::ForEach { span, .. } => span.line,
    }
}

fn stmt_end_line(stmt: &Stmt) -> u32 {
    match stmt {
        Stmt::Assign { span, .. } | Stmt::Return { span, .. } | Stmt::Call { span, .. } => {
            span.line + 1
        }
        Stmt::Branch { arms, span, .. } => {
            let mut end = span.line + 1;
            for arm in arms {
                for st in &arm.body {
                    end = end.max(stmt_end_line(st));
                }
            }
            end
        }
        Stmt::While { body, span, .. } | Stmt::ForEach { body, span, .. } => {
            let mut end = span.line + 1;
            for st in body {
                end = end.max(stmt_end_line(st));
            }
            end
        }
    }
}

fn render_stmt(stmt: &Stmt, lines: &[ClassifiedLine]) -> (String, u32) {
    let end = stmt_end_line(stmt);
    let html = match stmt {
        Stmt::Assign { name, value, .. } => format!(
            "<div class=\"card\"><span class=\"badge\">bind</span><code class=\"expr\">`{}` = {}</code></div>",
            escape(name),
            escape(&expr_display(value))
        ),
        Stmt::Return { value, .. } => format!(
            "<div class=\"card ret-card\"><span class=\"badge\">return</span><code class=\"expr\">{}</code></div>",
            escape(&expr_display(value))
        ),
        Stmt::Call { call, .. } => format!(
            "<div class=\"card call-card\"><span class=\"badge\">call</span><code class=\"expr\">{}</code></div>",
            escape(&call_display(call))
        ),
        Stmt::Branch { arms, .. } => {
            let mut inner =
                String::from("<div class=\"card branch-card\"><span class=\"badge\">branch</span>");
            let mut arm_cursor = stmt_start(stmt) + 1;
            for arm in arms {
                let label = match &arm.condition {
                    None => "else".to_string(),
                    Some(c) => expr_display(c),
                };
                // comments before first stmt in arm (approx: between arms hard; use body starts)
                let arm_first = arm.body.first().map(stmt_start);
                inner.push_str(&format!(
                    "<div class=\"arm\"><span class=\"badge\">arm</span><code class=\"expr\">{}</code>",
                    escape(&label)
                ));
                if let Some(af) = arm_first {
                    inner.push_str(&emit_comments(lines, arm_cursor, af));
                    arm_cursor = af;
                }
                inner.push_str("<div class=\"nested\">");
                for st in &arm.body {
                    let start = stmt_start(st);
                    inner.push_str(&emit_comments(lines, arm_cursor, start));
                    let (h, e) = render_stmt(st, lines);
                    inner.push_str(&h);
                    arm_cursor = e;
                }
                inner.push_str("</div></div>");
            }
            inner.push_str("</div>");
            inner
        }
        Stmt::While {
            condition, body, ..
        } => {
            let mut inner = format!(
                "<div class=\"card loop-card\"><span class=\"badge\">while</span><code class=\"expr\">{}</code><div class=\"nested\">",
                escape(&expr_display(condition))
            );
            let mut cursor = stmt_start(stmt) + 1;
            for st in body {
                let start = stmt_start(st);
                inner.push_str(&emit_comments(lines, cursor, start));
                let (h, e) = render_stmt(st, lines);
                inner.push_str(&h);
                cursor = e;
            }
            inner.push_str("</div></div>");
            inner
        }
        Stmt::ForEach {
            item,
            collection,
            body,
            ..
        } => {
            let mut inner = format!(
                "<div class=\"card loop-card\"><span class=\"badge\">foreach</span><code class=\"expr\">[{}]({})</code><div class=\"nested\">",
                escape(item),
                escape(collection)
            );
            let mut cursor = stmt_start(stmt) + 1;
            for st in body {
                let start = stmt_start(st);
                inner.push_str(&emit_comments(lines, cursor, start));
                let (h, e) = render_stmt(st, lines);
                inner.push_str(&h);
                cursor = e;
            }
            inner.push_str("</div></div>");
            inner
        }
    };
    (html, end)
}

fn call_display(call: &CallExpr) -> String {
    let mut s = format!("> {}", call.callee);
    for a in &call.args {
        match a {
            Arg::Positional(e) => {
                s.push(' ');
                s.push_str(&expr_display(e));
            }
            Arg::Named { name, value } => {
                s.push(' ');
                s.push_str(name);
                s.push('=');
                s.push_str(&expr_display(value));
            }
        }
    }
    s
}

/// Surface-syntax preview for view (not Debug).
pub fn expr_display(expr: &Expr) -> String {
    expr_prec(expr, 0)
}

fn expr_prec(expr: &Expr, parent_prec: u8) -> String {
    match expr {
        Expr::Literal(lit) => lit_display(lit),
        Expr::Var(name) => format!("`{name}`"),
        Expr::Interp(parts) => {
            let mut s = String::new();
            for p in parts {
                match p {
                    InterpPart::Lit(t) => s.push_str(t),
                    InterpPart::Var(n) => {
                        s.push('`');
                        s.push_str(n);
                        s.push('`');
                    }
                }
            }
            s
        }
        Expr::Unary { op, expr } => {
            let prec = 5;
            let inner = match op {
                UnaryOp::Not => format!("not {}", expr_prec(expr, prec)),
                UnaryOp::Neg => format!("-{}", expr_prec(expr, prec)),
            };
            wrap(inner, prec, parent_prec)
        }
        Expr::Binary { op, left, right } => {
            let (sym, prec) = bin_sym_prec(*op);
            let inner = format!(
                "{} {} {}",
                expr_prec(left, prec),
                sym,
                expr_prec(right, prec + 1) // left-assoc bias
            );
            wrap(inner, prec, parent_prec)
        }
        Expr::Call(call) => call_display(call),
        Expr::List(items) => {
            let parts: Vec<String> = items.iter().map(|e| expr_prec(e, 0)).collect();
            format!("[{}]", parts.join(", "))
        }
    }
}

fn wrap(inner: String, prec: u8, parent_prec: u8) -> String {
    if prec < parent_prec {
        format!("({inner})")
    } else {
        inner
    }
}

fn bin_sym_prec(op: BinaryOp) -> (&'static str, u8) {
    match op {
        BinaryOp::Or => ("or", 1),
        BinaryOp::And => ("and", 2),
        BinaryOp::Eq => ("==", 3),
        BinaryOp::Ne => ("!=", 3),
        BinaryOp::Lt => ("<", 3),
        BinaryOp::Le => ("<=", 3),
        BinaryOp::Gt => (">", 3),
        BinaryOp::Ge => (">=", 3),
        BinaryOp::Add => ("+", 4),
        BinaryOp::Sub => ("-", 4),
        BinaryOp::Mul => ("*", 5),
        BinaryOp::Div => ("/", 5),
    }
}

fn lit_display(lit: &Literal) -> String {
    match lit {
        Literal::None => "None".into(),
        Literal::Bool(true) => "True".into(),
        Literal::Bool(false) => "False".into(),
        Literal::Int(n) => n.to_string(),
        Literal::Text(t) => t.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr, Literal};

    #[test]
    fn expr_gt_is_surface() {
        let e = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Var("x".into())),
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };
        assert_eq!(expr_display(&e), "`x` > 0");
        assert!(!expr_display(&e).contains("Binary"));
    }

    #[test]
    fn structure_keeps_comments_and_surface_expr() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&module, src);
        assert!(html.contains("comment-card"));
        assert!(html.contains("print"));
        assert!(!html.contains("Binary {"));
        assert!(!html.contains("Literal(Int"));
    }
}

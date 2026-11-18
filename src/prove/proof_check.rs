//! `check --proof`：定理表的静态校验（设计 §7；默认关）。
//!
//! 校验项（全部 advisory 之外都是 Error 级诊断）：
//! - `proof.claim_malformed`  表头命中定理词表但列形不齐
//! - `proof.claim_ambiguous`  同时有 `类型` 与 `结论` 列
//! - `proof.claim_misplaced`  合法定理表出现在首条可执行语句之后
//! - `proof.claim_duplicate`  同一函数内定理 id 重复
//! - `proof.claim_unknown_symbol`  结论/前提引用了受限符号集之外的名字
//! - `proof.claim_unsupported`    谓词超出封闭集合（包含/长度/下标）或元数不对
//! - `proof.name_shadow`      保留名（`结果`/`轨迹`）被形参/局部变量遮蔽

use std::path::Path;

use crate::ast::{Expr, Function, Module, Stmt};
use crate::contract;
use crate::diagnostics::{Diagnostic, Severity, Span};

/// 定理作用域保留名（§7.3.4）。
pub const RESERVED_CLAIM_NAMES: &[&str] = &["结果", "轨迹"];

/// 谓词封闭集合（§7.3.5：改集合需重新评审 ADR）。
pub const PREDICATES: &[(&str, usize)] = &[("包含", 2), ("长度", 1), ("下标", 2)];

/// 入口：对整个模块做定理表校验（只看主文件；与契约检查同一范围，
/// import 库文件不重复校验——诊断的行号锚在主文件上才有意义）。
pub fn check_proofs(module: &Module, path: Option<&Path>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut function_names: Vec<String> = Vec::new();
    collect_function_names(module, &mut function_names);
    // 谓词名遮蔽是模块级事实（与单个函数无关），整个模块只报一次；
    // 诊断锚在被遮蔽的函数上（任意深度）。
    for pred in PREDICATES {
        if function_names.iter().any(|n| n == pred.0) {
            let Some(fun) = find_by_name(&module.functions, pred.0) else {
                continue;
            };
            out.push(proof_diag(
                module,
                path,
                fun.span,
                "proof.name_shadow",
                format!(
                    "函数名 `{}` 与定理谓词 `{}` 同名（谓词集合封闭：包含/长度/下标）",
                    fun.name, pred.0
                ),
                format!("把函数 `{}` 改名，避免定理表里的 `{}` 歧义", fun.name, pred.0),
            ));
        }
    }
    walk_checks(module, &module.functions, path, &mut out);
    out
}

fn walk_checks(module: &Module, funs: &[Function], path: Option<&Path>, out: &mut Vec<Diagnostic>) {
    for fun in funs {
        check_claim_duplicates(module, fun, path, out);
        check_name_shadow(module, fun, path, out);
        check_misplaced_and_malformed(module, fun, path, out);
        check_claim_symbols(module, fun, path, out);
        walk_checks(module, &fun.children, path, out);
    }
}

fn collect_function_names(module: &Module, out: &mut Vec<String>) {
    fn go(funs: &[Function], out: &mut Vec<String>) {
        for f in funs {
            out.push(f.name.clone());
            go(&f.children, out);
        }
    }
    go(&module.functions, out);
    for child in module.import_modules.values() {
        collect_function_names(child, out);
    }
}

/// 在函数树里按显示名找函数（诊断锚点用）。
fn find_by_name<'a>(funs: &'a [Function], name: &str) -> Option<&'a Function> {
    for f in funs {
        if f.name == name {
            return Some(f);
        }
        if let Some(inner) = find_by_name(&f.children, name) {
            return Some(inner);
        }
    }
    None
}

fn proof_diag(
    module: &Module,
    path: Option<&Path>,
    span: Span,
    code: &str,
    message: String,
    suggestion: String,
) -> Diagnostic {
    let _ = module;
    Diagnostic::new(path, span, message)
        .with_code(code)
        .with_severity(Severity::Error)
        .with_suggestion(suggestion)
}

/// 同一函数内定理 id 重复（§7.1 规则 3）。
fn check_claim_duplicates(module: &Module, fun: &Function, path: Option<&Path>, out: &mut Vec<Diagnostic>) {
    let mut seen: Vec<&str> = Vec::new();
    for claim in &fun.claims {
        if seen.contains(&claim.id.as_str()) {
            out.push(proof_diag(
                module,
                path,
                Span::new(claim.line, 1),
                "proof.claim_duplicate",
                format!("定理 `{}` 在 `{}` 内重复声明", claim.id, fun.name),
                "定理 id 在函数内必须唯一；重命名或删除重复行".into(),
            ));
        } else {
            seen.push(&claim.id);
        }
    }
}

/// 遮蔽检查（§7.3.4）：只看**自带定理表的函数**——形参 `结果`/`轨迹` 会
/// 与该函数定理作用域的输出绑定名冲突；没有定理表的函数不受影响。
fn check_name_shadow(
    module: &Module,
    fun: &Function,
    path: Option<&Path>,
    out: &mut Vec<Diagnostic>,
) {
    if fun.claims.is_empty() {
        return;
    }
    for param in &fun.params {
        if RESERVED_CLAIM_NAMES.contains(&param.name.as_str()) {
            out.push(proof_diag(
                module,
                path,
                param.span,
                "proof.name_shadow",
                format!(
                    "形参 `{}` 与定理作用域保留名冲突（`{}` 也要声明定理）",
                    param.name,
                    fun.name
                ),
                format!("把形参 `{}` 改名；`结果`/`轨迹` 在定理里指返回值与操作轨迹", param.name),
            ));
        }
    }
}

/// 错位与畸形：body 里仍是定理形的表（默认关时解析器不提取它们）。
fn check_misplaced_and_malformed(
    module: &Module,
    fun: &Function,
    path: Option<&Path>,
    out: &mut Vec<Diagnostic>,
) {
    let mut prefix_zone = true;
    for stmt in &fun.body {
        let (pairs, span) = match stmt {
            Stmt::Expr {
                value: Expr::Map(pairs),
                span,
            } => (pairs, *span),
            Stmt::Expr { value: Expr::List(_), .. } => continue,
            _ => {
                prefix_zone = false;
                continue;
            }
        };
        if let Some(assumption)=contract::parse_assumption_table(pairs,span.line){
            let (code,msg)=match assumption {
                Ok(_)=>("proof.assumption_misplaced",
                    "假设表必须在第一条可执行语句之前；此处不会自动生效".to_string()),
                Err(err)=>("proof.assumption_malformed",err)
            };
            out.push(proof_diag(module,path,span,code,msg,
                "移动并修正 | 假设 | 前置 | 后置 | 表到函数开头".into()));
            continue;
        }
        match contract::parse_claim_table(pairs, span.line) {
            Some(Ok(_rows)) => {
                // 合法定理形却还在 body：一定是位置不对（prefix zone 内会被提取）。
                out.push(proof_diag(
                    module,
                    path,
                    span,
                    "proof.claim_misplaced",
                    format!(
                        "`{}` 的定理表出现在首条可执行语句之后——它是普通表，不会成为定理",
                        fun.name
                    ),
                    "把定理表移到函数体开头（与契约表同一槽位）".into(),
                ));
            }
            Some(Err(err)) => {
                out.push(proof_diag(
                    module,
                    path,
                    span,
                    err.code,
                    err.message,
                    "对照定理表形状：| 定理 | 结论 | 前提 | 策略 |（前提/策略可省）".into(),
                ));
            }
            None => {}
        }
        let _ = prefix_zone;
    }
}

/// 受限符号集校验（§7.1 规则 4）：前提只看形参；结论可加 `结果`/`轨迹`。
fn check_claim_symbols(module: &Module, fun: &Function, path: Option<&Path>, out: &mut Vec<Diagnostic>) {
    if fun.claims.is_empty() {
        return;
    }
    let params: Vec<String> = fun.params.iter().map(|p| p.name.clone()).collect();
    for claim in &fun.claims {
        if let Some(tactic) = &claim.tactic {
            if tactic != "auto" && tactic != "none" {
                out.push(proof_diag(
                    module,
                    path,
                    Span::new(claim.line, 1),
                    "proof.claim_malformed",
                    format!("定理 `{}` 的策略 `{}` 不在 auto/none 内", claim.id, tactic),
                    "策略列只接受 auto（自动）或 none（永远 unverified）".into(),
                ));
            }
        }
        if let Some(premise) = &claim.premise {
            check_claim_expr(
                module,
                path,
                premise,
                claim,
                &params,
                false,
                out,
            );
        }
        check_claim_expr(module, path, &claim.conclusion, claim, &params, true, out);
    }
}

fn check_claim_expr(
    module: &Module,
    path: Option<&Path>,
    text: &str,
    claim: &crate::ast::Claim,
    params: &[String],
    conclusion_scope: bool,
    out: &mut Vec<Diagnostic>,
) {
    let tokens = match tokenize_claim(text) {
        Ok(t) => t,
        Err(msg) => {
            out.push(proof_diag(
                module,
                path,
                Span::new(claim.line, 1),
                "proof.claim_malformed",
                format!("定理 `{}` 的表达无法解析：{msg}", claim.id),
                "受限表达式：名字、数字、字符串、比较/算术/布尔、包含(轨迹,x)/长度(轨迹)/下标(轨迹,i)".into(),
            ));
            return;
        }
    };
    match parse_claim_expr(&tokens) {
        Ok((_, ast)) => collect_symbols(
            module,
            path,
            claim,
            &ast,
            params,
            conclusion_scope,
            out,
        ),
        Err(msg) => {
            out.push(proof_diag(
                module,
                path,
                Span::new(claim.line, 1),
                "proof.claim_malformed",
                format!("定理 `{}` 的表达无法解析：{msg}", claim.id),
                "受限表达式：名字、数字、字符串、比较/算术/布尔、包含(轨迹,x)/长度(轨迹)/下标(轨迹,i)".into(),
            ));
        }
    }
}

// ---------------------------------------------------------------- 受限表达式

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CTok {
    Name(String),
    Int(i64),
    Num(f64),
    Text(String),
    Bool(bool),
    Op(&'static str),
    LParen,
    RParen,
    Comma,
    Dot,
}

/// 词法：把受限表达文本切成记号（emit 与 check 共用）。
pub(crate) fn tokenize_claim(text: &str) -> Result<Vec<CTok>, String> {
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '(' => out.push(CTok::LParen),
            ')' => out.push(CTok::RParen),
            ',' | '，' => out.push(CTok::Comma),
            '.' | '。' => out.push(CTok::Dot),
            '"' | '“' | '”' => {
                // 字符串字面量（到下一个同类引号）。
                let close = c;
                let mut j = i + 1;
                let mut s = String::new();
                while j < chars.len() && chars[j] != close {
                    s.push(chars[j]);
                    j += 1;
                }
                if j >= chars.len() {
                    return Err("字符串没有闭合引号".into());
                }
                out.push(CTok::Text(s));
                i = j;
            }
            '`' => {
                // `` `名字` `` → 名字。
                let mut j = i + 1;
                let mut s = String::new();
                while j < chars.len() && chars[j] != '`' {
                    s.push(chars[j]);
                    j += 1;
                }
                if j >= chars.len() {
                    return Err("反引号名没有闭合".into());
                }
                out.push(CTok::Name(s));
                i = j;
            }
            '0'..='9' => {
                let mut j = i;
                let mut s = String::new();
                let mut dotted = false;
                while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                    if chars[j] == '.' {
                        if dotted {
                            break;
                        }
                        dotted = true;
                    }
                    s.push(chars[j]);
                    j += 1;
                }
                if dotted {
                    out.push(CTok::Num(s.parse::<f64>().map_err(|_| "数字解析失败")?));
                } else {
                    out.push(CTok::Int(s.parse::<i64>().map_err(|_| "整数解析失败")?));
                }
                i = j - 1;
            }
            '=' => out.push(CTok::Op("=")),
            '≠' | '!' => out.push(CTok::Op("≠")),
            '<' => out.push(CTok::Op("<")),
            '≤' => out.push(CTok::Op("≤")),
            '>' => out.push(CTok::Op(">")),
            '≥' => out.push(CTok::Op("≥")),
            '+' => out.push(CTok::Op("+")),
            '-' | '−' => out.push(CTok::Op("-")),
            '*' | '×' => out.push(CTok::Op("*")),
            '/' | '÷' => out.push(CTok::Op("/")),
            _ => {
                // 名字 / 中文词 / 关键字。
                if c.is_alphanumeric() || c == '_' {
                    let mut j = i;
                    let mut s = String::new();
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '.') {
                        s.push(chars[j]);
                        j += 1;
                    }
                    i = j - 1;
                    match s.as_str() {
                        "且" | "and" => out.push(CTok::Op("且")),
                        "或" | "or" => out.push(CTok::Op("或")),
                        "非" | "not" => out.push(CTok::Op("非")),
                        "真" | "true" | "True" => out.push(CTok::Bool(true)),
                        "假" | "false" | "False" => out.push(CTok::Bool(false)),
                        _ => {
                            // `名字.字段` 已并入 s（含点）。
                            out.push(CTok::Name(s));
                        }
                    }
                } else {
                    return Err(format!("无法识别的字符 `{c}`"));
                }
            }
        }
        i += 1;
    }
    Ok(out)
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // op/字面值记号保留：P1 语义求值与 Lean 脚本生成要用。
pub(crate) enum ClaimAst {
    Lit(CTok),
    /// 名字（可含 `.字段`；已合并进字符串）。
    Var(String),
    Call { name: String, args: Vec<ClaimAst> },
    Bin { op: &'static str, left: Box<ClaimAst>, right: Box<ClaimAst> },
    Un { op: &'static str, expr: Box<ClaimAst> },
}

struct P<'a> {
    toks: &'a [CTok],
    pos: usize,
}

impl<'a> P<'a> {
    fn peek(&self) -> Option<&CTok> {
        self.toks.get(self.pos)
    }
    fn next(&mut self) -> Option<CTok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn eat_op(&mut self, ops: &[&str]) -> Option<&'static str> {
        let CTok::Op(o) = self.peek()? else {
            return None;
        };
        if !ops.contains(o) {
            return None;
        }
        let hit: &'static str = match *o {
            "+" => "+",
            "-" => "-",
            "*" => "*",
            "/" => "/",
            "=" => "=",
            "≠" => "≠",
            "<" => "<",
            "≤" => "≤",
            ">" => ">",
            "≥" => "≥",
            "且" => "且",
            "或" => "或",
            "非" => "非",
            _ => return None,
        };
        self.pos += 1;
        Some(hit)
    }
}

pub(crate) fn parse_claim_expr(toks: &[CTok]) -> Result<(Vec<CTok>, ClaimAst), String> {
    let mut p = P { toks, pos: 0 };
    let ast = parse_or(&mut p)?;
    if p.pos != toks.len() {
        return Err(format!("表达式末尾有多余内容（第 {} 个记号起）", p.pos + 1));
    }
    Ok((toks.to_vec(), ast))
}

fn parse_or(p: &mut P) -> Result<ClaimAst, String> {
    let mut left = parse_and(p)?;
    while let Some(op) = p.eat_op(&["或"]) {
        let right = parse_and(p)?;
        left = ClaimAst::Bin { op, left: Box::new(left), right: Box::new(right) };
    }
    Ok(left)
}

fn parse_and(p: &mut P) -> Result<ClaimAst, String> {
    let mut left = parse_cmp(p)?;
    while let Some(op) = p.eat_op(&["且"]) {
        let right = parse_cmp(p)?;
        left = ClaimAst::Bin { op, left: Box::new(left), right: Box::new(right) };
    }
    Ok(left)
}

fn parse_cmp(p: &mut P) -> Result<ClaimAst, String> {
    let left = parse_add(p)?;
    if let Some(op) = p.eat_op(&["=", "≠", "<", "≤", ">", "≥"]) {
        let right = parse_add(p)?;
        return Ok(ClaimAst::Bin { op, left: Box::new(left), right: Box::new(right) });
    }
    Ok(left)
}

fn parse_add(p: &mut P) -> Result<ClaimAst, String> {
    let mut left = parse_mul(p)?;
    while let Some(op) = p.eat_op(&["+", "-"]) {
        let right = parse_mul(p)?;
        left = ClaimAst::Bin { op, left: Box::new(left), right: Box::new(right) };
    }
    Ok(left)
}

fn parse_mul(p: &mut P) -> Result<ClaimAst, String> {
    let mut left = parse_unary(p)?;
    while let Some(op) = p.eat_op(&["*", "/"]) {
        let right = parse_unary(p)?;
        left = ClaimAst::Bin { op, left: Box::new(left), right: Box::new(right) };
    }
    Ok(left)
}

fn parse_unary(p: &mut P) -> Result<ClaimAst, String> {
    if let Some(op) = p.eat_op(&["非"]) {
        let expr = parse_unary(p)?;
        return Ok(ClaimAst::Un { op, expr: Box::new(expr) });
    }
    if let Some(CTok::Op("-")) = p.peek() {
        p.pos += 1;
        let expr = parse_unary(p)?;
        return Ok(ClaimAst::Un { op: "-", expr: Box::new(expr) });
    }
    parse_atom(p)
}

fn parse_atom(p: &mut P) -> Result<ClaimAst, String> {
    match p.next() {
        Some(CTok::LParen) => {
            let e = parse_or(p)?;
            match p.next() {
                Some(CTok::RParen) => Ok(e),
                _ => Err("缺少右括号".into()),
            }
        }
        Some(t @ (CTok::Int(_) | CTok::Num(_) | CTok::Text(_) | CTok::Bool(_))) => Ok(ClaimAst::Lit(t)),
        Some(CTok::Name(name)) => {
            // 谓词调用：名字后紧跟 `(`。
            if matches!(p.peek(), Some(CTok::LParen)) {
                p.pos += 1;
                let mut args = Vec::new();
                if !matches!(p.peek(), Some(CTok::RParen)) {
                    loop {
                        args.push(parse_or(p)?);
                        match p.next() {
                            Some(CTok::Comma) => continue,
                            Some(CTok::RParen) => break,
                            _ => return Err("调用参数列表未闭合".into()),
                        }
                    }
                } else {
                    p.pos += 1;
                }
                return Ok(ClaimAst::Call { name, args });
            }
            Ok(ClaimAst::Var(name))
        }
        other => Err(format!("这里应该是一个名字或值，得到 {other:?}")),
    }
}

/// 收集受限符号并校验（§7.1 规则 4 + §7.3.5 谓词封闭）。
fn collect_symbols(
    module: &Module,
    path: Option<&Path>,
    claim: &crate::ast::Claim,
    ast: &ClaimAst,
    params: &[String],
    conclusion_scope: bool,
    out: &mut Vec<Diagnostic>,
) {
    match ast {
        ClaimAst::Lit(_) => {}
        ClaimAst::Var(name) => {
            // 名字可带 `.字段`（结果.ok / 回执.value）——只校验根名字。
            let root = name.split('.').next().unwrap_or(name);
            let known = params.contains(&root.to_string())
                || (conclusion_scope && RESERVED_CLAIM_NAMES.contains(&root));
            if !known {
                let suggestion = crate::check::suggest_closest(root, params);
                let mut d = proof_diag(
                    module,
                    path,
                    Span::new(claim.line, 1),
                    "proof.claim_unknown_symbol",
                    format!(
                        "定理 `{}` 引用了未知名字 `{root}`（{}作用域只允许{}）",
                        claim.id,
                        if conclusion_scope { "结论" } else { "前提" },
                        if conclusion_scope {
                            "形参、结果、轨迹"
                        } else {
                            "形参"
                        }
                    ),
                    suggestion
                        .map(|s| format!("是不是想写 `{s}`？"))
                        .unwrap_or_else(|| {
                            if conclusion_scope {
                                "只引用形参与 `结果`/`轨迹`".into()
                            } else {
                                "前提里只引用形参（入口状态求值）".into()
                            }
                        }),
                );
                d = d.with_severity(Severity::Error);
                out.push(d);
            }
        }
        ClaimAst::Call { name, args } => {
            match PREDICATES.iter().find(|(n, _)| n == name) {
                Some((_, arity)) if *arity == args.len() => {}
                Some((_, arity)) => out.push(proof_diag(
                    module,
                    path,
                    Span::new(claim.line, 1),
                    "proof.claim_unsupported",
                    format!(
                        "谓词 `{name}` 需要 {arity} 个参数，定理 `{}` 给了 {} 个",
                        claim.id,
                        args.len()
                    ),
                    "谓词集合封闭：包含(轨迹,x) / 长度(轨迹) / 下标(轨迹,i)".into(),
                )),
                None => out.push(proof_diag(
                    module,
                    path,
                    Span::new(claim.line, 1),
                    "proof.claim_unsupported",
                    format!(
                        "定理 `{}` 调用了 `{name}`——定理里只能用包含/长度/下标三个谓词",
                        claim.id
                    ),
                    "谓词集合封闭（§7.3.5）；要扩集合先改 ADR 0010".into(),
                )),
            }
            for arg in args {
                collect_symbols(module, path, claim, arg, params, conclusion_scope, out);
            }
        }
        ClaimAst::Bin { left, right, .. } => {
            collect_symbols(module, path, claim, left, params, conclusion_scope, out);
            collect_symbols(module, path, claim, right, params, conclusion_scope, out);
        }
        ClaimAst::Un { expr, .. } => {
            collect_symbols(module, path, claim, expr, params, conclusion_scope, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(text: &str) -> ClaimAst {
        let toks = tokenize_claim(text).expect("tokenize");
        parse_claim_expr(&toks).expect("parse").1
    }

    #[test]
    fn restricted_grammar_parses_design_examples() {
        // 设计 §7.1 的示例句全部可解析。
        let _ = parse_ok("包含(轨迹, \"人工审核\")");
        let _ = parse_ok("金额 > 10000");
        let _ = parse_ok("结果.ok = True");
        let _ = parse_ok("长度(轨迹) = 2");
        let _ = parse_ok("下标(轨迹, 0) = \"x\"");
        let _ = parse_ok("非 结果.ok = False 且 金额 ≥ 0");
        let _ = parse_ok("金额 + 1 ≤ 10 或 假");
    }

    #[test]
    fn tokenizer_rejects_unterminated_strings() {
        assert!(tokenize_claim("包含(轨迹, \"未闭合").is_err());
    }

    #[test]
    fn reserved_outputs_render_without_field_access_prefix() {
        // emit 侧翻译：结果/轨迹 是定理作用域绑定名，不加 v_ 前缀。
        let ast = parse_ok("轨迹");
        assert!(matches!(ast, ClaimAst::Var(ref n) if n == "轨迹"));
        let ast = parse_ok("结果.ok");
        assert!(matches!(ast, ClaimAst::Var(ref n) if n == "结果.ok"));
    }

    #[test]
    fn symbol_check_reports_unknown_and_suggests() {
        let module = Module::default();
        let claim = crate::ast::Claim {
            id: "引用未知".into(),
            conclusion: "额度 > 0".into(),
            premise: None,
            tactic: Some("auto".into()),
            line: 3,
        };
        let mut out = Vec::new();
        check_claim_expr(
            &module,
            None,
            "额度 > 0",
            &claim,
            &["金额".to_string()],
            true,
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].code, "proof.claim_unknown_symbol");
        // 最近建议（编辑距离）：额度 → 金额 不够近，无建议也要有消息。
        assert!(out[0].message.contains("额度"));
    }

    #[test]
    fn symbol_check_accepts_params_in_premise_and_outputs_in_conclusion() {
        let module = Module::default();
        let claim = crate::ast::Claim {
            id: "合法".into(),
            conclusion: "结果.ok = True".into(),
            premise: Some("金额 > 0".into()),
            tactic: None,
            line: 1,
        };
        let mut out = Vec::new();
        check_claim_expr(
            &module,
            None,
            "金额 > 0",
            &claim,
            &["金额".to_string()],
            false,
            &mut out,
        );
        check_claim_expr(
            &module,
            None,
            "结果.ok = True",
            &claim,
            &["金额".to_string()],
            true,
            &mut out,
        );
        assert!(out.is_empty(), "{out:?}");
    }

    #[test]
    fn premise_may_not_reference_outputs() {
        let module = Module::default();
        let claim = crate::ast::Claim {
            id: "前提越界".into(),
            conclusion: "结果.ok = True".into(),
            premise: Some("轨迹 = 0".into()),
            tactic: None,
            line: 1,
        };
        let mut out = Vec::new();
        check_claim_expr(
            &module,
            None,
            "轨迹 = 0",
            &claim,
            &[],
            false,
            &mut out,
        );
        assert!(out
            .iter()
            .any(|d| d.code == "proof.claim_unknown_symbol"), "{out:?}");
    }

    #[test]
    fn predicate_arity_and_closure_are_enforced() {
        let module = Module::default();
        let claim = crate::ast::Claim {
            id: "元数错".into(),
            conclusion: "包含(轨迹, \"a\", \"b\")".into(),
            premise: None,
            tactic: None,
            line: 1,
        };
        let mut out = Vec::new();
        check_claim_expr(
            &module,
            None,
            "包含(轨迹, \"a\", \"b\")",
            &claim,
            &[],
            true,
            &mut out,
        );
        assert!(out.iter().any(|d| d.code == "proof.claim_unsupported"), "{out:?}");

        let claim2 = crate::ast::Claim {
            id: "越界谓词".into(),
            conclusion: "求和(金额, 1) > 0".into(),
            premise: None,
            tactic: None,
            line: 2,
        };
        let mut out2 = Vec::new();
        check_claim_expr(
            &module,
            None,
            "求和(金额, 1) > 0",
            &claim2,
            &["金额".to_string()],
            true,
            &mut out2,
        );
        assert!(out2.iter().any(|d| d.code == "proof.claim_unsupported"), "{out2:?}");
    }
}

//! `marqdo check` —— 静态契约互查（T2.3 + T2.5）。
//!
//! 只读静态检查，不执行程序。三类必产诊断：
//!
//! 1. **防漂移（T2.5）**：契约行 vs 升参推断形参**双向互查**——契约表一经出现
//!    就是该单元形参的完整声明：契约多出的名字（错行）与未被覆盖的推断形参
//!    （新升参）都是 `contract.param_drift`。
//! 2. **未知类型名**：`Ty::Object` 必须是已知对象类型（防 `nubmer` 拼错，
//!    附最近建议）⇒ `contract.unknown_type`。
//! 3. **错位契约**：契约形状（含 `类型` 列）的未绑定表出现在不允许位置
//!    （提取阶段原样留在 body 里的就是）⇒ `contract.misplaced`。
//!
//! 任一 Error ⇒ 进程以非零码退出。诊断全部带 `doc_anchor` 锚点（T2.4）。

use std::path::Path;

use anyhow::Result;

use crate::ast::{Expr, Function, Module, Stmt};
use crate::contract::{self, Contract, Ty};
use crate::diagnostics::{Diagnostic, Severity, Span};
use crate::load;

/// 读取文件 + 装载（含 import）后做全量契约互查。
pub fn check_path(path: &Path) -> Result<Vec<Diagnostic>> {
    let module = load::load_module(path)?;
    Ok(check_module(&module, Some(path)))
}

/// 递归检查模块全部函数（含子函数）的契约。
pub fn check_module(module: &Module, path: Option<&Path>) -> Vec<Diagnostic> {
    let known = known_type_names(module);
    let mut out = Vec::new();
    walk(module, &module.functions, path, &known, &mut out);
    out
}

/// 已知对象类型名：本文件 `#` 对象 + import/use 绑定 + 导入库的 `#` 对象。
fn known_type_names(module: &Module) -> Vec<String> {
    let mut names = Vec::new();
    for f in &module.functions {
        if f.is_object() {
            names.push(f.name.clone());
        }
    }
    for i in &module.imports {
        names.push(i.bind.clone());
    }
    for u in &module.uses {
        names.push(u.bind.clone());
    }
    for lib in module.import_modules.values() {
        for f in &lib.functions {
            if f.is_object() {
                names.push(f.name.clone());
            }
        }
    }
    names
}

fn walk(
    module: &Module,
    funs: &[Function],
    path: Option<&Path>,
    known: &[String],
    out: &mut Vec<Diagnostic>,
) {
    for fun in funs {
        check_one(module, fun, path, known, out);
        walk(module, &fun.children, path, known, out);
    }
}

fn check_one(
    module: &Module,
    fun: &Function,
    path: Option<&Path>,
    known: &[String],
    out: &mut Vec<Diagnostic>,
) {
    if let Some(c) = &fun.contract {
        for p in &c.params {
            type_name_check(&p.ty, c, &fun.name, path, fun.span, known, out);
        }
        if let Some(r) = &c.returns {
            type_name_check(&r.ty, c, &fun.name, path, fun.span, known, out);
        }
        for f in &c.fields {
            type_name_check(&f.ty, c, &fun.name, path, fun.span, known, out);
        }
        // ---- T2.5 防漂移：契约行 vs 升参推断形参（双向） ----
        let inferred: Vec<String> = fun.params.iter().map(|p| p.name.clone()).collect();
        for p in &c.params {
            if !inferred.contains(&p.name) {
                out.push(c.diagnostic(
                    "contract.param_drift",
                    &fun.name,
                    format!(
                        "契约参数 `{}` 不是 `{}` 的形参（当前形参：[{}]）",
                        p.name,
                        fun.name,
                        inferred.join(", ")
                    ),
                    "删除契约表该行，或让该名在函数体被读取（升参）/ 用 `+` 行显式声明".to_string(),
                    path,
                    fun.span,
                ));
            }
        }
        for name in &inferred {
            if !c.params.iter().any(|p| p.name == *name) {
                let d = c
                    .diagnostic(
                        "contract.param_drift",
                        &fun.name,
                        format!(
                            "形参 `{name}` 未被契约覆盖（可能是新推断的升参；契约表一经出现即为完整声明）"
                        ),
                        "在契约表补一行 `| {name} | 类型 | 说明 |`，或删掉契约表回到纯动态".to_string(),
                        path,
                        fun.span,
                    )
                    .with_severity(Severity::Warning);
                out.push(d);
            }
        }
    }
    for (name, c) in &fun.var_contracts {
        for f in &c.fields {
            type_name_check(&f.ty, c, name, path, fun.span, known, out);
        }
    }
    // ---- 错位契约：契约形状的未绑定表被提取器原样留在 body ----
    for stmt in &fun.body {
        let Stmt::Expr { value: Expr::Map(pairs), span } = stmt else {
            continue;
        };
        if let Some(Ok(c)) = contract::parse_contract_table(pairs, span.line, span.line, Vec::new())
        {
            out.push(c.diagnostic(
                "contract.misplaced",
                &fun.name,
                "契约表位置不允许（形参/返回表只能在函数体首个可执行行之前；字段表只能紧邻绑定或对象体首成员之前），已按普通表处理".to_string(),
                "移动契约表到允许位置，或改表头避开契约形状（含 `类型` 列）".to_string(),
                path,
                *span,
            ));
        }
    }
    let _ = module;
}

/// 对象类型名必须已知；拼错（`nubmer`）⇒ `contract.unknown_type` + 最近建议。
fn type_name_check(
    ty: &Ty,
    c: &Contract,
    unit: &str,
    path: Option<&Path>,
    span: Span,
    known: &[String],
    out: &mut Vec<Diagnostic>,
) {
    let Ty::Object(name) = ty else { return };
    if known.iter().any(|k| k == name) {
        return;
    }
    let suggestion = match suggest(name, known) {
        Some(near) => format!("是不是想写 `{near}`？已知类型：{}", known.join(" / ")),
        None => format!("已知类型：{}", known.join(" / ")),
    };
    out.push(c.diagnostic(
        "contract.unknown_type",
        unit,
        format!("未知类型名 `{name}`（不是原语，也没有对应的 `#` 对象）"),
        suggestion,
        path,
        span,
    ));
}

const PRIMITIVES: &[&str] = &["text", "number", "bool", "list", "map", "any"];

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

fn suggest(name: &str, known: &[String]) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let mut best: Option<(usize, String)> = None;
    let candidates = PRIMITIVES.iter().map(|s| s.to_string()).chain(
        known.iter().cloned(),
    );
    for cand in candidates {
        let d = edit_distance(&lower, &cand.to_ascii_lowercase());
        if d <= 2 && best.as_ref().map(|(bd, _)| d < *bd).unwrap_or(true) {
            best = Some((d, cand));
        }
    }
    best.map(|(_, c)| c)
}

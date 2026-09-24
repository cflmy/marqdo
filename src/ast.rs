//! Marqdo AST (Phase I).

use std::collections::HashMap;

use crate::diagnostics::Span;
use crate::formula::Expr as FormulaExpr;
use crate::value::CodeBlock;

/// Frontmatter file import: `import bind:path.mq.md` (see `doc/design/module-namespace.md`).
#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub bind: String,
}

/// Frontmatter short-name bind: `import bind:lib.member` (same keyword as file import).
#[derive(Debug, Clone)]
pub struct Use {
    pub path: Vec<String>,
    pub bind: String,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub imports: Vec<Import>,
    pub uses: Vec<Use>,
    pub functions: Vec<Function>,
    /// Populated by the loader: bind name → imported module (not flattened).
    pub import_modules: HashMap<String, Module>,
    /// Unresolved Artifact Metadata (from frontmatter); filled by parse.
    pub metadata_raw: Vec<(String, crate::binding::MetaValue)>,
    /// Bound metadata after load-time Binding resolve (insertion order).
    pub metadata: Vec<(String, crate::value::Value)>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            imports: Vec::new(),
            uses: Vec::new(),
            functions: Vec::new(),
            import_modules: HashMap::new(),
            metadata_raw: Vec::new(),
            metadata: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
    /// True when collected by v0.3 body/prose inference (not an explicit `+` line).
    pub inferred: bool,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    /// Heading depth 1–6. Level 1 = object/type; level ≥ 2 = function/method.
    pub level: u8,
    pub span: Span,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub children: Vec<Function>,
    /// Base type name for `# Child = > Parent` (objects only).
    pub base: Option<String>,
    /// Prose `` `名` `` never read/assigned on the executable surface (not params).
    pub dead_binds: Vec<String>,
    /// 渐进式契约（T2.1）：形参/返回/对象字段契约（附着在文档表上，升参推断前提取）。
    pub contract: Option<crate::contract::Contract>,
    /// 集合契约（T2.1）：变量名 → 字段契约（紧邻 `` `名` = `` 绑定之前的文档表）。
    pub var_contracts: HashMap<String, crate::contract::Contract>,
}

impl Function {
    /// `# Name` — object / type (constructor body).
    pub fn is_object(&self) -> bool {
        self.level == 1
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        name: String,
        value: Expr,
        span: Span,
        /// Inclusive last source line (covers following `$$` fence or table rows).
        end_line: u32,
    },
    Return {
        value: Expr,
        span: Span,
    },
    /// Statement-form call (`> …`); return value discarded.
    Call {
        call: CallExpr,
        span: Span,
    },
    /// Bold expression statement (`**n + 1**`); value evaluated and discarded.
    Expr {
        value: Expr,
        span: Span,
    },
    Branch {
        arms: Vec<BranchArm>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    ForEach {
        item: String,
        collection: String,
        body: Vec<Stmt>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct BranchArm {
    pub condition: Option<Expr>, // None = else
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: String,
    /// Bare library path segments, e.g. `["time","parse"]` for `time.parse`.
    /// Mutually exclusive with `receiver` (instance method).
    pub path: Option<Vec<String>>,
    /// When set, `` `recv`.callee `` method call; receiver is a variable name.
    pub receiver: Option<String>,
    pub args: Vec<Arg>,
    /// Pre-bracket modifiers: `礼貌 [问候] x` → `["礼貌"]`, expanded to `礼貌=True` at eval.
    /// See [bracket-call-modifiers.md](../../doc/design/bracket-call-modifiers.md).
    pub pre_modifiers: Vec<String>,
    /// True when the call was authored with `[callee]` (even with no modifiers).
    /// View prefers `修饰 [callee] …` / `[callee] …` over `> callee …` when set.
    pub bracket_marked: bool,
}

impl CallExpr {
    /// Fold `pre_modifiers` into leading `name=True` named args (conflicts → Err).
    pub fn with_modifiers_expanded(&self) -> Result<CallExpr, String> {
        let mut call = self.clone();
        for m in &call.pre_modifiers {
            if call
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named { name, .. } if name == m))
            {
                return Err(format!(
                    "pre-bracket modifier `{m}` conflicts with named argument `{m}`"
                ));
            }
        }
        if call.pre_modifiers.is_empty() {
            return Ok(call);
        }
        let mut args = Vec::with_capacity(call.pre_modifiers.len() + call.args.len());
        for m in &call.pre_modifiers {
            args.push(Arg::Named {
                name: m.clone(),
                value: Expr::Literal(Literal::Bool(true)),
            });
        }
        args.append(&mut call.args);
        call.args = args;
        call.pre_modifiers.clear();
        Ok(call)
    }
}

#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Expr),
    Named { name: String, value: Expr },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Var(String),
    /// Text with embedded `` `var` `` segments.
    Interp(Vec<InterpPart>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call(CallExpr),
    List(Vec<Expr>),
    /// Object literal / horizontal table (`map` value).
    Map(Vec<(String, Expr)>),
    /// Collection index: `` [拿铁](菜单) `` / `` [`名`](菜单) `` (preferred), or
    /// legacy footnote `` `菜单`[^拿铁] `` / `` `菜单`[^`名`] ``.
    Index {
        base: Box<Expr>,
        /// Key or 1-based list index; see [`IndexKey`].
        label: IndexKey,
    },
    /// Parsed `$$…$$` formula tree (from assignment RHS).
    Formula(FormulaExpr),
    /// Bound ```lang … ``` fence (`code` value).
    Code(CodeBlock),
}

#[derive(Debug, Clone)]
pub enum InterpPart {
    Lit(String),
    Var(String),
    /// `` `base`[^label] `` — variable followed by footnote index inside interpolated
    /// text (chained labels fold into one part, e.g. `` `x`[^a][^b] `` → two labels).
    Index { base: String, labels: Vec<String> },
}

/// Map / list index key (`[key](coll)` or legacy `[^…]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexKey {
    /// Static text: `[拿铁](…)` / `[^拿铁]` / `[^1]`.
    Lit(String),
    /// Dynamic: `` [`品名`](…) `` / `` [^`品名`] `` — evaluate variable, then stringify as key.
    Var(String),
}

#[derive(Debug, Clone)]
pub enum Literal {
    None,
    Bool(bool),
    Int(i64),
    /// Decimal literal (`0.85`, `3.14`).
    Num(f64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Pretty-print AST for `--dump-ast`.
pub fn format_ast_dump(path: &str, module: &Module) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== marqdo: ast ({path}) ===\n"));
    if !module.imports.is_empty() {
        let specs: Vec<String> = module
            .imports
            .iter()
            .map(|i| format!("import {}:{}", i.bind, i.path))
            .collect();
        out.push_str(&format!("(imports {:?})\n", specs));
    }
    for fun in &module.functions {
        dump_fun(&mut out, fun, 0);
    }
    out.push_str("=== marqdo: end ast ===\n");
    out
}

fn dump_fun(out: &mut String, fun: &Function, depth: usize) {
    let pad = "  ".repeat(depth);
    let kind = if fun.is_object() { "object" } else { "fun" };
    let params = format_params(&fun.params);
    let dead = if fun.dead_binds.is_empty() {
        String::new()
    } else {
        format!(" dead={:?}", fun.dead_binds)
    };
    match &fun.base {
        Some(base) => out.push_str(&format!(
            "{pad}({kind} level={} {:?} extends={base:?} params={params}{dead} @{}\n",
            fun.level, fun.name, fun.span
        )),
        None => out.push_str(&format!(
            "{pad}({kind} level={} {:?} params={params}{dead} @{}\n",
            fun.level, fun.name, fun.span
        )),
    }
    for stmt in &fun.body {
        dump_stmt(out, stmt, depth + 1);
    }
    for child in &fun.children {
        dump_fun(out, child, depth + 1);
    }
    out.push_str(&format!("{pad})\n"));
}

fn dump_stmt(out: &mut String, stmt: &Stmt, depth: usize) {
    let pad = "  ".repeat(depth);
    match stmt {
        Stmt::Assign {
            name,
            value,
            span,
            end_line,
        } => {
            out.push_str(&format!(
                "{pad}(assign {name:?} {value:?} @{span}..{end_line})\n"
            ));
        }
        Stmt::Return { value, span } => {
            out.push_str(&format!("{pad}(return {value:?} @{span})\n"));
        }
        Stmt::Call { call, span } => {
            out.push_str(&format!(
                "{pad}(call recv={:?} {:?} {:?} @{span}\n",
                call.receiver, call.callee, call.args
            ));
        }
        Stmt::Expr { value, span } => {
            out.push_str(&format!("{pad}(expr {value:?} @{span})\n"));
        }
        Stmt::Branch { arms, span } => {
            out.push_str(&format!("{pad}(branch @{span}\n"));
            for arm in arms {
                out.push_str(&format!("{pad}  (arm cond={:?}\n", arm.condition));
                for s in &arm.body {
                    dump_stmt(out, s, depth + 2);
                }
                out.push_str(&format!("{pad}  )\n"));
            }
            out.push_str(&format!("{pad})\n"));
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            out.push_str(&format!("{pad}(while {condition:?} @{span}\n"));
            for s in body {
                dump_stmt(out, s, depth + 1);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Stmt::ForEach {
            item,
            collection,
            body,
            span,
        } => {
            out.push_str(&format!(
                "{pad}(foreach {item:?} in {collection:?} @{span}\n"
            ));
            for s in body {
                dump_stmt(out, s, depth + 1);
            }
            out.push_str(&format!("{pad})\n"));
        }
    }
}

fn format_params(params: &[Param]) -> String {
    let parts: Vec<String> = params
        .iter()
        .map(|p| {
            let mut s = p.name.clone();
            if p.default.is_some() {
                s.push('?');
            }
            if p.inferred {
                s.push_str(" inferred");
            }
            s
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

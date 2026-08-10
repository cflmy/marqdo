//! Marqdo AST (Phase I).

use std::collections::HashMap;

use crate::diagnostics::Span;
use crate::formula::Expr as FormulaExpr;
use crate::value::CodeBlock;

/// Frontmatter import: path + library bind name (see `doc/design/module-namespace.md`).
#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub bind: String,
}

/// Frontmatter `use` / `使用`: bind a short name to a library path (`time.format`).
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
}

impl Default for Module {
    fn default() -> Self {
        Self {
            imports: Vec::new(),
            uses: Vec::new(),
            functions: Vec::new(),
            import_modules: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
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
    /// Footnote index: `` `xs`[^1] `` / `` `m`[^key] `` (see tables-maps-footnotes).
    Index {
        base: Box<Expr>,
        /// Label inside `[^…]` (digits → 1-based list index; else map key).
        label: String,
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
}

#[derive(Debug, Clone)]
pub enum Literal {
    None,
    Bool(bool),
    Int(i64),
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
            .map(|i| format!("{} as {}", i.path, i.bind))
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
    match &fun.base {
        Some(base) => out.push_str(&format!(
            "{pad}({kind} level={} {:?} extends={base:?} params={:?} @{}\n",
            fun.level, fun.name, fun.params, fun.span
        )),
        None => out.push_str(&format!(
            "{pad}({kind} level={} {:?} params={:?} @{}\n",
            fun.level, fun.name, fun.params, fun.span
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

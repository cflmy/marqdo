//! Marqdo AST (Phase I).

use crate::diagnostics::Span;
use crate::formula::Expr as FormulaExpr;
use crate::value::CodeBlock;

#[derive(Debug, Clone)]
pub struct Module {
    pub imports: Vec<String>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    /// Heading depth 1–6. Level 1 = object/type; level ≥ 2 = function/method.
    pub level: u8,
    pub span: Span,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub children: Vec<Function>,
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
    /// Object literal / table row (`map` value).
    Map(Vec<(String, Expr)>),
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
        out.push_str(&format!("(imports {:?})\n", module.imports));
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
    out.push_str(&format!(
        "{pad}({kind} level={} {:?} params={:?} @{}\n",
        fun.level, fun.name, fun.params, fun.span
    ));
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

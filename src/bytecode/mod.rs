//! Bytecode chunk + opcodes (M5 prototype).

mod compile;
mod vm;
mod verify;
mod expression;
mod statement;
mod call;
mod source;
mod names;
pub use source::SourceFunction;
pub use call::{CallEmission, CallStage, CallDispatch, PathFallback};
pub use statement::{StatementRule,StatementEmission};
pub use expression::{ExpressionRule,ExpressionEmission,value_evidence as expression_value_evidence};
pub use verify::StackCertificate;

pub use compile::{compile_for_call, compile_module};
pub use vm::Vm;

use std::collections::HashMap;

use crate::diagnostics::Span;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Constant(usize),
    True,
    False,
    None_,
    Pop,
    GetLocal(usize),
    SetLocal(usize),
    Add,
    Sub,
    Mul,
    Div,
    Negate,
    Not,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    Lessequal,
    /// Absolute jump to instruction index.
    Jump(usize),
    JumpIfFalse(usize),
    Print,
    /// Pop prompt text, print it (no newline), read one stdin line → push Text.
    Input,
    BuildList(usize),
    /// Stack (bottom→top): k0, v0, k1, v1, … (n pairs); push Map.
    BuildMap(usize),
    /// Pop value → push `len` (text/list).
    Len,
    /// Pop value → push display text.
    Str,
    /// Pop value → push int (or fail).
    Int,
    /// Pop value → push type name text.
    TypeOf,
    /// Pop text → push trimmed text.
    Trim,
    /// Pop sep, then value → push list of text parts.
    Split,
    /// Pop sep, then list → push joined text.
    Join,
    GetIndex,
    /// Pop list/map → push list of foreach items (list elements or map keys).
    IterItems,
    /// Pop footnote label text, then base → push indexed value (1-based list / map key).
    FootnoteGet,
    /// Call function by index; argc values are on the stack (param order).
    Call(usize, usize),
    /// Evaluated actuals in authored order; layout preserves optional names.
    CallArgs(usize, usize),
    /// Runtime binding fault after all explicit actual effects.
    RejectDuplicateArgument(usize),
    /// Modifier/argument preparation failed before any actual is evaluated.
    RejectArgumentExpansion(usize),
    /// Skip a callee default when the parameter was explicitly supplied.
    JumpIfProvided(usize, usize),
    /// Entry/raw-call parameters without a value retain language None after defaults.
    FillMissingParameters,
    ResetLoop(usize),
    CheckLoop(usize),
    CheckArguments,
    RestoreReceiver,
    CheckFrameArguments,
    CheckReturn,
    /// Check the stack top against the named local's field contract.
    CheckBinding(usize),
    /// Checked footnote access; operand identifies the contracted base local.
    ContractFootnoteGet(usize),
    /// Pop function name text → call zero-arg top-level or lexically visible function.
    CallFn,
    /// After a constructor call: stamp name and module of functions[idx].
    TagInstance(usize),
    /// Method call: stack has argc args (param order) then receiver on top.
    /// constants[name_idx] is method name text; resolve via receiver `_type`.
    MethodCall(usize, usize),
    /// Resolve the receiver first, then bind against that method's signature.
    MethodCallArgs(usize, usize),
    /// Plugin call: stack has argc args in registered param order; name in constants.
    PluginCall(usize, usize),
    /// Host primitive: pop `argc` args (param order), push result.
    HostCall(u16, usize),
    /// Native dispatch with authored argument labels; binding happens at runtime.
    BuiltinCallArgs(usize, usize),
    HostCallArgs(u16, usize),
    PluginCallArgs(usize, usize),
    /// Two-component paths inspect actual receiver presence before static fallback.
    PathCallArgs(usize, usize),
    RejectCallResolution(usize),
    Return,
}

#[derive(Debug, Clone)]
pub struct FnChunk {
    /// Captured directly from the loaded declaration before emission.
    pub source_ast: SourceFunction,
    pub module_scope: u32,
    pub module_path: Vec<String>,
    pub source: crate::diagnostics::SourceIdentity,
    pub declaration_span: Span,
    pub name: String,
    pub params: Vec<String>,
    pub contract: Option<crate::contract::Contract>,
    pub var_contracts: HashMap<String, crate::contract::Contract>,
    pub receiver_aliases: bool,
    pub defaults: Vec<bool>,
    pub default_expressions: Vec<Option<usize>>,
    pub call_layouts: Vec<Vec<Option<String>>>,
    pub calls: Vec<CallEmission>,
    /// Source names are reserved before synthetic slots are allocated.
    pub source_local_count: usize,
    pub expressions: Vec<ExpressionEmission>,
    pub statements: Vec<StatementEmission>,
    pub statement_roots: Vec<usize>,
    pub body_start: usize,
    pub code: Vec<Op>,
    /// Parallel to `code`: source location for diagnostics / debug.
    pub spans: Vec<Span>,
    pub constants: Vec<Value>,
    /// Local slot names (params first).
    pub locals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FnChunk>,
    pub entry: usize,
    /// Applied by run(), not by named or nested calls to the entry function.
    pub entry_metadata: Vec<(String, Value)>,
    /// Top-level object name → function index (level-1).
    pub objects: Vec<(String, usize)>,
    /// Parallel to `objects`: base object flat index (`# Child = > Parent`).
    pub object_bases: Vec<Option<usize>>,
    /// (object_fn_idx, method_name) → method fn idx
    pub methods: Vec<(usize, String, usize)>,
    /// Parallel to `functions`: parent fn index for dynamic `call_fn` lookup.
    pub parents: Vec<Option<usize>>,
    /// Parallel to `functions`: child fn indices.
    pub children: Vec<Vec<usize>>,
    /// Dotted path (`计算器.加法`, `inc.加一`) → function index. Entry module exports plus imports.
    pub path_index: HashMap<Vec<String>, usize>,
}

impl Program {
    pub fn disassemble(&self) -> String {
        let mut out = String::new();
        out.push_str("=== marqdo: bytecode ===\n");
        for (fi, fun) in self.functions.iter().enumerate() {
            let entry = if fi == self.entry { "  <entry>" } else { "" };
            out.push_str(&format!(
                "fn[{fi}] {} params={:?} locals={:?}{entry}\n",
                fun.name, fun.params, fun.locals
            ));
            for (i, c) in fun.constants.iter().enumerate() {
                out.push_str(&format!("  const[{i}] = {c:?}\n"));
            }
            for (ip, op) in fun.code.iter().enumerate() {
                let sp = fun.spans.get(ip).copied().unwrap_or(Span::new(1, 1));
                out.push_str(&format!("  {ip:04} {op:?}  @{sp}\n"));
            }
        }
        out.push_str("=== marqdo: end bytecode ===\n");
        out
    }
}

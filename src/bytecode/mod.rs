//! Bytecode chunk + opcodes (M5 prototype).

mod compile;
mod vm;

pub use compile::compile_module;
pub use vm::Vm;

use crate::diagnostics::Span;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Constant(u16),
    True,
    False,
    None_,
    Pop,
    GetLocal(u8),
    SetLocal(u8),
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
    Jump(u16),
    JumpIfFalse(u16),
    Print,
    /// Pop prompt text, print it (no newline), read one stdin line → push Text.
    Input,
    BuildList(u16),
    /// Stack (bottom→top): k0, v0, k1, v1, … (n pairs); push Map.
    BuildMap(u16),
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
    Call(u16, u8),
    /// Pop function name text → call zero-arg top-level or lexically visible function.
    CallFn,
    /// After a constructor Call: tag top-of-stack map with `_type` = constants[idx].
    TagInstance(u16),
    /// Method call: stack has argc args (param order) then receiver on top.
    /// constants[name_idx] is method name text; resolve via receiver `_type`.
    MethodCall(u16, u8),
    /// Plugin call: stack has argc args in registered param order; name in constants.
    PluginCall(u16, u8),
    /// Host primitive: pop `argc` args (param order), push result.
    HostCall(u16, u8),
    Return,
}

#[derive(Debug, Clone)]
pub struct FnChunk {
    pub name: String,
    pub params: Vec<String>,
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
    /// Top-level object name → function index (level-1).
    pub objects: Vec<(String, usize)>,
    /// (object_fn_idx, method_name) → method fn idx
    pub methods: Vec<(usize, String, usize)>,
    /// Parallel to `functions`: parent fn index for dynamic `call_fn` lookup.
    pub parents: Vec<Option<usize>>,
    /// Parallel to `functions`: child fn indices.
    pub children: Vec<Vec<usize>>,
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

//! Bytecode chunk + opcodes (M5 prototype).

mod compile;
mod vm;

pub use compile::compile_module;
pub use vm::Vm;

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
    BuildList(u16),
    Len,
    GetIndex,
    /// Call function by index; argc values are on the stack (param order).
    Call(u16, u8),
    Return,
}

#[derive(Debug, Clone)]
pub struct FnChunk {
    pub name: String,
    pub params: Vec<String>,
    pub code: Vec<Op>,
    pub constants: Vec<Value>,
    /// Local slot names (params first).
    pub locals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FnChunk>,
    pub entry: usize,
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
                out.push_str(&format!("  {ip:04} {op:?}\n"));
            }
        }
        out.push_str("=== marqdo: end bytecode ===\n");
        out
    }
}

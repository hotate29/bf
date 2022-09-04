use std::fmt::Write as _;
use std::io::{self, Write};

use super::{type_::Type, var::Var};

pub struct FunctionBody {
    locals: Vec<LocalEntry>,
    pub code: Vec<Op>,
}
impl FunctionBody {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            code: Vec::new(),
        }
    }
    pub fn from_ops(ops: &[Op]) -> Self {
        Self {
            locals: Vec::new(),
            code: ops.to_vec(),
        }
    }
    pub fn push_local(&mut self, local_entry: LocalEntry) {
        self.locals.push(local_entry)
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let mut body_payload = Vec::new();

        let local_count = Var(self.locals.len() as u32);
        local_count.write(&mut body_payload)?;

        for local in &self.locals {
            local.write(&mut body_payload)?;
        }

        self.code.write(&mut body_payload)?;

        let body_size = Var(body_payload.len() as u32);
        body_size.write(&mut w)?;

        w.write_all(&body_payload)
    }
}

pub struct LocalEntry {
    pub count: Var<u32>,
    pub type_: Type,
}
impl LocalEntry {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        self.count.write(&mut w)?;
        self.type_.write(&mut w)
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Nop,
    End,
    Block { block_type: Type },
    Loop { block_type: Type },
    If { block_type: Type },
    Br { relative_depth: Var<u32> },
    BrIf { relative_depth: Var<u32> },

    Call { function_index: Var<u32> },

    Drop,

    GetLocal { local_index: Var<u32> },
    SetLocal { local_index: Var<u32> },
    TeeLocal { local_index: Var<u32> },

    I32Load8U(MemoryImmediate),
    I32Store(MemoryImmediate),
    I32Store8(MemoryImmediate),

    I32Const(Var<i32>),

    I32Eqz,
    I32Ne,

    I32Add,
    I32Sub,
    I32Mul,
}

impl Op {
    pub fn write_str(&self, s: &mut String) {
        match self {
            Op::Nop => (),
            Op::End => *s += "end",
            Op::Block { block_type } => {
                assert_eq!(block_type, &Type::Void);
                *s += "block";
            }
            Op::Loop { block_type } => {
                assert_eq!(block_type, &Type::Void);
                *s += "loop";
            }
            Op::If { block_type } => {
                assert_eq!(block_type, &Type::Void);
                *s += "if";
            }
            Op::Br { relative_depth } => write!(s, "br {}", relative_depth.0).unwrap(),
            Op::BrIf { relative_depth } => write!(s, "br_if {}", relative_depth.0).unwrap(),
            Op::Call { function_index } => write!(s, "call {}", function_index.0).unwrap(),
            Op::Drop => *s += "drop",
            Op::GetLocal { local_index } => write!(s, "local.get {}", local_index.0).unwrap(),
            Op::SetLocal { local_index } => write!(s, "local.set {}", local_index.0).unwrap(),
            Op::TeeLocal { local_index } => write!(s, "local.tee {}", local_index.0).unwrap(),
            Op::I32Load8U(offset) => write!(s, "i32.load8_u offset={}", offset.offset.0).unwrap(),
            Op::I32Store(offset) => write!(s, "i32.store offset={}", offset.offset.0).unwrap(),
            Op::I32Store8(offset) => write!(s, "i32.store8 offset={}", offset.offset.0).unwrap(),
            Op::I32Const(var) => write!(s, "i32.const {}", var.0).unwrap(),
            Op::I32Eqz => *s += "i32.eqz",
            Op::I32Ne => *s += "i32.ne",
            Op::I32Add => *s += "i32.add",
            Op::I32Sub => *s += "i32.sub",
            Op::I32Mul => *s += "i32.mul",
        }
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        match self {
            Op::Nop => w.write_all(&[0x01]),
            Op::End => w.write_all(&[0x0b]),
            Op::Block { block_type } => {
                w.write_all(&[0x02])?;
                block_type.write(&mut w)
            }
            Op::Loop { block_type } => {
                w.write_all(&[0x03])?;
                block_type.write(&mut w)
            }
            Op::If { block_type } => {
                w.write_all(&[0x04])?;
                block_type.write(&mut w)
            }
            Op::Br { relative_depth } => {
                w.write_all(&[0x0c])?;
                relative_depth.write(&mut w)
            }
            Op::BrIf { relative_depth } => {
                w.write_all(&[0x0d])?;
                relative_depth.write(&mut w)
            }
            Op::Call { function_index } => {
                w.write_all(&[0x10])?;
                function_index.write(&mut w)
            }
            Op::Drop => w.write_all(&[0x1a]),
            Op::GetLocal { local_index } => {
                w.write_all(&[0x20])?;
                local_index.write(w)
            }
            Op::SetLocal { local_index } => {
                w.write_all(&[0x21])?;
                local_index.write(w)
            }
            Op::TeeLocal { local_index } => {
                w.write_all(&[0x22])?;
                local_index.write(w)
            }
            Op::I32Load8U(memory_immediate) => {
                w.write_all(&[0x2d])?;
                memory_immediate.write(w)
            }
            Op::I32Store(memory_immediate) => {
                w.write_all(&[0x36])?;
                memory_immediate.write(w)
            }
            Op::I32Store8(memory_immediate) => {
                w.write_all(&[0x3a])?;
                memory_immediate.write(w)
            }
            Op::I32Const(literal) => {
                w.write_all(&[0x41])?;
                literal.write(w)
            }
            Op::I32Eqz => w.write_all(&[0x45]),
            Op::I32Ne => w.write_all(&[0x47]),
            Op::I32Add => w.write_all(&[0x6a]),
            Op::I32Sub => w.write_all(&[0x6b]),
            Op::I32Mul => w.write_all(&[0x6c]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryImmediate {
    flags: Var<u32>,
    offset: Var<u32>,
}
impl MemoryImmediate {
    pub fn i8(offset: u32) -> Self {
        Self {
            flags: Var(0),
            offset: Var(offset),
        }
    }
    pub fn i32(offset: u32) -> Self {
        Self {
            flags: Var(2),
            offset: Var(offset),
        }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        self.flags.write(&mut w)?;
        self.offset.write(&mut w)
    }
}

pub trait OpSlice {
    fn write(&self, w: impl Write) -> io::Result<()>;
    fn write_str(&self, s: &mut String);
}

impl OpSlice for [Op] {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        for op in self {
            op.write(&mut w)?
        }
        Ok(())
    }

    fn write_str(&self, s: &mut String) {
        for op in self {
            op.write_str(s);
            *s += "\n";
        }
    }
}

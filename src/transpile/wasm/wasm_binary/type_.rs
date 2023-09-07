use std::io::{self, Write};

use super::leb128::WriteLeb128;

pub enum Type {
    Func(FuncSignature),
    Value(ValueType),
}

impl Type {
    pub fn write(&self, w: impl Write) -> io::Result<()> {
        match self {
            Type::Func(func_signature) => func_signature.write(w),
            Type::Value(value_type) => value_type.write(w),
        }
    }
}

pub struct FuncSignature {
    pub params: Vec<ValueType>,
    pub result: Option<ValueType>,
}

impl FuncSignature {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        // Func
        (-0x20).write_leb128(&mut w)?;

        // Len
        let params_len = self.params.len() as u32;
        params_len.write_leb128(&mut w)?;

        for param_type in &self.params {
            param_type.write(&mut w)?
        }

        if let Some(result) = &self.result {
            true.write_leb128(&mut w)?;
            result.write(&mut w)?
        } else {
            false.write_leb128(w)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    I32,
    // I64,
    // F32,
    // F64,
    Void,
}
impl ValueType {
    fn opcode(&self) -> i8 {
        match self {
            ValueType::I32 => -0x1,
            // Type::I64 => -0x2,
            // Type::F32 => -0x3,
            // Type::F64 => -0x4,
            // Type::AnyFunc => -0x10,
            ValueType::Void => -0x40,
        }
    }
    pub fn write(&self, w: impl Write) -> io::Result<()> {
        match self {
            ValueType::I32 | ValueType::Void => self.opcode().write_leb128(w),
        }
    }
}

use std::io::{self, Write};

use super::leb128::WriteLeb128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    // I64,
    // F32,
    // F64,
    // AnyFunc,
    Func {
        params: Vec<Type>,
        result: Option<Box<Type>>,
    },
    Void,
}
impl Type {
    fn opcode(&self) -> i8 {
        match self {
            Type::I32 => -0x1,
            // Type::I64 => -0x2,
            // Type::F32 => -0x3,
            // Type::F64 => -0x4,
            // Type::AnyFunc => -0x10,
            Type::Func { .. } => -0x20,
            Type::Void => -0x40,
        }
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        match self {
            Type::I32 | Type::Void => self.opcode().write_leb128(w),
            // AnyFuncを使う予定は無いのでunimplemented!
            // Type::AnyFunc => unimplemented!(),
            Type::Func { params, result } => {
                // Func
                self.opcode().write_leb128(&mut w)?;

                // Len
                let params_len = params.len() as u32;
                params_len.write_leb128(&mut w)?;

                for param_type in params {
                    param_type.write(&mut w)?
                }

                if let Some(result) = result {
                    true.write_leb128(&mut w)?;
                    result.write(w)?
                } else {
                    false.write_leb128(w)?;
                }

                Ok(())
            }
        }
    }
}

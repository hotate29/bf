const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_BINARY_VERSION: u32 = 1;

mod var;

use std::io::{self, Write};

use var::Var;

enum Type {
    I32,
    I64,
    F32,
    F64,
    AnyFunc,
    Func {
        params: Vec<Type>,
        result: Option<Box<Type>>,
    },
    Void,
}
impl Type {
    fn opcode_int(&self) -> i8 {
        match self {
            Type::I32 => -1,
            Type::I64 => -2,
            Type::F32 => -3,
            Type::F64 => -4,
            Type::AnyFunc => -10,
            Type::Func { .. } => -20,
            Type::Void => -40,
        }
    }
    fn opcode_var(&self) -> Var<i8> {
        Var(self.opcode_int())
    }
    fn write(&self, mut writer: impl Write) -> io::Result<()> {
        match self {
            Type::I32 | Type::I64 | Type::F32 | Type::F64 | Type::Void => {
                self.opcode_var().write(writer)
            }
            // AnyFuncを使う予定は無いのでunimplemented!
            Type::AnyFunc => unimplemented!(),
            Type::Func { params, result } => {
                self.opcode_var().write(&mut writer)?;

                let params_len = Var(params.len() as u32);
                params_len.write(&mut writer)?;

                for param_type in params {
                    param_type.write(&mut writer)?
                }

                if let Some(result) = result {
                    Var(true).write(&mut writer)?;
                    result.write(writer)?
                } else {
                    Var(false).write(&mut writer)?;
                }

                Ok(())
            }
        }
    }
}

pub struct Module {}
impl Module {
    pub fn new() -> Self {
        Module {}
    }

    pub fn write(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(WASM_BINARY_MAGIC.to_le_bytes().as_slice())?;
        writer.write_all(WASM_BINARY_VERSION.to_le_bytes().as_slice())?;
        Ok(())
    }
}
enum Section {
    Type(Type),
    Import,
    Function,
    Table,
    Memory,
    Data,
    Global,
    Start,
    Element,
    Code,
}

use std::io::{self, Write};

use super::var::Var;

pub enum Type {
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
            Type::I32 => -0x1,
            Type::I64 => -0x2,
            Type::F32 => -0x3,
            Type::F64 => -0x4,
            Type::AnyFunc => -0x10,
            Type::Func { .. } => -0x20,
            Type::Void => -0x40,
        }
    }
    fn opcode_var(&self) -> Var<i8> {
        Var(self.opcode_int())
    }
    // 本当はimpl Writeとしたいけど、E0275エラーが発生してコンパイルが通らなかった。かなしい
    // https://doc.rust-lang.org/error-index.html#E0275
    pub fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
        match self {
            Type::I32 | Type::I64 | Type::F32 | Type::F64 | Type::Void => {
                self.opcode_var().write(w)
            }
            // AnyFuncを使う予定は無いのでunimplemented!
            Type::AnyFunc => unimplemented!(),
            Type::Func { params, result } => {
                // Func
                self.opcode_var().write(&mut w)?;

                // Len
                let params_len = Var(params.len() as u32);
                params_len.write(&mut w)?;

                for param_type in params {
                    param_type.write(&mut w)?
                }

                if let Some(result) = result {
                    Var(true).write(&mut w)?;
                    result.write(w)?
                } else {
                    Var(false).write(w)?;
                }

                Ok(())
            }
        }
    }
}

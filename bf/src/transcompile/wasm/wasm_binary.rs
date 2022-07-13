const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_BINARY_VERSION: u32 = 1;

mod section;
mod var;

use std::io::{self, prelude::*};

use section::{Section, TypeSection};
use var::Var;

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
    fn write(&self, mut w: &mut dyn Write) -> io::Result<()> {
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

struct Module {
    sections: Vec<Section>,
}
impl Module {
    fn new() -> Self {
        Module {
            sections: Vec::new(),
        }
    }

    fn write(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(WASM_BINARY_MAGIC.to_le_bytes().as_slice())?;
        w.write_all(WASM_BINARY_VERSION.to_le_bytes().as_slice())?;

        for section in &self.sections {
            section.write(&mut w)?;
        }
        Ok(())
    }
}

#[test]
fn a() {
    use crate::transcompile::wasm::wasm_binary::section::FunctionSection;
    use crate::transcompile::wasm::wasm_binary::section::{ImportEntry, ImportSection};

    use std::fs::File;

    let mut module = Module::new();

    let mut type_section = TypeSection::new();

    type_section.push(Type::Func {
        params: vec![Type::I32, Type::I32, Type::I32, Type::I32],
        result: Some(Box::new(Type::I32)),
    });

    module.sections.push(Section::Type(type_section));

    let mut import_section = ImportSection::new();

    let entry = ImportEntry::function(
        "wasi_unstable".to_string(),
        "fd_write".to_string(),
        Var(0_u32),
    );
    import_section.push(entry);

    module.sections.push(Section::Import(import_section));

    let mut function_section = FunctionSection::new();

    function_section.push(Var(0));

    module.sections.push(Section::Function(function_section));

    let mut file = File::create("aa.wasm").unwrap();
    module.write(&mut file).unwrap();
}

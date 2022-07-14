const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_BINARY_VERSION: u32 = 1;

mod code;
mod section;
mod type_;
mod var;

use std::io::{self, Write};

use section::Section;
use type_::Type;

pub struct Module {
    sections: Vec<Section>,
}
impl Module {
    fn new() -> Self {
        Module {
            sections: Vec::new(),
        }
    }

    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        w.write_all(WASM_BINARY_MAGIC.to_le_bytes().as_slice())?;
        w.write_all(WASM_BINARY_VERSION.to_le_bytes().as_slice())?;

        for section in &self.sections {
            section.write(&mut w)?;
        }
        Ok(())
    }
}

struct ModuleBuilder {
    imports: Vec<Import>,
    functions: Vec<Function>,

    module: Module,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            imports: Vec::new(),
            module: Module::new(),
        }
    }
    pub fn push_import(&mut self, import: Import) {
        self.imports.push(import);
    }
    pub fn push_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    pub fn into_module(self) -> Module {
        unimplemented!()
    }
}

struct Function {
    signature: Type,
    code: Vec<u8>,
    export_name: Option<String>,
}

enum Import {
    Function {
        module_name: String,
        field_name: String,
        signature: Type,
    },
}

#[test]
fn a() {
    use crate::transcompile::wasm::wasm_binary::{
        code::{FunctionBody, MemoryImmediate, Op},
        section::{
            CodeSection, ExportEntry, ExportSection, ExternalKind, FunctionSection, ImportEntry,
            ImportSection, MemorySection, MemoryType, ResizableLimits, TypeSection,
        },
        type_::Type,
        var::Var,
    };

    use std::fs::File;

    let mut module = Module::new();

    {
        let mut type_section = TypeSection::new();

        type_section.push(Type::Func {
            params: vec![],
            result: None,
        });

        type_section.push(Type::Func {
            params: vec![Type::I32, Type::I32, Type::I32, Type::I32],
            result: Some(Box::new(Type::I32)),
        });

        type_section.push(Type::Func {
            params: vec![Type::I32],
            result: None,
        });

        module.sections.push(Section::Type(type_section));
    }

    {
        let mut import_section = ImportSection::new();

        let entry =
            ImportEntry::function("wasi_unstable".to_string(), "fd_write".to_string(), Var(1));
        import_section.push(entry);

        module.sections.push(Section::Import(import_section));
    }

    {
        let mut function_section = FunctionSection::new();
        function_section.push(Var(2));
        function_section.push(Var(0));

        module.sections.push(Section::Function(function_section));
    }

    {
        let mut memory_section = MemorySection::new();

        let limits = MemoryType {
            limits: ResizableLimits {
                flags: Var(true),
                initial: Var(1),
                maximum: Some(Var(2)),
            },
        };

        memory_section.push(limits);

        module.sections.push(Section::Memory(memory_section));
    }

    {
        let mut export_section = ExportSection::new();

        let export_entry = ExportEntry {
            field: "memory".to_string(),
            kind: ExternalKind::Memory,
            index: Var(0),
        };
        export_section.push(export_entry);

        let export_entry = ExportEntry {
            field: "_start".to_string(),
            kind: ExternalKind::Function,
            index: Var(2),
        };
        export_section.push(export_entry);

        module.sections.push(Section::Export(export_section));
    }

    {
        let mut code_section = CodeSection::new();

        let mut print_char = FunctionBody::new();

        Op::I32Const(Var(0)).write(&mut print_char.code).unwrap();
        Op::GetLocal {
            local_index: Var(0),
        }
        .write(&mut print_char.code)
        .unwrap();
        Op::I32Store8(MemoryImmediate::zero())
            .write(&mut print_char.code)
            .unwrap();

        Op::I32Const(Var(4)).write(&mut print_char.code).unwrap();
        Op::I32Const(Var(0)).write(&mut print_char.code).unwrap();
        Op::I32Store(MemoryImmediate::i32())
            .write(&mut print_char.code)
            .unwrap();

        Op::I32Const(Var(8)).write(&mut print_char.code).unwrap();
        Op::I32Const(Var(1)).write(&mut print_char.code).unwrap();
        Op::I32Store(MemoryImmediate::i32())
            .write(&mut print_char.code)
            .unwrap();

        Op::I32Const(Var(1)).write(&mut print_char.code).unwrap();
        Op::I32Const(Var(4)).write(&mut print_char.code).unwrap();
        Op::I32Const(Var(1)).write(&mut print_char.code).unwrap();
        Op::I32Const(Var(12)).write(&mut print_char.code).unwrap();

        Op::Call {
            function_index: Var(0),
        }
        .write(&mut print_char.code)
        .unwrap();

        Op::Drop.write(&mut print_char.code).unwrap();
        Op::End.write(&mut print_char.code).unwrap();

        code_section.push(print_char);

        let mut main = FunctionBody::new();

        Op::I32Const(Var(97)).write(&mut main.code).unwrap();
        Op::Call {
            function_index: Var(1),
        }
        .write(&mut main.code)
        .unwrap();
        Op::End.write(&mut main.code).unwrap();

        code_section.push(main);

        module.sections.push(Section::Code(code_section));
    }

    let mut file = File::create("aa.wasm").unwrap();
    module.write(&mut file).unwrap();
}

const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_BINARY_VERSION: u32 = 1;

mod code;
mod section;
mod type_;
mod var;

use std::io::{self, prelude::*};

use section::Section;

struct Module {
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

#[test]
fn a() {
    use crate::transcompile::wasm::wasm_binary::{
        code::{FunctionBody, Op},
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
            index: Var(1),
        };
        export_section.push(export_entry);

        module.sections.push(Section::Export(export_section));
    }

    {
        let mut code_section = CodeSection::new();

        let mut function_body = FunctionBody::new();

        Op::End.write(&mut function_body.code).unwrap();

        code_section.push(function_body);

        module.sections.push(Section::Code(code_section));
    }

    let mut file = File::create("aa.wasm").unwrap();
    module.write(&mut file).unwrap();
}

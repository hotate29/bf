const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_BINARY_VERSION: u32 = 1;

pub mod code;
pub mod leb128;
pub mod section;
pub mod type_;

use std::io::{self, Write};

use self::{
    code::{FunctionBody, LocalEntry},
    section::{
        CodeSection, ExportEntry, ExportSection, ExternalKind, FunctionSection, ImportEntry,
        ImportSection, MemorySection, MemoryType, Section, TypeSection,
    },
    type_::{FuncSignature, Type},
};

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

pub struct ModuleBuilder {
    imports: Vec<Import>,
    functions: Vec<Function>,
    memory: Memory,
}

impl ModuleBuilder {
    // 面倒くさいので一旦
    pub fn new(memory: Memory) -> Self {
        Self {
            functions: Vec::new(),
            imports: Vec::new(),
            memory,
        }
    }
    /// `push_function`の前に行う
    pub fn push_import(&mut self, import: Import) -> u32 {
        assert!(self.functions.is_empty());
        self.imports.push(import);

        (self.imports.len() - 1) as u32
    }
    /// `push_import`の後に行う
    pub fn push_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    pub fn into_module(self) -> Module {
        let mut module = Module::new();

        let mut type_section = TypeSection::new();
        let mut import_section = ImportSection::new();
        let mut function_section = FunctionSection::new();
        let mut code_section = CodeSection::new();
        let mut export_section = ExportSection::new();
        let mut memory_section = MemorySection::new();

        // Importをゴニョゴニョ
        for import in self.imports {
            // ぐぬぬ
            match import {
                Import::Function {
                    module_name,
                    field_name,
                    signature,
                } => {
                    let type_index = type_section.push(Type::Func(signature));

                    let import_entry =
                        ImportEntry::function(module_name, field_name, type_index as u32);
                    import_section.push(import_entry);
                }
            }
        }

        // 実関数をぶちこむ
        for function in self.functions {
            let type_index = type_section.push(Type::Func(function.signature));

            let function_index =
                function_section.push(type_index as _) + import_section.import_entries.len();

            code_section.push(function.body);

            if let Some(export_name) = function.export_name {
                let export_entry = ExportEntry {
                    field: export_name,
                    kind: ExternalKind::Function,
                    index: function_index as u32,
                };

                export_section.push(export_entry);
            }
        }

        {
            let memory_index = memory_section.push(self.memory.mem_type);

            if let Some(export_name) = self.memory.export_name {
                let export_entry = ExportEntry {
                    field: export_name,
                    kind: ExternalKind::Memory,
                    index: memory_index as u32,
                };

                export_section.push(export_entry);
            }
        }

        module.sections.push(Section::Type(type_section));
        module.sections.push(Section::Import(import_section));
        module.sections.push(Section::Function(function_section));
        module.sections.push(Section::Memory(memory_section));
        module.sections.push(Section::Export(export_section));
        module.sections.push(Section::Code(code_section));

        module
    }
}

pub struct Function {
    pub signature: FuncSignature,
    pub body: FunctionBody,
    pub export_name: Option<String>,
}
impl Function {
    pub fn push_local(&mut self, entry: LocalEntry) {
        self.body.push_local(entry)
    }
}

pub enum Import {
    Function {
        module_name: String,
        field_name: String,
        signature: FuncSignature,
    },
}

pub struct Memory {
    pub mem_type: MemoryType,
    pub export_name: Option<String>,
}

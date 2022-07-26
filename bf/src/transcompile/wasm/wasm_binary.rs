const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_BINARY_VERSION: u32 = 1;

pub mod code;
pub mod section;
pub mod type_;
pub mod var;

use std::io::{self, Write};

use section::Section;
use type_::Type;

use self::{
    code::FunctionBody,
    section::{
        CodeSection, ExportEntry, ExportSection, ExternalKind, FunctionSection, ImportEntry,
        ImportSection, MemorySection, MemoryType, TypeSection,
    },
    var::Var,
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
    pub fn push_import(&mut self, import: Import) -> usize {
        assert!(self.functions.is_empty());
        self.imports.push(import);

        self.imports.len() - 1
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
                    let type_index = type_section.push(signature);

                    let import_entry =
                        ImportEntry::function(module_name, field_name, Var(type_index as _));
                    import_section.push(import_entry);
                }
            }
        }

        // 実関数をぶちこむ
        for function in self.functions {
            let type_index = type_section.push(function.signature);

            let function_index =
                function_section.push(Var(type_index as _)) + import_section.import_entries.len();

            code_section.push(function.body);

            if let Some(export_name) = function.export_name {
                let export_entry = ExportEntry {
                    field: export_name,
                    kind: ExternalKind::Function,
                    index: Var(function_index as _),
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
                    index: Var(memory_index as _),
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
    pub signature: Type,
    pub body: FunctionBody,
    pub export_name: Option<String>,
}

pub enum Import {
    Function {
        module_name: String,
        field_name: String,
        signature: Type,
    },
}

pub struct Memory {
    pub mem_type: MemoryType,
    pub export_name: Option<String>,
}

#[test]
fn a() {
    use crate::transcompile::wasm::wasm_binary::{
        code::{FunctionBody, MemoryImmediate, Op, OpSlice},
        section::{
            CodeSection, ExportEntry, ExportSection, ExternalKind, FunctionSection, ImportEntry,
            ImportSection, MemorySection, MemoryType, ResizableLimits, TypeSection,
        },
        type_::Type,
        var::Var,
    };

    use std::fs::File;

    let memory = Memory {
        mem_type: MemoryType {
            limits: ResizableLimits {
                initial: Var(1),
                maximum: None,
            },
        },
        export_name: Some("memory".to_string()),
    };

    let mut module_builder = ModuleBuilder::new(memory);

    let import_fd_write = Import::Function {
        module_name: "wasi_unstable".to_string(),
        field_name: "fd_write".to_string(),
        signature: Type::Func {
            params: vec![Type::I32, Type::I32, Type::I32, Type::I32],
            result: Some(Box::new(Type::I32)),
        },
    };

    module_builder.push_import(import_fd_write);

    let mut print_char = FunctionBody::new();

    let print_char_ops = [
        Op::I32Const(Var(0)),
        Op::GetLocal {
            local_index: Var(0),
        },
        Op::I32Store8(MemoryImmediate::zero()),
        Op::I32Const(Var(4)),
        Op::I32Const(Var(0)),
        Op::I32Store(MemoryImmediate::i32()),
        Op::I32Const(Var(8)),
        Op::I32Const(Var(1)),
        Op::I32Store(MemoryImmediate::i32()),
        Op::I32Const(Var(1)),
        Op::I32Const(Var(4)),
        Op::I32Const(Var(1)),
        Op::I32Const(Var(12)),
        Op::Call {
            function_index: Var(0),
        },
        Op::Drop,
        Op::End,
    ];

    print_char_ops.write(&mut print_char.code).unwrap();

    let print_char = Function {
        signature: Type::Func {
            params: vec![Type::I32],
            result: None,
        },
        body: print_char,
        export_name: None,
    };

    module_builder.push_function(print_char);

    let mut main = FunctionBody::new();

    let main_ops = [
        Op::I32Const(Var(97)),
        Op::Call {
            function_index: Var(1),
        },
        Op::End,
    ];

    main_ops.write(&mut main.code).unwrap();

    let main = Function {
        signature: Type::Func {
            params: vec![],
            result: None,
        },
        body: main,
        export_name: Some("_start".to_string()),
    };

    module_builder.push_function(main);

    let module = module_builder.into_module();
    let mut f = File::create("c.wasm").unwrap();
    module.write(&mut f).unwrap();

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

        let print_char_ops = [
            Op::I32Const(Var(0)),
            Op::GetLocal {
                local_index: Var(0),
            },
            Op::I32Store8(MemoryImmediate::zero()),
            Op::I32Const(Var(4)),
            Op::I32Const(Var(0)),
            Op::I32Store(MemoryImmediate::i32()),
            Op::I32Const(Var(8)),
            Op::I32Const(Var(1)),
            Op::I32Store(MemoryImmediate::i32()),
            Op::I32Const(Var(1)),
            Op::I32Const(Var(4)),
            Op::I32Const(Var(1)),
            Op::I32Const(Var(12)),
            Op::Call {
                function_index: Var(0),
            },
            Op::Drop,
            Op::End,
        ];

        print_char_ops.write(&mut print_char.code).unwrap();

        code_section.push(print_char);

        let mut main = FunctionBody::new();

        let main_ops = [
            Op::I32Const(Var(97)),
            Op::Call {
                function_index: Var(1),
            },
            Op::End,
        ];

        main_ops.write(&mut main.code).unwrap();

        code_section.push(main);

        module.sections.push(Section::Code(code_section));
    }

    let mut file = File::create("aa.wasm").unwrap();
    module.write(&mut file).unwrap();
}

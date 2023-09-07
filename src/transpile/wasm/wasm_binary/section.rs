use std::io::{self, Write};

use super::code::FunctionBody;
use super::leb128::WriteLeb128;
use super::type_::Type;

pub enum Section {
    Type(TypeSection),
    Import(ImportSection),
    Function(FunctionSection),
    Memory(MemorySection),
    Export(ExportSection),
    Code(CodeSection),
    // Table,
    // Data,
    // Global,
    // Element,
}
impl Section {
    fn section_id(&self) -> u8 {
        match self {
            Section::Type(_) => 1,
            Section::Import(_) => 2,
            Section::Function(_) => 3,
            // Section::Table => todo!(),
            Section::Memory(_) => 5,
            // Section::Global => todo!(),
            Section::Export(_) => 7,
            // Section::Element => todo!(),
            Section::Code(_) => 10,
            // Section::Data => todo!(),
        }
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let section_id = self.section_id();
        section_id.write_leb128(&mut w)?;

        let mut payload = Vec::new();

        match self {
            Section::Type(type_section) => type_section.write(&mut payload)?,
            Section::Import(import_section) => import_section.write(&mut payload)?,
            Section::Function(function_section) => function_section.write(&mut payload)?,
            Section::Memory(memory_section) => memory_section.write(&mut payload)?,
            Section::Export(export_section) => export_section.write(&mut payload)?,
            Section::Code(code_section) => code_section.write(&mut payload)?,
            // Section::Table | Section::Data | Section::Global | Section::Element => unimplemented!(),
        }

        let payload_size = payload.len() as u32;
        payload_size.write_leb128(&mut w)?;

        w.write_all(&payload)?;

        Ok(())
    }
}

pub struct TypeSection {
    types: Vec<Type>,
}
impl TypeSection {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }
    pub fn push(&mut self, ty: Type) -> usize {
        self.types.push(ty);
        self.types.len() - 1
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let type_count = self.types.len() as u32;
        type_count.write_leb128(&mut w)?;

        for ty in &self.types {
            ty.write(&mut w)?;
        }

        Ok(())
    }
}

pub struct ImportSection {
    pub import_entries: Vec<ImportEntry>,
}
impl ImportSection {
    pub fn new() -> Self {
        Self {
            import_entries: Vec::new(),
        }
    }
    pub fn push(&mut self, entry: ImportEntry) {
        self.import_entries.push(entry);
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let entry_count = self.import_entries.len() as u32;
        entry_count.write_leb128(&mut w)?;

        for entry in &self.import_entries {
            entry.write(&mut w)?;
        }
        Ok(())
    }
}

pub struct ImportEntry {
    module_str: String,
    field_str: String,
    import_type: ImportType,
}
impl ImportEntry {
    pub fn function(module: String, field: String, index: u32) -> Self {
        Self {
            module_str: module,
            field_str: field,
            import_type: ImportType::Function { type_: index },
        }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let module_len = self.module_str.len() as u32;
        module_len.write_leb128(&mut w)?;
        w.write_all(self.module_str.as_bytes())?;

        let field_len = self.field_str.len() as u32;
        field_len.write_leb128(&mut w)?;
        w.write_all(self.field_str.as_bytes())?;

        self.import_type.write(w)
    }
}

enum ImportType {
    Function { type_: u32 },
    // Table,
    // Memory,
    // Global,
}
impl ImportType {
    fn kind(&self) -> ExternalKind {
        match self {
            ImportType::Function { .. } => ExternalKind::Function,
        }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let import_kind = self.kind() as u8;
        w.write_all(&[import_kind])?;

        match self {
            ImportType::Function { type_ } => type_.write_leb128(w),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExternalKind {
    Function = 0,
    // Table = 1,
    Memory = 2,
    // Global = 3,
}

pub struct FunctionSection {
    types: Vec<u32>,
}
impl FunctionSection {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }
    pub fn push(&mut self, index: u32) -> usize {
        self.types.push(index);
        self.types.len() - 1
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let count = self.types.len() as u32;
        count.write_leb128(&mut w)?;

        for index in &self.types {
            index.write_leb128(&mut w)?;
        }
        Ok(())
    }
}

pub struct MemorySection {
    entries: Vec<MemoryType>,
}

impl MemorySection {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn push(&mut self, memory_type: MemoryType) -> usize {
        self.entries.push(memory_type);
        self.entries.len() - 1
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let count = self.entries.len() as u32;
        count.write_leb128(&mut w)?;

        for entry in &self.entries {
            entry.write(&mut w)?;
        }

        Ok(())
    }
}

pub struct MemoryType {
    pub limits: ResizableLimits,
}
impl MemoryType {
    fn write(&self, w: impl Write) -> io::Result<()> {
        self.limits.write(w)
    }
}

pub struct ResizableLimits {
    pub initial: u32,
    pub maximum: Option<u32>,
}
impl ResizableLimits {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let flags = self.maximum.is_some();
        flags.write_leb128(&mut w)?;

        self.initial.write_leb128(&mut w)?;

        if let Some(maximum) = &self.maximum {
            maximum.write_leb128(&mut w)?;
        }
        Ok(())
    }
}

pub struct ExportSection {
    entries: Vec<ExportEntry>,
}

impl ExportSection {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn push(&mut self, entry: ExportEntry) {
        self.entries.push(entry)
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let count = self.entries.len() as u32;
        count.write_leb128(&mut w)?;
        for entry in &self.entries {
            entry.write(&mut w)?
        }
        Ok(())
    }
}

pub struct ExportEntry {
    pub field: String,
    pub kind: ExternalKind,
    pub index: u32,
}

impl ExportEntry {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let field_len = self.field.len() as u32;
        field_len.write_leb128(&mut w)?;

        w.write_all(self.field.as_bytes())?;

        w.write_all(&[self.kind as u8])?;

        self.index.write_leb128(w)
    }
}

pub struct CodeSection {
    function_bodies: Vec<FunctionBody>,
}
impl CodeSection {
    pub fn new() -> Self {
        Self {
            function_bodies: Vec::new(),
        }
    }
    pub fn push(&mut self, function_body: FunctionBody) {
        self.function_bodies.push(function_body)
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let count = self.function_bodies.len() as u32;
        count.write_leb128(&mut w)?;

        for body in &self.function_bodies {
            body.write(&mut w)?;
        }
        Ok(())
    }
}

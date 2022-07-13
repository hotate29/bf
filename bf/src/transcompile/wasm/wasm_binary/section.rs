use std::io::{self, Write};

use super::var::Var;
use super::Type;

pub enum Section {
    Type(TypeSection),
    Import(ImportSection),
    Function(FunctionSection),
    Table,
    Memory(MemorySection),
    Data,
    Global,
    Start,
    Element,
    Code,
}
impl Section {
    fn section_id(&self) -> Var<u8> {
        match self {
            Section::Type(_) => Var(1u8),
            Section::Import(_) => Var(2u8),
            Section::Function(_) => Var(3u8),
            Section::Table => todo!(),
            Section::Memory(_) => Var(5u8),
            Section::Data => todo!(),
            Section::Global => todo!(),
            Section::Start => todo!(),
            Section::Element => todo!(),
            Section::Code => todo!(),
        }
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let section_id = self.section_id();
        section_id.write(&mut w)?;

        let mut payload = Vec::new();

        match self {
            Section::Type(type_section) => type_section.write(&mut payload)?,
            Section::Import(import_section) => import_section.write(&mut payload)?,
            Section::Function(function_section) => function_section.write(&mut payload)?,
            Section::Memory(memory_section) => memory_section.write(&mut payload)?,
            Section::Start => todo!(),
            Section::Code => todo!(),
            Section::Table | Section::Data | Section::Global | Section::Element => unimplemented!(),
        }

        let payload_size = Var(payload.len() as u32);
        payload_size.write(&mut w)?;

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
    pub fn push(&mut self, ty: Type) {
        self.types.push(ty);
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let type_count = Var(self.types.len() as u32);
        type_count.write(&mut w)?;

        for ty in &self.types {
            ty.write(&mut w)?;
        }

        Ok(())
    }
}

pub struct ImportSection {
    import_entries: Vec<ImportEntry>,
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
        let entry_count = Var(self.import_entries.len() as u32);
        entry_count.write(&mut w)?;

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
    pub fn function(module: String, field: String, index: Var<u32>) -> Self {
        Self {
            module_str: module,
            field_str: field,
            import_type: ImportType::Function { type_: index },
        }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let module_len = Var(self.module_str.len() as u32);
        module_len.write(&mut w)?;
        w.write_all(self.module_str.as_bytes())?;

        let field_len = Var(self.field_str.len() as u32);
        field_len.write(&mut w)?;
        w.write_all(self.field_str.as_bytes())?;

        self.import_type.write(w)
    }
}

enum ImportType {
    Function { type_: Var<u32> },
    // Table,
    // Memory,
    // Global,
}
impl ImportType {
    fn kind(&self) -> ImportKind {
        match self {
            ImportType::Function { .. } => ImportKind::Function,
        }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let import_kind = self.kind() as u8;
        w.write_all(&[import_kind])?;

        match self {
            ImportType::Function { type_ } => type_.write(w),
        }
    }
}

#[repr(u8)]
enum ImportKind {
    Function = 0,
    // Table = 1,
    // Memory = 2,
    // Global = 3,
}

pub struct FunctionSection {
    types: Vec<Var<u32>>,
}
impl FunctionSection {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }
    pub fn push(&mut self, index: Var<u32>) {
        self.types.push(index)
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let count = Var(self.types.len() as u32);
        count.write(&mut w)?;

        for index in &self.types {
            index.write(&mut w)?;
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
    pub fn push(&mut self, memory_type: MemoryType) {
        self.entries.push(memory_type)
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let count = Var(self.entries.len() as u32);
        count.write(&mut w)?;

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
    pub flags: Var<bool>,
    pub initial: Var<u32>,
    pub maximum: Option<Var<u32>>,
}
impl ResizableLimits {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        self.flags.write(&mut w)?;
        self.initial.write(&mut w)?;
        if let Some(maximum) = &self.maximum {
            maximum.write(&mut w)?;
        }
        Ok(())
    }
}

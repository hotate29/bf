use std::io::{self, Write};

use super::var::Var;
use super::Type;

pub enum Section {
    Type(TypeSection),
    Import(ImportSection),
    Function,
    Table,
    Memory,
    Data,
    Global,
    Start,
    Element,
    Code,
}
impl Section {
    pub fn write(&self, w: impl Write) -> io::Result<()> {
        match self {
            Section::Type(type_section) => type_section.write(w),
            Section::Import(import_section) => import_section.write(w),
            Section::Function => todo!(),
            Section::Table => unimplemented!(),
            Section::Memory => todo!(),
            Section::Data => unimplemented!(),
            Section::Global => unimplemented!(),
            Section::Start => todo!(),
            Section::Element => todo!(),
            Section::Code => todo!(),
        }
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
        let mut payload = Vec::new();

        let type_count = Var(self.types.len() as u32);
        type_count.write(&mut payload)?;

        for ty in &self.types {
            ty.write(&mut payload)?;
        }

        let id = Var(1u8);
        id.write(&mut w)?;

        let payload_len = Var(payload.len() as u32);
        payload_len.write(&mut w)?;

        w.write_all(&payload)
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
        let section_id = Var(2_u32);
        section_id.write(&mut w)?;

        let mut payload = Vec::new();

        let entry_count = Var(self.import_entries.len() as u32);
        entry_count.write(&mut payload)?;

        for entry in &self.import_entries {
            entry.write(&mut payload)?;
        }

        let payload_size = Var(payload.len() as u32);
        payload_size.write(&mut w)?;

        w.write_all(&payload)
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
    Table = 1,
    Memory = 2,
    Global = 3,
}

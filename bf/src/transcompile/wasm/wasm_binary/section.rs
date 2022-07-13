use std::io::{self, Write};

use super::var::Var;
use super::Type;

pub enum Section {
    Type(TypeSection),
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
impl Section {
    pub fn write(&self, w: impl Write) -> io::Result<()> {
        match self {
            Section::Type(type_section) => type_section.write(w),
            Section::Import => todo!(),
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

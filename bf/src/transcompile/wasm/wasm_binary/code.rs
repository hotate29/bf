use std::io::{self, Write};

use super::{var::Var, Type};

pub struct FunctionBody {
    locals: Vec<LocalEntry>,
    pub code: Vec<u8>,
}
impl FunctionBody {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            code: Vec::new(),
        }
    }
    pub fn push_local(&mut self, local_entry: LocalEntry) {
        self.locals.push(local_entry)
    }
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let mut body_payload = Vec::new();

        let local_count = Var(self.locals.len() as u32);
        local_count.write(&mut body_payload)?;

        for local in &self.locals {
            local.write(&mut body_payload)?;
        }

        body_payload.write_all(&self.code)?;

        let body_size = Var(body_payload.len() as u32);
        body_size.write(&mut w)?;

        w.write_all(&body_payload)
    }
}

pub struct LocalEntry {
    count: Var<u32>,
    type_: Type,
}
impl LocalEntry {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        self.count.write(&mut w)?;
        self.type_.write(&mut w)
    }
}

pub enum Op {
    Nop,
    End,
}

impl Op {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        match self {
            Op::Nop => w.write_all(&[0x01]),
            Op::End => w.write_all(&[0x0b]),
        }
    }
}

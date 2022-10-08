use std::io::{self, Write};

pub trait VarInt {}

impl VarInt for bool {}
impl VarInt for i8 {}
impl VarInt for u8 {}
impl VarInt for i32 {}
impl VarInt for u32 {}
impl VarInt for i64 {}

#[derive(Debug, Clone, Copy)]
pub struct Var<T: VarInt>(pub T);

impl Var<bool> {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        let value = self.0;
        leb128::write::unsigned(&mut w, value as u64)?;
        Ok(())
    }
}

// 以下繰り返し
impl Var<u8> {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, self.0 as u64)?;
        Ok(())
    }
}

impl Var<i8> {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.0 as i64)?;
        Ok(())
    }
}

impl Var<u32> {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, self.0 as u64)?;
        Ok(())
    }
}

impl Var<i32> {
    pub fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.0 as i64)?;
        Ok(())
    }
}

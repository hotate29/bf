use std::io::{self, Write};

pub trait VarInt {}

impl VarInt for bool {}
impl VarInt for i8 {}
impl VarInt for u8 {}
impl VarInt for i32 {}
impl VarInt for u32 {}
impl VarInt for i64 {}

pub struct Var<T: VarInt>(pub T);

pub trait VarImpl<T: VarInt> {
    fn write(&self, writer: impl Write) -> io::Result<()>;
}

impl VarImpl<bool> for Var<bool> {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let value = self.0;
        leb128::write::unsigned(&mut w, value as u64)?;
        Ok(())
    }
}

// 以下繰り返し
impl VarImpl<u8> for Var<u8> {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, self.0 as u64)?;
        Ok(())
    }
}

impl VarImpl<i8> for Var<i8> {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.0 as i64)?;
        Ok(())
    }
}

impl VarImpl<u32> for Var<u32> {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, self.0 as u64)?;
        Ok(())
    }
}

impl VarImpl<i32> for Var<i32> {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.0 as i64)?;
        Ok(())
    }
}

impl VarImpl<i64> for Var<i64> {
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.0)?;
        Ok(())
    }
}

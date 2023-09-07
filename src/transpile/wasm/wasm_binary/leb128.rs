use std::io::{self, Write};

pub trait WriteLeb128 {
    fn write_leb128(&self, w: impl Write) -> io::Result<()>;
}

impl WriteLeb128 for bool {
    fn write_leb128(&self, mut w: impl Write) -> io::Result<()> {
        let value = *self as u64;
        leb128::write::unsigned(&mut w, value)?;
        Ok(())
    }
}

impl WriteLeb128 for i8 {
    fn write_leb128(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, *self as i64)?;
        Ok(())
    }
}

impl WriteLeb128 for u8 {
    fn write_leb128(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, *self as u64)?;
        Ok(())
    }
}

impl WriteLeb128 for i32 {
    fn write_leb128(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, *self as i64)?;
        Ok(())
    }
}

impl WriteLeb128 for u32 {
    fn write_leb128(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, *self as i64)?;
        Ok(())
    }
}

impl WriteLeb128 for i64 {
    fn write_leb128(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, *self)?;
        Ok(())
    }
}

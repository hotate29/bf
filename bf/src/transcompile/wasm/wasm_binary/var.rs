use std::io::{self, Write};

pub trait VarInt {}

impl VarInt for bool {}
impl VarInt for i8 {}
impl VarInt for u8 {}
impl VarInt for i32 {}
impl VarInt for u32 {}
impl VarInt for i64 {}

pub struct Var<T: VarInt> {
    value: T,
}

pub trait VarImpl<T: VarInt> {
    #[allow(clippy::new_ret_no_self)]
    fn new(value: T) -> Var<T>;
    fn value(&self) -> T;
    fn write(&self, writer: impl Write) -> io::Result<()>;
}

impl VarImpl<bool> for Var<bool> {
    fn new(value: bool) -> Var<bool> {
        Self { value }
    }
    fn value(&self) -> bool {
        self.value
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        let value = self.value;
        leb128::write::unsigned(&mut w, value as u64)?;
        Ok(())
    }
}

// 以下繰り返し
impl VarImpl<u8> for Var<u8> {
    fn new(value: u8) -> Var<u8> {
        Self { value }
    }
    fn value(&self) -> u8 {
        self.value
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, self.value as u64)?;
        Ok(())
    }
}

impl VarImpl<i8> for Var<i8> {
    fn new(value: i8) -> Var<i8> {
        Self { value }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.value as i64)?;
        Ok(())
    }
    fn value(&self) -> i8 {
        self.value
    }
}

impl VarImpl<u32> for Var<u32> {
    fn new(value: u32) -> Var<u32> {
        Self { value }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::unsigned(&mut w, self.value as u64)?;
        Ok(())
    }
    fn value(&self) -> u32 {
        self.value
    }
}

impl VarImpl<i32> for Var<i32> {
    fn new(value: i32) -> Var<i32> {
        Self { value }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.value as i64)?;
        Ok(())
    }
    fn value(&self) -> i32 {
        self.value
    }
}

impl VarImpl<i64> for Var<i64> {
    fn new(value: i64) -> Var<i64> {
        Self { value }
    }
    fn write(&self, mut w: impl Write) -> io::Result<()> {
        leb128::write::signed(&mut w, self.value)?;
        Ok(())
    }
    fn value(&self) -> i64 {
        self.value
    }
}

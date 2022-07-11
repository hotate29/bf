use std::marker::PhantomData;

pub trait VarInt {}

impl VarInt for bool {}
impl VarInt for i8 {}
impl VarInt for u8 {}
impl VarInt for i32 {}
impl VarInt for u32 {}
impl VarInt for i64 {}

pub struct Var<T: VarInt> {
    value: Vec<u8>,
    _marker: PhantomData<T>,
}

pub trait VarImpl<T: VarInt> {
    #[allow(clippy::new_ret_no_self)]
    fn new(value: T) -> Var<T>;
    fn read(&self) -> T;
}

impl VarImpl<bool> for Var<bool> {
    fn new(value: bool) -> Var<bool> {
        let mut buffer = Vec::new();
        leb128::write::unsigned(&mut buffer, value as u64).unwrap();
        Self {
            value: buffer,
            _marker: PhantomData,
        }
    }

    fn read(&self) -> bool {
        let value = leb128::read::unsigned(&mut self.value.as_slice()).unwrap();

        // u64からboolへの`(Try)Into`実装が無い
        if value == 1 {
            true
        } else if value == 0 {
            false
        } else {
            unreachable!()
        }
    }
}

// 以下繰り返し
impl VarImpl<u8> for Var<u8> {
    fn new(value: u8) -> Var<u8> {
        let mut buffer = Vec::new();
        leb128::write::unsigned(&mut buffer, value as u64).unwrap();
        Self {
            value: buffer,
            _marker: PhantomData,
        }
    }

    fn read(&self) -> u8 {
        leb128::read::unsigned(&mut self.value.as_slice())
            .unwrap()
            .try_into()
            .unwrap()
    }
}

impl VarImpl<i8> for Var<i8> {
    fn new(value: i8) -> Var<i8> {
        let mut buffer = Vec::new();
        leb128::write::signed(&mut buffer, value as i64).unwrap();
        Self {
            value: buffer,
            _marker: PhantomData,
        }
    }

    fn read(&self) -> i8 {
        leb128::read::signed(&mut self.value.as_slice())
            .unwrap()
            .try_into()
            .unwrap()
    }
}

impl VarImpl<u32> for Var<u32> {
    fn new(value: u32) -> Var<u32> {
        let mut buffer = Vec::new();
        leb128::write::unsigned(&mut buffer, value as u64).unwrap();
        Self {
            value: buffer,
            _marker: PhantomData,
        }
    }

    fn read(&self) -> u32 {
        leb128::read::unsigned(&mut self.value.as_slice())
            .unwrap()
            .try_into()
            .unwrap()
    }
}

impl VarImpl<i32> for Var<i32> {
    fn new(value: i32) -> Var<i32> {
        let mut buffer = Vec::new();
        leb128::write::signed(&mut buffer, value as i64).unwrap();
        Self {
            value: buffer,
            _marker: PhantomData,
        }
    }

    fn read(&self) -> i32 {
        leb128::read::signed(&mut self.value.as_slice())
            .unwrap()
            .try_into()
            .unwrap()
    }
}

impl VarImpl<i64> for Var<i64> {
    fn new(value: i64) -> Var<i64> {
        let mut buffer = Vec::new();
        leb128::write::signed(&mut buffer, value as i64).unwrap();
        Self {
            value: buffer,
            _marker: PhantomData,
        }
    }

    fn read(&self) -> i64 {
        leb128::read::signed(&mut self.value.as_slice()).unwrap()
    }
}

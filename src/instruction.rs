use std::cmp::Ordering;

use serde::Serialize;

use crate::parse::Token;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize)]
pub enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    Add(isize, Value),
    Sub(isize, Value),
    /// mem[左isize] += mem[右isize] * value
    MulAdd(isize, isize, u8),
    /// mem[左isize] -= mem[右isize] * value
    MulSub(isize, isize, u8),
    Output(isize, usize),
    Input(isize, usize),
    // SetValue(isize, u8),
    SetValue(isize, Value),
}

impl Instruction {
    #[inline]
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Greater => Some(Self::PtrIncrement(1)),
            Token::Less => Some(Self::PtrDecrement(1)),
            Token::Plus => Some(Self::Add(0, 1.into())),
            Token::Minus => Some(Self::Sub(0, 1.into())),
            Token::Period => Some(Self::Output(0, 1)),
            Token::Comma => Some(Self::Input(0, 1)),
            Token::LeftBracket | Token::RightBracket => None,
        }
    }
    pub fn to_compressed_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(format!("{}>", n)),
            Instruction::PtrDecrement(n) => Some(format!("{}<", n)),
            Instruction::Add(0, Value::Const(n)) => Some(format!("{}+", n)),
            Instruction::Sub(0, Value::Const(n)) => Some(format!("{}-", n)),
            Instruction::Output(0, n) => Some(format!("{}.", n)),
            Instruction::Input(0, n) => Some(format!("{},", n)),
            Instruction::MulAdd(_, _, _)
            | Instruction::MulSub(_, _, _)
            | Instruction::Add(_, _)
            | Instruction::Sub(_, _)
            | Instruction::Output(_, _)
            | Instruction::Input(_, _)
            | Instruction::SetValue(_, _) => None,
        }
    }
    pub fn to_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(">".repeat(n)),
            Instruction::PtrDecrement(n) => Some("<".repeat(n)),
            Instruction::Add(0, Value::Const(n)) => Some("+".repeat(n as usize)),
            Instruction::Sub(0, Value::Const(n)) => Some("-".repeat(n as usize)),
            Instruction::Output(0, n) => Some(".".repeat(n)),
            Instruction::Input(0, n) => Some(",".repeat(n)),
            Instruction::MulAdd(_, _, _)
            | Instruction::MulSub(_, _, _)
            | Instruction::Add(_, _)
            | Instruction::Sub(_, _)
            | Instruction::Output(_, _)
            | Instruction::Input(_, _)
            | Instruction::SetValue(_, _) => None,
        }
    }
    #[inline]
    pub fn merge(self, instruction: Instruction) -> Option<Instruction> {
        use Instruction::*;

        match (self, instruction) {
            (Add(x_offset, Value::Const(x)), Add(y_offset, Value::Const(y)))
                if x_offset == y_offset =>
            {
                Some(Add(x_offset, Value::Const(x.wrapping_add(y))))
            }
            (Sub(y_offset, Value::Const(y)), Add(x_offset, Value::Const(x)))
            | (Add(x_offset, Value::Const(x)), Sub(y_offset, Value::Const(y)))
                if x_offset == y_offset =>
            {
                let x = x as i32;
                let y = y as i32;

                let z = x - y;

                match z.cmp(&0) {
                    Ordering::Less => Some(Sub(x_offset, (z.abs() as u8).into())),
                    Ordering::Greater => Some(Add(x_offset, (z as u8).into())),
                    Ordering::Equal => Some(Add(x_offset, 0.into())),
                }
            }
            (Sub(x_offset, Value::Const(x)), Sub(y_offset, Value::Const(y)))
                if x_offset == y_offset =>
            {
                Some(Sub(x_offset, x.wrapping_add(y).into()))
            }
            (PtrIncrement(x), PtrIncrement(y)) => Some(PtrIncrement(x + y)),
            (PtrDecrement(y), PtrIncrement(x)) | (PtrIncrement(x), PtrDecrement(y)) => {
                let x = x as isize;
                let y = y as isize;

                let z = x - y;

                match z.cmp(&0) {
                    Ordering::Less => Some(PtrDecrement(z.abs() as usize)),
                    Ordering::Greater => Some(PtrIncrement(z as usize)),
                    Ordering::Equal => Some(PtrIncrement(0)),
                }
            }
            (PtrDecrement(x), PtrDecrement(y)) => Some(PtrDecrement(x + y)),
            (SetValue(offset_x, _), rhs @ SetValue(offset_y, _)) if offset_x == offset_y => {
                Some(rhs)
            }
            (SetValue(offset_x, value), Add(offset_y, Value::Const(add_value)))
                if offset_x == offset_y =>
            {
                Some(SetValue(
                    offset_x,
                    value.map_const(|v| v.wrapping_add(add_value)),
                ))
            }
            (
                SetValue(offset_x, value @ Value::Const(_)),
                Sub(offset_y, Value::Const(sub_value)),
            ) if offset_x == offset_y => Some(SetValue(
                offset_x,
                value.map_const(|v| v.wrapping_sub(sub_value)),
            )),
            (SetValue(offset_x, Value::Const(0)), Add(to_offset, Value::Memory(from_offset)))
                if offset_x == to_offset =>
            {
                Some(SetValue(offset_x, Value::Memory(from_offset)))
            }
            (Add(x_offset, _) | Sub(x_offset, _), SetValue(y_offset, Value::Const(0)))
                if x_offset == y_offset =>
            {
                Some(instruction)
            }
            (Output(x_offset, x), Output(y_offset, y)) if x_offset == y_offset => {
                Some(Output(x_offset, x + y))
            }
            (ins, instruction) | (instruction, ins) if ins.is_no_action() => Some(instruction),
            (_, _) => None,
        }
    }
    #[inline]
    pub fn is_no_action(&self) -> bool {
        matches!(
            self,
            Instruction::PtrIncrement(0)
                | Instruction::PtrDecrement(0)
                | Instruction::Add(_, Value::Const(0))
                | Instruction::Sub(_, Value::Const(0))
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize)]
pub enum Value {
    Const(u8),
    Memory(isize),
}
impl Value {
    #[inline]
    pub fn get_const(self) -> Option<u8> {
        match self {
            Value::Const(value) => Some(value),
            Value::Memory(_) => None,
        }
    }
    #[inline]
    pub fn get_or(self, f: impl FnOnce(isize) -> u8) -> u8 {
        match self {
            Value::Const(value) => value,
            Value::Memory(offset) => f(offset),
        }
    }
    #[inline]
    pub fn map_const(self, f: impl FnOnce(u8) -> u8) -> Self {
        if let Self::Const(value) = self {
            Self::Const(f(value))
        } else {
            self
        }
    }
    #[inline]
    pub fn map_offset(self, f: impl FnOnce(isize) -> isize) -> Self {
        if let Self::Memory(offset) = self {
            Self::Memory(f(offset))
        } else {
            self
        }
    }
}
impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::Const(value)
    }
}

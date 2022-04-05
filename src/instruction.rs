use std::cmp::Ordering;

use serde::Serialize;

use crate::parse::Token;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize)]
pub enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    Add(u8),
    AddOffset(isize, u8),
    AddTo(isize),
    Sub(u8),
    SubOffset(isize, u8),
    SubTo(isize),
    MulAdd(isize, u8),
    Output(usize),
    OutputOffset(usize, isize),
    Input(usize),
    ZeroSet,
    ZeroSetOffset(isize),
    Copy(isize),
}

impl Instruction {
    #[inline]
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Greater => Some(Self::PtrIncrement(1)),
            Token::Less => Some(Self::PtrDecrement(1)),
            Token::Plus => Some(Self::Add(1)),
            Token::Minus => Some(Self::Sub(1)),
            Token::Period => Some(Self::Output(1)),
            Token::Comma => Some(Self::Input(1)),
            Token::LeftBracket | Token::RightBracket => None,
        }
    }
    pub fn to_compressed_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(format!("{}>", n)),
            Instruction::PtrDecrement(n) => Some(format!("{}<", n)),
            Instruction::Add(n) => Some(format!("{}+", n)),
            Instruction::Sub(n) => Some(format!("{}-", n)),
            Instruction::Output(n) => Some(format!("{}.", n)),
            Instruction::Input(n) => Some(format!("{},", n)),
            Instruction::AddTo(_)
            | Instruction::SubTo(_)
            | Instruction::MulAdd(_, _)
            | Instruction::ZeroSet
            | Instruction::Copy(_)
            | Instruction::AddOffset(_, _)
            | Instruction::SubOffset(_, _)
            | Instruction::OutputOffset(_, _)
            | Instruction::ZeroSetOffset(_) => None,
        }
    }
    pub fn to_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(">".repeat(n)),
            Instruction::PtrDecrement(n) => Some("<".repeat(n)),
            Instruction::Add(n) => Some("+".repeat(n as usize)),
            Instruction::Sub(n) => Some("-".repeat(n as usize)),
            Instruction::Output(n) => Some(".".repeat(n)),
            Instruction::Input(n) => Some(",".repeat(n)),
            Instruction::AddTo(_)
            | Instruction::SubTo(_)
            | Instruction::MulAdd(_, _)
            | Instruction::ZeroSet
            | Instruction::Copy(_)
            | Instruction::AddOffset(_, _)
            | Instruction::SubOffset(_, _)
            | Instruction::OutputOffset(_, _)
            | Instruction::ZeroSetOffset(_) => None,
        }
    }
    #[inline]
    pub fn merge(self, instruction: Instruction) -> Option<Instruction> {
        use Instruction::*;

        match (self, instruction) {
            (Add(x), Add(y)) => Some(Add(x.wrapping_add(y))),
            (Sub(y), Add(x)) | (Add(x), Sub(y)) => {
                let x = x as i32;
                let y = y as i32;

                let z = x - y;

                match z.cmp(&0) {
                    Ordering::Less => Some(Sub(z.abs() as u8)),
                    Ordering::Greater => Some(Add(z as u8)),
                    Ordering::Equal => Some(Add(0)),
                }
            }
            (Sub(x), Sub(y)) => Some(Sub(x.wrapping_add(y) % u8::MAX)),
            (PtrIncrement(x), PtrIncrement(y)) => Some(PtrIncrement(x + y)),
            (PtrDecrement(y), PtrIncrement(x)) | (PtrIncrement(x), PtrDecrement(y)) => {
                let x = x as isize;
                let y = y as isize;

                let z = x - y;

                match z.cmp(&0) {
                    Ordering::Less => Some(PtrDecrement(z.abs() as usize)),
                    Ordering::Greater => Some(PtrIncrement(z as usize)),
                    Ordering::Equal => Some(Add(0)),
                }
            }
            (PtrDecrement(x), PtrDecrement(y)) => Some(PtrDecrement(x + y)),
            (ZeroSet, ZeroSet) => Some(ZeroSet),
            (ZeroSet, AddTo(_) | SubTo(_)) => Some(ZeroSet),
            (Add(_) | Sub(_), ZeroSet) => Some(ZeroSet),
            // (AddOffset(x_offset, x), AddOffset(y_offset, y)) if x_offset == y_offset => {
            //     Some(AddOffset(x_offset, x.wrapping_add(y)))
            // }
            // (SubOffset(x_offset, x), SubOffset(y_offset, y)) if x_offset == y_offset => {
            //     Some(SubOffset(x_offset, x.wrapping_add(y)))
            // }
            // (AddOffset(offset, x), Add(y)) => Some(AddOffset(offset, x.wrapping_add(y))),
            // (SubOffset(offset, x), Sub(y)) => Some(SubOffset(offset, x.wrapping_add(y))),
            // (AddOffset(add_offset, x), SubOffset(sub_offset, y)) if add_offset == sub_offset => {
            //     let x = x as i16;
            //     let y = y as i16;
            //     let z = x - y;
            //     if z < 0 {
            //         Some(SubOffset(add_offset, (-z) as u8))
            //     } else {
            //         Some(AddOffset(add_offset, z as u8))
            //     }
            // }
            // (SubOffset(sub_offset, x), AddOffset(add_offset, y)) if add_offset == sub_offset => {
            //     let x = x as i16;
            //     let y = y as i16;
            //     let z = x - y;
            //     if z < 0 {
            //         Some(AddOffset(add_offset, (-z) as u8))
            //     } else {
            //         Some(SubOffset(add_offset, z as u8))
            //     }
            // }
            (ins, instruction) if ins.is_no_action() => Some(instruction),
            (_, _) => None,
        }
    }
    #[inline]
    pub fn is_no_action(&self) -> bool {
        matches!(
            self,
            Instruction::PtrIncrement(0)
                | Instruction::PtrDecrement(0)
                | Instruction::Add(0)
                | Instruction::AddOffset(_, 0)
                | Instruction::Sub(0)
                | Instruction::SubOffset(_, 0)
        )
    }
}

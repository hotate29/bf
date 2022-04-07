use std::cmp::Ordering;

use serde::Serialize;

use crate::parse::Token;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize)]
pub enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    AddOffset(isize, u8),
    AddTo(isize, isize),
    SubOffset(isize, u8),
    SubTo(isize, isize),
    /// mem[左isize] += mem[右isize] * value
    MulAdd(isize, isize, u8),
    /// mem[左isize] -= mem[右isize] * value
    MulSub(isize, isize, u8),
    Output(usize),
    OutputOffset(isize, usize),
    Input(usize),
    InputOffset(isize, usize),
    ZeroSet,
    ZeroSetOffset(isize),
}

impl Instruction {
    #[inline]
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Greater => Some(Self::PtrIncrement(1)),
            Token::Less => Some(Self::PtrDecrement(1)),
            Token::Plus => Some(Self::AddOffset(0, 1)),
            Token::Minus => Some(Self::SubOffset(0, 1)),
            Token::Period => Some(Self::Output(1)),
            Token::Comma => Some(Self::Input(1)),
            Token::LeftBracket | Token::RightBracket => None,
        }
    }
    pub fn to_compressed_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(format!("{}>", n)),
            Instruction::PtrDecrement(n) => Some(format!("{}<", n)),
            Instruction::AddOffset(0, n) => Some(format!("{}+", n)),
            Instruction::SubOffset(0, n) => Some(format!("{}-", n)),
            Instruction::Output(n) => Some(format!("{}.", n)),
            Instruction::Input(n) => Some(format!("{},", n)),
            Instruction::AddTo(_, _)
            | Instruction::SubTo(_, _)
            | Instruction::MulAdd(_, _, _)
            | Instruction::MulSub(_, _, _)
            | Instruction::ZeroSet
            | Instruction::AddOffset(_, _)
            | Instruction::SubOffset(_, _)
            | Instruction::OutputOffset(_, _)
            | Instruction::ZeroSetOffset(_)
            | Instruction::InputOffset(_, _) => None,
        }
    }
    pub fn to_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(">".repeat(n)),
            Instruction::PtrDecrement(n) => Some("<".repeat(n)),
            Instruction::AddOffset(0, n) => Some("+".repeat(n as usize)),
            Instruction::SubOffset(0, n) => Some("-".repeat(n as usize)),
            Instruction::Output(n) => Some(".".repeat(n)),
            Instruction::Input(n) => Some(",".repeat(n)),
            Instruction::AddTo(_, _)
            | Instruction::SubTo(_, _)
            | Instruction::MulAdd(_, _, _)
            | Instruction::MulSub(_, _, _)
            | Instruction::ZeroSet
            | Instruction::AddOffset(_, _)
            | Instruction::SubOffset(_, _)
            | Instruction::OutputOffset(_, _)
            | Instruction::ZeroSetOffset(_)
            | Instruction::InputOffset(_, _) => None,
        }
    }
    #[inline]
    pub fn merge(self, instruction: Instruction) -> Option<Instruction> {
        use Instruction::*;

        match (self, instruction) {
            (AddOffset(x_offset, x), AddOffset(y_offset, y)) if x_offset == y_offset => {
                Some(AddOffset(x_offset, x.wrapping_add(y)))
            }
            (SubOffset(y_offset, y), AddOffset(x_offset, x))
            | (AddOffset(x_offset, x), SubOffset(y_offset, y))
                if x_offset == y_offset =>
            {
                let x = x as i32;
                let y = y as i32;

                let z = x - y;

                match z.cmp(&0) {
                    Ordering::Less => Some(SubOffset(x_offset, z.abs() as u8)),
                    Ordering::Greater => Some(AddOffset(0, z as u8)),
                    Ordering::Equal => Some(AddOffset(0, 0)),
                }
            }
            (SubOffset(x_offset, x), SubOffset(y_offset, y)) if x_offset == y_offset => {
                Some(SubOffset(x_offset, x.wrapping_add(y) % u8::MAX))
            }
            (PtrIncrement(x), PtrIncrement(y)) => Some(PtrIncrement(x + y)),
            (PtrDecrement(y), PtrIncrement(x)) | (PtrIncrement(x), PtrDecrement(y)) => {
                let x = x as isize;
                let y = y as isize;

                let z = x - y;

                match z.cmp(&0) {
                    Ordering::Less => Some(PtrDecrement(z.abs() as usize)),
                    Ordering::Greater => Some(PtrIncrement(z as usize)),
                    Ordering::Equal => Some(AddOffset(0, 0)),
                }
            }
            (PtrDecrement(x), PtrDecrement(y)) => Some(PtrDecrement(x + y)),
            (ZeroSet, ZeroSet) => Some(ZeroSet),
            (AddOffset(0, _) | SubOffset(0, _), ZeroSet) => Some(ZeroSet),
            (Output(x), Output(y)) => Some(Output(x + y)),
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
                | Instruction::AddOffset(_, 0)
                | Instruction::SubOffset(_, 0)
        )
    }
}

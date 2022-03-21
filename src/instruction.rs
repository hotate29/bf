use std::cmp::Ordering;

use serde::Serialize;

use crate::parse::Token;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize)]
pub enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    Add(u8),
    AddTo(usize),
    AddToRev(usize),
    Sub(u8),
    SubTo(usize),
    SubToRev(usize),
    MulAdd(usize, u8),
    MulAddRev(usize, u8),
    Output(usize),
    Input(usize),
    ZeroSet,
    Copy(usize),
    CopyRev(usize),
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
            | Instruction::AddToRev(_)
            | Instruction::SubTo(_)
            | Instruction::SubToRev(_)
            | Instruction::MulAdd(_, _)
            | Instruction::MulAddRev(_, _)
            | Instruction::ZeroSet
            | Instruction::Copy(_)
            | Instruction::CopyRev(_) => None,
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
            | Instruction::AddToRev(_)
            | Instruction::SubTo(_)
            | Instruction::SubToRev(_)
            | Instruction::MulAdd(_, _)
            | Instruction::MulAddRev(_, _)
            | Instruction::ZeroSet
            | Instruction::Copy(_)
            | Instruction::CopyRev(_) => None,
        }
    }
    #[inline]
    pub fn merge(self, instruction: Instruction) -> Option<Instruction> {
        use Instruction::*;

        match (self, instruction) {
            (Add(x), Add(y)) => Some(Add((x + y) % u8::MAX)),
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
            (Sub(x), Sub(y)) => Some(Sub((x + y) % u8::MAX)),
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
            (ZeroSet, inst) | (inst, ZeroSet) if matches!(inst, Add(_) | Sub(_)) => Some(inst),
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
                | Instruction::Sub(0)
        )
    }
}

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
}

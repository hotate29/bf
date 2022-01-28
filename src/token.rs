use std::{
    fmt::{Display, Write},
    mem::swap,
};

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Token {
    Greater,
    Less,
    Plus,
    Minus,
    Period,
    Comma,
    LeftBracket,
    RightBracket,
    Other(char),
}
impl Token {
    pub fn from_char(c: char) -> Token {
        use Token::*;
        match c {
            '>' => Greater,
            '<' => Less,
            '+' => Plus,
            '-' => Minus,
            '.' => Period,
            ',' => Comma,
            '[' => LeftBracket,
            ']' => RightBracket,
            c => Other(c),
        }
    }
    pub fn as_char(&self) -> char {
        match self {
            Token::Greater => '>',
            Token::Less => '<',
            Token::Plus => '+',
            Token::Minus => '-',
            Token::Period => '.',
            Token::Comma => ',',
            Token::LeftBracket => '[',
            Token::RightBracket => ']',
            Token::Other(c) => *c,
        }
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    source.chars().map(Token::from_char).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum MiddleToken {
    Token(Token, usize),
    WhileBegin,
    WhileEnd,
}

impl MiddleToken {
    pub fn to_instruction(self) -> Option<Instruction> {
        match self {
            MiddleToken::Token(Token::Other(_), _)
            | MiddleToken::WhileBegin
            | MiddleToken::WhileEnd => None,
            MiddleToken::Token(token, count) => match token {
                Token::Greater => Some(Instruction::PtrIncrement(count)),
                Token::Less => Some(Instruction::PtrDecrement(count)),
                Token::Plus => Some(Instruction::Add(count)),
                Token::Minus => Some(Instruction::Sub(count)),
                Token::Period => Some(Instruction::Output(count)),
                Token::Comma => Some(Instruction::Input(count)),
                Token::LeftBracket | Token::RightBracket | Token::Other(_) => unreachable!(),
            },
        }
    }
}

impl Display for MiddleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiddleToken::Token(token, count) => match token {
                Token::LeftBracket | Token::RightBracket | Token::Other(_) => unreachable!(),
                token => token.as_char().to_string().repeat(*count).fmt(f),
            },
            MiddleToken::WhileBegin => f.write_char('['),
            MiddleToken::WhileEnd => f.write_char(']'),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("角括弧が対応してないよ!")]
    InvalidBracket,
}

pub fn middle_token(tokens: &[Token]) -> Result<Vec<MiddleToken>, ParseError> {
    let mut middle_tokens = Vec::new();

    let mut prev = None;
    let mut prev_count = 0;

    for token in tokens
        .iter()
        .filter(|token| !matches!(token, Token::Other(_)))
        .copied()
    {
        match token {
            Token::LeftBracket | Token::RightBracket => {
                if prev_count != 0 {
                    middle_tokens.push(MiddleToken::Token(prev.unwrap(), prev_count));
                }
                match token {
                    Token::LeftBracket => middle_tokens.push(MiddleToken::WhileBegin),
                    Token::RightBracket => middle_tokens.push(MiddleToken::WhileEnd),
                    _ => unreachable!(),
                }
                prev = None;
                prev_count = 0;
                continue;
            }
            _ => (),
        }
        if prev == None {
            prev = Some(token);
            prev_count = 1;
        } else if prev == Some(token) {
            prev_count += 1;
        } else {
            let mut token = Some(token);
            swap(&mut prev, &mut token);

            match token.unwrap() {
                Token::LeftBracket | Token::RightBracket | Token::Other(_) => (),
                token => middle_tokens.push(MiddleToken::Token(token, prev_count)),
            }
            prev_count = 1;
        }
    }
    if let Some(token) = prev {
        match token {
            Token::LeftBracket | Token::RightBracket | Token::Other(_) => (),
            token => middle_tokens.push(MiddleToken::Token(token, prev_count)),
        }
    }

    let mut begin = 0;
    for token in &middle_tokens {
        match token {
            MiddleToken::WhileBegin => begin += 1,
            MiddleToken::WhileEnd => begin -= 1,
            MiddleToken::Token(_, _) => (),
        }
        if begin < 0 {
            return Err(ParseError::InvalidBracket);
        }
    }
    if begin != 0 {
        return Err(ParseError::InvalidBracket);
    }

    Ok(middle_tokens)
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize)]
pub enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    Add(usize),
    MoveAdd(usize),
    MoveAddRev(usize),
    Sub(usize),
    MoveSub(usize),
    MoveSubRev(usize),
    MulAdd(usize, u8),
    MulAddRev(usize, u8),
    Output(usize),
    Input(usize),
    SetValue(usize, u8),
}

impl Instruction {
    pub fn to_string(self) -> Option<String> {
        match self {
            Instruction::PtrIncrement(n) => Some(format!(">{}", n)),
            Instruction::PtrDecrement(n) => Some(format!("<{}", n)),
            Instruction::Add(n) => Some(format!("+{}", n)),
            Instruction::Sub(n) => Some(format!("-{}", n)),
            Instruction::Output(n) => Some(format!(".{}", n)),
            Instruction::Input(n) => Some(format!(",{}", n)),
            Instruction::MoveAdd(_)
            | Instruction::MoveAddRev(_)
            | Instruction::MoveSub(_)
            | Instruction::MoveSubRev(_)
            | Instruction::SetValue(_, _)
            | Instruction::MulAdd(_, _)
            | Instruction::MulAddRev(_, _) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{middle_token, tokenize, MiddleToken, ParseError, Token};

    #[test]
    fn test_token_from_char() {
        fn helper(c: char, assert_token: Token) {
            let token = Token::from_char(c);
            assert_eq!(token, assert_token);
        }

        helper('>', Token::Greater);
        helper('<', Token::Less);
        helper('+', Token::Plus);
        helper('-', Token::Minus);
        helper('.', Token::Period);
        helper(',', Token::Comma);
        helper('[', Token::LeftBracket);
        helper(']', Token::RightBracket);

        helper('a', Token::Other('a'));
        helper('1', Token::Other('1'));
    }

    #[test]
    fn test_middle_token() {
        use Token::*;

        fn helper(source: &str, assert_middle_token: Result<Vec<MiddleToken>, ParseError>) {
            let tokens = tokenize(source);
            let middle_tokens = middle_token(&tokens);
            assert_eq!(middle_tokens, assert_middle_token);
        }

        helper("", Ok(vec![]));
        helper("brainfuck", Ok(vec![]));
        helper("bra+inf+uck", Ok(vec![MiddleToken::Token(Plus, 2)]));

        helper("+", Ok(vec![MiddleToken::Token(Plus, 1)]));

        helper("+++", Ok(vec![MiddleToken::Token(Plus, 3)]));
        helper("---", Ok(vec![MiddleToken::Token(Minus, 3)]));
        helper(">>>", Ok(vec![MiddleToken::Token(Greater, 3)]));
        helper("<<<", Ok(vec![MiddleToken::Token(Less, 3)]));
        helper("...", Ok(vec![MiddleToken::Token(Period, 3)]));
        helper(",,,", Ok(vec![MiddleToken::Token(Comma, 3)]));

        helper(
            "[[]]",
            Ok(vec![
                MiddleToken::WhileBegin,
                MiddleToken::WhileBegin,
                MiddleToken::WhileEnd,
                MiddleToken::WhileEnd,
            ]),
        );
        helper(
            "[+++-]",
            Ok(vec![
                MiddleToken::WhileBegin,
                MiddleToken::Token(Plus, 3),
                MiddleToken::Token(Minus, 1),
                MiddleToken::WhileEnd,
            ]),
        );

        helper("[", Err(ParseError::InvalidBracket));
        helper("]", Err(ParseError::InvalidBracket));
    }
}

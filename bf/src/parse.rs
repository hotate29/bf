use std::{collections::LinkedList, fmt::Display};

use serde::Serialize;
use thiserror::Error;

use crate::instruction::Instruction;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("角括弧が対応してないよ!")]
    InvalidBracket,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Token {
    /// \>
    Greater,
    /// \<
    Less,
    /// \+
    Plus,
    /// \-
    Minus,
    /// .
    Period,
    /// ,
    Comma,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
}
impl Token {
    pub fn from_char(c: char) -> Option<Token> {
        match c {
            '>' => Some(Token::Greater),
            '<' => Some(Token::Less),
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '.' => Some(Token::Period),
            ',' => Some(Token::Comma),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            _ => None,
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
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_char().fmt(f)
    }
}

pub fn tokenize(code: &str) -> Vec<Token> {
    code.chars().filter_map(Token::from_char).collect()
}
pub type Nodes = LinkedList<Node>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Node {
    Loop(Nodes),
    Instruction(Instruction),
}
impl Node {
    pub fn from_tokens(tokens: impl IntoIterator<Item = Token>) -> Result<Nodes, ParseError> {
        fn inner(
            nod: &mut Nodes,
            depth: usize,
            token_iterator: &mut impl Iterator<Item = Token>,
        ) -> Result<(), ParseError> {
            while let Some(token) = token_iterator.next() {
                match token {
                    Token::LeftBracket => {
                        let mut inner_node = Nodes::new();
                        inner(&mut inner_node, depth + 1, token_iterator)?;
                        nod.push_back(Node::Loop(inner_node));
                    }
                    Token::RightBracket => {
                        // 深さ0の時点で]に遭遇するとエラー
                        // 例: ], []]
                        if depth == 0 {
                            return Err(ParseError::InvalidBracket);
                        } else {
                            return Ok(());
                        };
                    }
                    token => {
                        nod.push_back(Node::Instruction(Instruction::from_token(&token).unwrap()))
                    }
                }
            }
            // 深さ0の場合を除いて、]以外で関数を抜けた場合はエラー、
            // 例: [, [[]
            if depth == 0 {
                Ok(())
            } else {
                Err(ParseError::InvalidBracket)
            }
        }

        let mut nods = Nodes::new();

        let mut tokens = tokens.into_iter();

        inner(&mut nods, 0, &mut tokens)?;

        Ok(nods)
    }
    pub fn as_instruction(&self) -> Option<Instruction> {
        match self {
            Node::Loop(_) => None,
            Node::Instruction(ins) => Some(*ins),
        }
    }
}

impl From<Instruction> for Node {
    fn from(ins: Instruction) -> Self {
        Self::Instruction(ins)
    }
}

#[cfg(test)]
mod test {
    use crate::parse::Token;

    #[test]
    fn test_token_from_char() {
        fn helper(c: char, assert_token: Option<Token>) {
            let token = Token::from_char(c);
            assert_eq!(token, assert_token);
        }

        helper('>', Some(Token::Greater));
        helper('<', Some(Token::Less));
        helper('+', Some(Token::Plus));
        helper('-', Some(Token::Minus));
        helper('.', Some(Token::Period));
        helper(',', Some(Token::Comma));
        helper('[', Some(Token::LeftBracket));
        helper(']', Some(Token::RightBracket));

        helper('a', None);
        helper('1', None);
    }
}

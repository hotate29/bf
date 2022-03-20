use std::fmt::Display;

use serde::Serialize;

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

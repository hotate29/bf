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

pub fn tokenize(code: &str) -> Vec<Token> {
    code.chars().map(Token::from_char).collect()
}

#[cfg(test)]
mod test {
    use crate::parse::Token;

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
}

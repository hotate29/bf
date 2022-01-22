#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    PtrIncrement,
    PtrDecrement,
    Increment,
    Decrement,
    Output,
    Input,
    WhileBegin,
    WhileEnd,
    Other(char),
}
impl Token {
    pub fn from_char(c: char) -> Token {
        use Token::*;
        match c {
            '>' => PtrIncrement,
            '<' => PtrDecrement,
            '+' => Increment,
            '-' => Decrement,
            '.' => Output,
            ',' => Input,
            '[' => WhileBegin,
            ']' => WhileEnd,
            c => Other(c),
        }
    }
    pub fn as_char(&self) -> char {
        match self {
            Token::PtrIncrement => '>',
            Token::PtrDecrement => '<',
            Token::Increment => '+',
            Token::Decrement => '-',
            Token::Output => '.',
            Token::Input => ',',
            Token::WhileBegin => '[',
            Token::WhileEnd => ']',
            Token::Other(c) => *c,
        }
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    source.chars().map(Token::from_char).collect()
}

#[cfg(test)]
mod test {
    use super::Token;

    #[test]
    fn test_token_from_char() {
        fn helper(c: char, assert_token: Token) {
            let token = Token::from_char(c);
            assert_eq!(token, assert_token);
        }

        helper('>', Token::PtrIncrement);
        helper('<', Token::PtrDecrement);
        helper('+', Token::Increment);
        helper('-', Token::Decrement);
        helper('.', Token::Output);
        helper(',', Token::Input);
        helper('[', Token::WhileBegin);
        helper(']', Token::WhileEnd);

        helper('a', Token::Other('a'));
        helper('1', Token::Other('1'));
    }
}

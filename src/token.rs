#[derive(Debug, Clone, Copy)]
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

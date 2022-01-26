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
    fn to_instruction(self) -> Option<Instruction> {
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
    AddTo(usize),
    Sub(usize),
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
            Instruction::AddTo(_) | Instruction::SetValue(_, _) => None,
        }
    }
}

// [++[>>]-][]+
// root Node
//   |-while
//   | |-(+2)
//   | |-while
//   | |  |-(>2)
//   | |-(-1)
//   |
//   |-while
//   |-(+1)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum ExprKind {
    Instructions(Vec<Instruction>),
    While(Node),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Node(pub Vec<ExprKind>);

impl Node {
    pub fn from_source(source: &str) -> Result<Node, ParseError> {
        let tokens = tokenize(source);
        let middle_token = middle_token(&tokens)?;
        Ok(Node::from_middle_tokens(&middle_token))
    }
    pub fn from_middle_tokens(tokens: &[MiddleToken]) -> Node {
        fn inner(tokens: &[MiddleToken]) -> (usize, Node) // (どれだけ進んだか, Node)
        {
            let mut exprs = Vec::new();
            let mut index = 0;
            let mut last_while_end_index = None;

            while index < tokens.len() {
                let token = tokens[index];

                match token {
                    MiddleToken::Token(_, _) => index += 1,
                    MiddleToken::WhileBegin => {
                        {
                            let sub_tokens = &tokens[last_while_end_index.unwrap_or(0)..index];
                            if !sub_tokens.is_empty() {
                                exprs.push(ExprKind::Instructions(
                                    sub_tokens
                                        .iter()
                                        .map(|token| token.to_instruction().unwrap())
                                        .collect(),
                                ));
                            }
                        }
                        {
                            index += 1;
                            let (count, while_node) = inner(&tokens[index..]);
                            index += count;
                            last_while_end_index = Some(index);
                            exprs.push(ExprKind::While(while_node));
                        }
                    }
                    MiddleToken::WhileEnd => {
                        {
                            let sub_tokens = &tokens[last_while_end_index.unwrap_or(0)..index];
                            if !sub_tokens.is_empty() {
                                let expr = ExprKind::Instructions(
                                    sub_tokens
                                        .iter()
                                        .map(|token| token.to_instruction().unwrap())
                                        .collect(),
                                );
                                exprs.push(expr)
                            }
                        }

                        let node = Node(exprs);
                        return (index + 1, node);
                    }
                }
            }

            let range = last_while_end_index.unwrap_or(0)..index;
            if !range.is_empty() {
                exprs.push(ExprKind::Instructions(
                    tokens[range]
                        .iter()
                        .map(|token| token.to_instruction().unwrap())
                        .collect(),
                ))
            }
            (index, Node(exprs))
        }
        let (c, node) = inner(tokens);
        assert_eq!(c, tokens.len());
        node
    }
    pub fn to_string(&self) -> Option<String> {
        fn inner(node: &Node, out: &mut String) {
            for expr in &node.0 {
                match expr {
                    ExprKind::Instructions(instructions) => {
                        for instruction in instructions {
                            if let Some(s) = instruction.to_string() {
                                out.push_str(&s);
                            } else {
                                out.push_str("None");
                            }
                        }
                    }
                    ExprKind::While(while_node) => {
                        out.push('[');
                        inner(while_node, out);
                        out.push(']');
                    }
                }
            }
        }
        let mut out = String::new();
        inner(self, &mut out);
        Some(out)
    }
}

#[cfg(test)]
mod test {
    use super::{
        middle_token, tokenize, ExprKind, Instruction, MiddleToken, Node, ParseError, Token,
    };

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
    #[test]
    fn test_node_from_middle_token() {
        fn helper(source: &str, assert_node: Node) {
            let root_node = Node::from_source(source).unwrap();
            assert_eq!(root_node, assert_node);
        }

        helper(
            "+++",
            Node(vec![ExprKind::Instructions(vec![Instruction::Add(3)])]),
        );
        helper(
            "+++[]",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![])),
            ]),
        );
        helper(
            "+++[---]",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![Instruction::Sub(
                    3,
                )])])),
            ]),
        );
        helper(
            "+++[---]+++",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![Instruction::Sub(
                    3,
                )])])),
                ExprKind::Instructions(vec![Instruction::Add(3)]),
            ]),
        );
        helper(
            "+++[--[]]>>><<<",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![
                    ExprKind::Instructions(vec![Instruction::Sub(2)]),
                    ExprKind::While(Node(vec![])),
                ])),
                ExprKind::Instructions(vec![
                    Instruction::PtrIncrement(3),
                    Instruction::PtrDecrement(3),
                ]),
            ]),
        );
        helper(
            "+++[--[]]>>><<<[.,]",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![
                    ExprKind::Instructions(vec![Instruction::Sub(2)]),
                    ExprKind::While(Node(vec![])),
                ])),
                ExprKind::Instructions(vec![
                    Instruction::PtrIncrement(3),
                    Instruction::PtrDecrement(3),
                ]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![
                    Instruction::Output(1),
                    Instruction::Input(1),
                ])])),
            ]),
        );
    }
}

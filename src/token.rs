use std::mem::swap;

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
                Token::Minus => Some(Instruction::Decrement(count)),
                Token::Period => Some(Instruction::Output(count)),
                Token::Comma => Some(Instruction::Input(count)),
                Token::LeftBracket | Token::RightBracket | Token::Other(_) => unreachable!(),
            },
        }
    }
}

pub fn middle_token(tokens: &[Token]) -> Vec<MiddleToken> {
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

    middle_tokens
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    Add(usize),
    Decrement(usize),
    Output(usize),
    Input(usize),
    SetValue(usize, u8),
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ExprKind {
    Instructions(Vec<Instruction>),
    While(Node),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node(pub Vec<ExprKind>);

pub fn node(tokens: &[MiddleToken]) -> Node {
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

#[cfg(test)]
mod test {
    use crate::token::{node, ExprKind, Instruction, MiddleToken, Node};

    use super::{middle_token, tokenize, Token};

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

        fn helper(source: &str, assert_middle_token: &[MiddleToken]) {
            let tokens = tokenize(source);
            let middle_tokens = middle_token(&tokens);
            assert_eq!(middle_tokens, assert_middle_token);
        }

        helper("", &[]);
        helper("brainfuck", &[]);
        helper("bra+inf+uck", &[MiddleToken::Token(Plus, 2)]);

        helper("+", &[MiddleToken::Token(Plus, 1)]);

        helper("+++", &[MiddleToken::Token(Plus, 3)]);
        helper("---", &[MiddleToken::Token(Minus, 3)]);
        helper(">>>", &[MiddleToken::Token(Greater, 3)]);
        helper("<<<", &[MiddleToken::Token(Less, 3)]);
        helper("...", &[MiddleToken::Token(Period, 3)]);
        helper(",,,", &[MiddleToken::Token(Comma, 3)]);

        helper(
            "[[]]",
            &[
                MiddleToken::WhileBegin,
                MiddleToken::WhileBegin,
                MiddleToken::WhileEnd,
                MiddleToken::WhileEnd,
            ],
        );
        helper(
            "[+++-]",
            &[
                MiddleToken::WhileBegin,
                MiddleToken::Token(Plus, 3),
                MiddleToken::Token(Minus, 1),
                MiddleToken::WhileEnd,
            ],
        );
    }
    #[test]
    fn test_nodes() {
        fn helper(s: &str, assert_node: Node) {
            let tokens = tokenize(s);
            let middle_tokens = middle_token(&tokens);
            let root_node = node(&middle_tokens);
            assert_eq!(root_node, assert_node);
        }

        helper(
            "+++",
            Node(vec![ExprKind::Instructions(vec![Instruction::Add(
                3,
            )])]),
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
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![
                    Instruction::Decrement(3),
                ])])),
            ]),
        );
        helper(
            "+++[---]+++",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![
                    Instruction::Decrement(3),
                ])])),
                ExprKind::Instructions(vec![Instruction::Add(3)]),
            ]),
        );
        helper(
            "+++[--[]]>>><<<",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(3)]),
                ExprKind::While(Node(vec![
                    ExprKind::Instructions(vec![Instruction::Decrement(2)]),
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
                    ExprKind::Instructions(vec![Instruction::Decrement(2)]),
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

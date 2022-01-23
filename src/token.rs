use std::mem::swap;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
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
                Token::PtrIncrement => Some(Instruction::PtrIncrement(count)),
                Token::PtrDecrement => Some(Instruction::PtrDecrement(count)),
                Token::Increment => Some(Instruction::Increment(count)),
                Token::Decrement => Some(Instruction::Decrement(count)),
                Token::Output => Some(Instruction::Output(count)),
                Token::Input => Some(Instruction::Input(count)),
                Token::WhileBegin | Token::WhileEnd | Token::Other(_) => unreachable!(),
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
            Token::WhileBegin | Token::WhileEnd => {
                if prev_count != 0 {
                    middle_tokens.push(MiddleToken::Token(prev.unwrap(), prev_count));
                }
                match token {
                    Token::WhileBegin => middle_tokens.push(MiddleToken::WhileBegin),
                    Token::WhileEnd => middle_tokens.push(MiddleToken::WhileEnd),
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
                Token::WhileBegin | Token::WhileEnd | Token::Other(_) => (),
                token => middle_tokens.push(MiddleToken::Token(token, prev_count)),
            }
            prev_count = 1;
        }
    }
    if let Some(token) = prev {
        match token {
            Token::WhileBegin | Token::WhileEnd | Token::Other(_) => (),
            token => middle_tokens.push(MiddleToken::Token(token, prev_count)),
        }
    }

    middle_tokens
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
enum Instruction {
    PtrIncrement(usize),
    PtrDecrement(usize),
    Increment(usize),
    Decrement(usize),
    Output(usize),
    Input(usize),
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
#[derive(Debug, Serialize)]
pub enum ExprKind {
    Tokens(Vec<Instruction>),
    While(Node),
}

#[derive(Debug, Serialize)]
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
                        let sub_tokens = &tokens[..index];
                        if !sub_tokens.is_empty() {
                            exprs.push(ExprKind::Tokens(
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
                        let sub_tokens = &tokens[..index];
                        if !sub_tokens.is_empty() {
                            let expr = ExprKind::Tokens(
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
            exprs.push(ExprKind::Tokens(
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
    use crate::token::MiddleToken;

    use super::{middle_token, tokenize, Token};

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
        helper("bra+inf+uck", &[MiddleToken::Token(Increment, 2)]);

        helper("+", &[MiddleToken::Token(Increment, 1)]);

        helper("+++", &[MiddleToken::Token(Increment, 3)]);
        helper("---", &[MiddleToken::Token(Decrement, 3)]);
        helper(">>>", &[MiddleToken::Token(PtrIncrement, 3)]);
        helper("<<<", &[MiddleToken::Token(PtrDecrement, 3)]);
        helper("...", &[MiddleToken::Token(Output, 3)]);
        helper(",,,", &[MiddleToken::Token(Input, 3)]);

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
                MiddleToken::Token(Increment, 3),
                MiddleToken::Token(Decrement, 1),
                MiddleToken::WhileEnd,
            ],
        );
    }
}

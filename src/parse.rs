use std::fmt::Display;

use serde::Serialize;

use crate::token::{Instruction, ParseError};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Token {
    /// >
    Greater,
    /// >
    Less,
    /// +
    Plus,
    /// -
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

impl ExprKind {
    pub fn concat(&self, other: &ExprKind) -> Option<ExprKind> {
        if let (
            ExprKind::Instructions(self_instructions),
            ExprKind::Instructions(other_instructions),
        ) = (self, other)
        {
            let mut self_instructions = self_instructions.clone();
            self_instructions.extend(other_instructions);

            Some(ExprKind::Instructions(self_instructions))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Node(pub Vec<ExprKind>);

impl Node {
    pub fn from_source(source: &str) -> Result<Node, ParseError> {
        let tokens = tokenize(source);
        Ok(Node::from_tokens(&tokens))
    }
    pub fn from_tokens(tokens: &[Token]) -> Node {
        fn inner(tokens: &[Token]) -> (usize, Node) // (どれだけ進んだか, Node)
        {
            let mut exprs = Vec::new();
            let mut index = 0;
            let mut last_while_end_index = None;

            while index < tokens.len() {
                let token = tokens[index];

                match token {
                    Token::LeftBracket => {
                        {
                            let sub_tokens = &tokens[last_while_end_index.unwrap_or(0)..index];
                            if !sub_tokens.is_empty() {
                                exprs.push(ExprKind::Instructions(
                                    sub_tokens
                                        .iter()
                                        .filter_map(Instruction::from_token)
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
                    Token::RightBracket => {
                        {
                            let sub_tokens = &tokens[last_while_end_index.unwrap_or(0)..index];
                            if !sub_tokens.is_empty() {
                                let expr = ExprKind::Instructions(
                                    sub_tokens
                                        .iter()
                                        .filter_map(Instruction::from_token)
                                        .collect(),
                                );
                                exprs.push(expr)
                            }
                        }

                        let node = Node(exprs);
                        return (index + 1, node);
                    }
                    _ => index += 1,
                }
            }

            let range = last_while_end_index.unwrap_or(0)..index;
            if !range.is_empty() {
                exprs.push(ExprKind::Instructions(
                    tokens[range]
                        .iter()
                        .filter_map(Instruction::from_token)
                        .collect(),
                ))
            }
            (index, Node(exprs))
        }
        let (c, node) = inner(tokens);
        assert_eq!(c, tokens.len());
        node
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        fn inner(node: &Node, out: &mut String) {
            for expr in &node.0 {
                match expr {
                    ExprKind::Instructions(instructions) => {
                        for instruction in instructions {
                            if let Some(s) = instruction.to_compressed_string() {
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
        out
    }
}

#[cfg(test)]
mod test {
    use crate::parse::{ExprKind, Node, Token};
    use crate::token::Instruction;

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

    #[test]
    fn test_node_from_token() {
        fn helper(source: &str, assert_node: Node) {
            let root_node = Node::from_source(source).unwrap();
            assert_eq!(root_node, assert_node);
        }

        helper(
            "+++",
            Node(vec![ExprKind::Instructions(vec![Instruction::Add(1); 3])]),
        );
        helper(
            "+++[]",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(1); 3]),
                ExprKind::While(Node(vec![])),
            ]),
        );
        helper(
            "+++[---]",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(1); 3]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![
                    Instruction::Sub(1);
                    3
                ])])),
            ]),
        );
        helper(
            "+++[---]+++",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(1); 3]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![
                    Instruction::Sub(1);
                    3
                ])])),
                ExprKind::Instructions(vec![Instruction::Add(1); 3]),
            ]),
        );
        helper(
            "+++[--[]]>>><<<",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(1); 3]),
                ExprKind::While(Node(vec![
                    ExprKind::Instructions(vec![Instruction::Sub(1); 2]),
                    ExprKind::While(Node(vec![])),
                ])),
                ExprKind::Instructions(vec![
                    Instruction::PtrIncrement(1),
                    Instruction::PtrIncrement(1),
                    Instruction::PtrIncrement(1),
                    Instruction::PtrDecrement(1),
                    Instruction::PtrDecrement(1),
                    Instruction::PtrDecrement(1),
                ]),
            ]),
        );
        helper(
            "+++[--[]]>>><<<[.,]",
            Node(vec![
                ExprKind::Instructions(vec![Instruction::Add(1); 3]),
                ExprKind::While(Node(vec![
                    ExprKind::Instructions(vec![Instruction::Sub(1); 2]),
                    ExprKind::While(Node(vec![])),
                ])),
                ExprKind::Instructions(vec![
                    Instruction::PtrIncrement(1),
                    Instruction::PtrIncrement(1),
                    Instruction::PtrIncrement(1),
                    Instruction::PtrDecrement(1),
                    Instruction::PtrDecrement(1),
                    Instruction::PtrDecrement(1),
                ]),
                ExprKind::While(Node(vec![ExprKind::Instructions(vec![
                    Instruction::Output(1),
                    Instruction::Input(1),
                ])])),
            ]),
        );
    }
}

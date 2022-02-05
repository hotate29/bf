use serde::Serialize;

use crate::token::{middle_token, tokenize, Instruction, MiddleToken, ParseError};

mod copy;
mod copy_rev;
mod move_add;
mod move_add_rev;
mod move_sub;
mod move_sub_rev;
mod mul_add;
mod mul_add_rev;
mod sub_copy;
mod zeroset;

pub trait Optimizer {
    fn optimize_exprs(&self, _node: &[ExprKind]) -> Option<(usize, Vec<ExprKind>)> {
        None
    }
    fn optimize_while(&self, _expr: &ExprKind) -> Option<ExprKind> {
        None
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

pub fn optimize(mut root_node: Node, optimizers: &[Box<dyn Optimizer>]) -> Node {
    fn inner(node: &mut Node, optimizers: &[Box<dyn Optimizer>]) {
        for optimizer in optimizers {
            for i in 0..node.0.len() {
                let (front_exprs, back_exprs) = &node.0.split_at(i);

                if let Some((offset, optimized_exprs)) = optimizer.optimize_exprs(back_exprs) {
                    let mut exprs = front_exprs.to_vec();
                    exprs.extend(optimized_exprs);
                    exprs.extend_from_slice(&back_exprs[offset..]);

                    node.0 = exprs;
                }
            }
        }
        for expr in &mut node.0 {
            // ExprKindを最適化する
            if let ExprKind::While(while_node) = expr {
                inner(while_node, optimizers);
            }
            for optimizer in optimizers {
                if let Some(optimized_expr) = optimizer.optimize_while(expr) {
                    *expr = optimized_expr;
                }
            }
        }
        // ExprKind::Instructionsが何個も続くと後の最適化で困るので、一つにまとめる。
        if let Some(expr) = node
            .0
            .iter()
            .try_fold(ExprKind::Instructions(vec![]), |i, expr| i.concat(expr))
        {
            node.0 = vec![expr];
        }
    }

    inner(&mut root_node, optimizers);

    root_node
}

pub fn all_optimizer() -> Vec<Box<dyn Optimizer>> {
    vec![
        Box::new(zeroset::ZeroSetOptimizer),
        Box::new(mul_add::MulAddOptimizer),
        Box::new(mul_add_rev::MulAddRevOptimizer),
        Box::new(move_add::MoveAddOptimizer),
        Box::new(move_add_rev::MoveAddRevOptimizer),
        Box::new(move_sub::MoveSubOptimizer),
        Box::new(move_sub_rev::MoveSubRevOptimizer),
        Box::new(copy::CopyOptimizer),
        Box::new(copy_rev::CopyRevOptimizer),
        Box::new(sub_copy::SubCopyOptimizer),
    ]
}

#[cfg(test)]
pub(crate) mod test {
    use crate::token::Instruction;

    use super::{ExprKind, Node, Optimizer};

    pub fn expr_helper(source: &str, assert_expr: Option<ExprKind>, optimizer: impl Optimizer) {
        let root_node = Node::from_source(source).unwrap();

        if let [expr] = root_node.0.as_slice() {
            let optimized_expr = optimizer.optimize_while(expr);
            assert_eq!(optimized_expr, assert_expr);
        } else {
            panic!("変なテストデータ")
        }
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

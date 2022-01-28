use if_chain::if_chain;
use log::info;
use serde::Serialize;

use crate::token::{middle_token, tokenize, Instruction, MiddleToken, ParseError};

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
}

impl ToString for Node {
    fn to_string(&self) -> String {
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
        out
    }
}

pub fn optimize(mut root_node: Node) -> Node {
    fn inner(node: &mut Node) {
        if let Some(optimized_node) = opt_set_value(node) {
            info!("optimize: opt_set_value");
            *node = optimized_node;
        }
        for expr in &mut node.0 {
            // ExprKindを最適化する
            if let Some(optimized_expr) = opt_zeroset(expr) {
                info!("optimize: opt_zeroset");
                *expr = optimized_expr;
            }
            if let Some(optimized_expr) = opt_move_add(expr) {
                info!("optimize: opt_move_add");
                *expr = optimized_expr;
            }
            if let Some(optimized_expr) = opt_move_add_rev(expr) {
                info!("optimize: opt_move_add_rev");
                *expr = optimized_expr;
            }
            if let Some(optimized_expr) = opt_move_sub(expr) {
                info!("optimize: opt_move_sub");
                *expr = optimized_expr;
            }
            if let Some(optimized_expr) = opt_move_sub_rev(expr) {
                info!("optimize: opt_move_sub_rev");
                *expr = optimized_expr;
            }

            if let ExprKind::While(while_node) = expr {
                inner(while_node);
            }
        }
    }

    inner(&mut root_node);

    root_node
}

/// [-]をSetValue(0)に変換する
fn opt_zeroset(expr: &ExprKind) -> Option<ExprKind> {
    if_chain! {
        if let ExprKind::While(while_node) = expr;
        if let [ExprKind::Instructions(instructions)] = while_node.0.as_slice();
        if let [Instruction::Sub(1)] = instructions.as_slice();
        then {
            Some(ExprKind::Instructions(vec![Instruction::SetValue(0, 0)]))
        }
        else {
            None
        }
    }
}

/// +n[-x>+m<x]>をSetToValue(x, n*m)に変換する
fn opt_set_value(node: &Node) -> Option<Node> {
    for i in 0..node.0.len() {
        let front_kinds = &node.0[0..i];

        if_chain! {
            if let &[ExprKind::Instructions(instructions), ExprKind::While(while_node), ExprKind::Instructions(s), last_kinds @ ..] =
            &node.0.as_slice();
            if let [front_instructions @ .., Instruction::Add(n)] = instructions.as_slice();
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(ptrinc_count), Instruction::Add(x), Instruction::PtrDecrement(ptrdec_count)] = while_instructions.as_slice();
            if ptrinc_count == ptrdec_count;
            then {
                eprintln!("a {} {} {}", n * x, n, x);
                let x = n * x;
                let x = (x % u8::MAX as usize) as u8;

                let mut node_kinds = front_kinds.to_vec();

                let mut instructions = front_instructions.to_vec();
                instructions.push(Instruction::SetValue(*ptrinc_count, x));
                instructions.extend_from_slice(s);
                node_kinds.push(ExprKind::Instructions(instructions));

                node_kinds.extend_from_slice(last_kinds);

                let optimized_node = Node(node_kinds);
                return Some(optimized_node);
            }
        }
    }
    None
}

fn opt_move_add(expr: &ExprKind) -> Option<ExprKind> {
    if_chain! {
        if let ExprKind::While(while_node) = expr;
        if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
        if let [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Add(1), Instruction::PtrDecrement(ptr_decrement)] = while_instructions.as_slice();
        if ptr_increment == ptr_decrement;
        then {
            let optimized_expr = ExprKind::Instructions(vec![
                Instruction::MoveAdd(*ptr_increment),
            ]);
            Some(optimized_expr)
        }
        else {
            None
        }
    }
}

fn opt_move_add_rev(expr: &ExprKind) -> Option<ExprKind> {
    if_chain! {
        if let ExprKind::While(while_node) = expr;
        if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
        if let [Instruction::Sub(1), Instruction::PtrDecrement(x), Instruction::Add(1), Instruction::PtrIncrement(y)] = while_instructions.as_slice();
        if x == y;
        then {
            let expr = ExprKind::Instructions(vec![Instruction::MoveAddRev(*x)]);
            Some(expr)
        }
        else {
            None
        }
    }
}

fn opt_move_sub(expr: &ExprKind) -> Option<ExprKind> {
    if_chain! {
        if let ExprKind::While(while_node) = expr;
        if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
        if let [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Sub(1), Instruction::PtrDecrement(ptr_decrement)] = while_instructions.as_slice();
        if ptr_increment == ptr_decrement;
        then {
            let optimized_expr = ExprKind::Instructions(vec![
                Instruction::MoveSub(*ptr_increment),
            ]);
            Some(optimized_expr)
        }
        else {
            None
        }
    }
}

fn opt_move_sub_rev(expr: &ExprKind) -> Option<ExprKind> {
    if_chain! {
        if let ExprKind::While(while_node) = expr;
        if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
        if let [Instruction::Sub(1), Instruction::PtrDecrement(x), Instruction::Sub(1), Instruction::PtrIncrement(y)] = while_instructions.as_slice();
        if x == y;
        then {
            let expr = ExprKind::Instructions(vec![Instruction::MoveSubRev(*x)]);
            Some(expr)
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::token::Instruction;

    use super::{
        opt_move_add, opt_move_add_rev, opt_move_sub, opt_set_value, opt_zeroset, ExprKind, Node,
    };

    #[test]
    fn test_opt_zeroset() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = opt_zeroset(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }

        helper(
            "[-]",
            Some(ExprKind::Instructions(vec![Instruction::SetValue(0, 0)])),
        );
        helper("[>]", None);
    }

    #[test]
    fn test_opt_set_value() {
        fn helper(source: &str, assert_node: Option<Node>) {
            let root_node = Node::from_source(source).unwrap();

            let optimized_node = opt_set_value(&root_node);
            assert_eq!(optimized_node, assert_node);
        }
        helper(
            ">++++++++++[->++++++++++<]>",
            Some(Node(vec![ExprKind::Instructions(vec![
                Instruction::PtrIncrement(1),
                Instruction::SetValue(1, 100),
                Instruction::PtrIncrement(1),
            ])])),
        );
        helper(
            ">++[->+++<]>",
            Some(Node(vec![ExprKind::Instructions(vec![
                Instruction::PtrIncrement(1),
                Instruction::SetValue(1, 6),
                Instruction::PtrIncrement(1),
            ])])),
        );

        helper("++++++++++[->+++++++++<<]>", None);
    }

    #[test]
    fn test_opt_move_add() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = opt_move_add(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }

        helper(
            "[->+<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveAdd(1)])),
        );
        helper(
            "[->>>>>>>>>>+<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveAdd(10)])),
        );

        helper("[->+<<]", None);
    }
    #[test]
    fn test_opt_move_add_rev() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = opt_move_add_rev(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }

        helper(
            "[-<+>]",
            Some(ExprKind::Instructions(vec![Instruction::MoveAddRev(1)])),
        );
        helper(
            "[-<<<<<<<<<<+>>>>>>>>>>]",
            Some(ExprKind::Instructions(vec![Instruction::MoveAddRev(10)])),
        );

        helper("[->+<]", None);
    }
    #[test]
    fn test_opt_move_sub() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = opt_move_sub(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }
        helper(
            "[->-<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveSub(1)])),
        );
        helper(
            "[->>>>>>>>>>-<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveSub(10)])),
        );

        helper("[->+<<]", None);
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

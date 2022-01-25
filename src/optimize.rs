use if_chain::if_chain;

use crate::token::{ExprKind, Instruction, Node};

pub fn optimize(mut root_node: Node) -> Node {
    fn inner(node: &mut Node) {
        if let Some(optimized_node) = opt_set_value(node) {
            *node = optimized_node;
        }
        for expr in &mut node.0 {
            // ExprKindを最適化する
            if let Some(optimized_expr) = opt_zeroset(expr) {
                *expr = optimized_expr;
            }
            if let Some(optimized_expr) = opt_move_add(expr) {
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
                Instruction::AddTo(*ptr_increment),
                Instruction::SetValue(0, 0),
            ]);
            Some(optimized_expr)
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        optimize::opt_move_add,
        token::{middle_token, node, tokenize, ExprKind, Instruction, Node},
    };

    use super::{opt_set_value, opt_zeroset};

    #[test]
    fn test_opt_zeroset() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let tokens = tokenize(source);
            let middle_tokens = middle_token(&tokens);
            let root_node = node(&middle_tokens);
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
            let tokens = tokenize(source);
            let middle_tokens = middle_token(&tokens);
            let root_node = node(&middle_tokens);

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
            let tokens = tokenize(source);
            let middle_tokens = middle_token(&tokens);
            let root_node = node(&middle_tokens);
            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = opt_move_add(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }

        helper(
            "[->+<]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddTo(1),
                Instruction::SetValue(0, 0),
            ])),
        );
        helper(
            "[->>>>>>>>>>+<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddTo(10),
                Instruction::SetValue(0, 0),
            ])),
        );

        helper("[->+<<]", None);
    }
}

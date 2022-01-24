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
        if let [Instruction::Decrement(1)] = instructions.as_slice();
        then {
            Some(ExprKind::Instructions(vec![Instruction::SetValue(0)]))
        }
        else {
            None
        }
    }
}

fn opt_set_value(node: &Node) -> Option<Node> {
    for i in 0..node.0.len() {
        let front_kinds = &node.0[0..i];

        if_chain! {
            if let &[ExprKind::Instructions(instructions), ExprKind::While(while_node), ExprKind::Instructions(s), last_kinds @ ..] =
            &node.0.as_slice();
            if let [front_instructions @ .., Instruction::Increment(n)] = instructions.as_slice();
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Decrement(1), Instruction::PtrIncrement(ptrinc_count), Instruction::Increment(x), Instruction::PtrDecrement(ptrdec_count)] = while_instructions.as_slice();
            if ptrinc_count == ptrdec_count;
            then {
                eprintln!("a {} {} {}", n * x, n, x);
                let x = n * x;
                let x = (x % u8::MAX as usize) as u8;

                let mut node_kinds = front_kinds.to_vec();

                let mut instructions = front_instructions.to_vec();
                instructions.push(Instruction::SetToValue(*ptrinc_count, x));
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

#[cfg(test)]
mod tests {
    use crate::token::{ExprKind, Instruction, Node};

    use super::opt_zeroset;

    #[test]
    fn test_opt_zeroset() {
        let expr = ExprKind::While(Node(vec![ExprKind::Instructions(vec![
            Instruction::Decrement(1),
        ])]));

        let optimized_expr = opt_zeroset(&expr).unwrap();
        assert_eq!(
            optimized_expr,
            ExprKind::Instructions(vec![Instruction::SetValue(0)])
        );
    }
}

use if_chain::if_chain;

use crate::token::{ExprKind, Instruction, Node};

pub fn optimize(mut root_node: Node) -> Node {
    fn inner(node: &mut Node) {
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

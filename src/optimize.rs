use crate::token::{ExprKind, Instruction, Node};

pub fn optimize(mut root_node: Node) -> Node {
    fn inner(expr: &mut ExprKind) {
        if let Some(optimized_node) = opt_zeroset(expr) {
            *expr = optimized_node;
        }
        if let ExprKind::While(while_node) = expr {
            for expr in &mut while_node.0 {
                inner(expr)
            }
        }
    }

    for expr in &mut root_node.0 {
        inner(expr);
    }

    root_node
}

/// [-]をSetValue(0)に変換する
fn opt_zeroset(expr: &ExprKind) -> Option<ExprKind> {
    if let ExprKind::While(while_node) = expr {
        if let [ExprKind::Instructions(instructions)] = while_node.0.as_slice() {
            if let [Instruction::Decrement(1)] = instructions.as_slice() {
                return Some(ExprKind::Instructions(vec![Instruction::SetValue(0)]));
            }
        }
    }

    None
}

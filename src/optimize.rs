use crate::token::{ExprKind, Node};

pub fn optimize(mut root_node: Node) -> Node {
    fn inner(expr: &mut ExprKind) {
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

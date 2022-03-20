pub use crate::parse::{ExprKind, Node};

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
mod test {
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
}

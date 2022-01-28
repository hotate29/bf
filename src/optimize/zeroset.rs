use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct ZeroSetOptimizer;

impl Optimizer for ZeroSetOptimizer {
    /// [-]をSetValue(0)に変換する
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1)] = instructions.as_slice();
            then {
                info!("optimize!");
                Some(ExprKind::Instructions(vec![Instruction::SetValue(0, 0)]))
            }
            else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        optimize::{zeroset::ZeroSetOptimizer, ExprKind, Node, Optimizer},
        token::Instruction,
    };

    #[test]
    fn test_opt_zeroset() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = ZeroSetOptimizer.optimize_expr(expr);
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
}

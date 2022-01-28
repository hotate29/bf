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
    use super::ZeroSetOptimizer;
    use crate::{
        optimize::{test::expr_helper, ExprKind},
        token::Instruction,
    };

    #[test]
    fn test_opt_zeroset() {
        expr_helper(
            "[-]",
            Some(ExprKind::Instructions(vec![Instruction::SetValue(0, 0)])),
            ZeroSetOptimizer,
        );
        expr_helper("[>]", None, ZeroSetOptimizer);
    }
}

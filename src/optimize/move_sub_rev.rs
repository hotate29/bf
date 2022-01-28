use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MoveSubRevOptimizer;

impl Optimizer for MoveSubRevOptimizer {
    /// [-<<<->>>]を変換する
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrDecrement(x), Instruction::Sub(1), Instruction::PtrIncrement(y)] = while_instructions.as_slice();
            if x == y;
            then {
                info!("optimize!");
                let expr = ExprKind::Instructions(vec![Instruction::MoveSubRev(*x)]);
                Some(expr)
            }
            else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::MoveSubRevOptimizer;
    use crate::{
        optimize::{test::expr_helper, ExprKind},
        token::Instruction,
    };

    #[test]
    fn test_opt_move_sub_rev() {
        expr_helper(
            "[-<->]",
            Some(ExprKind::Instructions(vec![Instruction::MoveSubRev(1)])),
            MoveSubRevOptimizer,
        );
        expr_helper(
            "[-<<<<<<<<<<->>>>>>>>>>]",
            Some(ExprKind::Instructions(vec![Instruction::MoveSubRev(10)])),
            MoveSubRevOptimizer,
        );

        expr_helper("[->+<]", None, MoveSubRevOptimizer);
    }
}

use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MoveSubOptimizer;

impl Optimizer for MoveSubOptimizer {
    /// [->>>-<<<]を変換する
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Sub(1), Instruction::PtrDecrement(ptr_decrement)] = while_instructions.as_slice();
            if ptr_increment == ptr_decrement;
            then {
                info!("optimize!");
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
}

#[cfg(test)]
mod test {
    use super::MoveSubOptimizer;
    use crate::{
        optimize::{test::expr_helper, ExprKind},
        token::Instruction,
    };

    #[test]
    fn test_opt_move_sub() {
        expr_helper(
            "[->-<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveSub(1)])),
            MoveSubOptimizer,
        );
        expr_helper(
            "[->>>>>>>>>>-<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveSub(10)])),
            MoveSubOptimizer,
        );

        expr_helper("[->+<<]", None, MoveSubOptimizer);
    }
}

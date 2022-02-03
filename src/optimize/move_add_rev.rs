use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MoveAddRevOptimizer;

impl Optimizer for MoveAddRevOptimizer {
    /// [-<<<+>>>]を変換する
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrDecrement(x), Instruction::Add(1), Instruction::PtrIncrement(y)]
                 | [Instruction::PtrDecrement(x), Instruction::Add(1), Instruction::PtrIncrement(y), Instruction::Sub(1)] = while_instructions.as_slice();
            if x == y;
            then {
                info!("optimize!");
                let expr = ExprKind::Instructions(vec![Instruction::AddToRev(*x), Instruction::ZeroSet]);
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
    use super::MoveAddRevOptimizer;
    use crate::{
        optimize::{test::expr_helper, ExprKind},
        token::Instruction,
    };

    #[test]
    fn test_opt_move_add_rev() {
        expr_helper(
            "[-<+>]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddToRev(1),
                Instruction::ZeroSet,
            ])),
            MoveAddRevOptimizer,
        );
        expr_helper(
            "[-<<<<<<<<<<+>>>>>>>>>>]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddToRev(10),
                Instruction::ZeroSet,
            ])),
            MoveAddRevOptimizer,
        );

        expr_helper("[->+<]", None, MoveAddRevOptimizer);
    }
}

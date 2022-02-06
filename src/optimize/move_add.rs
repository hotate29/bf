use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MoveAddOptimizer;

impl Optimizer for MoveAddOptimizer {
    /// [->>>+<<<]を変換する
    fn optimize_while(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Add(1), Instruction::PtrDecrement(ptr_decrement)]
                    // -が後ろにある場合
                 | [Instruction::PtrIncrement(ptr_increment), Instruction::Add(1), Instruction::PtrDecrement(ptr_decrement), Instruction::Sub(1)]
                    = while_instructions.as_slice();
            if ptr_increment == ptr_decrement;
            then {
                info!("optimize!");
                let optimized_expr = ExprKind::Instructions(vec![
                    Instruction::AddTo(*ptr_increment), Instruction::ZeroSet
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
    use super::MoveAddOptimizer;
    use crate::{
        optimize::{test::expr_helper, ExprKind},
        token::Instruction,
    };

    #[test]
    fn test_opt_move_add() {
        expr_helper(
            "[->+<]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddTo(1),
                Instruction::ZeroSet,
            ])),
            MoveAddOptimizer,
        );
        expr_helper(
            "[>+<-]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddTo(1),
                Instruction::ZeroSet,
            ])),
            MoveAddOptimizer,
        );
        expr_helper(
            "[->>>>>>>>>>+<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![
                Instruction::AddTo(10),
                Instruction::ZeroSet,
            ])),
            MoveAddOptimizer,
        );

        expr_helper("[->+<<]", None, MoveAddOptimizer);
    }
}

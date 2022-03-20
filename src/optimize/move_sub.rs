use if_chain::if_chain;
use log::info;

use crate::instruction::Instruction;

use super::Optimizer;
use crate::parse::ExprKind;

pub struct MoveSubOptimizer;

impl Optimizer for MoveSubOptimizer {
    /// [->>>-<<<]を変換する
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Sub(1), Instruction::PtrDecrement(ptr_decrement)]
                    = while_instructions.as_slice();
            if ptr_increment == ptr_decrement;
            then {
                info!("optimize!");
                let optimized_expr = ExprKind::Instructions(vec![
                    Instruction::SubTo(*ptr_increment), Instruction::ZeroSet
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
    use crate::{optimize::test::expr_helper, parse::ExprKind, instruction::Instruction};

    #[test]
    fn test_opt_move_sub() {
        expr_helper(
            "[->-<]",
            Some(ExprKind::Instructions(vec![
                Instruction::SubTo(1),
                Instruction::ZeroSet,
            ])),
            MoveSubOptimizer,
        );
        expr_helper(
            "[->>>>>>>>>>-<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![
                Instruction::SubTo(10),
                Instruction::ZeroSet,
            ])),
            MoveSubOptimizer,
        );

        expr_helper("[->+<<]", None, MoveSubOptimizer);
    }
}

use if_chain::if_chain;
use log::info;

use crate::{instruction::Instruction, parse::ExprKind};

use super::Optimizer;

pub struct MulAddRevOptimizer;

impl Optimizer for MulAddRevOptimizer {
    /// [<+++++++>-]を変換する
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::PtrDecrement(ptr_decrement), Instruction::Add(add_count), Instruction::PtrIncrement(ptr_increment), Instruction::Sub(1)]
                    = while_instructions.as_slice();
            if ptr_decrement == ptr_increment && *add_count > 1;
            then {
                info!("optimize!");

                let expr =
                    ExprKind::Instructions(
                        vec![
                            Instruction::MulAddRev(*ptr_decrement, *add_count),
                            Instruction::ZeroSet,
                        ]
                    );

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
    use super::MulAddRevOptimizer;
    use crate::{instruction::Instruction, optimize::test::expr_helper, parse::ExprKind};

    #[test]
    fn test_opt_move_add_rev() {
        expr_helper(
            "[<+++>-]",
            Some(ExprKind::Instructions(vec![
                Instruction::MulAddRev(1, 3),
                Instruction::ZeroSet,
            ])),
            MulAddRevOptimizer,
        );
        expr_helper(
            "[<<+++++>>-]",
            Some(ExprKind::Instructions(vec![
                Instruction::MulAddRev(2, 5),
                Instruction::ZeroSet,
            ])),
            MulAddRevOptimizer,
        );
        expr_helper("[<+>-]", None, MulAddRevOptimizer);

        expr_helper("[->+<<]", None, MulAddRevOptimizer);
    }
}

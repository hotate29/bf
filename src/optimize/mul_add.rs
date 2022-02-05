use if_chain::if_chain;
use log::info;

use crate::{optimize::ExprKind, token::Instruction};

use super::Optimizer;

pub struct MulAddOptimizer;

impl Optimizer for MulAddOptimizer {
    /// [->>>+++<<<]を変換する
    fn optimize_while(&self, expr: &ExprKind) -> Option<ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::PtrIncrement(ptr_increment), Instruction::Add(add_count), Instruction::PtrDecrement(ptr_decrement), Instruction::Sub(1)]
                |  [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Add(add_count), Instruction::PtrDecrement(ptr_decrement)]
                = while_instructions.as_slice();
            if ptr_increment == ptr_decrement && *add_count > 1;
            then {
                info!("optimize!");
                let expr =
                    ExprKind::Instructions(
                        vec![
                            Instruction::MulAdd(
                                *ptr_increment,
                                *add_count,
                                ),
                            Instruction::ZeroSet
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
    use super::MulAddOptimizer;
    use crate::{
        optimize::{test::expr_helper, ExprKind},
        token::Instruction,
    };

    #[test]
    fn test_mul_add() {
        expr_helper("[->+<]", None, MulAddOptimizer);
        expr_helper("[>+<-]", None, MulAddOptimizer);
        expr_helper(
            "[->>>+++++<<<]",
            Some(ExprKind::Instructions(vec![
                Instruction::MulAdd(3, 5),
                Instruction::ZeroSet,
            ])),
            MulAddOptimizer,
        );
        expr_helper("[->>>-----<<<]", None, MulAddOptimizer);
    }
}

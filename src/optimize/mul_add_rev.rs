use if_chain::if_chain;
use log::info;

use crate::{optimize::ExprKind, token::Instruction};

use super::Optimizer;

pub struct MulAddRevOptimizer;

impl Optimizer for MulAddRevOptimizer {
    /// [<+++++++>-]を変換する
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::PtrDecrement(ptr_decrement), Instruction::Add(add_count), Instruction::PtrIncrement(ptr_increment), Instruction::Sub(1)] = while_instructions.as_slice();
            if ptr_decrement == ptr_increment;
            then {
                info!("optimize!");
                let add_count = (*add_count % u8::MAX as usize) as u8;

                let expr = ExprKind::Instructions(vec![Instruction::MulAddRev(*ptr_decrement, add_count)]);

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
    use crate::{
        optimize::{mul_add_rev::MulAddRevOptimizer, ExprKind, Node, Optimizer},
        token::Instruction,
    };

    #[test]
    fn test_opt_move_add_rev() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = MulAddRevOptimizer.optimize_expr(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }

        helper(
            "[<+++>-]",
            Some(ExprKind::Instructions(vec![Instruction::MulAddRev(1, 3)])),
        );
        helper(
            "[<<+++++>>-]",
            Some(ExprKind::Instructions(vec![Instruction::MulAddRev(2, 5)])),
        );

        helper("[->+<<]", None);
    }
}

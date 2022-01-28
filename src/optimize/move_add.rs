use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MoveAddOptimizer;

impl Optimizer for MoveAddOptimizer {
    /// [->>>+<<<]を変換する
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Add(1), Instruction::PtrDecrement(ptr_decrement)] = while_instructions.as_slice();
            if ptr_increment == ptr_decrement;
            then {
                info!("optimize!");
                let optimized_expr = ExprKind::Instructions(vec![
                    Instruction::MoveAdd(*ptr_increment),
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
    use crate::{
        optimize::{move_add::MoveAddOptimizer, ExprKind, Node, Optimizer},
        token::Instruction,
    };

    #[test]
    fn test_opt_move_add() {
        fn helper(source: &str, assert_expr: Option<ExprKind>) {
            let root_node = Node::from_source(source).unwrap();

            if let [expr] = root_node.0.as_slice() {
                let optimized_expr = MoveAddOptimizer.optimize_expr(expr);
                assert_eq!(optimized_expr, assert_expr);
            } else {
                panic!("変なテストデータ")
            }
        }

        helper(
            "[->+<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveAdd(1)])),
        );
        helper(
            "[->>>>>>>>>>+<<<<<<<<<<]",
            Some(ExprKind::Instructions(vec![Instruction::MoveAdd(10)])),
        );

        helper("[->+<<]", None);
    }
}

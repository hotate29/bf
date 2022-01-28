use if_chain::if_chain;
use log::info;

use crate::{optimize::ExprKind, token::Instruction};

use super::Optimizer;

pub struct MulAddOptimizer;

impl Optimizer for MulAddOptimizer {
    /// [->>>+++<<<]を変換する
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if let ExprKind::While(while_node) = expr {
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice() {
                if let [Instruction::PtrIncrement(ptr_increment), Instruction::Add(add_count), Instruction::PtrDecrement(ptr_decrement), Instruction::Sub(1)]
                | [Instruction::Sub(1), Instruction::PtrIncrement(ptr_increment), Instruction::Add(add_count), Instruction::PtrDecrement(ptr_decrement)] =
                    while_instructions.as_slice()
                {
                    if ptr_increment == ptr_decrement {
                        info!("optimize!");
                        let add_count = (add_count % u8::MAX as usize) as u8;
                        let expr = ExprKind::Instructions(vec![Instruction::MulAdd(
                            *ptr_increment,
                            add_count,
                        )]);
                        return Some(expr);
                    }
                }
            }
        }
        None
    }
}

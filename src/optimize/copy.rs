use if_chain::if_chain;
use log::info;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct CopyOptimizer;

impl Optimizer for CopyOptimizer {
    fn optimize_expr(&self, expr: &super::ExprKind) -> Option<super::ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(x), Instruction::Add(1), Instruction::PtrIncrement(y), Instruction::Add(1), Instruction::PtrDecrement(z)] =
                    while_instructions.as_slice();
            if x + y == *z;
            then {
                info!("optimize!");
                let expr = ExprKind::Instructions(vec![
                    Instruction::Copy(*x),
                    Instruction::Copy(x + y),
                    Instruction::ZeroSet,
                ]);
                Some(expr)
            }
            else {
                None
            }
        }
    }
}

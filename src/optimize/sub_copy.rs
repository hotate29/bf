use if_chain::if_chain;
use log::info;

use super::Optimizer;
use crate::instruction::Instruction;
use crate::parse::ExprKind;

pub struct SubCopyOptimizer;

impl Optimizer for SubCopyOptimizer {
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        // [-<-<+>>]
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrDecrement(x), Instruction::Sub(1), Instruction::PtrDecrement(y), Instruction::Add(1), Instruction::PtrIncrement(z)]
                    = while_instructions.as_slice();
            if x + y == *z;
            then {
                info!("optimize!");
                let expr = ExprKind::Instructions(
                    vec![Instruction::SubToRev(*x),Instruction::CopyRev(*x+y),Instruction::ZeroSet]
                );
                Some(expr)
            }
            else {
                None
            }
        }.or_else(||
            // [->-<<+>]
            if_chain! {
                if let ExprKind::While(while_node) = expr;
                if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
                if let [Instruction::Sub(1), Instruction::PtrIncrement(x), Instruction::Sub(1), Instruction::PtrDecrement(y), Instruction::Add(1), Instruction::PtrIncrement(z)] = while_instructions.as_slice();
                if x + z == *y;
                then {
                    info!("optimize! 2");
                    let expr = ExprKind::Instructions(
                        vec![Instruction::SubTo(*x), Instruction::CopyRev(y-x), Instruction::ZeroSet]
                    );
                    Some(expr)
                }
                else {
                    None
                }
            }
        ).or_else(||
            // [<->-<+>]
            if_chain! {
                if let ExprKind::While(while_node) = expr;
                if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
                if let [Instruction::PtrDecrement(x), Instruction::Sub(1), Instruction::PtrIncrement(y), Instruction::Sub(1), Instruction::PtrDecrement(z), Instruction::Add(1), Instruction::PtrIncrement(a)] =
                        while_instructions.as_slice();
                if x + z == y + a;
                then {
                    info!("optimize! 3");
                    let expr = ExprKind::Instructions(
                        vec![Instruction::SubToRev(*x), Instruction::CopyRev(*z), Instruction::ZeroSet]
                    );
                    Some(expr)
                }
                else {
                    None
                }
            }
        )
    }
}

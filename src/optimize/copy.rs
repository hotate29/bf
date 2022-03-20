use if_chain::if_chain;
use log::info;

use super::Optimizer;
use crate::parse::ExprKind;
use crate::token::Instruction;

pub struct CopyOptimizer;

impl Optimizer for CopyOptimizer {
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        // [->+>+<<]
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1), Instruction::PtrIncrement(x), Instruction::Add(1), Instruction::PtrIncrement(y), Instruction::Add(1), Instruction::PtrDecrement(z)]
                    = while_instructions.as_slice();
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
        }.or_else(||
            // [->>+<+<]
            if_chain! {
                if let ExprKind::While(while_node) = expr;
                if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
                if let [Instruction::Sub(1), Instruction::PtrIncrement(x), Instruction::Add(1), Instruction::PtrDecrement(y), Instruction::Add(1), Instruction::PtrDecrement(z)] =
                        while_instructions.as_slice();
                if *x == y + z;
                then {
                    info!("optimize! 2");
                    let expr = ExprKind::Instructions(vec![
                        Instruction::Copy(*x),
                        Instruction::Copy(x - y),
                        Instruction::ZeroSet,
                    ]);
                    Some(expr)
                }
                else {
                    None
                }
            }
        ).or_else(||
            // [->>>+<+<+<]
            if_chain! {
                if let ExprKind::While(while_node) = expr;
                if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
                if let [Instruction::Sub(1), Instruction::PtrIncrement(x), Instruction::Add(1), Instruction::PtrDecrement(y), Instruction::Add(1), Instruction::PtrDecrement(z), Instruction::Add(1), Instruction::PtrDecrement(a)] =
                        while_instructions.as_slice();
                if *x == y + z + a;
                then {
                    info!("optimize! 3");
                    let expr = ExprKind::Instructions(vec![
                        Instruction::Copy(*x),
                        Instruction::Copy(x - y),
                        Instruction::Copy(x - y - z),
                        Instruction::ZeroSet,
                    ]);
                    Some(expr)
                }
                else {
                    None
                }
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::CopyOptimizer;
    use crate::{optimize::test::expr_helper, parse::ExprKind, token::Instruction};

    #[test]
    fn test_opt_copy() {
        expr_helper(
            "[->+>+<<]",
            Some(ExprKind::Instructions(vec![
                Instruction::Copy(1),
                Instruction::Copy(2),
                Instruction::ZeroSet,
            ])),
            CopyOptimizer,
        );
        expr_helper("[->+>+<<<]", None, CopyOptimizer);

        expr_helper(
            "[->>+<+<]",
            Some(ExprKind::Instructions(vec![
                Instruction::Copy(2),
                Instruction::Copy(1),
                Instruction::ZeroSet,
            ])),
            CopyOptimizer,
        );
        expr_helper("[->>>+<+<]", None, CopyOptimizer);

        expr_helper(
            "[->>>+<+<+<]",
            Some(ExprKind::Instructions(vec![
                Instruction::Copy(3),
                Instruction::Copy(2),
                Instruction::Copy(1),
                Instruction::ZeroSet,
            ])),
            CopyOptimizer,
        );
        expr_helper("[->>>>+<+<+<]", None, CopyOptimizer);
    }
}

use if_chain::if_chain;
use log::info;

use super::Optimizer;
use crate::parse::ExprKind;
use crate::instruction::Instruction;

pub struct ZeroSetOptimizer;

impl Optimizer for ZeroSetOptimizer {
    /// [-]をSetValue(0)に変換する
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if_chain! {
            if let ExprKind::While(while_node) = expr;
            if let [ExprKind::Instructions(instructions)] = while_node.0.as_slice();
            if let [Instruction::Sub(1)]
                 | [Instruction::Add(1)]
                    = instructions.as_slice();
            then {
                info!("optimize!");
                Some(ExprKind::Instructions(vec![Instruction::ZeroSet]))
            }
            else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::ZeroSetOptimizer;
    use crate::{optimize::test::expr_helper, parse::ExprKind, instruction::Instruction};

    #[test]
    fn test_opt_zeroset() {
        expr_helper(
            "[-]",
            Some(ExprKind::Instructions(vec![Instruction::ZeroSet])),
            ZeroSetOptimizer,
        );
        expr_helper(
            "[+]",
            Some(ExprKind::Instructions(vec![Instruction::ZeroSet])),
            ZeroSetOptimizer,
        );
        expr_helper("[>]", None, ZeroSetOptimizer);
    }
}

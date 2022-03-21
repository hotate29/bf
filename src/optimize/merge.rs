use super::Optimizer;
use crate::{instruction::Instruction, parse::ExprKind};

pub struct MergeOptimizer;

impl Optimizer for MergeOptimizer {
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if let ExprKind::Instructions(instructions) = expr {
            let new_expr =
                instructions
                    .iter()
                    .fold(vec![], |mut new_expr: Vec<Instruction>, instruction| {
                        if let Some(top) = new_expr.pop() {
                            match top.merge(*instruction) {
                                Some(instruction) => {
                                    new_expr.push(instruction);
                                }
                                None => {
                                    new_expr.push(top);
                                    new_expr.push(*instruction);
                                }
                            }
                        } else {
                            new_expr.push(*instruction)
                        };
                        new_expr
                    });

            Some(ExprKind::Instructions(new_expr))
        } else {
            None
        }
    }
}

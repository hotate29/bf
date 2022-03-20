use std::cmp::Ordering;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MergeOptimizer;

impl Optimizer for MergeOptimizer {
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if let ExprKind::Instructions(instructions) = expr {
            use Instruction::*;

            let new_expr = instructions
                .iter()
                .fold(vec![], |mut new_expr, instruction| {
                    let top = new_expr.pop();

                    if let Some(top) = top {
                        match (top, *instruction) {
                            (Add(x), Add(y)) => new_expr.push(Add((x + y) % u8::MAX)),
                            (Sub(y), Add(x)) | (Add(x), Sub(y)) => {
                                let x = x as i32;
                                let y = y as i32;

                                let z = x - y;

                                match z.cmp(&0) {
                                    Ordering::Less => new_expr.push(Sub(z.abs() as u8)),
                                    Ordering::Greater => new_expr.push(Add(z as u8)),
                                    Ordering::Equal => (),
                                }
                            }
                            (Sub(x), Sub(y)) => new_expr.push(Sub((x + y) % u8::MAX)),
                            (PtrIncrement(x), PtrIncrement(y)) => {
                                new_expr.push(PtrIncrement(x + y))
                            }
                            (PtrDecrement(y), PtrIncrement(x))
                            | (PtrIncrement(x), PtrDecrement(y)) => {
                                let x = x as isize;
                                let y = y as isize;

                                let z = x - y;

                                match z.cmp(&0) {
                                    Ordering::Less => new_expr.push(PtrDecrement(z.abs() as usize)),
                                    Ordering::Greater => new_expr.push(PtrIncrement(z as usize)),
                                    Ordering::Equal => (),
                                }
                            }
                            (PtrDecrement(x), PtrDecrement(y)) => {
                                new_expr.push(PtrDecrement(x + y))
                            }
                            (prev, instruction) => {
                                new_expr.push(prev);
                                new_expr.push(instruction)
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

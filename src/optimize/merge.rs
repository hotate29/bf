use std::cmp::Ordering;

use crate::token::Instruction;

use super::{ExprKind, Optimizer};

pub struct MergeOptimizer;

impl Optimizer for MergeOptimizer {
    fn optimize_exprs(&self, node: &[ExprKind]) -> Option<(usize, Vec<ExprKind>)> {
        let mut new_node = vec![];

        for expr in node {
            if let ExprKind::Instructions(instructions) = expr {
                use Instruction::*;
                let mut new_expr = Vec::new();

                for instruction in instructions {
                    match (new_expr.pop(), *instruction) {
                        (Some(Add(x)), Add(y)) => new_expr.push(Add((x + y) % u8::MAX)),
                        (Some(Sub(y)), Add(x)) | (Some(Add(x)), Sub(y)) => {
                            let x = x as i32;
                            let y = y as i32;

                            let z = x - y;

                            match z.cmp(&0) {
                                Ordering::Less => new_expr.push(Sub(z.abs() as u8)),
                                Ordering::Greater => new_expr.push(Add(z as u8)),
                                Ordering::Equal => (),
                            }
                        }
                        (Some(Sub(x)), Sub(y)) => new_expr.push(Sub((x + y) % u8::MAX)),
                        (Some(PtrIncrement(x)), PtrIncrement(y)) => {
                            new_expr.push(PtrIncrement(x + y))
                        }
                        (Some(PtrDecrement(y)), PtrIncrement(x))
                        | (Some(PtrIncrement(x)), PtrDecrement(y)) => {
                            let x = x as isize;
                            let y = y as isize;

                            let z = x - y;

                            match z.cmp(&0) {
                                Ordering::Less => new_expr.push(PtrDecrement(z.abs() as usize)),
                                Ordering::Greater => new_expr.push(PtrIncrement(z as usize)),
                                Ordering::Equal => (),
                            }
                        }
                        (Some(PtrDecrement(x)), PtrDecrement(y)) => {
                            new_expr.push(PtrDecrement(x + y))
                        }

                        (prev, instruction) => {
                            if let Some(instruction) = prev {
                                new_expr.push(instruction)
                            };
                            new_expr.push(instruction)
                        }
                    }
                }
                new_node.push(ExprKind::Instructions(new_expr))
            } else {
                new_node.push(expr.clone())
            }
        }
        Some((new_node.len(), new_node))
    }
}

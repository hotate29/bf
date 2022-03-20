use std::cmp::Ordering;

use crate::token::Instruction;

use super::Optimizer;
use crate::parse::ExprKind;

fn merge_instruction(a: Instruction, b: Instruction) -> Option<Instruction> {
    use Instruction::*;

    match (a, b) {
        (Add(x), Add(y)) => Some(Add((x + y) % u8::MAX)),
        (Sub(y), Add(x)) | (Add(x), Sub(y)) => {
            let x = x as i32;
            let y = y as i32;

            let z = x - y;

            match z.cmp(&0) {
                Ordering::Less => Some(Sub(z.abs() as u8)),
                Ordering::Greater => Some(Add(z as u8)),
                Ordering::Equal => Some(Add(0)),
            }
        }
        (Sub(x), Sub(y)) => Some(Sub((x + y) % u8::MAX)),
        (PtrIncrement(x), PtrIncrement(y)) => Some(PtrIncrement(x + y)),
        (PtrDecrement(y), PtrIncrement(x)) | (PtrIncrement(x), PtrDecrement(y)) => {
            let x = x as isize;
            let y = y as isize;

            let z = x - y;

            match z.cmp(&0) {
                Ordering::Less => Some(PtrDecrement(z.abs() as usize)),
                Ordering::Greater => Some(PtrIncrement(z as usize)),
                Ordering::Equal => Some(Add(0)),
            }
        }
        (PtrDecrement(x), PtrDecrement(y)) => Some(PtrDecrement(x + y)),
        (_, _) => None,
    }
}

pub struct MergeOptimizer;

impl Optimizer for MergeOptimizer {
    fn optimize_expr(&self, expr: &ExprKind) -> Option<ExprKind> {
        if let ExprKind::Instructions(instructions) = expr {
            let new_expr = instructions
                .iter()
                .fold(vec![], |mut new_expr, instruction| {
                    if let Some(top) = new_expr.pop() {
                        match merge_instruction(top, *instruction) {
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

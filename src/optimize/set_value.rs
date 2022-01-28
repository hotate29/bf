use if_chain::if_chain;
use log::info;

use crate::{
    optimize::{ExprKind, Node},
    token::Instruction,
};

use super::Optimizer;

pub struct SetValueOptimizer;

impl Optimizer for SetValueOptimizer {
    /// +n[-x>+m<x]>をSetToValue(x, n*m)に変換する
    fn optimize_node(&self, node: &super::Node) -> Option<super::Node> {
        for i in 0..node.0.len() {
            let front_kinds = &node.0[0..i];

            if_chain! {
                if let &[ExprKind::Instructions(instructions), ExprKind::While(while_node), ExprKind::Instructions(s), last_kinds @ ..] =
                &node.0.as_slice();
                if let [front_instructions @ .., Instruction::Add(n)] = instructions.as_slice();
                if let [ExprKind::Instructions(while_instructions)] = while_node.0.as_slice();
                if let [Instruction::Sub(1), Instruction::PtrIncrement(ptrinc_count), Instruction::Add(x), Instruction::PtrDecrement(ptrdec_count)] = while_instructions.as_slice();
                if ptrinc_count == ptrdec_count;
                then {
                    info!("optimize!");
                    eprintln!("a {} {} {}", n * x, n, x);
                    let x = n * x;
                    let x = (x % u8::MAX as usize) as u8;

                    let mut node_kinds = front_kinds.to_vec();

                    let mut instructions = front_instructions.to_vec();
                    instructions.push(Instruction::SetValue(*ptrinc_count, x));
                    instructions.extend_from_slice(s);
                    node_kinds.push(ExprKind::Instructions(instructions));

                    node_kinds.extend_from_slice(last_kinds);

                    let optimized_node = Node(node_kinds);
                    return Some(optimized_node);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{
        optimize::{set_value::SetValueOptimizer, Optimizer},
        token::Instruction,
    };

    use super::{ExprKind, Node};

    #[test]
    fn test_opt_set_value() {
        fn helper(source: &str, assert_node: Option<Node>) {
            let root_node = Node::from_source(source).unwrap();

            let optimized_node = SetValueOptimizer.optimize_node(&root_node);
            assert_eq!(optimized_node, assert_node);
        }
        helper(
            ">++++++++++[->++++++++++<]>",
            Some(Node(vec![ExprKind::Instructions(vec![
                Instruction::PtrIncrement(1),
                Instruction::SetValue(1, 100),
                Instruction::PtrIncrement(1),
            ])])),
        );
        helper(
            ">++[->+++<]>",
            Some(Node(vec![ExprKind::Instructions(vec![
                Instruction::PtrIncrement(1),
                Instruction::SetValue(1, 6),
                Instruction::PtrIncrement(1),
            ])])),
        );

        helper("++++++++++[->+++++++++<<]>", None);
    }
}

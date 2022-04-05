use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    instruction::Instruction::{self, *},
    parse::{Node, Nodes},
};

pub struct AddOptimizer;

impl Optimizer for AddOptimizer {
    fn optimize_node(&self, node: &Node) -> Option<Nodes> {
        if let Node::Loop(loop_nodes) = node {
            if loop_nodes.len() == 4 {
                let mut nodes_iter = loop_nodes.iter();

                if let [Node::Instruction(Sub(1)), Node::Instruction(PtrIncrement(ptr_increment)), Node::Instruction(Add(1)), Node::Instruction(PtrDecrement(ptr_decrement))]
                | [Node::Instruction(PtrIncrement(ptr_increment)), Node::Instruction(Add(1)), Node::Instruction(PtrDecrement(ptr_decrement)), Node::Instruction(Sub(1))] = {
                    [
                        nodes_iter.next()?,
                        nodes_iter.next()?,
                        nodes_iter.next()?,
                        nodes_iter.next()?,
                    ]
                } {
                    if ptr_increment == ptr_decrement {
                        return Some(
                            [AddTo(*ptr_increment as isize).into(), ZeroSet.into()].into(),
                        );
                    }
                }

                let mut nodes_iter = loop_nodes.iter();

                if let [Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(ptr_increment)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(ptr_decrement))]
                | [Node::Instruction(PtrDecrement(ptr_increment)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(ptr_decrement)), Node::Instruction(Sub(1))] = [
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                ] {
                    if ptr_decrement == ptr_increment {
                        return Some(
                            [AddTo(-(*ptr_decrement as isize)).into(), ZeroSet.into()].into(),
                        );
                    }
                }
            }
        }
        None
    }
}

pub struct SubOptimizer;

impl Optimizer for SubOptimizer {
    fn optimize_node(&self, node: &Node) -> Option<Nodes> {
        if let Node::Loop(loop_nodes) = node {
            if loop_nodes.len() == 4 {
                let mut nodes_iter = loop_nodes.iter();

                if let [Node::Instruction(Sub(1)), Node::Instruction(PtrIncrement(ptr_increment)), Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(ptr_decrement))]
                | [Node::Instruction(PtrIncrement(ptr_increment)), Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(ptr_decrement)), Node::Instruction(Sub(1))] = {
                    [
                        nodes_iter.next()?,
                        nodes_iter.next()?,
                        nodes_iter.next()?,
                        nodes_iter.next()?,
                    ]
                } {
                    if ptr_increment == ptr_decrement {
                        return Some(
                            [SubTo(*ptr_increment as isize).into(), ZeroSet.into()].into(),
                        );
                    }
                }

                let mut nodes_iter = loop_nodes.iter();

                if let [Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(ptr_increment)), Node::Instruction(Sub(1)), Node::Instruction(PtrIncrement(ptr_decrement))]
                | [Node::Instruction(PtrDecrement(ptr_increment)), Node::Instruction(Sub(1)), Node::Instruction(PtrIncrement(ptr_decrement)), Node::Instruction(Sub(1))] = [
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                ] {
                    if ptr_decrement == ptr_increment {
                        return Some(
                            [SubTo(-(*ptr_decrement as isize)).into(), ZeroSet.into()].into(),
                        );
                    }
                }
            }
        }
        None
    }
}

fn merge_instruction(nodes: Nodes) -> Nodes {
    let mut new_nodes = Nodes::new();

    for node in nodes {
        new_nodes.push_back(node);

        while let Some(merged_inst) = new_nodes
            .iter()
            .nth_back(1)
            .zip(new_nodes.back())
            .and_then(|(back2, back)| back2.as_instruction().zip(back.as_instruction()))
            .and_then(|(back2, back)| back2.merge(back))
        {
            new_nodes.pop_back().unwrap();
            new_nodes.pop_back().unwrap();
            if !merged_inst.is_no_action() {
                new_nodes.push_back(merged_inst.into())
            }
        }
    }

    new_nodes
}

#[derive(Debug)]
struct Instructions {
    inner: Vec<Instruction>,
}
impl Instructions {
    fn from_ins(ins: Instruction) -> Self {
        Self { inner: vec![ins] }
    }
    fn push(&mut self, ins: Instruction) {
        self.inner.push(ins);

        while let Some(merged_inst) = self
            .inner
            .iter()
            .nth_back(1)
            .zip(self.inner.last())
            .and_then(|(back2, back)| back2.merge(*back))
        {
            self.inner.pop().unwrap();
            self.inner.pop().unwrap();
            if !merged_inst.is_no_action() {
                self.inner.push(merged_inst)
            }
        }
    }
    fn inner(&self) -> &Vec<Instruction> {
        &self.inner
    }
}

pub fn offset_opt(nodes: &Nodes) -> Nodes {
    #[derive(Debug)]
    enum Nod {
        Loop(Nodes),
        Instructions(Nodes),
    }

    #[derive(Debug, Default)]
    struct State {
        pointer_offset: isize,
        offset_map: BTreeMap<isize, Instructions>,
        output_order: Vec<isize>,
    }
    impl State {
        fn push_instruction(&mut self, ins: Instruction) {
            if matches!(ins, Output(_)) && self.output_order.last() != Some(&self.pointer_offset) {
                self.output_order.push(self.pointer_offset);
            }
            match ins {
                PtrIncrement(inc) => self.pointer_offset += inc as isize,
                PtrDecrement(dec) => self.pointer_offset -= dec as isize,
                ins @ (Add(_) | Sub(_) | Output(_) | ZeroSet) => {
                    self.offset_map
                        .entry(self.pointer_offset)
                        .and_modify(|instructions| instructions.push(ins))
                        .or_insert_with(|| Instructions::from_ins(ins));
                }
                Input(_) => todo!(),
                _ => panic!(),
            };
        }
        fn into_nodes(mut self) -> Nodes {
            let mut out_nodes = Nodes::new();

            // 出力の順番をちゃんと
            for order in self.output_order {
                let instructions = self.offset_map.remove(&order).unwrap();
                for instruction in instructions.inner() {
                    let instruction = match instruction {
                        Add(value) => AddOffset(order, *value),
                        Sub(value) => SubOffset(order, *value),
                        Output(repeat) => OutputOffset(*repeat, order),
                        Input(_) => todo!(),
                        ZeroSet => ZeroSetOffset(order),
                        _ => panic!(),
                    };
                    out_nodes.push_back(instruction.into());
                }
            }

            for (offset, instructions) in self.offset_map {
                for instruction in instructions.inner {
                    let instruction = match instruction {
                        Add(value) => AddOffset(offset, value),
                        Sub(value) => SubOffset(offset, value),
                        // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                        Output(repeat) => OutputOffset(repeat, offset),
                        Input(_) => todo!(),
                        ZeroSet => ZeroSetOffset(offset),
                        _ => panic!(),
                    };
                    out_nodes.push_back(instruction.into());
                }
            }
            match self.pointer_offset.cmp(&0) {
                Ordering::Less => {
                    out_nodes.push_back(PtrDecrement(self.pointer_offset.abs() as usize).into())
                }
                Ordering::Greater => {
                    out_nodes.push_back(PtrIncrement(self.pointer_offset as usize).into())
                }
                Ordering::Equal => (),
            }
            out_nodes
        }
    }

    fn inner(nodes: &Nodes, is_loop: bool) -> Nod {
        let mut new_nodes = Nodes::new();

        let mut state = State::default();

        let mut has_loop = false;
        let mut has_output = false;

        for node in nodes {
            match node {
                Node::Loop(loop_nodes) => {
                    has_loop = true;

                    let mut instructions = state.into_nodes();

                    new_nodes.append(&mut instructions);

                    match inner(loop_nodes, true) {
                        Nod::Loop(loop_nodes) => new_nodes.push_back(Node::Loop(loop_nodes)),
                        Nod::Instructions(mut instructions) => new_nodes.append(&mut instructions),
                    }

                    state = State::default();
                }
                Node::Instruction(instruction) => {
                    has_output |= matches!(instruction, Output(_));

                    state.push_instruction(*instruction);
                }
            }
        }

        if state.pointer_offset == 0
            && !has_loop
            && is_loop
            // [->>>.<<<]を弾く
            && !has_output
            && state.offset_map
                .get(&0)
                .filter(|ins| ins.inner() == &[Sub(1)])
                .is_some()
        {
            // 最適化をするぞ！バリバリ！
            // 注: ここで出力するのは命令列で、ループではない。これの扱いをどうする？

            for (offset, instructions) in state.offset_map {
                for instruction in instructions.inner {
                    let instruction = match instruction {
                        Add(1) => AddTo(offset),
                        Add(value) => MulAdd(offset, value),
                        Sub(1) if offset == 0 => continue,
                        Sub(1) => SubTo(offset),
                        Output(repeat) => OutputOffset(repeat, offset),
                        // Input(_) => todo!(),
                        // ZeroSet => ZeroSetOffset(offset),
                        _ => panic!(),
                    };
                    new_nodes.push_back(instruction.into());
                }
            }
            new_nodes.push_back(ZeroSet.into());
            Nod::Instructions(new_nodes)
        } else {
            let mut instructions = state.into_nodes();
            new_nodes.append(&mut instructions);

            Nod::Loop(new_nodes)
        }
    }
    match inner(nodes, false) {
        Nod::Loop(nodes) | Nod::Instructions(nodes) => nodes,
    }
}

pub trait Optimizer {
    fn optimize_node(&self, node: &Node) -> Option<Nodes>;
}

pub struct ZeroSetOptimizer;

impl Optimizer for ZeroSetOptimizer {
    fn optimize_node(&self, node: &Node) -> Option<Nodes> {
        if let Node::Loop(loop_nodes) = node {
            if loop_nodes.len() == 1 {
                if let Node::Instruction(Add(1) | Sub(1)) = loop_nodes.front()? {
                    let nodes = [ZeroSet.into()].into();
                    return Some(nodes);
                }
                if let Node::Instruction(AddOffset(offset, 1) | SubOffset(offset, 1)) =
                    loop_nodes.front()?
                {
                    let nodes = [ZeroSetOffset(*offset).into()].into();
                    return Some(nodes);
                }
            }
        }
        None
    }
}

pub fn all_optimizers() -> Vec<Box<dyn Optimizer>> {
    vec![
        Box::new(ZeroSetOptimizer),
        Box::new(AddOptimizer),
        Box::new(SubOptimizer),
    ]
}

pub fn optimize(nodes: Nodes, optimizers: &[Box<dyn Optimizer>]) -> Nodes {
    // eprintln!("{nodes:?}");
    fn inner(nodes: Nodes, optimizers: &[Box<dyn Optimizer>]) -> Nodes {
        let nodes = merge_instruction(nodes);
        let mut new_nodes = Nodes::new();

        for node in nodes {
            let node = if let Node::Loop(loop_nodes) = node {
                let loop_nodes = merge_instruction(loop_nodes);
                Node::Loop(inner(loop_nodes, optimizers))
            } else {
                node
            };

            let optimized_node = optimizers
                .iter()
                .find_map(|optimizer| optimizer.optimize_node(&node));

            if let Some(mut optimized_node) = optimized_node {
                new_nodes.append(&mut optimized_node);
            } else {
                new_nodes.push_back(node);
            }
        }
        new_nodes
    }

    inner(nodes, optimizers)
}

#[cfg(test)]
mod test {
    use super::{
        merge_instruction, offset_opt, AddOptimizer, Optimizer, SubOptimizer, ZeroSetOptimizer,
    };
    use crate::{
        instruction::Instruction::*,
        parse::{tokenize, Node, Nodes},
    };

    use rstest::rstest;

    #[test]
    fn test_merge_instruction() {
        let nodes = [
            Add(1).into(),
            Sub(1).into(),
            PtrIncrement(1).into(),
            PtrDecrement(1).into(),
            Add(1).into(),
        ]
        .into();
        assert_eq!(merge_instruction(nodes), [Add(1).into()].into());
    }
    fn optimize_node(code: &str, optimizer: impl Optimizer) -> Option<Nodes> {
        let tokens = tokenize(code);
        let mut nodes = Node::from_tokens(tokens).unwrap();
        if nodes.len() == 1 {
            if let Node::Loop(loop_nodes) = nodes.pop_front().unwrap() {
                let merged_loop_node = merge_instruction(loop_nodes);
                let loop_node = Node::Loop(merged_loop_node);

                optimizer.optimize_node(&loop_node)
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    fn assert_node(optimizer: impl Optimizer, code: &str, node: Option<Nodes>) {
        let optimized_node = optimize_node(code, optimizer);
        assert_eq!(node, optimized_node);
    }

    #[rstest(input, expected,
        case("[-]", Some([ZeroSet.into()].into())),
        case("[+]", Some([ZeroSet.into()].into())),
        case("[++]", None),
    )]
    fn test_zeroset_opt(input: &str, expected: Option<Nodes>) {
        assert_node(ZeroSetOptimizer, input, expected);
    }

    #[rstest(input, expected,
        case("[->>>+<<<]", Some([AddTo(3).into(), ZeroSet.into()].into())),
        case("[>>>+<<<-]", Some([AddTo(3).into(), ZeroSet.into()].into())),
        case("[-<<<+>>>]", Some([AddTo(-3).into(), ZeroSet.into()].into())),
        case("[<<<+>>>-]", Some([AddTo(-3).into(), ZeroSet.into()].into())),
        case("[-<<<++>>>]", None),
    )]
    fn test_add_opt(input: &str, expected: Option<Nodes>) {
        assert_node(AddOptimizer, input, expected);
    }

    #[rstest(input, expected,
        case("[->>>-<<<]", Some([SubTo(3).into(), ZeroSet.into()].into())),
        case("[>>>-<<<-]", Some([SubTo(3).into(), ZeroSet.into()].into())),
        case("[-<<<->>>]", Some([SubTo(-3).into(), ZeroSet.into()].into())),
        case("[<<<->>>-]", Some([SubTo(-3).into(), ZeroSet.into()].into())),
        case("[-<<<-->>>]", None),
    )]
    fn test_sub_opt(input: &str, expected: Option<Nodes>) {
        assert_node(SubOptimizer, input, expected);
    }

    #[rstest(input, expected,
        case("", [].into()),
        case("+++", [AddOffset(0, 3).into()].into()),
        case("+++---", [].into()),
        case(">+++<-", [SubOffset(0, 1).into(), AddOffset(1, 3).into()].into()),
        case(">+++", [AddOffset(1, 3).into(), PtrIncrement(1).into()].into()),
        case("[[[]]]", [Node::Loop([Node::Loop([Node::Loop([].into())].into())].into())].into()),
        case("->+<", [SubOffset(0, 1).into(), AddOffset(1, 1).into()].into()),
        case("[->>>-<<<]", [SubTo(3).into(), ZeroSet.into()].into()),
        case("+++>-<[->>>-<<<]", [AddOffset(0, 3).into(),SubOffset(1, 1).into(), SubTo(3).into(), ZeroSet.into()].into()),
        case("[>>>-<<<-]", [SubTo(3).into(), ZeroSet.into()].into()),
        case("[>>>->+<<<<-]", [SubTo(3).into(), AddTo(4).into(), ZeroSet.into()].into()),
        case("+++[>>>[-][[->+<]]<<<]", [AddOffset(0, 3).into(), Node::Loop([PtrIncrement(3).into(), ZeroSet.into(), Node::Loop([AddTo(1).into(), ZeroSet.into()].into()), PtrDecrement(3).into()].into())].into()),
        case("[->>>.<<<]", [Node::Loop([SubOffset(0, 1).into(), OutputOffset(1,3).into()].into())].into()),
        case("[->+>+>++>+++<<<<]", [AddTo(1).into(), AddTo(2).into(), MulAdd(3, 2).into(), MulAdd(4, 3).into(), ZeroSet.into()].into()),
        // TODO: MulSubを実装する
        #[should_panic]
        case("[-<<<-->>>]", [Node::Loop([SubOffset(0, 1).into(), SubOffset(-3, 2).into()].into())].into()),
    )]
    fn test_offset_opt(input: &str, expected: Nodes) {
        let tokens = tokenize(input);
        let nodes = Node::from_tokens(tokens).unwrap();

        let optimized_node = offset_opt(&nodes);
        assert_eq!(optimized_node, expected)
    }
}

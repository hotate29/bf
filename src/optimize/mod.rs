use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    instruction::Instruction::{self, *},
    parse::{Node, Nodes},
};

fn loop_opt(node: &Node) -> Option<Nodes> {
    let mut new_nodes = Nodes::new();
    if let Node::Loop(loop_nodes) = node {
        let mut f = false;

        for node in loop_nodes {
            let ins = node.as_instruction()?;
            match ins {
                AddOffset(0, 1) | SubOffset(0, 1) => {
                    f = true;
                    new_nodes.push_back(Node::Instruction(ZeroSet))
                }
                AddOffset(offset, 1) => {
                    let ins = if offset < 0 {
                        AddToRev(-offset as usize)
                    } else {
                        AddTo(offset as usize)
                    };

                    new_nodes.push_front(Node::Instruction(ins));
                }
                AddOffset(offset, value) => {
                    let ins = if offset < 0 {
                        MulAddRev(-offset as usize, value)
                    } else {
                        MulAdd(offset as usize, value)
                    };

                    new_nodes.push_front(Node::Instruction(ins));
                }
                SubOffset(offset, 1) => {
                    let ins = if offset < 0 {
                        SubToRev(-offset as usize)
                    } else {
                        SubTo(offset as usize)
                    };

                    new_nodes.push_front(Node::Instruction(ins));
                }
                PtrIncrement(_) | PtrDecrement(_) => return None,
                _ => return None,
            }
        }
        if f {
            return Some(new_nodes);
        } else {
            return None;
        }
    }
    None
}

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
                            [
                                Node::Instruction(AddTo(*ptr_increment)),
                                Node::Instruction(ZeroSet),
                            ]
                            .into(),
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
                            [
                                Node::Instruction(AddToRev(*ptr_decrement)),
                                Node::Instruction(ZeroSet),
                            ]
                            .into(),
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
                            [
                                Node::Instruction(SubTo(*ptr_increment)),
                                Node::Instruction(ZeroSet),
                            ]
                            .into(),
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
                            [
                                Node::Instruction(SubToRev(*ptr_decrement)),
                                Node::Instruction(ZeroSet),
                            ]
                            .into(),
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
                new_nodes.push_back(Node::Instruction(merged_inst))
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
    enum Nod {
        Loop(Nodes),
        Instructions(Nodes),
    }

    fn inner(nodes: &Nodes) -> Nod {
        let mut new_nodes = Nodes::new();

        let mut pointer_offset = 0;
        let mut offset_map: BTreeMap<isize, Instructions> = BTreeMap::new();

        let mut has_loop = false;

        for node in nodes {
            match node {
                Node::Loop(loop_nodes) => {
                    has_loop = true;

                    for (offset, instructions) in offset_map {
                        for instruction in instructions.inner {
                            let instruction = match instruction {
                                Add(value) => AddOffset(offset, value),
                                Sub(value) => SubOffset(offset, value),
                                Output(_) => OutputOffset(offset),
                                Input(_) => todo!(),
                                ZeroSet => ZeroSetOffset(offset),
                                _ => panic!(),
                            };
                            new_nodes.push_back(Node::Instruction(instruction));
                        }
                    }

                    match pointer_offset.cmp(&0) {
                        Ordering::Less => new_nodes
                            .push_back(Node::Instruction(PtrDecrement(-pointer_offset as usize))),
                        Ordering::Greater => new_nodes
                            .push_back(Node::Instruction(PtrIncrement(pointer_offset as usize))),
                        Ordering::Equal => (),
                    }

                    match inner(loop_nodes) {
                        Nod::Loop(loop_nodes) => new_nodes.push_back(Node::Loop(loop_nodes)),
                        Nod::Instructions(mut instructions) => new_nodes.append(&mut instructions),
                    }

                    offset_map = BTreeMap::new();
                    pointer_offset = 0;
                }
                Node::Instruction(instruction) => {
                    match instruction {
                        PtrIncrement(inc) => pointer_offset += *inc as isize,
                        PtrDecrement(dec) => pointer_offset -= *dec as isize,
                        ins @ (Add(_) | Sub(_) | Output(_) | ZeroSet) => {
                            offset_map
                                .entry(pointer_offset)
                                .and_modify(|instructions| instructions.push(*ins))
                                .or_insert_with(|| Instructions::from_ins(*ins));
                        }
                        Input(_) => todo!(),
                        _ => panic!(),
                    };
                }
            }
        }

        if pointer_offset == 0
            && !has_loop
            && offset_map
                .get(&0)
                .filter(|ins| ins.inner() == &[Sub(1)])
                .is_some()
        {
            // 最適化をするぞ！バリバリ！
            // 注: ここで出力するのは命令列で、ループではない。これの扱いをどうする？
            for (offset, instructions) in (offset_map) {
                for instruction in instructions.inner {
                    let instruction = match instruction {
                        Add(1) if offset > 0 => AddTo(offset as usize),
                        Add(1) if offset < 0 => AddToRev(-offset as usize),
                        Add(value) if offset > 0 => MulAdd(offset as usize, value),
                        Add(value) if offset < 0 => MulAddRev(-offset as usize, value),
                        Sub(1) if offset > 0 => SubTo(offset as usize),
                        Sub(1) if offset < 0 => SubToRev(-offset as usize),
                        Sub(1) if offset == 0 => continue,
                        Output(_) => OutputOffset(offset),
                        // Input(_) => todo!(),
                        ZeroSet => ZeroSetOffset(offset),
                        _ => panic!(),
                    };
                    new_nodes.push_back(Node::Instruction(instruction));
                }
            }
            new_nodes.push_back(Node::Instruction(ZeroSet));
            Nod::Instructions(new_nodes)
        } else {
            for (offset, instructions) in offset_map {
                for instruction in instructions.inner {
                    let instruction = match instruction {
                        Add(value) => AddOffset(offset, value),
                        Sub(value) => SubOffset(offset, value),
                        Output(_) => OutputOffset(offset),
                        Input(_) => todo!(),
                        ZeroSet => ZeroSetOffset(offset),
                        _ => panic!(),
                    };
                    new_nodes.push_back(Node::Instruction(instruction));
                }
            }
            match pointer_offset.cmp(&0) {
                Ordering::Less => {
                    new_nodes.push_back(Node::Instruction(PtrDecrement(-pointer_offset as usize)))
                }
                Ordering::Greater => {
                    new_nodes.push_back(Node::Instruction(PtrIncrement(pointer_offset as usize)))
                }
                Ordering::Equal => (),
            }
            Nod::Loop(new_nodes)
        }
    }
    match inner(nodes) {
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
                    let nodes = Nodes::from([Node::Instruction(ZeroSet)]);
                    return Some(nodes);
                }
                if let Node::Instruction(AddOffset(offset, 1) | SubOffset(offset, 1)) =
                    loop_nodes.front()?
                {
                    let nodes = Nodes::from([Node::Instruction(ZeroSetOffset(*offset))]);
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
    use super::{merge_instruction, AddOptimizer, Optimizer, SubOptimizer, ZeroSetOptimizer};
    use crate::{
        instruction::Instruction::*,
        parse::{tokenize, Node, Nodes},
    };

    use rstest::rstest;

    #[test]
    fn test_merge_instruction() {
        let nodes = [
            Node::Instruction(Add(1)),
            Node::Instruction(Sub(1)),
            Node::Instruction(PtrIncrement(1)),
            Node::Instruction(PtrDecrement(1)),
            Node::Instruction(Add(1)),
        ]
        .into();
        assert_eq!(merge_instruction(nodes), [Node::Instruction(Add(1))].into());
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
        case("[-]", Some([Node::Instruction(ZeroSet)].into())),
        case("[+]", Some([Node::Instruction(ZeroSet)].into())),
        case("[++]", None),
    )]
    fn test_zeroset_opt(input: &str, expected: Option<Nodes>) {
        assert_node(ZeroSetOptimizer, input, expected);
    }

    #[rstest(input, expected,
        case("[->>>+<<<]", Some([Node::Instruction(AddTo(3)), Node::Instruction(ZeroSet)].into())),
        case("[>>>+<<<-]", Some([Node::Instruction(AddTo(3)), Node::Instruction(ZeroSet)].into())),
        case("[-<<<+>>>]", Some([Node::Instruction(AddToRev(3)), Node::Instruction(ZeroSet)].into())),
        case("[<<<+>>>-]", Some([Node::Instruction(AddToRev(3)), Node::Instruction(ZeroSet)].into())),
        case("[-<<<++>>>]", None),
    )]
    fn test_add_opt(input: &str, expected: Option<Nodes>) {
        assert_node(AddOptimizer, input, expected);
    }

    #[rstest(input, expected,
        case("[->>>-<<<]", Some([Node::Instruction(SubTo(3)), Node::Instruction(ZeroSet)].into())),
        case("[>>>-<<<-]", Some([Node::Instruction(SubTo(3)), Node::Instruction(ZeroSet)].into())),
        case("[-<<<->>>]", Some([Node::Instruction(SubToRev(3)), Node::Instruction(ZeroSet)].into())),
        case("[<<<->>>-]", Some([Node::Instruction(SubToRev(3)), Node::Instruction(ZeroSet)].into())),
        case("[-<<<-->>>]", None),
    )]
    fn test_sub_opt(input: &str, expected: Option<Nodes>) {
        assert_node(SubOptimizer, input, expected);
    }
}

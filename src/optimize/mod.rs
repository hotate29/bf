use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    instruction::Instruction::{self, *},
    parse::{Node, Nodes},
};

fn zeroset_opt(node: &Node) -> Option<Nodes> {
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

fn add_opt(node: &Node) -> Option<Nodes> {
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

fn sub_opt(node: &Node) -> Option<Nodes> {
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

fn copy_opt(node: &Node) -> Option<Nodes> {
    if let Node::Loop(loop_nodes) = node {
        if loop_nodes.len() == 6 {
            let mut nodes_iter = loop_nodes.iter();
            if let [Node::Instruction(Sub(1)), Node::Instruction(PtrIncrement(x)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(y)), Node::Instruction(Add(1)), Node::Instruction(PtrDecrement(z))] = [
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
            ] {
                if x + y == *z {
                    return Some(
                        [
                            Node::Instruction(Copy(*x)),
                            Node::Instruction(Copy(x + y)),
                            Node::Instruction(ZeroSet),
                        ]
                        .into(),
                    );
                }
            }
        }
    }
    if let Node::Loop(loop_nodes) = node {
        if loop_nodes.len() == 6 {
            let mut nodes_iter = loop_nodes.iter();
            if let [Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(x)), Node::Instruction(Add(1)), Node::Instruction(PtrDecrement(y)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(z))] = [
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
            ] {
                if x + y == *z {
                    return Some(
                        [
                            Node::Instruction(CopyRev(*x)),
                            Node::Instruction(CopyRev(x + y)),
                            Node::Instruction(ZeroSet),
                        ]
                        .into(),
                    );
                }
            }
        }
    }
    if let Node::Loop(loop_nodes) = node {
        if loop_nodes.len() == 6 {
            let mut nodes_iter = loop_nodes.iter();
            if let [Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(x)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(y)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(z))] = [
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
            ] {
                if *x == y + z {
                    return Some(
                        [
                            Node::Instruction(CopyRev(*x)),
                            Node::Instruction(CopyRev(x - y)),
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

fn sub_add_opt(node: &Node) -> Option<Nodes> {
    if let Node::Loop(loop_nodes) = node {
        if loop_nodes.len() == 7 {
            let mut nodes_iter = loop_nodes.iter();

            if let [Node::Instruction(PtrDecrement(dec)), Node::Instruction(Sub(1)), Node::Instruction(PtrIncrement(inc)), Node::Instruction(Sub(1)), Node::Instruction(PtrDecrement(dec2)), Node::Instruction(Add(1)), Node::Instruction(PtrIncrement(inc2))] = {
                [
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                ]
            } {
                if dec == inc && dec2 == inc2 {
                    eprintln!("match! {:?}", loop_nodes);
                    return Some(
                        [
                            Node::Instruction(SubToRev(*dec)),
                            Node::Instruction(AddToRev(*dec2)),
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

fn merge_instruction(nodes: Nodes) -> Nodes {
    let mut new_nodes = Nodes::new();

    for node in nodes {
        new_nodes.push_back(node);

        while let Some(merged_inst) =
            new_nodes
                .iter()
                .nth_back(1)
                .zip(new_nodes.back())
                .and_then(|(back2, back)| {
                    if let (Node::Instruction(back2), Node::Instruction(back)) = (back2, back) {
                        back2.merge(*back)
                    } else {
                        None
                    }
                })
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
    fn inner(nodes: &Nodes) -> Nodes {
        let mut new_nodes = Nodes::new();

        let mut pointer_offset = 0isize;
        let mut offset_instructions: BTreeMap<isize, Instructions> = BTreeMap::new();
        for node in nodes {
            match node {
                Node::Loop(loop_nodes) => {
                    // offset_instructionsに溜めていたものを吐き出す。
                    for (offset, instruction) in offset_instructions.iter() {
                        let mut instructions = instruction
                            .inner()
                            .iter()
                            .copied()
                            .map(|ins| match ins {
                                Add(value) => AddOffset(*offset, value),
                                Sub(value) => SubOffset(*offset, value),
                                Output(_) => OutputOffset(*offset),
                                Input(_) => todo!(),
                                _ => unreachable!(),
                            })
                            .map(Node::Instruction)
                            .collect();
                        new_nodes.append(&mut instructions)
                    }
                    match pointer_offset.cmp(&0) {
                        Ordering::Less => new_nodes
                            .push_back(Node::Instruction(PtrDecrement(-pointer_offset as usize))),
                        Ordering::Equal => (),
                        Ordering::Greater => new_nodes
                            .push_back(Node::Instruction(PtrIncrement(pointer_offset as usize))),
                    };

                    let loop_nodes = inner(loop_nodes);
                    new_nodes.push_back(Node::Loop(loop_nodes));

                    pointer_offset = 0;
                    offset_instructions.clear()
                }
                Node::Instruction(ins) => match ins {
                    PtrIncrement(inc) => pointer_offset += *inc as isize,
                    PtrDecrement(dec) => pointer_offset -= *dec as isize,
                    Add(_) | Sub(_) | Output(_) | Input(_) => {
                        offset_instructions
                            .entry(pointer_offset)
                            .and_modify(|i: &mut Instructions| i.push(*ins))
                            .or_insert_with(|| Instructions::from_ins(*ins));
                    }
                    _ => panic!("{ins:?}"),
                },
            };
        }

        // let include_loop =
        //     new_nodes.iter().any(|node| matches!(node, Node::Loop(_))) && !new_nodes.is_empty();
        // let optimizable = !include_loop
        //     && pointer_offset == 0
        //     && offset_instructions
        //         .get(&0)
        //         .filter(|ins| ins.inner() == &[Sub(1)])
        //         .is_some();

        // let mut instructions = instruction
        //     .inner()
        //     .iter()
        //     .copied()
        //     .filter_map(|ins| match ins {
        //         Add(1) if offset < &0 => Some(AddToRev(-offset as usize)),
        //         Add(1) if offset > &0 => Some(AddTo(*offset as usize)),
        //         Add(value) if offset > &0 => Some(MulAdd(*offset as usize, value)),
        //         Add(value) if offset < &0 => Some(MulAddRev(-offset as usize, value)),
        //         Add(_) if offset == &0 => unreachable!(),
        //         Sub(1) if offset == &0 => None, // これは後で
        //         Sub(1) if offset < &0 => Some(SubToRev(-offset as usize)),
        //         Sub(1) if offset > &0 => Some(SubTo(*offset as usize)),
        //         Output(_) => Some(OutputOffset(*offset)),
        //         Input(_) => todo!(),
        //         _ => unreachable!(),
        //     })
        //     .map(Nod::Instruction)
        //     .collect();
        // new_nodes.append(&mut instructions);
        // new_nodes.push_back(Nod::Instruction(ZeroSet));
        // }

        for (offset, instruction) in offset_instructions.iter() {
            let mut instructions = instruction
                .inner()
                .iter()
                .copied()
                .map(|ins| match ins {
                    Add(value) => AddOffset(*offset, value),
                    Sub(value) => SubOffset(*offset, value),
                    Output(_) => OutputOffset(*offset),
                    Input(_) => todo!(),
                    _ => unreachable!(),
                })
                .map(Node::Instruction)
                .collect();
            new_nodes.append(&mut instructions)
        }
        match pointer_offset.cmp(&0) {
            Ordering::Less => {
                new_nodes.push_back(Node::Instruction(PtrDecrement(-pointer_offset as usize)))
            }
            Ordering::Equal => (),
            Ordering::Greater => {
                new_nodes.push_back(Node::Instruction(PtrIncrement(pointer_offset as usize)))
            }
        };

        new_nodes
    }
    // eprintln!("{pointer_offset}, {instructions:?}");
    inner(nodes)
    // unimplemented!()
}

pub fn optimize(nodes: Nodes) -> Nodes {
    // eprintln!("{nodes:?}");
    fn inner(nodes: Nodes) -> Nodes {
        let nodes = merge_instruction(nodes);
        let mut new_nodes = Nodes::new();

        for node in nodes {
            let node = if let Node::Loop(loop_nodes) = node {
                Node::Loop(merge_instruction(loop_nodes))
            } else {
                node
            };

            // if let Some(mut optimized_nodes) = loop_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // }
            if let Some(mut optimized_nodes) = zeroset_opt(&node) {
                new_nodes.append(&mut optimized_nodes);
            } else if let Some(mut optimized_nodes) = add_opt(&node) {
                new_nodes.append(&mut optimized_nodes);
            } else if let Some(mut optimized_nodes) = sub_opt(&node) {
                new_nodes.append(&mut optimized_nodes);
            } else if let Some(mut optimized_nodes) = sub_add_opt(&node) {
                new_nodes.append(&mut optimized_nodes);
            } else if let Some(mut optimized_nodes) = copy_opt(&node) {
                new_nodes.append(&mut optimized_nodes);
            } else if let Node::Loop(nodes) = node {
                let node = Node::Loop(inner(nodes));
                new_nodes.push_back(node);
            } else {
                new_nodes.push_back(node);
            }
        }
        new_nodes
    }

    inner(nodes)
}

#[cfg(test)]
mod test {
    use crate::{
        instruction::Instruction::*,
        parse::{tokenize, Node, Nodes},
    };

    use super::{add_opt, merge_instruction, zeroset_opt};

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
    fn optimize_node(code: &str, optimizer: impl FnOnce(&Node) -> Option<Nodes>) -> Option<Nodes> {
        let tokens = tokenize(code);
        let mut nodes = Node::from_tokens(tokens).unwrap();
        if nodes.len() == 1 {
            if let Node::Loop(loop_nodes) = nodes.pop_front().unwrap() {
                let merged_loop_node = merge_instruction(loop_nodes);
                let loop_node = Node::Loop(merged_loop_node);

                optimizer(&loop_node)
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    fn assert_node(
        optimizer: impl FnOnce(&Node) -> Option<Nodes>,
        code: &str,
        node: Option<Nodes>,
    ) {
        let optimized_node = optimize_node(code, optimizer);
        assert_eq!(node, optimized_node);
    }

    #[test]
    fn test_zeroset_opt() {
        assert_node(
            zeroset_opt,
            "[-]",
            Some([Node::Instruction(ZeroSet)].into()),
        );

        assert_node(
            zeroset_opt,
            "[+]",
            Some([Node::Instruction(ZeroSet)].into()),
        );

        assert_node(zeroset_opt, "[--]", None);
    }

    #[test]
    fn test_add_opt() {
        assert_node(
            add_opt,
            "[->>>+<<<]",
            Some([Node::Instruction(AddTo(3)), Node::Instruction(ZeroSet)].into()),
        );
        assert_node(
            add_opt,
            "[>>>+<<<-]",
            Some([Node::Instruction(AddTo(3)), Node::Instruction(ZeroSet)].into()),
        );

        assert_node(
            add_opt,
            "[-<<<+>>>]",
            Some([Node::Instruction(AddToRev(3)), Node::Instruction(ZeroSet)].into()),
        );
        assert_node(
            add_opt,
            "[<<<+>>>-]",
            Some([Node::Instruction(AddToRev(3)), Node::Instruction(ZeroSet)].into()),
        );

        assert_node(add_opt, "[-<<<++>>>]", None);
    }
}

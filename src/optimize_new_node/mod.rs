use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    instruction::{
        self,
        Instruction::{self, *},
    },
    parse::{Nod, Nods},
};

fn zeroset_opt(node: &Nod) -> Option<Nods> {
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 1 {
            if let Nod::Instruction(Add(1) | Sub(1)) = loop_nodes.front()? {
                let nodes = Nods::from([Nod::Instruction(ZeroSet)]);
                return Some(nodes);
            }
            if let Nod::Instruction(AddOffset(offset, 1) | SubOffset(offset, 1)) =
                loop_nodes.front()?
            {
                let nodes = Nods::from([Nod::Instruction(ZeroSetOffset(*offset))]);
                return Some(nodes);
            }
        }
    }
    None
}

fn loop_opt(node: &Nod) -> Option<Nods> {
    let mut new_nodes = Nods::new();
    if let Nod::Loop(loop_nodes) = node {
        for node in loop_nodes {
            let ins = node.as_instruction()?;
            match ins {
                AddOffset(offset, value) if value == 1 => {
                    let ins = if offset < 0 {
                        AddToRev(-offset as usize)
                    } else {
                        AddTo(offset as usize)
                    };

                    new_nodes.push_front(Nod::Instruction(ins));
                }
                AddOffset(offset, value) => {
                    let ins = if offset < 0 {
                        MulAddRev(-offset as usize, value)
                    } else {
                        MulAdd(offset as usize, value)
                    };

                    new_nodes.push_front(Nod::Instruction(ins));
                }
                SubOffset(0, 1) => {
                    new_nodes.push_back(Nod::Instruction(ZeroSet));
                }
                PtrIncrement(_) | PtrDecrement(_) => return None,
                _ => return None,
            }
        }

        return Some(new_nodes);
    }
    None
}

fn add_opt(node: &Nod) -> Option<Nods> {
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 4 {
            let mut nodes_iter = loop_nodes.iter();

            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrIncrement(ptr_increment)), Nod::Instruction(Add(1)), Nod::Instruction(PtrDecrement(ptr_decrement))]
            | [Nod::Instruction(PtrIncrement(ptr_increment)), Nod::Instruction(Add(1)), Nod::Instruction(PtrDecrement(ptr_decrement)), Nod::Instruction(Sub(1))] = {
                [
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                ]
            } {
                if ptr_increment == ptr_decrement {
                    {
                        return Some(
                            [
                                Nod::Instruction(AddTo(*ptr_increment)),
                                Nod::Instruction(ZeroSet),
                            ]
                            .into(),
                        );
                    }
                }
            }

            let mut nodes_iter = loop_nodes.iter();

            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(ptr_increment)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(ptr_decrement))]
            | [Nod::Instruction(PtrDecrement(ptr_increment)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(ptr_decrement)), Nod::Instruction(Sub(1))] = [
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
            ] {
                if ptr_decrement == ptr_increment {
                    {
                        return Some(
                            [
                                Nod::Instruction(AddToRev(*ptr_decrement)),
                                Nod::Instruction(ZeroSet),
                            ]
                            .into(),
                        );
                    }
                }
            }
        }
    }
    None
}

fn sub_opt(node: &Nod) -> Option<Nods> {
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 4 {
            let mut nodes_iter = loop_nodes.iter();

            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrIncrement(ptr_increment)), Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(ptr_decrement))]
            | [Nod::Instruction(PtrIncrement(ptr_increment)), Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(ptr_decrement)), Nod::Instruction(Sub(1))] = {
                [
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                    nodes_iter.next()?,
                ]
            } {
                if ptr_increment == ptr_decrement {
                    {
                        return Some(
                            [
                                Nod::Instruction(SubTo(*ptr_increment)),
                                Nod::Instruction(ZeroSet),
                            ]
                            .into(),
                        );
                    }
                }
            }

            let mut nodes_iter = loop_nodes.iter();

            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(ptr_increment)), Nod::Instruction(Sub(1)), Nod::Instruction(PtrIncrement(ptr_decrement))]
            | [Nod::Instruction(PtrDecrement(ptr_increment)), Nod::Instruction(Sub(1)), Nod::Instruction(PtrIncrement(ptr_decrement)), Nod::Instruction(Sub(1))] = [
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
                nodes_iter.next()?,
            ] {
                if ptr_decrement == ptr_increment {
                    {
                        return Some(
                            [
                                Nod::Instruction(SubToRev(*ptr_decrement)),
                                Nod::Instruction(ZeroSet),
                            ]
                            .into(),
                        );
                    }
                }
            }
        }
    }
    None
}

fn copy_opt(node: &Nod) -> Option<Nods> {
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 6 {
            let mut nodes_iter = loop_nodes.iter();
            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrIncrement(x)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(y)), Nod::Instruction(Add(1)), Nod::Instruction(PtrDecrement(z))] = [
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
                            Nod::Instruction(Copy(*x)),
                            Nod::Instruction(Copy(x + y)),
                            Nod::Instruction(ZeroSet),
                        ]
                        .into(),
                    );
                }
            }
        }
    }
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 6 {
            let mut nodes_iter = loop_nodes.iter();
            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(x)), Nod::Instruction(Add(1)), Nod::Instruction(PtrDecrement(y)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(z))] = [
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
                            Nod::Instruction(CopyRev(*x)),
                            Nod::Instruction(CopyRev(x + y)),
                            Nod::Instruction(ZeroSet),
                        ]
                        .into(),
                    );
                }
            }
        }
    }
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 6 {
            let mut nodes_iter = loop_nodes.iter();
            if let [Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(x)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(y)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(z))] = [
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
                            Nod::Instruction(CopyRev(*x)),
                            Nod::Instruction(CopyRev(x - y)),
                            Nod::Instruction(ZeroSet),
                        ]
                        .into(),
                    );
                }
            }
        }
    }
    None
}

fn sub_add_opt(node: &Nod) -> Option<Nods> {
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 7 {
            let mut nodes_iter = loop_nodes.iter();

            if let [Nod::Instruction(PtrDecrement(dec)), Nod::Instruction(Sub(1)), Nod::Instruction(PtrIncrement(inc)), Nod::Instruction(Sub(1)), Nod::Instruction(PtrDecrement(dec2)), Nod::Instruction(Add(1)), Nod::Instruction(PtrIncrement(inc2))] = {
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
                            Nod::Instruction(SubToRev(*dec)),
                            Nod::Instruction(AddToRev(*dec2)),
                            Nod::Instruction(ZeroSet),
                        ]
                        .into(),
                    );
                }
            }
        }
    }
    None
}

fn merge_instruction(nodes: Nods) -> Nods {
    let mut new_nodes = Nods::new();

    for node in nodes {
        new_nodes.push_back(node);

        while let Some(merged_inst) =
            new_nodes
                .iter()
                .nth_back(1)
                .zip(new_nodes.back())
                .and_then(|(back2, back)| {
                    if let (Nod::Instruction(back2), Nod::Instruction(back)) = (back2, back) {
                        back2.merge(*back)
                    } else {
                        None
                    }
                })
        {
            new_nodes.pop_back().unwrap();
            new_nodes.pop_back().unwrap();
            if !merged_inst.is_no_action() {
                new_nodes.push_back(Nod::Instruction(merged_inst))
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
    fn new() -> Self {
        Self { inner: vec![] }
    }
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

pub fn offset_opt(nodes: &Nods) -> Option<Nods> {
    fn inner(nodes: &Nods) -> Nods {
        let mut new_nodes = Nods::new();

        let mut pointer_offset = 0isize;
        let mut offset_instructions: BTreeMap<isize, Instructions> = BTreeMap::new();

        for node in nodes {
            match node {
                Nod::Loop(loop_nodes) => {
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
                            .map(Nod::Instruction)
                            .collect();
                        new_nodes.append(&mut instructions)
                    }
                    match pointer_offset.cmp(&0) {
                        Ordering::Less => new_nodes
                            .push_back(Nod::Instruction(PtrDecrement(-pointer_offset as usize))),
                        Ordering::Equal => (),
                        Ordering::Greater => new_nodes
                            .push_back(Nod::Instruction(PtrIncrement(pointer_offset as usize))),
                    };

                    let loop_nodes = inner(loop_nodes);
                    new_nodes.push_back(Nod::Loop(loop_nodes));

                    pointer_offset = 0;
                    offset_instructions.clear()
                }
                Nod::Instruction(ins) => match ins {
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
                .map(Nod::Instruction)
                .collect();
            new_nodes.append(&mut instructions)
        }
        match pointer_offset.cmp(&0) {
            Ordering::Less => {
                new_nodes.push_back(Nod::Instruction(PtrDecrement(-pointer_offset as usize)))
            }
            Ordering::Equal => (),
            Ordering::Greater => {
                new_nodes.push_back(Nod::Instruction(PtrIncrement(pointer_offset as usize)))
            }
        };

        new_nodes
    }
    // eprintln!("{pointer_offset}, {instructions:?}");
    Some(inner(nodes))
    // unimplemented!()
}

pub fn optimize(nodes: Nods) -> Nods {
    fn inner(nodes: Nods) {
        let mut new_nodes = Nods::new();

        for node in nodes {
            // let node = if let Nod::Loop(loop_nodes) = node {
            //     Nod::Loop(merge_instruction(loop_nodes))
            // } else {
            //     node
            // };

            // if let Some(mut optimized_nodes) = zeroset_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // } else
            // if let Some(mut optimized_nodes) = loop_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // }
            // else if let Some(mut optimized_nodes) = add_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // } else if let Some(mut optimized_nodes) = sub_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // } else if let Some(mut optimized_nodes) = sub_add_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // } else if let Some(mut optimized_nodes) = copy_opt(&node) {
            //     new_nodes.append(&mut optimized_nodes);
            // }
            // else
            if let Nod::Loop(nodes) = node {
                let node = Nod::Loop(optimize(nodes));
                new_nodes.push_back(node);
            } else {
                new_nodes.push_back(node);
            }
        }
    }

    offset_opt(&nodes).unwrap()
}

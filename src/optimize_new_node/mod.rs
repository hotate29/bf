use crate::{
    instruction::Instruction::*,
    parse::{Nod, Nods},
};

fn zeroset_opt(node: &Nod) -> Option<Nods> {
    if let Nod::Loop(loop_nodes) = node {
        if loop_nodes.len() == 1 {
            if let Nod::Instruction(Add(1) | Sub(1)) = loop_nodes.front()? {
                let nodes = Nods::from([Nod::Instruction(ZeroSet)]);
                return Some(nodes);
            }
        }
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

pub fn offset_opt(nodes: &Nods) -> Nods {
    fn inner(nodes: &Nods) -> Nods {
        let mut new_nodes = Nods::new();

        let mut pointer_offset = 0isize;

        for node in nodes {
            match node {
                Nod::Loop(loop_nodes) => {
                    match pointer_offset.cmp(&0) {
                        std::cmp::Ordering::Less => new_nodes
                            .push_back(Nod::Instruction(PtrDecrement((-pointer_offset) as usize))),
                        std::cmp::Ordering::Greater => new_nodes
                            .push_back(Nod::Instruction(PtrIncrement(pointer_offset as usize))),
                        std::cmp::Ordering::Equal => (),
                    };
                    pointer_offset = 0;
                    new_nodes.push_back(Nod::Loop(inner(loop_nodes)));
                }
                Nod::Instruction(ins) => {
                    match *ins {
                        PtrIncrement(inc) => {
                            pointer_offset += inc as isize;
                            continue;
                        }
                        PtrDecrement(dec) => {
                            pointer_offset -= dec as isize;
                            continue;
                        }
                        _ => (),
                    }

                    match *ins {
                        // とりあえず、基本命令だけ
                        PtrIncrement(_) | PtrDecrement(_) => {
                            unreachable!()
                        }
                        Add(value) => {
                            new_nodes.push_back(Nod::Instruction(AddOffset(pointer_offset, value)))
                        }
                        Sub(value) => {
                            new_nodes.push_back(Nod::Instruction(SubOffset(pointer_offset, value)))
                        }
                        Output(_) => {
                            new_nodes.push_back(Nod::Instruction(OutputOffset(pointer_offset)))
                        }
                        ins => new_nodes.push_back(Nod::Instruction(ins)),
                    }

                    if let Some(merged_instruction) = new_nodes
                        .iter()
                        .nth_back(1)
                        .and_then(|nod| nod.as_instruction())
                        .zip(new_nodes.back().and_then(|nod| nod.as_instruction()))
                        .and_then(|(last2_ins, last_ins)| last2_ins.merge(last_ins))
                    {
                        // eprintln!("aa");
                        new_nodes.pop_back();
                        new_nodes.pop_back();
                        if !merged_instruction.is_no_action() {
                            new_nodes.push_back(Nod::Instruction(merged_instruction))
                        }
                    }
                }
            }

            // eprintln!("{pointer_offset}, {new_nodes:?}");
        }

        match pointer_offset.cmp(&0) {
            std::cmp::Ordering::Less => {
                new_nodes.push_back(Nod::Instruction(PtrDecrement((-pointer_offset) as usize)))
            }
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Greater => {
                new_nodes.push_back(Nod::Instruction(PtrIncrement(pointer_offset as usize)))
            }
        }
        new_nodes
    }
    // eprintln!("{pointer_offset}, {instructions:?}");
    inner(nodes)
    // unimplemented!()
}

pub fn optimize(nodes: Nods) -> Nods {
    let nodes = offset_opt(&nodes);
    let mut new_nodes = Nods::new();

    for node in nodes {
        let node = if let Nod::Loop(loop_nodes) = node {
            Nod::Loop(merge_instruction(loop_nodes))
        } else {
            node
        };

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
        } else if let Nod::Loop(nodes) = node {
            let node = Nod::Loop(optimize(nodes));
            new_nodes.push_back(node);
        } else {
            new_nodes.push_back(node);
        }
    }

    merge_instruction(new_nodes)
}

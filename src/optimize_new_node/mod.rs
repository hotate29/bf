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

pub fn optimize(nodes: Nods) -> Nods {
    let nodes = merge_instruction(nodes);
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

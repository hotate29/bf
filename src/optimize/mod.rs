use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    instruction::Instruction::{self, *},
    parse::{Node, Nodes},
};

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
    inner: Vec<(usize, Instruction)>,
}
impl Instructions {
    fn from_ins(num: usize, ins: Instruction) -> Self {
        Self {
            inner: vec![(num, ins)],
        }
    }
    fn push(&mut self, num: usize, ins: Instruction) {
        self.inner.push((num, ins));

        while let Some(merged_inst) = self
            .inner
            .iter()
            .nth_back(1)
            .zip(self.inner.last())
            .and_then(|((back2_num, back2), (_, back))| {
                back2.merge(*back).map(|ins| (*back2_num, ins))
            })
        {
            self.inner.pop().unwrap();
            self.inner.pop().unwrap();
            if !merged_inst.1.is_no_action() {
                self.inner.push(merged_inst)
            }
        }
    }
    fn instructions(&self) -> impl Iterator<Item = &Instruction> {
        self.inner.iter().map(|(_, ins)| ins)
    }
    // fn inner(&self) -> &Vec<Instruction> {
    //     &self.inner
    // }
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
        ins_count: usize,
        output_order: Vec<isize>,
    }
    impl State {
        fn push_instruction(&mut self, ins: Instruction) {
            self.ins_count += 1;
            if matches!(ins, Output(_)) && self.output_order.last() != Some(&self.pointer_offset) {
                self.output_order.push(self.pointer_offset);
            }
            match ins {
                PtrIncrement(inc) => self.pointer_offset += inc as isize,
                PtrDecrement(dec) => self.pointer_offset -= dec as isize,
                ins @ (Add(_) | Sub(_) | Output(_) | Input(_) | ZeroSet) => {
                    self.offset_map
                        .entry(self.pointer_offset)
                        .and_modify(|instructions| instructions.push(self.ins_count, ins))
                        .or_insert_with(|| Instructions::from_ins(self.ins_count, ins));
                }
                _ => panic!(),
            };
        }
        fn into_nodes(self) -> Nodes {
            let mut insss = self
                .offset_map
                .into_iter()
                .flat_map(|(offset, instructions)| {
                    instructions.inner.into_iter().map(move |(id, ins)| {
                        let ins = match ins {
                            Add(value) => AddOffset(offset, value),
                            Sub(value) => SubOffset(offset, value),
                            Output(repeat) => OutputOffset(offset, repeat),
                            Input(repeat) => InputOffset(offset, repeat),
                            ZeroSet => ZeroSetOffset(offset),
                            _ => panic!(),
                        };
                        (id, ins)
                    })
                })
                .collect::<Vec<_>>();

            insss.sort_unstable_by_key(|(num, _)| *num);
            let mut out_nodes = insss
                .into_iter()
                .map(|(_, ins)| ins)
                .map(Into::into)
                .collect::<Nodes>();

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
                .filter(|ins| ins.inner.len() == 1 && (ins.inner[0].1 == Sub(1) || ins.inner[0].1 == Add(1)))
                .is_some()
        {
            // 最適化をするぞ！バリバリ！
            // 注: ここで出力するのは命令列で、ループではない。これの扱いをどうする？

            for (offset, instructions) in state.offset_map {
                for instruction in instructions.instructions() {
                    let instruction = match instruction {
                        // 最後にZeroSetにする
                        Sub(1) | Add(1) if offset == 0 => continue,
                        Add(1) => AddTo(offset),
                        Add(value) => MulAdd(offset, *value),
                        Sub(1) => SubTo(offset),
                        Sub(value) => MulSub(offset, *value),
                        // Output(repeat) => OutputOffset(repeat, offset),
                        // Input(_) => todo!(),
                        // ZeroSet => ZeroSetOffset(offset),
                        ins => panic!("{ins:?}"),
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

pub fn optimize(nodes: &Nodes) -> Nodes {
    offset_opt(nodes)
}

#[cfg(test)]
mod test {
    use super::{merge_instruction, offset_opt};
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

    #[rstest(input, expected,
        case("[-]", [ZeroSet.into()].into()),
        case("[+]", [ZeroSet.into()].into()),
        case("[++]", [Node::Loop([AddOffset(0, 2).into()].into())].into()),
        case("[->>>+<<<]", [AddTo(3).into(), ZeroSet.into()].into()),
        case("[>>>+<<<-]", [AddTo(3).into(), ZeroSet.into()].into()),
        case("[-<<<+>>>]", [AddTo(-3).into(), ZeroSet.into()].into()),
        case("[<<<+>>>-]", [AddTo(-3).into(), ZeroSet.into()].into()),
        case("[-<<<++>>>]", [MulAdd(-3, 2).into(), ZeroSet.into()].into()),
        case("[->>>-<<<]", [SubTo(3).into(), ZeroSet.into()].into()),
        case("[>>>-<<<-]", [SubTo(3).into(), ZeroSet.into()].into()),
        case("[-<<<->>>]", [SubTo(-3).into(), ZeroSet.into()].into()),
        case("[<<<->>>-]", [SubTo(-3).into(), ZeroSet.into()].into()),
        // case("[-<<<-->>>]", [SubOffset(0, 1).into(), SubOffset(-3, 2).into()].into()),
        case("", [].into()),
        case("+++", [AddOffset(0, 3).into()].into()),
        case("+++---", [].into()),
        case(">+++<-", [AddOffset(1, 3).into(), SubOffset(0, 1).into()].into()),
        case(">+++", [AddOffset(1, 3).into(), PtrIncrement(1).into()].into()),
        case("[[[]]]", [Node::Loop([Node::Loop([Node::Loop([].into())].into())].into())].into()),
        case("->+<", [SubOffset(0, 1).into(), AddOffset(1, 1).into()].into()),
        case("[->>>-<<<]", [SubTo(3).into(), ZeroSet.into()].into()),
        case("+++>-<[->>>-<<<]", [AddOffset(0, 3).into(),SubOffset(1, 1).into(), SubTo(3).into(), ZeroSet.into()].into()),
        case("[>>>-<<<-]", [SubTo(3).into(), ZeroSet.into()].into()),
        case("[>>>->+<<<<-]", [SubTo(3).into(), AddTo(4).into(), ZeroSet.into()].into()),
        case("+++[>>>[-][[->+<]]<<<]", [AddOffset(0, 3).into(), Node::Loop([PtrIncrement(3).into(), ZeroSet.into(), Node::Loop([AddTo(1).into(), ZeroSet.into()].into()), PtrDecrement(3).into()].into())].into()),
        case("[->>>.<<<]", [Node::Loop([SubOffset(0, 1).into(), OutputOffset(3, 1).into()].into())].into()),
        case("[->+>+>++>+++<<<<]", [AddTo(1).into(), AddTo(2).into(), MulAdd(3, 2).into(), MulAdd(4, 3).into(), ZeroSet.into()].into()),
        case("[-<<<-->>>]", [MulSub(-3, 2).into(), ZeroSet.into()].into()),
    )]
    fn test_optimize(input: &str, expected: Nodes) {
        let tokens = tokenize(input);
        let nodes = Node::from_tokens(tokens).unwrap();

        let optimized_node = offset_opt(&nodes);
        assert_eq!(optimized_node, expected)
    }
}

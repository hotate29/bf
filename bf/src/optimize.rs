use std::{cmp::Ordering, collections::BTreeMap, mem::swap};

use crate::{
    instruction::{
        Instruction::{self, *},
        Value,
    },
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
            .and_then(|(back, back2)| back.as_instruction().zip(back2.as_instruction()))
            .and_then(|(back, back2)| back.merge(back2))
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
    }
    impl State {
        fn push_instruction(&mut self, ins: Instruction) {
            self.ins_count += 1;
            match ins {
                PtrIncrement(inc) => self.pointer_offset += inc as isize,
                PtrDecrement(dec) => self.pointer_offset -= dec as isize,
                ins @ (Add(0, _)
                | Sub(0, _)
                | Output(0)
                | Input(0)
                | SetValue(0, Value::Const(0))) => {
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
                            Add(0, value) => Add(offset, value),
                            Sub(0, value) => Sub(offset, value),
                            Output(0) => Output(offset),
                            Input(0) => Input(offset),
                            SetValue(0, Value::Const(0)) => SetValue(offset, Value::Const(0)),
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
                Ordering::Less => out_nodes
                    .push_back(PtrDecrement(self.pointer_offset.unsigned_abs() as usize).into()),
                Ordering::Greater => {
                    out_nodes.push_back(PtrIncrement(self.pointer_offset as usize).into())
                }
                Ordering::Equal => (),
            }
            out_nodes
        }
        fn take(&mut self) -> Self {
            let mut a = Self::default();
            swap(self, &mut a);
            a
        }
    }

    fn inner(nodes: &Nodes, is_loop: bool) -> Nod {
        let mut new_nodes = Nodes::new();

        let mut state = State::default();

        let mut has_loop = false;
        let mut has_io = false;

        for node in nodes {
            match node {
                Node::Loop(loop_nodes) => {
                    has_loop = true;

                    let mut instructions = state.take().into_nodes();

                    new_nodes.append(&mut instructions);

                    match inner(loop_nodes, true) {
                        Nod::Loop(loop_nodes) => {
                            new_nodes.push_back(Node::Loop(loop_nodes));

                            // ループを抜けたときにポインターが指している値は必ず0なので、一応命令を挿入しておく。
                            new_nodes.push_back(SetValue(0, 0.into()).into());
                        }
                        Nod::Instructions(mut instructions) => new_nodes.append(&mut instructions),
                    }
                }
                Node::Instruction(instruction) => {
                    has_io |= matches!(instruction, Output(0) | Input(0));

                    state.push_instruction(*instruction);
                }
            }
        }

        let loop_counter_ins = state
            .offset_map
            .get(&0)
            .filter(|ins| ins.inner.len() == 1)
            .map(|ins| ins.inner[0].1);

        let optimizable = state.pointer_offset == 0 && !has_loop && is_loop && !has_io; // [->>>.<<<]を弾く

        // ポインターが指している値の回数だけ操作をするタイプのループ
        if optimizable && loop_counter_ins == Some(Sub(0, 1.into())) {
            // 最適化をするぞ！バリバリ！
            // 注: ここで出力するのは命令列で、ループではない。これの扱いをどうする？

            for (offset, instructions) in state.offset_map {
                for instruction in instructions.instructions() {
                    let instruction = match instruction {
                        // 最後にZeroSetにする
                        Sub(0, Value::Const(1)) if offset == 0 => continue,
                        Add(0, Value::Const(1)) => Add(offset, Value::Memory(0)),
                        Add(0, value @ Value::Const(_)) => MulAdd(offset, Value::Memory(0), *value),
                        Sub(0, Value::Const(1)) => Sub(offset, Value::Memory(0)),
                        Sub(0, value @ Value::Const(_)) => MulSub(offset, Value::Memory(0), *value),
                        ins => panic!("{ins:?}"),
                    };
                    new_nodes.push_back(instruction.into());
                }
            }
            new_nodes.push_back(SetValue(0, 0.into()).into());

            Nod::Instructions(new_nodes)
        }
        // (255 - ポインターが指している値)の回数だけ操作をするタイプのループ。これはよくわからないので簡単なやつだけ。
        else if optimizable && loop_counter_ins == Some(Add(0, 1.into())) {
            Nod::Instructions([SetValue(0, 0.into()).into()].into())
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

fn offset_merge(nodes: Nodes) -> Nodes {
    let mut simplified_nodes = SimplifiedNodes::new();
    for node in nodes {
        match node {
            Node::Loop(loop_nodes) => {
                simplified_nodes.push_back(Node::Loop(offset_merge(loop_nodes)))
            }
            ins_node @ Node::Instruction(_) => simplified_nodes.push_back(ins_node),
        };
    }
    simplified_nodes.into_nodes()
}

pub fn optimize(nodes: &Nodes) -> Nodes {
    let nodes = offset_opt(nodes);
    offset_merge(nodes)
}
#[derive(Debug, Default)]
struct SimplifiedNodes {
    nodes: Nodes,
    pointer_offset: isize,
}
impl SimplifiedNodes {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    fn push_back(&mut self, node: Node) {
        match node {
            loop_node @ Node::Loop(_) => {
                // ポインターを動かしておく
                match self.pointer_offset.cmp(&0) {
                    Ordering::Less => self.nodes.push_back(
                        PtrDecrement(self.pointer_offset.unsigned_abs() as usize).into(),
                    ),
                    Ordering::Greater => self
                        .nodes
                        .push_back(PtrIncrement(self.pointer_offset as usize).into()),
                    Ordering::Equal => (),
                }
                self.pointer_offset = 0;

                self.nodes.push_back(loop_node);
            }
            Node::Instruction(ins) => {
                // ポインターいじいじ
                match ins {
                    PtrIncrement(inc) => {
                        self.pointer_offset += inc as isize;
                        return;
                    }
                    PtrDecrement(dec) => {
                        self.pointer_offset -= dec as isize;
                        return;
                    }
                    _ => (),
                };

                let ins = match ins {
                    PtrIncrement(_) | PtrDecrement(_) => unreachable!(),
                    Add(offset, value) => Add(
                        self.pointer_offset + offset,
                        value.map_offset(|offset| self.pointer_offset + offset),
                    ),
                    Sub(offset, value) => Sub(
                        self.pointer_offset + offset,
                        value.map_offset(|offset| self.pointer_offset + offset),
                    ),
                    MulAdd(to_offset, lhs, rhs) => MulAdd(
                        self.pointer_offset + to_offset,
                        lhs.map_offset(|offset| self.pointer_offset + offset),
                        rhs.map_offset(|offset| self.pointer_offset + offset),
                    ),
                    MulSub(to_offset, lhs, rhs) => MulSub(
                        self.pointer_offset + to_offset,
                        lhs.map_offset(|offset| self.pointer_offset + offset),
                        rhs.map_offset(|offset| self.pointer_offset + offset),
                    ),
                    Output(offset) => Output(self.pointer_offset + offset),
                    Input(offset) => Input(self.pointer_offset + offset),
                    SetValue(offset, value) => SetValue(
                        self.pointer_offset + offset,
                        value.map_offset(|offset| self.pointer_offset + offset),
                    ),
                };
                self.nodes.push_back(ins.into());
            }
        }
    }
    fn into_nodes(self) -> Nodes {
        let mut nodes = merge_instruction(self.nodes);

        match self.pointer_offset.cmp(&0) {
            Ordering::Less => {
                nodes.push_back(PtrDecrement(self.pointer_offset.unsigned_abs() as usize).into())
            }
            Ordering::Greater => nodes.push_back(PtrIncrement(self.pointer_offset as usize).into()),
            Ordering::Equal => (),
        };

        nodes
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        instruction::Value,
        parse::{tokenize, Node, Nodes},
    };

    use rstest::rstest;

    #[test]
    fn test_merge_instruction() {
        let nodes = [
            Add(0, 1.into()).into(),
            Sub(0, 1.into()).into(),
            PtrIncrement(1).into(),
            PtrDecrement(1).into(),
            Add(0, 1.into()).into(),
        ]
        .into();
        assert_eq!(merge_instruction(nodes), [Add(0, 1.into()).into()].into());
    }

    #[rstest(input, expected,
        case("[-]", [SetValue(0, 0.into()).into()].into()),
        case("[+]", [SetValue(0, 0.into()).into()].into()),
        case("[++]", [Node::Loop([Add(0, 2.into()).into()].into()), SetValue(0, 0.into()).into()].into()),
        case("[->>>+<<<]", [Add(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[>>>+<<<-]", [Add(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[-<<<+>>>]", [Add(-3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[<<<+>>>-]", [Add(-3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[-<<<++>>>]", [MulAdd(-3, Value::Memory(0), 2.into()).into(), SetValue(0, 0.into()).into()].into()),
        case("[->>>-<<<]", [Sub(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[>>>-<<<-]", [Sub(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[-<<<->>>]", [Sub(-3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[<<<->>>-]", [Sub(-3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        // case("[-<<<-->>>]", [SubOffset(0, 1).into(), SubOffset(-3, 2).into()].into()),
        case("", [].into()),
        case("+++", [Add(0, 3.into()).into()].into()),
        case("+++---", [].into()),
        case(">+++<-", [Add(1, 3.into()).into(), Sub(0, 1.into()).into()].into()),
        case(">+++", [Add(1, 3.into()).into(), PtrIncrement(1).into()].into()),
        case("[[[]]]", [Node::Loop([Node::Loop([Node::Loop([].into()), SetValue(0, 0.into()).into()].into()), SetValue(0, 0.into()).into()].into()), SetValue(0, 0.into()).into()].into()),
        case("->+<", [Sub(0, 1.into()).into(), Add(1, 1.into()).into()].into()),
        case("[->>>-<<<]", [Sub(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("+++>-<[->>>-<<<]", [Add(0, 3.into()).into(),Sub(1, 1.into()).into(), Sub(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[>>>-<<<-]", [Sub(3, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("[>>>->+<<<<-]", [Sub(3, Value::Memory(0)).into(), Add(4, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()),
        case("+++[>>>[-][[->+<]]<<<]", [Add(0, 3.into()).into(), Node::Loop([SetValue(3, 0.into()).into(), PtrIncrement(3).into(), Node::Loop([Add(1, Value::Memory(0)).into(), SetValue(0, 0.into()).into()].into()), SetValue(0, 0.into()).into(), PtrDecrement(3).into()].into()), SetValue(0, 0.into()).into()].into()),
        case("[->>>.<<<]", [Node::Loop([Sub(0, 1.into()).into(), Output(3).into()].into()), SetValue(0, 0.into()).into()].into()),
        case("[->+>+>++>+++<<<<]", [Add(1, Value::Memory(0)).into(), Add(2, Value::Memory(0)).into(), MulAdd(3, Value::Memory(0), 2.into()).into(), MulAdd(4, Value::Memory(0), 3.into()).into(), SetValue(0, 0.into()).into()].into()),
        case("[-<<<-->>>]", [MulSub(-3, Value::Memory(0), 2.into()).into(), SetValue(0, 0.into()).into()].into()),
        case(".>.<.", [Output(0).into(), Output(1).into(), Output(0).into()].into()),
    )]
    fn test_optimize(input: &str, expected: Nodes) {
        let tokens = tokenize(input);
        let nodes = Node::from_tokens(tokens).unwrap();

        let optimized_node = optimize(&nodes);
        assert_eq!(optimized_node, expected)
    }

    #[rstest(input, expected,
        case([Add(0, 1.into()).into(), PtrIncrement(1).into(), Sub(0, Value::Const(1)).into(), Add(2, 2.into()).into()].into(), [Add(0, 1.into()).into(), Sub(1, Value::Const(1)).into(), Add(3, 2.into()).into(), PtrIncrement(1).into()].into())
    )]
    fn test_simplified_nodes(input: Nodes, expected: Nodes) {
        let mut simplified_nodes = SimplifiedNodes::new();

        for node in input {
            simplified_nodes.push_back(node);
        }

        let simplified_nodes = simplified_nodes.into_nodes();

        assert_eq!(simplified_nodes, expected)
    }
}

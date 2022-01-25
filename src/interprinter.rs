use crate::token::{ExprKind, Instruction, Node};

use std::io::prelude::*;

struct State<R: Read, W: Write> {
    pointer: usize,
    memory: Vec<u8>,
    input_reader: R,
    output_writer: W,
}
impl<R: Read, W: Write> State<R, W> {
    fn at_mut(&mut self, offset: usize) -> &mut u8 {
        if self.memory.len() <= self.pointer + offset {
            self.memory.resize(self.pointer * 2 + offset, 0);
        }
        &mut self.memory[self.pointer + offset]
    }
    fn add(&mut self, value: u8) {
        let a = self.at_mut(0);
        *a = a.wrapping_add(value);
    }
    fn sub(&mut self, value: u8) {
        let a = self.at_mut(0);
        *a = a.wrapping_sub(value);
    }
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
        self.at_mut(0); // メモリーを伸ばす
    }
    fn pointer_sub(&mut self, value: usize) {
        self.pointer -= value;
    }
    fn set_to_value(&mut self, offset: usize, value: u8) {
        *self.at_mut(offset) = value;
    }
    fn output(&mut self) {
        write!(
            self.output_writer,
            "{}",
            char::from_u32(self.memory[self.pointer] as u32).unwrap()
        )
        .unwrap();
        self.output_writer.flush().unwrap();
    }
    fn input(&mut self) {
        let mut buf = [0];
        self.input_reader.read_exact(&mut buf).unwrap();
        *self.at_mut(0) = buf[0];
    }
}

pub struct InterPrinter<R: Read, W: Write> {
    state: State<R, W>,
    root_node: Node,
}
impl<R: Read, W: Write> InterPrinter<R, W> {
    pub fn new(root_node: Node, input: R, output: W) -> Self {
        let state = State {
            pointer: 0,
            memory: vec![0; 30000],
            input_reader: input,
            output_writer: output,
        };

        Self { state, root_node }
    }
    pub fn start(&mut self) {
        fn inner<R: Read, W: Write>(state: &mut State<R, W>, node: &Node) {
            for expr in &node.0 {
                match expr {
                    ExprKind::Instructions(instructions) => {
                        for instruction in instructions {
                            match instruction {
                                Instruction::PtrIncrement(n) => state.pointer_add(*n),
                                Instruction::PtrDecrement(n) => state.pointer_sub(*n),
                                Instruction::Add(n) => {
                                    state.add((n % u8::MAX as usize) as u8);
                                }
                                Instruction::Sub(n) => {
                                    state.sub((n % u8::MAX as usize) as u8);
                                }
                                Instruction::Output(n) => {
                                    for _ in 0..*n {
                                        state.output()
                                    }
                                }
                                Instruction::Input(n) => {
                                    for _ in 0..*n {
                                        state.input()
                                    }
                                }
                                Instruction::SetValue(offset, v) => {
                                    state.set_to_value(*offset, *v)
                                }
                            }
                        }
                    }
                    ExprKind::While(node) => {
                        while state.memory[state.pointer] != 0 {
                            inner(state, node)
                        }
                    }
                }
            }
        }
        inner(&mut self.state, &self.root_node);
    }
}

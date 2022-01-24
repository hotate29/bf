use crate::token::{ExprKind, Instruction, Node};

use std::io::prelude::*;

struct State<R: Read, W: Write> {
    pointer: usize,
    memory: Vec<u8>,
    input_reader: R,
    output_writer: W,
}
impl<R: Read, W: Write> State<R, W> {
    fn add(&mut self, value: u8) {
        self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(value);
    }
    fn sub(&mut self, value: u8) {
        self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(value);
    }
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
        if self.pointer >= self.memory.len() {
            self.memory.resize(self.memory.len() * 2 + value, 0);
        }
    }
    fn pointer_sub(&mut self, value: usize) {
        self.pointer -= value;
    }
    fn set_value(&mut self, value: u8) {
        self.memory[self.pointer] = value
    }
    fn set_to_value(&mut self, offset: usize, value: u8) {
        if self.pointer + offset >= self.memory.len() {
            self.memory.resize(self.memory.len() * 2 + offset, 0);
        }
        self.memory[self.pointer + offset] = value;
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
        self.set_value(buf[0]);
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
            memory: vec![0],
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
                                Instruction::Increment(n) => {
                                    state.add((n % u8::MAX as usize) as u8);
                                }
                                Instruction::Decrement(n) => {
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
                                Instruction::SetValue(v) => state.set_value(*v),
                                Instruction::SetToValue(offset, v) => {
                                    state.set_to_value(*offset, *v)
                                }
                            }
                        }
                    }
                    crate::token::ExprKind::While(node) => {
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

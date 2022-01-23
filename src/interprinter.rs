use crate::token::{middle_token, node, tokenize, ExprKind, Instruction, Node};

use std::io::{prelude::*, stdin, stdout};

struct State {
    pointer: usize,
    memory: Vec<u8>,
}
impl State {
    fn add(&mut self, value: u8) {
        self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(value);
    }
    fn sub(&mut self, value: u8) {
        self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(value);
    }
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
    }
    fn pointer_sub(&mut self, value: usize) {
        self.pointer -= value;
    }
    fn output(&self) {
        print!(
            "{}",
            char::from_u32(self.memory[self.pointer] as u32).unwrap()
        );
        stdout().flush().unwrap();
    }
    fn input(&mut self) {
        let mut buf = [0];
        stdin().read_exact(&mut buf).unwrap();
        self.memory[self.pointer] = buf[0];
    }
}

pub struct InterPrinter {
    state: State,
    root_node: Node,
}
impl InterPrinter {
    pub fn new(source: &str, memory_len: usize) -> Self {
        let tokens = tokenize(source);
        let middle_tokens = middle_token(&tokens);
        let root_node = node(&middle_tokens);

        let state = State {
            pointer: 0,
            memory: vec![0; memory_len],
        };

        Self { state, root_node }
    }
    pub fn start(&mut self) {
        fn inner(state: &mut State, node: &Node) {
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

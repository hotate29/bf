use crate::optimize::{ExprKind, Node};
use crate::token::Instruction;

use std::io::prelude::*;

struct State<R: Read, W: Write> {
    pointer: usize,
    memory: Vec<u8>,
    input_reader: R,
    output_writer: W,
}
impl<R: Read, W: Write> State<R, W> {
    fn at(&mut self, offset: usize) -> u8 {
        if self.memory.len() <= self.pointer + offset {
            self.memory.resize(self.pointer * 2 + offset, 0);
        }
        self.memory[self.pointer + offset]
    }
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
    fn add_offset(&mut self, offset: usize, value: u8) {
        let a = self.at_mut(offset);
        *a = a.wrapping_add(value);
    }
    fn sub(&mut self, value: u8) {
        let a = self.at_mut(0);
        *a = a.wrapping_sub(value);
    }
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
        self.at(0); // メモリーを伸ばす
    }
    fn pointer_sub(&mut self, value: usize) {
        if self.pointer < value {
            panic!("メモリがマイナス")
        }
        self.pointer -= value;
    }
    fn set_to_value(&mut self, offset: usize, value: u8) {
        *self.at_mut(offset) = value;
    }
    fn output(&mut self) {
        self.output_writer
            .write_all(&[self.memory[self.pointer]])
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
                                Instruction::MoveAdd(offset) => {
                                    let from = state.at(0);
                                    state.add_offset(*offset, from);
                                    *state.at_mut(0) = 0;
                                }
                                Instruction::MoveAddRev(offset) => {
                                    if state.at(0) == 0 {
                                        continue;
                                    }
                                    let from = state.at(0);
                                    state.memory[state.pointer - offset] =
                                        state.memory[state.pointer - offset].wrapping_add(from);
                                    *state.at_mut(0) = 0;
                                }
                                Instruction::MoveSub(offset) => {
                                    let from = state.at(0);
                                    let v = state.at_mut(*offset);
                                    *v = v.wrapping_sub(from);
                                    *state.at_mut(0) = 0;
                                }
                                Instruction::MoveSubRev(offset) => {
                                    if state.at(0) == 0 {
                                        continue;
                                    }
                                    let from = state.at(0);
                                    state.memory[state.pointer - offset] =
                                        state.memory[state.pointer - offset].wrapping_sub(from);
                                    *state.at_mut(0) = 0;
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
                                Instruction::SetValue(offset, v) => state.set_to_value(*offset, *v),
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

#[cfg(test)]
mod test {
    use std::{fs, io};

    use crate::{optimize::optimize, optimize::Node};

    use super::InterPrinter;

    #[test]
    fn test_memory_extend() {
        let source = ">".repeat(30001);
        let root_node = Node::from_source(&source).unwrap();

        InterPrinter::new(root_node, io::empty(), io::sink()).start();
    }

    // デバックビルドだとめちゃくちゃ時間がかかるので、デフォルトでは実行しないようになっている
    // 実行する場合は、`cargo test --release -- --ignored`
    #[test]
    #[ignore]
    fn test_interprinter_mandelbrot() {
        let mandelbrot_source = fs::read_to_string("mandelbrot.bf").unwrap();
        let assert_mandelbrot = fs::read_to_string("mandelbrot").unwrap();

        let root_node = Node::from_source(&mandelbrot_source).unwrap();

        let mut output_buffer = Vec::new();
        InterPrinter::new(root_node, io::empty(), &mut output_buffer).start();

        let output_string = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output_string, assert_mandelbrot);
    }

    #[test]
    #[ignore]
    fn test_optimized_interprinter_mandelbrot() {
        let mandelbrot_source = fs::read_to_string("mandelbrot.bf").unwrap();
        let assert_mandelbrot = fs::read_to_string("mandelbrot").unwrap();

        let root_node = Node::from_source(&mandelbrot_source).unwrap();
        let root_node = optimize(root_node);

        let mut output_buffer = Vec::new();
        InterPrinter::new(root_node, io::empty(), &mut output_buffer).start();

        let output_string = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output_string, assert_mandelbrot);
    }
}

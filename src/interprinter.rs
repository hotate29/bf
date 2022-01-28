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

#[derive(Debug)]
enum CInstruction {
    Instruction(Instruction),
    WhileBegin,
    WhileEnd,
}
impl CInstruction {
    fn from_instruction(instruction: Instruction) -> Self {
        Self::Instruction(instruction)
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
        fn inner(c_instruction: &mut Vec<CInstruction>, node: &Node) {
            for expr in &node.0 {
                match expr {
                    ExprKind::Instructions(ins) => c_instruction
                        .extend(ins.iter().map(|ins| CInstruction::from_instruction(*ins))),
                    ExprKind::While(while_node) => {
                        c_instruction.push(CInstruction::WhileBegin);
                        inner(c_instruction, while_node);
                        c_instruction.push(CInstruction::WhileEnd);
                    }
                }
            }
        }

        let mut instructions = vec![];
        inner(&mut instructions, &self.root_node);

        let mut while_stack = vec![0];
        let mut while_begin_jump_table = vec![0; instructions.len()];
        for (i, instruction) in instructions.iter().enumerate() {
            match instruction {
                CInstruction::Instruction(_) => (),
                CInstruction::WhileBegin => while_stack.push(i),
                CInstruction::WhileEnd => while_begin_jump_table[i] = while_stack.pop().unwrap(),
            }
        }

        let mut while_stack = vec![0];
        let mut while_end_jump_table = vec![0; instructions.len()];
        for (i, instruction) in instructions.iter().enumerate().rev() {
            match instruction {
                CInstruction::Instruction(_) => (),
                CInstruction::WhileBegin => while_end_jump_table[i] = while_stack.pop().unwrap(),
                CInstruction::WhileEnd => while_stack.push(i + 1),
            }
        }

        let mut now = 0;

        while now < instructions.len() {
            match &instructions[now] {
                CInstruction::Instruction(instruction) => {
                    match instruction {
                        Instruction::PtrIncrement(n) => self.state.pointer_add(*n),
                        Instruction::PtrDecrement(n) => self.state.pointer_sub(*n),
                        Instruction::Add(n) => {
                            self.state.add((n % u8::MAX as usize) as u8);
                        }
                        Instruction::MoveAdd(offset) => {
                            let from = self.state.at(0);
                            self.state.add_offset(*offset, from);
                            *self.state.at_mut(0) = 0;
                        }
                        Instruction::MoveAddRev(offset) => {
                            if self.state.at(0) != 0 {
                                let from = self.state.at(0);
                                self.state.memory[self.state.pointer - offset] = self.state.memory
                                    [self.state.pointer - offset]
                                    .wrapping_add(from);
                                *self.state.at_mut(0) = 0;
                            }
                        }
                        Instruction::MoveSub(offset) => {
                            let from = self.state.at(0);
                            let v = self.state.at_mut(*offset);
                            *v = v.wrapping_sub(from);
                            *self.state.at_mut(0) = 0;
                        }
                        Instruction::MoveSubRev(offset) => {
                            if self.state.at(0) != 0 {
                                let from = self.state.at(0);
                                self.state.memory[self.state.pointer - offset] = self.state.memory
                                    [self.state.pointer - offset]
                                    .wrapping_sub(from);
                                *self.state.at_mut(0) = 0;
                            }
                        }
                        Instruction::Sub(n) => {
                            self.state.sub((n % u8::MAX as usize) as u8);
                        }
                        Instruction::MulAddRev(offset, value) => {
                            if self.state.at(0) != 0 {
                                let a = self.state.at(0).wrapping_mul(*value);
                                self.state.memory[self.state.pointer - offset] =
                                    self.state.memory[self.state.pointer - offset].wrapping_add(a);
                                *self.state.at_mut(0) = 0
                            }
                        }
                        Instruction::Output(n) => {
                            for _ in 0..*n {
                                self.state.output()
                            }
                        }
                        Instruction::Input(n) => {
                            for _ in 0..*n {
                                self.state.input()
                            }
                        }
                        Instruction::SetValue(offset, v) => self.state.set_to_value(*offset, *v),
                    };
                    now += 1
                }
                CInstruction::WhileBegin if self.state.at(0) == 0 => {
                    now = while_end_jump_table[now]
                }
                CInstruction::WhileBegin => now += 1,
                CInstruction::WhileEnd => now = while_begin_jump_table[now],
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{fs, io};

    use crate::{
        optimize::Node,
        optimize::{all_optimizer, optimize},
    };

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
        let root_node = optimize(root_node, &all_optimizer());

        let mut output_buffer = Vec::new();
        InterPrinter::new(root_node, io::empty(), &mut output_buffer).start();

        let output_string = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output_string, assert_mandelbrot);
    }
    #[test]
    fn test_hello_world_interprinter() {
        let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

        let root_node = Node::from_source(hello_world).unwrap();

        let mut output = vec![];
        InterPrinter::new(root_node, io::empty(), &mut output).start();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "Hello World!\n");
    }
    #[test]
    fn test_optimized_hello_world_interprinter() {
        let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

        let root_node = Node::from_source(hello_world).unwrap();
        let root_node = optimize(root_node, &all_optimizer());

        let mut output = vec![];
        InterPrinter::new(root_node, io::empty(), &mut output).start();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "Hello World!\n");
    }
}

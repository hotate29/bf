use crate::optimize::{ExprKind, Node};
use crate::token::Instruction;

use std::io::prelude::*;

struct State {
    pointer: usize,
    memory: Vec<u8>,
}
impl State {
    fn at_offet(&mut self, offset: usize) -> u8 {
        if self.memory.len() <= self.pointer + offset {
            self.memory.resize(self.pointer * 2 + offset, 0);
        }
        self.memory[self.pointer + offset]
    }
    fn at_rev(&self, offset: usize) -> u8 {
        if offset > self.pointer {
            panic!("メモリがマイナス")
        }
        self.memory[self.pointer - offset]
    }
    fn at_offset_mut(&mut self, offset: usize) -> &mut u8 {
        if self.memory.len() <= self.pointer + offset {
            self.memory.resize(self.pointer * 2 + offset, 0);
        }
        &mut self.memory[self.pointer + offset]
    }
    fn at_mut_rev(&mut self, offset: usize) -> &mut u8 {
        if offset > self.pointer {
            panic!("メモリがマイナス")
        }
        &mut self.memory[self.pointer - offset]
    }
    fn add(&mut self, offset: usize, value: u8) {
        let a = self.at_offset_mut(offset);
        *a = a.wrapping_add(value);
    }
    fn add_rev(&mut self, offset: usize, value: u8) {
        let a = self.at_mut_rev(offset);
        *a = a.wrapping_add(value);
    }
    fn sub(&mut self, offset: usize, value: u8) {
        let a = self.at_offset_mut(offset);
        *a = a.wrapping_sub(value);
    }
    fn sub_rev(&mut self, offset: usize, value: u8) {
        let a = self.at_mut_rev(offset);
        *a = a.wrapping_sub(value);
    }
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
        self.at_offet(0); // メモリーを伸ばす
    }
    fn pointer_sub(&mut self, value: usize) {
        if self.pointer < value {
            panic!("ポインターがマイナス")
        }
        self.pointer -= value;
    }
    fn output(&self, writer: &mut impl Write) {
        let value = self.memory[self.pointer];
        writer.write_all(&[value]).unwrap();
        writer.flush().unwrap();
    }
    fn input(&mut self, reader: &mut impl Read) {
        let mut buf = [0];
        reader.read_exact(&mut buf).unwrap();
        *self.at_offset_mut(0) = buf[0];
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

fn node_to_c_instructions(node: &Node) -> Vec<CInstruction> {
    fn inner(c_instruction: &mut Vec<CInstruction>, node: &Node) {
        for expr in &node.0 {
            match expr {
                ExprKind::Instructions(ins) => {
                    c_instruction.extend(ins.iter().map(|ins| CInstruction::from_instruction(*ins)))
                }
                ExprKind::While(while_node) => {
                    c_instruction.push(CInstruction::WhileBegin);
                    inner(c_instruction, while_node);
                    c_instruction.push(CInstruction::WhileEnd);
                }
            }
        }
    }

    let mut instructions = vec![];
    inner(&mut instructions, node);
    instructions
}

pub struct InterPrinter<R: Read, W: Write> {
    state: State,
    input: R,
    output: W,
    instructions: Vec<CInstruction>,
    now: usize,
    while_begin_jump_table: Vec<usize>,
    while_end_jump_table: Vec<usize>,
}
impl<R: Read, W: Write> InterPrinter<R, W> {
    pub fn builder<'a>() -> InterPrinterBuilder<'a, R, W> {
        InterPrinterBuilder::default()
    }
    fn new(root_node: &Node, memory_len: usize, input: R, output: W) -> Self {
        let state = State {
            pointer: 0,
            memory: vec![0; memory_len],
        };

        let instructions = node_to_c_instructions(root_node);

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

        Self {
            state,
            instructions,
            while_begin_jump_table,
            while_end_jump_table,
            now: 0,
            input,
            output,
        }
    }
    pub fn memory(&self) -> &Vec<u8> {
        &self.state.memory
    }
    pub fn pointer(&self) -> usize {
        self.state.pointer
    }
    pub fn now(&self) -> usize {
        self.now
    }
}

impl<R: Read, W: Write> Iterator for InterPrinter<R, W> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if self.now < self.instructions.len() {
            match self.instructions[self.now] {
                CInstruction::Instruction(instruction) => {
                    match instruction {
                        Instruction::PtrIncrement(n) => self.state.pointer_add(n),
                        Instruction::PtrDecrement(n) => self.state.pointer_sub(n),
                        Instruction::Add(n) => {
                            self.state.add(0, n);
                        }
                        Instruction::AddTo(offset) | Instruction::Copy(offset) => {
                            let value = self.state.at_offet(0);
                            self.state.add(offset, value);
                        }
                        Instruction::AddToRev(offset) | Instruction::CopyRev(offset) => {
                            let value = self.state.at_offet(0);
                            if value != 0 {
                                self.state.add_rev(offset, value);
                            }
                        }
                        Instruction::Sub(n) => {
                            self.state.sub(0, n);
                        }
                        Instruction::SubTo(offset) => {
                            let value = self.state.at_offet(0);
                            self.state.sub(offset, value);
                        }
                        Instruction::SubToRev(offset) => {
                            let value = self.state.at_offet(0);
                            if value != 0 {
                                self.state.sub_rev(offset, value);
                            }
                        }
                        Instruction::MulAdd(offset, value) => {
                            let value = self.state.at_offet(0).wrapping_mul(value);
                            self.state.add(offset, value);
                        }
                        Instruction::MulAddRev(offset, value) => {
                            let value = self.state.at_offet(0).wrapping_mul(value);
                            if value != 0 {
                                self.state.add_rev(offset, value);
                            }
                        }
                        Instruction::Output(n) => {
                            for _ in 0..n {
                                self.state.output(&mut self.output);
                            }
                        }
                        Instruction::Input(n) => {
                            for _ in 0..n {
                                self.state.input(&mut self.input);
                            }
                        }
                        Instruction::ZeroSet => *self.state.at_offset_mut(0) = 0,
                    };
                    self.now += 1
                }
                CInstruction::WhileBegin if self.state.at_offet(0) == 0 => {
                    self.now = self.while_end_jump_table[self.now]
                }
                CInstruction::WhileBegin => self.now += 1,
                CInstruction::WhileEnd => self.now = self.while_begin_jump_table[self.now],
            }
            Some(())
        } else {
            None
        }
    }
}

pub struct InterPrinterBuilder<'a, R: Read, W: Write> {
    root_node: Option<&'a Node>,
    memory_len: usize,
    input: Option<R>,
    output: Option<W>,
}
impl<'a, R: Read, W: Write> Default for InterPrinterBuilder<'a, R, W> {
    fn default() -> Self {
        Self {
            root_node: Default::default(),
            memory_len: 1,
            input: Default::default(),
            output: Default::default(),
        }
    }
}
impl<'a, R: Read, W: Write> InterPrinterBuilder<'a, R, W> {
    pub fn root_node(self, root_node: &'a Node) -> Self {
        Self {
            root_node: Some(root_node),
            ..self
        }
    }
    pub fn memory_len(self, memory_len: usize) -> Self {
        assert!(memory_len > 0);
        Self { memory_len, ..self }
    }
    pub fn input(self, input: R) -> Self {
        Self {
            input: Some(input),
            ..self
        }
    }
    pub fn output(self, output: W) -> Self {
        Self {
            output: Some(output),
            ..self
        }
    }
    pub fn build(self) -> InterPrinter<R, W> {
        let Self {
            root_node,
            memory_len,
            input,
            output,
        } = self;

        let root_node = root_node.unwrap();
        let input = input.unwrap();
        let output = output.unwrap();

        InterPrinter::new(root_node, memory_len, input, output)
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

        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
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

        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .count();

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

        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .count();

        let output_string = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output_string, assert_mandelbrot);
    }
    #[test]
    fn test_hello_world_interprinter() {
        let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

        let root_node = Node::from_source(hello_world).unwrap();

        let mut output_buffer = vec![];

        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .count();

        let output = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output, "Hello World!\n");
    }
    #[test]
    fn test_optimized_hello_world_interprinter() {
        let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

        let root_node = Node::from_source(hello_world).unwrap();
        let root_node = optimize(root_node, &all_optimizer());

        let mut output_buffer = vec![];

        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .count();

        let output = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output, "Hello World!\n");
    }
}

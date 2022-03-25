use crate::instruction::Instruction;
use crate::parse::{ExprKind, Node, Nods};

use std::io::prelude::*;

struct State {
    pointer: usize,
    memory: Vec<u8>,
}
impl State {
    fn memory_extend(&mut self, offset: usize) {
        if self.memory.len() <= (self.pointer + offset + 1) {
            self.memory.resize(self.pointer * 2 + offset + 1, 0);
        }
    }
    fn at(&self) -> u8 {
        self.memory[self.pointer]
    }
    fn at_offset(&mut self, offset: isize) -> u8 {
        if offset < 0 {
            assert!(
                self.pointer >= (-offset as usize),
                "マイナスのインデックスを参照"
            );
            self.memory[self.pointer - (-offset as usize)]
        } else {
            self.memory_extend(offset as usize);
            self.memory[self.pointer + offset as usize]
        }
    }
    fn at_offset_mut(&mut self, offset: isize) -> &mut u8 {
        if offset < 0 {
            assert!(
                self.pointer >= (-offset as usize),
                "マイナスのインデックスを参照"
            );
            &mut self.memory[self.pointer - (-offset as usize)]
        } else {
            self.memory_extend(offset as usize);
            &mut self.memory[self.pointer + offset as usize]
        }
    }
    fn add(&mut self, offset: isize, value: u8) {
        let a = self.at_offset_mut(offset);
        *a = a.wrapping_add(value);
    }
    fn sub(&mut self, offset: isize, value: u8) {
        let a = self.at_offset_mut(offset);
        *a = a.wrapping_sub(value);
    }
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
        self.memory_extend(value); // メモリーを伸ばす
    }
    fn pointer_sub(&mut self, value: usize) {
        assert!(self.pointer >= value, "ポインターがマイナスに");
        self.pointer -= value;
    }
    fn output(&mut self, offset: isize, writer: &mut impl Write) {
        let value = self.at_offset(offset);
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

fn node_to_c_instructions(nodes: &Nods) -> Vec<CInstruction> {
    fn inner(c_instruction: &mut Vec<CInstruction>, nodes: &Nods) {
        for node in nodes {
            match node {
                crate::parse::Nod::Loop(loop_nodes) => {
                    c_instruction.push(CInstruction::WhileBegin);
                    inner(c_instruction, loop_nodes);
                    c_instruction.push(CInstruction::WhileEnd);
                }
                crate::parse::Nod::Instruction(instruction) => {
                    c_instruction.push(CInstruction::from_instruction(*instruction))
                }
            }
        }
    }

    let mut instructions = vec![];
    inner(&mut instructions, nodes);
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
    fn new(root_node: &Nods, memory_len: usize, input: R, output: W) -> Self {
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
                            let value = self.state.at();
                            self.state.add(offset as isize, value);
                        }
                        Instruction::AddToRev(offset) | Instruction::CopyRev(offset) => {
                            let value = self.state.at();
                            if value != 0 {
                                self.state.add(offset as isize, value);
                            }
                        }
                        Instruction::Sub(n) => {
                            self.state.sub(0, n);
                        }
                        Instruction::SubTo(offset) => {
                            let value = self.state.at();
                            self.state.sub(offset as isize, value);
                        }
                        Instruction::SubToRev(offset) => {
                            let value = self.state.at();
                            if value != 0 {
                                self.state.sub(offset as isize, value);
                            }
                        }
                        Instruction::MulAdd(offset, value) => {
                            let value = self.state.at().wrapping_mul(value);
                            self.state.add(offset as isize, value);
                        }
                        Instruction::MulAddRev(offset, value) => {
                            let value = self.state.at().wrapping_mul(value);
                            if value != 0 {
                                self.state.add(-(offset as isize), value);
                            }
                        }
                        Instruction::Output(n) => {
                            for _ in 0..n {
                                self.state.output(0, &mut self.output);
                            }
                        }
                        Instruction::Input(n) => {
                            for _ in 0..n {
                                self.state.input(&mut self.input);
                            }
                        }
                        Instruction::ZeroSet => *self.state.at_offset_mut(0) = 0,
                        Instruction::ZeroSetOffset(offset) => *self.state.at_offset_mut(offset) = 0,
                        Instruction::AddOffset(offset, value) => self.state.add(offset, value),
                        Instruction::SubOffset(offset, value) => self.state.sub(offset, value),
                        Instruction::OutputOffset(offset) => {
                            self.state.output(offset, &mut self.output);
                        }
                        ins => panic!("unimplemented instruction. {ins:?}"),
                    };
                    self.now += 1
                }
                CInstruction::WhileBegin if self.state.at() == 0 => {
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
    root_new_node: Option<&'a Nods>,
    memory_len: usize,
    input: Option<R>,
    output: Option<W>,
}
impl<'a, R: Read, W: Write> Default for InterPrinterBuilder<'a, R, W> {
    fn default() -> Self {
        Self {
            root_node: Default::default(),
            root_new_node: Default::default(),
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
    pub fn root_new_node(self, root_new_node: &'a Nods) -> Self {
        Self {
            root_new_node: Some(root_new_node),
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
            root_new_node,
        } = self;

        let root_new_node = root_new_node.unwrap();

        let input = input.unwrap();
        let output = output.unwrap();

        InterPrinter::new(root_new_node, memory_len, input, output)
    }
}

#[cfg(test)]
mod test {
    use std::{fs, io};

    use crate::{
        optimize::{all_optimizer, optimize},
        parse::Node,
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

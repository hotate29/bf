use crate::instruction::Instruction;
use crate::parse::Nodes;

use std::io::{self, Read, Write};

use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

struct Memory(Vec<u8>);

impl Memory {
    #[inline]
    fn extend(&mut self, index: usize) {
        if self.0.len() <= index + 1 {
            self.0.resize(self.0.len() * 2 + index + 1, 0);
        }
    }
    #[inline]
    fn inner(&self) -> &Vec<u8> {
        &self.0
    }
    #[inline]
    fn get(&mut self, index: usize) -> u8 {
        *self.get_mut(index)
    }
    #[inline]
    fn get_mut(&mut self, index: usize) -> &mut u8 {
        self.extend(index);
        &mut self.0[index]
    }
}

struct State {
    pointer: usize,
    memory: Memory,
}
impl State {
    #[inline]
    fn at(&mut self) -> u8 {
        self.memory.get(self.pointer)
    }
    #[inline]
    fn at_offset(&mut self, offset: isize) -> Result<u8> {
        self.at_offset_mut(offset).map(|v| *v)
    }
    #[inline]
    fn at_offset_mut(&mut self, offset: isize) -> Result<&mut u8> {
        let p = self.pointer as isize + offset;
        if p >= 0 {
            Ok(self.memory.get_mut(p as usize))
        } else {
            Err(Error::NegativePointer(p))
        }
    }
    #[inline]
    fn add(&mut self, offset: isize, value: u8) -> Result<()> {
        self.at_offset_mut(offset)
            .map(|a| *a = a.wrapping_add(value))
    }
    #[inline]
    fn sub(&mut self, offset: isize, value: u8) -> Result<()> {
        self.at_offset_mut(offset)
            .map(|a| *a = a.wrapping_sub(value))
    }
    #[inline]
    fn pointer_add(&mut self, value: usize) {
        self.pointer += value;
    }
    #[inline]
    fn pointer_sub(&mut self, value: usize) -> Result<()> {
        if self.pointer < value {
            Err(Error::NegativePointer(
                self.pointer as isize - value as isize,
            ))
        } else {
            self.pointer -= value;
            Ok(())
        }
    }
    #[inline]
    fn output(&mut self, offset: isize, writer: &mut impl Write) -> Result<()> {
        let value = self.at_offset(offset)?;
        writer.write_all(&[value])?;
        writer.flush()?;
        Ok(())
    }
    #[inline]
    fn input(&mut self, offset: isize, reader: &mut impl Read) -> Result<()> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        *self.at_offset_mut(offset)? = buf[0];
        Ok(())
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

fn node_to_c_instructions(nodes: &Nodes) -> Vec<CInstruction> {
    fn inner(c_instruction: &mut Vec<CInstruction>, nodes: &Nodes) {
        for node in nodes {
            match node {
                crate::parse::Node::Loop(loop_nodes) => {
                    c_instruction.push(CInstruction::WhileBegin);
                    inner(c_instruction, loop_nodes);
                    c_instruction.push(CInstruction::WhileEnd);
                }
                crate::parse::Node::Instruction(instruction) => {
                    c_instruction.push(CInstruction::from_instruction(*instruction))
                }
            }
        }
    }

    let mut instructions = vec![];
    inner(&mut instructions, nodes);
    instructions
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    NegativePointer(isize),
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
    fn new(root_node: &Nodes, memory_len: usize, input: R, output: W) -> Self {
        let state = State {
            pointer: 0,
            memory: Memory(vec![0; memory_len]),
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
        self.state.memory.inner()
    }
    pub fn pointer(&self) -> usize {
        self.state.pointer
    }
    pub fn now(&self) -> usize {
        self.now
    }
    #[inline]
    fn step(&mut self) -> Result<()> {
        if self.now < self.instructions.len() {
            match self.instructions[self.now] {
                CInstruction::Instruction(instruction) => {
                    match instruction {
                        Instruction::PtrIncrement(n) => self.state.pointer_add(n),
                        Instruction::PtrDecrement(n) => self.state.pointer_sub(n)?,
                        Instruction::Add(offset, value) => self.state.add(offset, value)?,
                        Instruction::AddValue(to_offset, value) => {
                            let value =
                                value.get_or(|offset| self.state.at_offset(offset).unwrap());
                            if value != 0 {
                                self.state.add(to_offset, value)?;
                            }
                        }
                        Instruction::AddTo(to_offset, offset) => {
                            let value = self.state.at_offset(offset)?;
                            if value != 0 {
                                self.state.add(to_offset, value)?;
                            }
                        }
                        Instruction::Sub(to_offset, value) => {
                            let value =
                                value.get_or(|offset| self.state.at_offset(offset).unwrap());
                            if value != 0 {
                                self.state.sub(to_offset, value)?;
                            }
                        }
                        Instruction::MulAdd(to_offset, offset, value) => {
                            let n = self.state.at_offset(offset)?;
                            // 後ろを参照するので、ここはちゃんと確認
                            if n != 0 {
                                self.state.add(to_offset, n.wrapping_mul(value))?;
                            }
                        }
                        Instruction::MulSub(to_offset, offset, value) => {
                            let n = self.state.at_offset(offset)?;
                            // 後ろを参照するので、ここはちゃんと確認
                            if n != 0 {
                                self.state.sub(to_offset, n.wrapping_mul(value))?;
                            }
                        }
                        Instruction::Output(offset, repeat) => {
                            for _ in 0..repeat {
                                self.state.output(offset, &mut self.output)?
                            }
                        }
                        Instruction::Input(offset, repeat) => {
                            for _ in 0..repeat {
                                self.state.input(offset, &mut self.input)?;
                            }
                        }
                        Instruction::SetValue(offset, value) => {
                            let value =
                                value.get_or(|offset| self.state.at_offset(offset).unwrap());
                            *self.state.at_offset_mut(offset)? = value;
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
        }
        Ok(())
    }
}

impl<R: Read, W: Write> Iterator for InterPrinter<R, W> {
    type Item = Result<()>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.now < self.instructions.len() {
            Some(self.step())
        } else {
            None
        }
    }
}

pub struct InterPrinterBuilder<'a, R: Read, W: Write> {
    root_node: Option<&'a Nodes>,
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
    pub fn root_node(self, root_node: &'a Nodes) -> Self {
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
        interprinter::Memory,
        optimize::optimize,
        parse::{tokenize, Node, Nodes},
    };

    use super::InterPrinter;

    fn node_from_source(source: &str) -> Nodes {
        let tokens = tokenize(source);
        Node::from_tokens(tokens).unwrap()
    }
    fn node_from_source_optimized(source: &str) -> Nodes {
        let tokens = tokenize(source);
        let nodes = Node::from_tokens(tokens).unwrap();
        optimize(&nodes)
    }

    #[test]
    fn test_memory_extend() {
        {
            let mut memory = Memory(Vec::new());
            memory.get(0); // 自動で伸びるはず...!

            assert!(!memory.0.is_empty());
        }

        {
            let mut memory = Memory(Vec::new());
            memory.get_mut(0); // 自動で伸びるはず...!2

            assert!(!memory.0.is_empty());
        }
    }

    // デバックビルドだとめちゃくちゃ時間がかかるので、デフォルトでは実行しないようになっている
    // 実行する場合は、`cargo test --release -- --ignored`
    #[test]
    #[ignore]
    fn test_interprinter_mandelbrot() {
        let mandelbrot_source = fs::read_to_string("mandelbrot.bf").unwrap();
        let assert_mandelbrot = fs::read_to_string("mandelbrot").unwrap();

        let root_node = node_from_source(&mandelbrot_source);

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

        let root_node = node_from_source_optimized(&mandelbrot_source);

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

        let root_node = node_from_source(hello_world);

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

        let root_node = node_from_source_optimized(hello_world);

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

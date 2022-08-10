use crate::instruction::Instruction;
use crate::parse::Nodes;

use std::io::{self, Read, Write};

use log::{trace, warn};
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

struct Memory(Vec<u8>);

impl Memory {
    #[inline]
    fn extend(&mut self, index: usize) {
        if self.0.len() <= index + 1 {
            let extend_len = self.0.len() * 2 + index + 1;

            trace!("extend! {} -> {}", self.0.len(), extend_len);
            self.0.resize(extend_len, 0);
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
        if &buf == b"\r" {
            warn!("\\r!!!");
        }

        *self.at_offset_mut(offset)? = buf[0];
        Ok(())
    }
}

#[derive(Debug)]
enum CInstruction {
    Instruction(Instruction),
    // 行き先
    WhileBegin(usize),
    WhileEnd(usize),
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
                    let instructions_len = c_instruction.len();
                    c_instruction.push(CInstruction::WhileBegin(0));
                    let begin_index = c_instruction.len() - 1;

                    inner(c_instruction, loop_nodes);

                    // これまでの長さ + ループ内の長さ
                    c_instruction[begin_index] = CInstruction::WhileBegin(c_instruction.len() + 1);

                    c_instruction.push(CInstruction::WhileEnd(instructions_len));
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
    #[error("I/O Error: {0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    NegativePointer(isize),
}

pub struct InterPreter<R: Read, W: Write> {
    state: State,
    input: R,
    output: W,
    instructions: Vec<CInstruction>,
    now: usize,
}
impl<R: Read, W: Write> InterPreter<R, W> {
    pub fn builder<'a>() -> InterPreterBuilder<'a, R, W> {
        InterPreterBuilder::default()
    }
    fn new(root_node: &Nodes, memory_len: usize, input: R, output: W) -> Self {
        let state = State {
            pointer: 0,
            memory: Memory(vec![0; memory_len]),
        };

        let instructions = node_to_c_instructions(root_node);

        Self {
            state,
            instructions,
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
    pub fn iter(&mut self) -> InterPreterIter<'_, R, W> {
        InterPreterIter(self)
    }
    #[inline]
    fn step(&mut self) -> Result<()> {
        if let Some(ins) = self.instructions.get(self.now) {
            match *ins {
                CInstruction::Instruction(instruction) => {
                    match instruction {
                        Instruction::PtrIncrement(n) => self.state.pointer_add(n),
                        Instruction::PtrDecrement(n) => self.state.pointer_sub(n)?,
                        Instruction::Add(to_offset, value) => {
                            let value =
                                value.get_or(|offset| self.state.at_offset(offset).unwrap());
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
                        Instruction::MulAdd(to_offset, lhs, rhs) => {
                            let n = lhs.get_or(|offset| self.state.at_offset(offset).unwrap());
                            // 後ろを参照するので、ここはちゃんと確認
                            if n != 0 {
                                let value =
                                    rhs.get_or(|offset| self.state.at_offset(offset).unwrap());
                                self.state.add(to_offset, n.wrapping_mul(value))?;
                            }
                        }
                        Instruction::MulSub(to_offset, lhs, rhs) => {
                            let n = lhs.get_or(|offset| self.state.at_offset(offset).unwrap());
                            // 後ろを参照するので、ここはちゃんと確認
                            if n != 0 {
                                let value =
                                    rhs.get_or(|offset| self.state.at_offset(offset).unwrap());
                                self.state.sub(to_offset, n.wrapping_mul(value))?;
                            }
                        }
                        Instruction::Output(offset) => {
                            self.state.output(offset, &mut self.output)?
                        }
                        Instruction::Input(offset) => {
                            self.state.input(offset, &mut self.input)?;
                        }
                        Instruction::SetValue(offset, value) => {
                            let value =
                                value.get_or(|offset| self.state.at_offset(offset).unwrap());
                            *self.state.at_offset_mut(offset)? = value;
                        }
                    };
                    self.now += 1
                }
                CInstruction::WhileBegin(to) if self.state.at() == 0 => self.now = to,
                CInstruction::WhileBegin(_) => self.now += 1,
                CInstruction::WhileEnd(to) => self.now = to,
            }
        }

        Ok(())
    }
}

impl<R: Read, W: Write> Iterator for InterPreterIter<'_, R, W> {
    type Item = Result<()>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.now < self.0.instructions.len() {
            Some(self.0.step())
        } else {
            None
        }
    }
}

pub struct InterPreterBuilder<'a, R: Read, W: Write> {
    root_node: Option<&'a Nodes>,
    memory_len: usize,
    input: Option<R>,
    output: Option<W>,
}
impl<'a, R: Read, W: Write> Default for InterPreterBuilder<'a, R, W> {
    fn default() -> Self {
        Self {
            root_node: Default::default(),
            memory_len: 1,
            input: Default::default(),
            output: Default::default(),
        }
    }
}
impl<'a, R: Read, W: Write> InterPreterBuilder<'a, R, W> {
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
    pub fn build(self) -> InterPreter<R, W> {
        let Self {
            root_node,
            memory_len,
            input,
            output,
        } = self;

        let root_node = root_node.unwrap();
        let input = input.unwrap();
        let output = output.unwrap();

        InterPreter::new(root_node, memory_len, input, output)
    }
}

pub struct InterPreterIter<'a, R: Read, W: Write>(&'a mut InterPreter<R, W>);

#[cfg(test)]
mod test {
    use std::io;

    use crate::{
        interpreter::Memory,
        optimize::optimize,
        parse::{tokenize, Node, Nodes},
    };

    use super::InterPreter;

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
    fn test_interpreter_mandelbrot() {
        let mandelbrot_source = include_str!("../../bf_codes/mandelbrot.bf");
        let assert_mandelbrot = include_str!("../../bf_codes/mandelbrot.out");

        let root_node = node_from_source(mandelbrot_source);

        let mut output_buffer = Vec::new();

        InterPreter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .iter()
            .count();

        let output_string = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output_string, assert_mandelbrot);
    }

    #[test]
    #[ignore]
    fn test_optimized_interpreter_mandelbrot() {
        let mandelbrot_source = include_str!("../../bf_codes/mandelbrot.bf");
        let assert_mandelbrot = include_str!("../../bf_codes/mandelbrot.out");

        let root_node = node_from_source_optimized(mandelbrot_source);

        let mut output_buffer = Vec::new();

        InterPreter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .iter()
            .count();

        let output_string = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output_string, assert_mandelbrot);
    }
    #[test]
    fn test_hello_world_interpreter() {
        let hello_world_code = include_str!("../../bf_codes/hello_world.bf");
        let hello_world = include_str!("../../bf_codes/hello_world.out");

        let root_node = node_from_source(hello_world_code);

        let mut output_buffer = vec![];

        InterPreter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .iter()
            .count();

        let output = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output, hello_world);
    }
    #[test]
    fn test_optimized_hello_world_interpreter() {
        let hello_world_code = include_str!("../../bf_codes/hello_world.bf");
        let hello_world = include_str!("../../bf_codes/hello_world.out");

        let root_node = node_from_source_optimized(hello_world_code);

        let mut output_buffer = vec![];

        InterPreter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(&mut output_buffer)
            .build()
            .iter()
            .count();

        let output = String::from_utf8(output_buffer).unwrap();
        assert_eq!(output, hello_world);
    }
}

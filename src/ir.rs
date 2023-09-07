use crate::parse::Ast;
use crate::transpile::wasm::wasm_binary::code::{MemoryImmediate, Op as WOp};
use crate::transpile::wasm::wasm_binary::type_::ValueType;

// offsetは負の値もとる事ができる。WebAssemblyメモリ操作命令は正のoffsetしか受け付けないので、出力時によしなにする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Op {
    Add(i32, i32),
    MovePtr(i32),
    /// Mul(to, x, offset)
    ///
    /// [ptr + to + off] += [ptr + off]*x
    Mul(i32, i32, i32),
    Set(i32, i32),
    Out(i32),
    Input(i32),
    // メモリ上の要素をx個ごとに見て、0ならループを抜ける
    Lick(i32),
}
impl Op {
    pub fn ptr(of: i32) -> Self {
        Op::MovePtr(of)
    }
    pub fn is_nop(&self) -> bool {
        matches!(self, Op::Add(0, _) | Op::Mul(_, 0, _) | Op::MovePtr(0))
    }
    pub fn map_offset(self, func: impl FnOnce(i32) -> i32) -> Option<Op> {
        match self {
            Op::Add(x, offset) => Some(Op::Add(x, func(offset))),
            Op::Mul(to, x, offset) => Some(Op::Mul(to, x, func(offset))),
            Op::Set(x, offset) => Some(Op::Set(x, func(offset))),
            Op::Out(offset) => Some(Op::Out(func(offset))),
            Op::Input(offset) => Some(Op::Input(func(offset))),
            _ => None,
        }
    }
    pub fn offset(self) -> Option<i32> {
        match self {
            Op::Add(_, offset) => Some(offset),
            Op::Mul(_, _, offset) => Some(offset),
            Op::Set(_, offset) => Some(offset),
            Op::Out(offset) => Some(offset),
            Op::Input(offset) => Some(offset),
            _ => None,
        }
    }
    fn to_wasm_ops(self, ops: &mut Vec<WOp>) {
        if let Some(offset) = self.offset() {
            if offset.is_negative() {
                panic!();
            }
        }
        match self {
            Op::Add(value, offset) => {
                let add_ops = [
                    WOp::GetLocal { local_index: 0 },
                    WOp::GetLocal { local_index: 0 },
                    WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                    WOp::I32Const(value),
                    WOp::I32Add,
                    WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
                ];

                ops.extend(add_ops);
            }
            Op::MovePtr(offset) => {
                let ptr_add_ops = [
                    WOp::GetLocal { local_index: 0 },
                    WOp::I32Const(offset),
                    WOp::I32Add,
                    WOp::SetLocal { local_index: 0 },
                ];

                ops.extend(ptr_add_ops);
            }
            Op::Mul(x, y, offset) => {
                if y == 1 {
                    let mul_ops = [
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Const(x),
                        WOp::I32Add,
                        WOp::TeeLocal { local_index: 1 },
                        WOp::GetLocal { local_index: 1 },
                        WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                        WOp::I32Add,
                        WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
                    ];
                    ops.extend(mul_ops);
                } else if y == -1 {
                    let mul_ops = [
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Const(x),
                        WOp::I32Add,
                        WOp::TeeLocal { local_index: 1 },
                        WOp::GetLocal { local_index: 1 },
                        WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                        WOp::I32Sub,
                        WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
                    ];
                    ops.extend(mul_ops);
                } else {
                    let mul_ops = [
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Const(x),
                        WOp::I32Add,
                        WOp::TeeLocal { local_index: 1 },
                        WOp::GetLocal { local_index: 1 },
                        WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                        WOp::I32Const(y),
                        WOp::I32Mul,
                        WOp::I32Add,
                        WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
                    ];
                    ops.extend(mul_ops);
                }
            }
            Op::Set(value, offset) => {
                let clear_ops = [
                    WOp::GetLocal { local_index: 0 },
                    WOp::I32Const(value),
                    WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
                ];

                ops.extend(clear_ops);
            }
            Op::Out(offset) => {
                let out_ops = [
                    WOp::GetLocal { local_index: 0 },
                    WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                    WOp::Call { function_index: 2 },
                ];

                ops.extend(out_ops);
            }
            Op::Input(offset) => {
                let input_ops = [
                    WOp::GetLocal { local_index: 0 },
                    WOp::Call { function_index: 3 },
                    WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
                ];

                ops.extend(input_ops)
            }
            Op::Lick(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockItem {
    Op(Op),
    Loop(Block),
    If(Block),
}
impl BlockItem {
    pub fn is_block(&self) -> bool {
        matches!(self, BlockItem::Loop(_) | BlockItem::If(_))
    }
    pub fn op(&self) -> Option<Op> {
        match self {
            BlockItem::Op(op) => Some(*op),
            _ => None,
        }
    }
    pub fn map_block(&self, func: impl FnOnce(&Block) -> Block) -> Option<Self> {
        match self {
            BlockItem::Loop(block) => Some(BlockItem::Loop(func(block))),
            BlockItem::If(block) => Some(BlockItem::If(func(block))),
            _ => None,
        }
    }
    pub fn map_op(&self, func: impl FnOnce(Op) -> Op) -> Option<Self> {
        match self {
            BlockItem::Op(op) => Some(BlockItem::Op(func(*op))),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

impl From<&[Ast]> for Block {
    fn from(ast: &[Ast]) -> Self {
        Block::from_items(
            ast.iter()
                .filter_map(|item| match item {
                    Ast::PtrInc => Some(BlockItem::Op(Op::ptr(1))),
                    Ast::PtrDec => Some(BlockItem::Op(Op::ptr(-1))),
                    Ast::Inc => Some(BlockItem::Op(Op::Add(1, 0))),
                    Ast::Dec => Some(BlockItem::Op(Op::Add(-1, 0))),
                    Ast::Read => Some(BlockItem::Op(Op::Input(0))),
                    Ast::Write => Some(BlockItem::Op(Op::Out(0))),
                    Ast::Loop(loop_items) => Some(BlockItem::Loop(loop_items.as_slice().into())),
                    Ast::_Invalid => None,
                })
                .collect(),
        )
    }
}

impl Block {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_items(items: Vec<BlockItem>) -> Self {
        Self { items }
    }
    pub fn push_item(&mut self, item: BlockItem) {
        self.items.push(item)
    }
    pub fn from_ast(ast: &[Ast]) -> Self {
        Self::from(ast)
    }
    pub(crate) fn to_wasm_ops(&self, ops: &mut Vec<WOp>) {
        for item in &self.items {
            match item {
                BlockItem::Op(op) => {
                    op.to_wasm_ops(ops);
                }
                BlockItem::Loop(loop_block) => {
                    let loop_ops = [
                        WOp::Loop {
                            block_type: ValueType::Void,
                        },
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Load8U(MemoryImmediate::i8(0)),
                        WOp::If {
                            block_type: ValueType::Void,
                        },
                    ];

                    ops.extend(loop_ops);

                    loop_block.to_wasm_ops(ops);

                    let loop_ops = [WOp::Br { relative_depth: 1 }, WOp::End, WOp::End];

                    ops.extend(loop_ops);
                }
                BlockItem::If(if_block) => {
                    let if_ops = [
                        WOp::GetLocal { local_index: 0 },
                        WOp::I32Load8U(MemoryImmediate::i8(0)),
                        WOp::If {
                            block_type: ValueType::Void,
                        },
                    ];

                    ops.extend(if_ops);

                    if_block.to_wasm_ops(ops);

                    ops.push(WOp::End);
                }
            }
        }
    }
}

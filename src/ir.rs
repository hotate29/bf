use crate::parse::Ast;

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
}

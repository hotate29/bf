use crate::parse::Ast;
use crate::transpile::wasm::wasm_binary::code::{MemoryImmediate, Op as WOp};
use crate::transpile::wasm::wasm_binary::type_::Type;
use crate::transpile::wasm::wasm_binary::var::Var;

// WebAssemblyのメモリ操作命令に付いているoffsetを使いたいので、offsetは正の整数のみ受け入れるようにしている。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op<T = u32> {
    Add(u32, T),
    Sub(u32, T),
    PtrAdd(u32),
    PtrSub(u32),
    /// Mul(to, x, offset)
    ///
    /// [ptr + to + off] += [ptr + off]*x
    Mul(i32, i32, T),
    Set(i32, T),
    Out(T),
    Input(T),
}
impl<T> Op<T> {
    pub fn ptr(of: i32) -> Self {
        if of < 0 {
            Op::PtrSub(-of as u32)
        } else {
            Op::PtrAdd(of as u32)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockItem {
    Op(Op),
    Loop(Block),
    If(Block),
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
                    Ast::PtrInc => Some(BlockItem::Op(Op::PtrAdd(1))),
                    Ast::PtrDec => Some(BlockItem::Op(Op::PtrSub(1))),
                    Ast::Inc => Some(BlockItem::Op(Op::Add(1, 0))),
                    Ast::Dec => Some(BlockItem::Op(Op::Sub(1, 0))),
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
                BlockItem::Op(op) => match op {
                    Op::Add(value, offset) => {
                        let add_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::i8(*offset)),
                            WOp::I32Const(Var(*value as i32)),
                            WOp::I32Add,
                            WOp::I32Store8(MemoryImmediate::i8(*offset)),
                        ];

                        ops.extend(add_ops);
                    }
                    Op::Sub(value, offset) => {
                        // Addと大体おなじ
                        let sub_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::i8(*offset)),
                            WOp::I32Const(Var(*value as i32)),
                            WOp::I32Sub,
                            WOp::I32Store8(MemoryImmediate::i8(*offset)),
                        ];

                        ops.extend(sub_ops);
                    }
                    Op::PtrAdd(value) => {
                        let ptr_add_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Const(Var(*value as i32)),
                            WOp::I32Add,
                            WOp::SetLocal {
                                local_index: Var(0),
                            },
                        ];

                        ops.extend(ptr_add_ops);
                    }
                    Op::PtrSub(value) => {
                        let ptr_sub_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Const(Var(*value as i32)),
                            WOp::I32Sub,
                            WOp::SetLocal {
                                local_index: Var(0),
                            },
                        ];

                        ops.extend(ptr_sub_ops);
                    }
                    Op::Mul(x, y, offset) => {
                        let mul_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Const(Var(*x)),
                            WOp::I32Add,
                            WOp::TeeLocal {
                                local_index: Var(1),
                            },
                            WOp::GetLocal {
                                local_index: Var(1),
                            },
                            WOp::I32Load8U(MemoryImmediate::i8(*offset)),
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::i8(*offset)),
                            WOp::I32Const(Var(*y)),
                            WOp::I32Mul,
                            WOp::I32Add,
                            WOp::I32Store8(MemoryImmediate::i8(*offset)),
                        ];

                        ops.extend(mul_ops);
                    }
                    Op::Set(value, offset) => {
                        let clear_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Const(Var(*value)),
                            WOp::I32Store8(MemoryImmediate::i8(*offset)),
                        ];

                        ops.extend(clear_ops);
                    }
                    Op::Out(offset) => {
                        let out_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::i8(*offset)),
                            WOp::Call {
                                function_index: Var(2),
                            },
                        ];

                        ops.extend(out_ops);
                    }
                    Op::Input(offset) => {
                        let input_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::Call {
                                function_index: Var(3),
                            },
                            WOp::I32Store8(MemoryImmediate::i8(*offset)),
                        ];

                        ops.extend(input_ops)
                    }
                },
                BlockItem::Loop(loop_block) => {
                    let loop_ops = [
                        WOp::Loop {
                            block_type: Type::Void,
                        },
                        WOp::GetLocal {
                            local_index: Var(0),
                        },
                        WOp::I32Load8U(MemoryImmediate::i8(0)),
                        WOp::If {
                            block_type: Type::Void,
                        },
                    ];

                    ops.extend(loop_ops);

                    loop_block.to_wasm_ops(ops);

                    let loop_ops = [
                        WOp::Br {
                            relative_depth: Var(1),
                        },
                        WOp::End,
                        WOp::End,
                    ];

                    ops.extend(loop_ops);
                }
                BlockItem::If(if_block) => {
                    let if_ops = [
                        WOp::GetLocal {
                            local_index: Var(0),
                        },
                        WOp::I32Load8U(MemoryImmediate::i8(0)),
                        WOp::If {
                            block_type: Type::Void,
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

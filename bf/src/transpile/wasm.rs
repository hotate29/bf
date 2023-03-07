use std::io;

mod opt;
mod wasm_binary;

use wasm_binary::{
    code::{FunctionBody, LocalEntry, MemoryImmediate, Op as WOp, OpSlice},
    section::{MemoryType, ResizableLimits},
    type_::Type,
    var::Var,
    Function, Import, Memory, ModuleBuilder,
};

use crate::{error::Error, parse::Ast};

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
    fn ptr(of: i32) -> Self {
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

impl From<Ast> for Block {
    fn from(ast: Ast) -> Self {
        (&ast).into()
    }
}

impl From<&Ast> for Block {
    fn from(ast: &Ast) -> Self {
        fn inner(block: &mut Block, ast: &Ast) {
            for item in ast.inner() {
                let blockitem = match item {
                    crate::parse::Item::Op(op) => {
                        let op = match op {
                            crate::parse::Op::Add => Op::Add(1, 0),
                            crate::parse::Op::Sub => Op::Sub(1, 0),
                            crate::parse::Op::PtrAdd => Op::PtrAdd(1),
                            crate::parse::Op::PtrSub => Op::PtrSub(1),
                            crate::parse::Op::Output => Op::Out(0),
                            crate::parse::Op::Input => Op::Input(0),
                        };
                        BlockItem::Op(op)
                    }
                    crate::parse::Item::Loop(ast) => BlockItem::Loop(ast.into()),
                };

                block.push_item(blockitem);
            }
        }

        let capacity = ast.inner().len();
        let mut block = Block::from_items(Vec::with_capacity(capacity));

        inner(&mut block, ast);

        block
    }
}

impl Block {
    fn new() -> Self {
        Self::default()
    }
    pub fn from_bf(bf: &str) -> Result<Self, Error> {
        let ast: Ast = bf.parse()?;
        Ok(ast.into())
    }
    fn from_items(items: Vec<BlockItem>) -> Self {
        Self { items }
    }
    fn push_item(&mut self, item: BlockItem) {
        self.items.push(item)
    }
    pub fn optimize(&self, is_top_level: bool) -> Block {
        let mut block = self.clone();

        if is_top_level {
            block.items.insert(0, BlockItem::Op(Op::Set(0, 0)));
        }

        let mut block = opt::merge(&block);

        opt::unwrap(&mut block);
        opt::clear(&mut block);
        opt::mul(&mut block);
        let mut block = opt::merge(&block);
        opt::if_opt(&mut block);
        opt::offset_opt(&block)
    }
    fn to_wasm_ops(&self, ops: &mut Vec<WOp>) {
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

pub fn bf_to_wasm(bf: &str, optimize: bool, mut w: impl io::Write) -> Result<(), Error> {
    let mut block = Block::from_bf(bf)?;
    if optimize {
        block = block.optimize(optimize);
    }

    to_wasm(&block, &mut w)?;
    Ok(())
}

pub fn bf_to_wat(bf: &str, optimize: bool, mut w: impl io::Write) -> Result<(), Error> {
    let mut block = Block::from_bf(bf)?;
    if optimize {
        block = block.optimize(optimize);
    }

    to_wat(&block, &mut w)?;
    Ok(())
}

pub fn to_wat(block: &Block, mut out: impl io::Write) -> io::Result<()> {
    // Base Wasmer
    // https://github.com/wasmerio/wasmer/blob/75a98ab171bee010b9a7cd0f836919dc4519dcaf/lib/wasi/tests/stdio.rs
    writeln!(
        out,
        r#"(module
    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_unstable" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (memory (export "memory") 1)
    (func $print_char (param $char i32)
    i32.const 0
    local.get $char
    i32.store8

    ;; Creating a new io vector within linear memory
    (i32.store (i32.const 4) (i32.const 0))  ;; iov.iov_base - This is a pointer to the start of the 'hello world\n' string
    (i32.store (i32.const 8) (i32.const 1))  ;; iov.iov_len - The length of the 'hello world\n' string

    (call $fd_write
        (i32.const 1) ;; file_descriptor - 1 for stdout
        (i32.const 4) ;; *iovs - The pointer to the iov array, which is stored at memory location 0
        (i32.const 1) ;; iovs_len - We're printing 1 string stored in an iov - so one.
        (i32.const 12) ;; nwritten - A place in memory to store the number of bytes written
    )
    drop ;; Discard the number of bytes written from the top of the stack
    )
    (func $input_char (result i32)
        (i32.store (i32.const 4) (i32.const 0))
        (i32.store (i32.const 8) (i32.const 1))

        (call $fd_read
            (i32.const 0)
            (i32.const 4)
            (i32.const 1)
            (i32.const 12)
        )
        drop

        i32.const 0
        i32.load8_u
    )
    (func $main (export "_start") (local i32 i32)"#,
    )?;

    let mut main = Vec::new();
    main.extend([
        WOp::I32Const(Var(40)),
        WOp::SetLocal {
            local_index: Var(0),
        },
    ]);
    block.to_wasm_ops(&mut main);
    // テキスト形式だといらない
    // ops.push(WOp::End);

    main.write_str(2, &mut out)?;
    writeln!(
        out,
"    )
)"
    )

    // let body = block.to_wat(40);
}

fn print_char(wd_write_index: Var<u32>) -> Function {
    let print_char_ops = [
        WOp::I32Const(Var(0)),
        WOp::GetLocal {
            local_index: Var(0),
        },
        WOp::I32Store8(MemoryImmediate::i8(0)),
        WOp::I32Const(Var(4)),
        WOp::I32Const(Var(0)),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(Var(8)),
        WOp::I32Const(Var(1)),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(Var(1)),
        WOp::I32Const(Var(4)),
        WOp::I32Const(Var(1)),
        WOp::I32Const(Var(12)),
        WOp::Call {
            function_index: wd_write_index,
        },
        WOp::Drop,
        WOp::End,
    ];

    Function {
        signature: Type::Func {
            params: vec![Type::I32],
            result: None,
        },
        body: FunctionBody::from_ops(print_char_ops.to_vec()),
        export_name: None,
    }
}

fn input_char(fd_read_index: Var<u32>) -> Function {
    let input_char_ops = [
        WOp::I32Const(Var(4)),
        WOp::I32Const(Var(0)),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(Var(8)),
        WOp::I32Const(Var(1)),
        WOp::I32Store8(MemoryImmediate::i8(0)),
        WOp::I32Const(Var(0)),
        WOp::I32Const(Var(4)),
        WOp::I32Const(Var(1)),
        WOp::I32Const(Var(12)),
        WOp::Call {
            function_index: fd_read_index,
        },
        WOp::Drop,
        WOp::I32Const(Var(0)),
        WOp::I32Load8U(MemoryImmediate::i8(0)),
        WOp::End,
    ];

    Function {
        signature: Type::Func {
            params: vec![],
            result: Some(Box::new(Type::I32)),
        },
        body: FunctionBody::from_ops(input_char_ops.to_vec()),
        export_name: None,
    }
}

pub fn to_wasm(block: &Block, mut buffer: impl io::Write) -> io::Result<()> {
    let mut module_builder = ModuleBuilder::new(Memory {
        mem_type: MemoryType {
            limits: ResizableLimits {
                initial: Var(1),
                maximum: None,
            },
        },
        export_name: Some("memory".to_string()),
    });

    let import_fd_write = Import::Function {
        module_name: "wasi_unstable".to_string(),
        field_name: "fd_write".to_string(),
        signature: Type::Func {
            params: vec![Type::I32, Type::I32, Type::I32, Type::I32],
            result: Some(Box::new(Type::I32)),
        },
    };

    let fd_write = module_builder.push_import(import_fd_write);

    let import_fd_read = Import::Function {
        module_name: "wasi_unstable".to_string(),
        field_name: "fd_read".to_string(),
        signature: Type::Func {
            params: vec![Type::I32, Type::I32, Type::I32, Type::I32],
            result: Some(Box::new(Type::I32)),
        },
    };

    let fd_read = module_builder.push_import(import_fd_read);

    module_builder.push_function(print_char(fd_write));
    module_builder.push_function(input_char(fd_read));

    let mut main = Function {
        signature: Type::Func {
            params: vec![],
            result: None,
        },
        body: FunctionBody::new(),
        export_name: Some("_start".to_string()),
    };

    let ptr = LocalEntry {
        count: Var(2),
        type_: Type::I32,
    };
    main.push_local(ptr);

    // ポインタの初期値を40に設定する。40未満はI/Oで使うために確保する。
    // 40未満をいじった場合の動作は未定義（I/O関連がこわれるかも？）
    main.body.code.extend([
        WOp::I32Const(Var(40)),
        WOp::SetLocal {
            local_index: Var(0),
        },
    ]);
    block.to_wasm_ops(&mut main.body.code);
    main.body.code.push(WOp::End);

    module_builder.push_function(main);

    let module = module_builder.into_module();
    module.write(&mut buffer)
}

#[cfg(target_arch = "wasm32")]
pub mod w {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn bf_to_wasm(bf: &str) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();

        super::bf_to_wasm(bf, true, &mut buffer).map_err(|e| e.to_string())?;
        Ok(buffer)
    }
}

use std::io;

pub mod wasm_binary;

use wasm_binary::{
    code::{FunctionBody, LocalEntry, MemoryImmediate, Op as WOp, OpSlice},
    section::{MemoryType, ResizableLimits},
    Function, Import, Memory, ModuleBuilder,
};

use crate::ir::{Block, BlockItem, Op};

use self::wasm_binary::type_::{FuncSignature, ValueType};

fn op_to_wop(op: Op, wops: &mut Vec<WOp>) {
    if let Some(offset) = op.offset() {
        if offset.is_negative() {
            panic!();
        }
    }
    match op {
        Op::Add(value, offset) => {
            let add_ops = [
                WOp::GetLocal { local_index: 0 },
                WOp::GetLocal { local_index: 0 },
                WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                WOp::I32Const(value),
                WOp::I32Add,
                WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
            ];

            wops.extend(add_ops);
        }
        Op::MovePtr(offset) => {
            let ptr_add_ops = [
                WOp::GetLocal { local_index: 0 },
                WOp::I32Const(offset),
                WOp::I32Add,
                WOp::SetLocal { local_index: 0 },
            ];

            wops.extend(ptr_add_ops);
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
                wops.extend(mul_ops);
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
                wops.extend(mul_ops);
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
                wops.extend(mul_ops);
            }
        }
        Op::Set(value, offset) => {
            let clear_ops = [
                WOp::GetLocal { local_index: 0 },
                WOp::I32Const(value),
                WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
            ];

            wops.extend(clear_ops);
        }
        Op::Out(offset) => {
            let out_ops = [
                WOp::GetLocal { local_index: 0 },
                WOp::I32Load8U(MemoryImmediate::i8(offset as u32)),
                WOp::Call { function_index: 2 },
            ];

            wops.extend(out_ops);
        }
        Op::Input(offset) => {
            let input_ops = [
                WOp::GetLocal { local_index: 0 },
                WOp::Call { function_index: 3 },
                WOp::I32Store8(MemoryImmediate::i8(offset as u32)),
            ];

            wops.extend(input_ops)
        }
        Op::Lick(_) => unimplemented!(),
    }
}

fn block_to_wop(block: &Block, wops: &mut Vec<WOp>) {
    for item in &block.items {
        match item {
            BlockItem::Op(op) => {
                op_to_wop(*op, wops);
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

                wops.extend(loop_ops);

                block_to_wop(loop_block, wops);

                let loop_ops = [WOp::Br { relative_depth: 1 }, WOp::End, WOp::End];

                wops.extend(loop_ops);
            }
            BlockItem::If(if_block) => {
                let if_ops = [
                    WOp::GetLocal { local_index: 0 },
                    WOp::I32Load8U(MemoryImmediate::i8(0)),
                    WOp::If {
                        block_type: ValueType::Void,
                    },
                ];

                wops.extend(if_ops);

                block_to_wop(if_block, wops);

                wops.push(WOp::End);
            }
        }
    }
}

pub fn block_to_wat(block: &Block, mut out: impl io::Write) -> io::Result<()> {
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
    main.extend([WOp::I32Const(40), WOp::SetLocal { local_index: 0 }]);
    block_to_wop(block, &mut main);
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

fn print_char(wd_write_index: u32) -> Function {
    let print_char_ops = [
        WOp::I32Const(0),
        WOp::GetLocal { local_index: 0 },
        WOp::I32Store8(MemoryImmediate::i8(0)),
        WOp::I32Const(4),
        WOp::I32Const(0),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(8),
        WOp::I32Const(1),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(1),
        WOp::I32Const(4),
        WOp::I32Const(1),
        WOp::I32Const(12),
        WOp::Call {
            function_index: wd_write_index,
        },
        WOp::Drop,
        WOp::End,
    ];

    Function {
        signature: FuncSignature {
            params: vec![ValueType::I32],
            result: None,
        },
        body: FunctionBody::from_ops(print_char_ops.to_vec()),
        export_name: None,
    }
}

fn input_char(fd_read_index: u32) -> Function {
    let input_char_ops = [
        WOp::I32Const(4),
        WOp::I32Const(0),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(8),
        WOp::I32Const(1),
        WOp::I32Store8(MemoryImmediate::i8(0)),
        WOp::I32Const(0),
        WOp::I32Const(4),
        WOp::I32Const(1),
        WOp::I32Const(12),
        WOp::Call {
            function_index: fd_read_index,
        },
        WOp::Drop,
        WOp::I32Const(0),
        WOp::I32Load8U(MemoryImmediate::i8(0)),
        WOp::End,
    ];

    Function {
        signature: FuncSignature {
            params: vec![],
            result: Some(ValueType::I32),
        },
        body: FunctionBody::from_ops(input_char_ops.to_vec()),
        export_name: None,
    }
}

pub fn block_to_wasm(block: &Block, mut buffer: impl io::Write) -> io::Result<()> {
    let mut module_builder = ModuleBuilder::new(Memory {
        mem_type: MemoryType {
            limits: ResizableLimits {
                initial: 1,
                maximum: None,
            },
        },
        export_name: Some("memory".to_string()),
    });

    let import_fd_write = Import::Function {
        module_name: "wasi_unstable".to_string(),
        field_name: "fd_write".to_string(),
        signature: FuncSignature {
            params: vec![
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
            ],
            result: Some(ValueType::I32),
        },
    };

    let fd_write = module_builder.push_import(import_fd_write);

    let import_fd_read = Import::Function {
        module_name: "wasi_unstable".to_string(),
        field_name: "fd_read".to_string(),
        signature: FuncSignature {
            params: vec![
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
            ],
            result: Some(ValueType::I32),
        },
    };

    let fd_read = module_builder.push_import(import_fd_read);

    module_builder.push_function(print_char(fd_write));
    module_builder.push_function(input_char(fd_read));

    let mut main = Function {
        signature: FuncSignature {
            params: vec![],
            result: None,
        },
        body: FunctionBody::new(),
        export_name: Some("_start".to_string()),
    };

    let ptr = LocalEntry {
        count: 2,
        type_: ValueType::I32,
    };
    main.push_local(ptr);

    // ポインタの初期値を40に設定する。40未満はI/Oで使うために確保する。
    // 40未満をいじった場合の動作は未定義（I/O関連がこわれるかも？）
    main.body
        .code
        .extend([WOp::I32Const(40), WOp::SetLocal { local_index: 0 }]);

    block_to_wop(block, &mut main.body.code);

    main.body.code.push(WOp::End);

    module_builder.push_function(main);

    let module = module_builder.into_module();
    module.write(&mut buffer)
}

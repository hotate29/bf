use std::{fmt::Write, str::Chars};

mod opt;
mod wasm_binary;

use wasm_binary::code::{Op as WOp, OpSlice};
use wasm_binary::type_::Type;
use wasm_binary::var::Var;
use wasm_binary::{Function, Import};

use crate::transcompile::wasm::wasm_binary::code::{FunctionBody, LocalEntry, MemoryImmediate};
use crate::transcompile::wasm::wasm_binary::section::{MemoryType, ResizableLimits};
use crate::transcompile::wasm::wasm_binary::{Memory, ModuleBuilder};

#[derive(Debug, Clone, Copy)]
enum Op {
    Add(u32),
    Sub(u32),
    PtrAdd(u32),
    PtrSub(u32),
    Mul(i32, i32),
    Clear,
    Out,
    Input,
}
impl Op {
    fn ptr(of: i32) -> Self {
        if of <= 0 {
            Op::PtrSub(-of as u32)
        } else {
            Op::PtrAdd(of as u32)
        }
    }
}

#[derive(Debug, Clone)]
enum BlockItem {
    Op(Op),
    Loop(Block),
}

#[derive(Debug, Clone, Default)]
struct Block {
    items: Vec<BlockItem>,
}

impl Block {
    fn new() -> Self {
        Self::default()
    }
    fn push_item(&mut self, item: BlockItem) {
        self.items.push(item)
    }
    pub fn optimize(&self) -> Block {
        let mut block = opt::merge(self);

        opt::unwrap(&mut block);
        let block = opt::clear(&block);
        let block = opt::mul(&block);
        opt::merge(&block)
    }
    fn to_wat(&self, memory_base_address: i32) -> String {
        fn block_to_wat(
            block: &Block,
            wat: &mut String,
            loop_stack: &mut Vec<u32>,
            mut loop_count: u32,
        ) {
            for item in &block.items {
                match item {
                    BlockItem::Op(op) => match op {
                        Op::Add(n) => {
                            writeln!(
                                wat,
                                "
                                ;; Add
                                local.get $pointer
                                local.get $pointer
                                i32.load8_u
                                i32.const {n}
                                i32.add
                                i32.store8"
                            )
                            .unwrap();
                        }
                        Op::Sub(n) => {
                            writeln!(
                                wat,
                                "
                                ;; Sub
                                local.get $pointer
                                local.get $pointer
                                i32.load8_u
                                i32.const {n}
                                i32.sub
                                i32.store8"
                            )
                            .unwrap();
                        }
                        Op::PtrAdd(n) => {
                            writeln!(
                                wat,
                                "
                                ;; Pointer Add
                                local.get $pointer
                                i32.const {n}
                                i32.add
                                local.set $pointer"
                            )
                            .unwrap();
                        }
                        Op::PtrSub(n) => {
                            writeln!(
                                wat,
                                "
                                ;; Pointer Sub
                                local.get $pointer
                                i32.const {n}
                                i32.sub
                                local.set $pointer"
                            )
                            .unwrap();
                        }
                        Op::Mul(of, x) => {
                            writeln!(
                                wat,
                                "
                                ;; Mul
                                local.get $pointer
                                i32.load8_u

                                (if (i32.ne (i32.const 0))
                                    (then
                                        local.get $pointer
                                        i32.const {of}
                                        i32.add
                                        local.set $ptr

                                        local.get $ptr

                                        local.get $ptr
                                        i32.load8_u

                                        local.get $pointer
                                        i32.load8_u
                                        i32.const {x}
                                        i32.mul

                                        i32.add
                                        i32.store8
                                    )
                                )
                            "
                            )
                            .unwrap();
                        }
                        Op::Clear => {
                            writeln!(
                                *wat,
                                "
                                ;; Clear
                                local.get $pointer
                                i32.const 0
                                i32.store8"
                            )
                            .unwrap();
                        }
                        Op::Out => {
                            writeln!(
                                wat,
                                "
                                ;; Out
                                local.get $pointer
                                i32.load8_u
                                call $print_char"
                            )
                            .unwrap();
                        }
                        Op::Input => {
                            writeln!(
                                wat,
                                "
                                ;; Input
                                local.get $pointer
                                call $input_char"
                            )
                            .unwrap();
                        }
                    },
                    BlockItem::Loop(block) => {
                        loop_stack.push(loop_count);

                        let loop_label = format!("loop_{loop_count}");
                        let exit_label = format!("exit_{loop_count}");
                        writeln!(
                            wat,
                            "
                            (block ${exit_label}
                                    (loop ${loop_label}
                                        local.get $pointer
                                        i32.load8_u

                                        (br_if ${exit_label} (i32.eqz))\n
                            "
                        )
                        .unwrap();

                        loop_count += 1;
                        block_to_wat(block, wat, loop_stack, loop_count);

                        loop_stack.pop().unwrap();

                        writeln!(wat, "(br ${loop_label})))").unwrap();
                    }
                }
            }
        }

        let mut wat = String::new();

        writeln!(
            wat,
            "(local $pointer i32) (local $ptr i32) i32.const {memory_base_address} local.set $pointer"
        )
        .unwrap();

        let mut loop_stack = vec![];

        block_to_wat(self, &mut wat, &mut loop_stack, 0);

        wat
    }
    fn to_wasm(&self) -> Vec<u8> {
        // ぐぬぬ
        fn block_to_wasm(block: &Block, mut buffer: &mut Vec<u8>) {
            for item in &block.items {
                match item {
                    BlockItem::Op(op) => match op {
                        Op::Add(value) => {
                            let add_ops = [
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Load8U(MemoryImmediate::zero()),
                                WOp::I32Const(Var(*value as i32)),
                                WOp::I32Add,
                                WOp::I32Store8(MemoryImmediate::zero()),
                            ];

                            add_ops.write(&mut buffer).unwrap();
                        }
                        Op::Sub(value) => {
                            // Addと大体おなじ
                            let sub_ops = [
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Load8U(MemoryImmediate::zero()),
                                WOp::I32Const(Var(*value as i32)),
                                WOp::I32Sub,
                                WOp::I32Store8(MemoryImmediate::zero()),
                            ];

                            sub_ops.write(&mut buffer).unwrap();
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

                            ptr_add_ops.write(&mut buffer).unwrap();
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

                            ptr_sub_ops.write(&mut buffer).unwrap();
                        }
                        Op::Mul(of, x) => {
                            let mul_ops = [
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Load8U(MemoryImmediate::zero()),
                                WOp::If {
                                    block_type: Type::Void,
                                },
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Const(Var(*of as i32)),
                                WOp::I32Add,
                                WOp::SetLocal {
                                    local_index: Var(1),
                                },
                                WOp::GetLocal {
                                    local_index: Var(1),
                                },
                                WOp::GetLocal {
                                    local_index: Var(1),
                                },
                                WOp::I32Load8U(MemoryImmediate::zero()),
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Load8U(MemoryImmediate::zero()),
                                WOp::I32Const(Var(*x as i32)),
                                WOp::I32Mul,
                                WOp::I32Add,
                                WOp::I32Store8(MemoryImmediate::zero()),
                                WOp::End,
                            ];

                            mul_ops.write(&mut buffer).unwrap();
                        }
                        Op::Clear => {
                            let clear_ops = [
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Const(Var(0)),
                                WOp::I32Store8(MemoryImmediate::zero()),
                            ];

                            clear_ops.write(&mut buffer).unwrap();
                        }
                        Op::Out => {
                            let out_ops = [
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::I32Load8U(MemoryImmediate::zero()),
                                WOp::Call {
                                    function_index: Var(2),
                                },
                            ];

                            out_ops.write(&mut buffer).unwrap();
                        }
                        Op::Input => {
                            let input_ops = [
                                WOp::GetLocal {
                                    local_index: Var(0),
                                },
                                WOp::Call {
                                    function_index: Var(3),
                                },
                            ];

                            input_ops.write(&mut buffer).unwrap()
                        }
                    },
                    BlockItem::Loop(loop_block) => {
                        let loop_ops = [
                            WOp::Block {
                                block_type: Type::Void,
                            },
                            WOp::Loop {
                                block_type: Type::Void,
                            },
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::zero()),
                            WOp::I32Eqz,
                            WOp::BrIf {
                                relative_depth: Var(1),
                            },
                        ];

                        loop_ops.write(&mut buffer).unwrap();

                        block_to_wasm(loop_block, buffer);

                        let loop_ops = [
                            WOp::Br {
                                relative_depth: Var(0),
                            },
                            WOp::End,
                            WOp::End,
                        ];

                        loop_ops.write(&mut buffer).unwrap();
                    }
                }
            }
        }

        let mut module_builder = ModuleBuilder::new(Memory {
            mem_type: MemoryType {
                limits: ResizableLimits {
                    flags: Var(false),
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

        module_builder.push_import(import_fd_write);

        let import_fd_read = Import::Function {
            module_name: "wasi_unstable".to_string(),
            field_name: "fd_read".to_string(),
            signature: Type::Func {
                params: vec![Type::I32, Type::I32, Type::I32, Type::I32],
                result: Some(Box::new(Type::I32)),
            },
        };

        module_builder.push_import(import_fd_read);

        let mut print_char = FunctionBody::new();

        let print_char_ops = [
            WOp::I32Const(Var(0)),
            WOp::GetLocal {
                local_index: Var(0),
            },
            WOp::I32Store8(MemoryImmediate::zero()),
            WOp::I32Const(Var(4)),
            WOp::I32Const(Var(0)),
            WOp::I32Store(MemoryImmediate::i32()),
            WOp::I32Const(Var(8)),
            WOp::I32Const(Var(1)),
            WOp::I32Store(MemoryImmediate::i32()),
            WOp::I32Const(Var(1)),
            WOp::I32Const(Var(4)),
            WOp::I32Const(Var(1)),
            WOp::I32Const(Var(12)),
            WOp::Call {
                function_index: Var(0),
            },
            WOp::Drop,
            WOp::End,
        ];
        print_char_ops.write(&mut print_char.code).unwrap();

        let print_char = Function {
            signature: Type::Func {
                params: vec![Type::I32],
                result: None,
            },
            body: print_char,
            export_name: None,
        };

        module_builder.push_function(print_char);

        let mut input_char = FunctionBody::new();

        let input_char_ops = [
            WOp::I32Const(Var(4)),
            WOp::GetLocal {
                local_index: Var(0),
            },
            WOp::I32Store(MemoryImmediate::i32()),
            WOp::I32Const(Var(8)),
            WOp::I32Const(Var(1)),
            WOp::I32Store8(MemoryImmediate::zero()),
            WOp::I32Const(Var(0)),
            WOp::I32Const(Var(4)),
            WOp::I32Const(Var(1)),
            WOp::I32Const(Var(12)),
            WOp::Call {
                function_index: Var(1),
            },
            WOp::Drop,
            WOp::End,
        ];
        input_char_ops.write(&mut input_char.code).unwrap();

        let input_char = Function {
            signature: Type::Func {
                params: vec![Type::I32],
                result: None,
            },
            body: input_char,
            export_name: None,
        };

        module_builder.push_function(input_char);

        let mut main = FunctionBody::new();

        main.push_local(LocalEntry {
            count: Var(2),
            type_: Type::I32,
        });

        [
            WOp::I32Const(Var(40)),
            WOp::SetLocal {
                local_index: Var(0),
            },
        ]
        .write(&mut main.code)
        .unwrap();

        block_to_wasm(self, &mut main.code);

        WOp::End.write(&mut main.code).unwrap();

        let main = Function {
            signature: Type::Func {
                params: vec![],
                result: None,
            },
            body: main,
            export_name: Some("_start".to_string()),
        };

        module_builder.push_function(main);

        let mut wasm = Vec::new();
        let module = module_builder.into_module();
        module.write(&mut wasm).unwrap();
        wasm
    }
}

fn bf_to_block(bf: &str) -> Block {
    fn inner(block: &mut Block, chars: &mut Chars) {
        while let Some(char) = chars.next() {
            match char {
                '+' => block.push_item(BlockItem::Op(Op::Add(1))),
                '-' => block.push_item(BlockItem::Op(Op::Sub(1))),
                '>' => block.push_item(BlockItem::Op(Op::PtrAdd(1))),
                '<' => block.push_item(BlockItem::Op(Op::PtrSub(1))),
                '.' => block.push_item(BlockItem::Op(Op::Out)),
                ',' => block.push_item(BlockItem::Op(Op::Input)),
                '[' => {
                    let mut b = Block::new();
                    inner(&mut b, chars);
                    block.push_item(BlockItem::Loop(b));
                }
                ']' => return,
                _ => (),
            }
        }
    }
    let mut block = Block::new();
    let mut bf_chars = bf.chars();

    inner(&mut block, &mut bf_chars);

    block
}

pub fn bf_to_wat(bf: &str) -> String {
    let mut block = bf_to_block(bf);
    block.items.insert(0, BlockItem::Op(Op::Clear));

    let optimized_block = block.optimize();

    let body = optimized_block.to_wat(40);

    // Base Wasmer
    // https://github.com/wasmerio/wasmer/blob/75a98ab171bee010b9a7cd0f836919dc4519dcaf/lib/wasi/tests/stdio.rs
    format!(
        r#"(module
    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_unstable" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (memory (export "memory") 1)
    (func $input_char (param $ptr i32)
        (i32.store (i32.const 4) (local.get $ptr))
        (i32.store (i32.const 8) (i32.const 1))

        (call $fd_read
            (i32.const 0)
            (i32.const 4)
            (i32.const 1)
            (i32.const 12)
        )
        drop
    )
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
    (func $main (export "_start") {body})
)
"#,
    )
}

pub fn bf_to_wasm(bf: &str) -> Vec<u8> {
    let mut block = bf_to_block(bf);

    block.items.insert(0, BlockItem::Op(Op::Clear));
    // eprintln!("{block:?}");
    // eprintln!();

    let block = block.optimize();

    eprintln!("{block:?}");

    block.to_wasm()
}

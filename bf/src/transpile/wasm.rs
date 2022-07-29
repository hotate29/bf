use std::{fmt::Write, io, str::Chars};

mod opt;
mod wasm_binary;

use wasm_binary::{
    code::{FunctionBody, LocalEntry, MemoryImmediate, Op as WOp, OpSlice},
    section::{MemoryType, ResizableLimits},
    type_::Type,
    var::Var,
    Function, Import, Memory, ModuleBuilder,
};

#[derive(Debug, Clone, Copy)]
enum Op<T = u32> {
    Add(u32, T),
    Sub(u32, T),
    PtrAdd(u32),
    PtrSub(u32),
    /// [p+x+off] += [p+off]*y
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

#[derive(Debug, Clone)]
enum BlockItem {
    Op(Op),
    Loop(Block),
    If(Block),
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    items: Vec<BlockItem>,
}

impl Block {
    fn new() -> Self {
        Self::default()
    }
    fn push_item(&mut self, item: BlockItem) {
        self.items.push(item)
    }
    pub fn optimize(&self, top_level: bool) -> Block {
        let mut block = self.clone();

        if top_level {
            block.items.insert(0, BlockItem::Op(Op::Set(0, 0)));
        }

        let mut block = opt::merge(&block);

        opt::unwrap(&mut block);
        opt::clear(&mut block);
        let block = opt::mul(&block);
        let block = opt::merge(&block);
        opt::offset_opt(&block)
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
                        Op::Add(n, offset) => {
                            writeln!(
                                wat,
                                "
                                ;; Add
                                local.get $pointer
                                local.get $pointer
                                i32.load8_u offset={offset}
                                i32.const {n}
                                i32.add
                                i32.store8 offset={offset}"
                            )
                            .unwrap();
                        }
                        Op::Sub(n, offset) => {
                            writeln!(
                                wat,
                                "
                                ;; Sub
                                local.get $pointer
                                local.get $pointer
                                i32.load8_u offset={offset}
                                i32.const {n}
                                i32.sub
                                i32.store8 offset={offset}"
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
                        Op::Mul(x, y, offset) => {
                            writeln!(
                                wat,
                                "
                                ;; Mul
                                local.get $pointer
                                i32.const {x}
                                i32.add
                                local.set $ptr

                                local.get $ptr

                                local.get $ptr
                                i32.load8_u offset={offset}

                                local.get $pointer
                                i32.load8_u offset={offset}
                                i32.const {y}
                                i32.mul

                                i32.add
                                i32.store8 offset={offset}
                            "
                            )
                            .unwrap();
                        }
                        Op::Set(value, offset) => {
                            writeln!(
                                *wat,
                                "
                                ;; Clear
                                local.get $pointer
                                i32.const {value}
                                i32.store8 offset={offset}"
                            )
                            .unwrap();
                        }
                        Op::Out(offset) => {
                            writeln!(
                                wat,
                                "
                                ;; Out
                                local.get $pointer
                                i32.load8_u offset={offset}
                                call $print_char"
                            )
                            .unwrap();
                        }
                        Op::Input(offset) => {
                            writeln!(
                                wat,
                                "
                                ;; Input
                                local.get $pointer
                                call $input_char
                                i32.store8 offset={offset}"
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
                    BlockItem::If(if_block) => {
                        writeln!(
                            wat,
                            "
                        ;; If
                        local.get $pointer
                        i32.load8_u

                        (if
                            (then
                        "
                        )
                        .unwrap();

                        block_to_wat(if_block, wat, loop_stack, loop_count);

                        writeln!(
                            wat,
                            "
                            )
                        )
                        "
                        )
                        .unwrap();
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
    fn to_wasm_codes(&self, mut buffer: &mut Vec<u8>) {
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
                            WOp::I32Load8U(MemoryImmediate::zero(*offset)),
                            WOp::I32Const(Var(*value as i32)),
                            WOp::I32Add,
                            WOp::I32Store8(MemoryImmediate::zero(*offset)),
                        ];

                        add_ops.write(&mut buffer).unwrap();
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
                            WOp::I32Load8U(MemoryImmediate::zero(*offset)),
                            WOp::I32Const(Var(*value as i32)),
                            WOp::I32Sub,
                            WOp::I32Store8(MemoryImmediate::zero(*offset)),
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
                    Op::Mul(x, y, offset) => {
                        let mul_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Const(Var(*x as i32)),
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
                            WOp::I32Load8U(MemoryImmediate::zero(*offset)),
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::zero(*offset)),
                            WOp::I32Const(Var(*y as i32)),
                            WOp::I32Mul,
                            WOp::I32Add,
                            WOp::I32Store8(MemoryImmediate::zero(*offset)),
                        ];

                        mul_ops.write(&mut buffer).unwrap();
                    }
                    Op::Set(value, offset) => {
                        let clear_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Const(Var(*value)),
                            WOp::I32Store8(MemoryImmediate::zero(*offset)),
                        ];

                        clear_ops.write(&mut buffer).unwrap();
                    }
                    Op::Out(offset) => {
                        let out_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::I32Load8U(MemoryImmediate::zero(*offset)),
                            WOp::Call {
                                function_index: Var(2),
                            },
                        ];

                        out_ops.write(&mut buffer).unwrap();
                    }
                    Op::Input(offset) => {
                        let input_ops = [
                            WOp::GetLocal {
                                local_index: Var(0),
                            },
                            WOp::Call {
                                function_index: Var(3),
                            },
                            WOp::I32Store8(MemoryImmediate::zero(*offset)),
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
                        WOp::I32Load8U(MemoryImmediate::zero(0)),
                        WOp::I32Eqz,
                        WOp::BrIf {
                            relative_depth: Var(1),
                        },
                    ];

                    loop_ops.write(&mut buffer).unwrap();

                    loop_block.to_wasm_codes(buffer);

                    let loop_ops = [
                        WOp::Br {
                            relative_depth: Var(0),
                        },
                        WOp::End,
                        WOp::End,
                    ];

                    loop_ops.write(&mut buffer).unwrap();
                }
                BlockItem::If(if_block) => {
                    let if_ops = [
                        WOp::GetLocal {
                            local_index: Var(0),
                        },
                        WOp::I32Load8U(MemoryImmediate::zero(0)),
                        WOp::If {
                            block_type: Type::Void,
                        },
                    ];

                    if_ops.write(&mut buffer).unwrap();

                    if_block.to_wasm_codes(buffer);

                    WOp::End.write(&mut buffer).unwrap();
                }
            }
        }
    }
}

pub fn bf_to_block(bf: &str) -> Block {
    fn inner(block: &mut Block, chars: &mut Chars) {
        while let Some(char) = chars.next() {
            match char {
                '+' => block.push_item(BlockItem::Op(Op::Add(1, 0))),
                '-' => block.push_item(BlockItem::Op(Op::Sub(1, 0))),
                '>' => block.push_item(BlockItem::Op(Op::PtrAdd(1))),
                '<' => block.push_item(BlockItem::Op(Op::PtrSub(1))),
                '.' => block.push_item(BlockItem::Op(Op::Out(0))),
                ',' => block.push_item(BlockItem::Op(Op::Input(0))),
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

pub fn to_wat(block: Block, mut out: impl io::Write) -> io::Result<()> {
    let body = block.to_wat(40);

    // Base Wasmer
    // https://github.com/wasmerio/wasmer/blob/75a98ab171bee010b9a7cd0f836919dc4519dcaf/lib/wasi/tests/stdio.rs
    writeln!(
        out,
        r#"(module
    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_unstable" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (memory (export "memory") 1)
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

pub fn to_wasm(block: Block, mut buffer: impl io::Write) -> io::Result<()> {
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

    let mut print_char = Function {
        signature: Type::Func {
            params: vec![Type::I32],
            result: None,
        },
        body: FunctionBody::new(),
        export_name: None,
    };

    let print_char_ops = [
        WOp::I32Const(Var(0)),
        WOp::GetLocal {
            local_index: Var(0),
        },
        WOp::I32Store8(MemoryImmediate::zero(0)),
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
            function_index: Var(0),
        },
        WOp::Drop,
        WOp::End,
    ];
    print_char_ops.write(&mut print_char.body.code).unwrap();

    module_builder.push_function(print_char);

    let mut input_char = Function {
        signature: Type::Func {
            params: vec![],
            result: Some(Box::new(Type::I32)),
        },
        body: FunctionBody::new(),
        export_name: None,
    };

    let input_char_ops = [
        WOp::I32Const(Var(4)),
        WOp::I32Const(Var(0)),
        WOp::I32Store(MemoryImmediate::i32(0)),
        WOp::I32Const(Var(8)),
        WOp::I32Const(Var(1)),
        WOp::I32Store8(MemoryImmediate::zero(0)),
        WOp::I32Const(Var(0)),
        WOp::I32Const(Var(4)),
        WOp::I32Const(Var(1)),
        WOp::I32Const(Var(12)),
        WOp::Call {
            function_index: Var(1),
        },
        WOp::Drop,
        WOp::I32Const(Var(0)),
        WOp::I32Load8U(MemoryImmediate::zero(0)),
        WOp::End,
    ];
    input_char_ops.write(&mut input_char.body.code).unwrap();

    module_builder.push_function(input_char);

    let mut main = Function {
        signature: Type::Func {
            params: vec![],
            result: None,
        },
        body: FunctionBody::new(),
        export_name: Some("_start".to_string()),
    };

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
    .write(&mut main.body.code)
    .unwrap();

    block.to_wasm_codes(&mut main.body.code);

    WOp::End.write(&mut main.body.code).unwrap();

    module_builder.push_function(main);

    let module = module_builder.into_module();
    module.write(&mut buffer)
}

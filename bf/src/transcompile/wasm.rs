use std::{fmt::Write, str::Chars};

// use wasmtime::{Engine, Linker, Module, Store};
// use wasmtime_wasi::WasiCtxBuilder;

mod opt;

#[derive(Debug, Clone, Copy)]
enum Op {
    Add(u32),
    Sub(u32),
    PtrAdd(u32),
    PtrSub(u32),
    Clear,
    Out,
    Input,
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
                            *wat += "local.get $pointer\n";
                            writeln!(wat, "i32.const {n}").unwrap();
                            *wat += "call $add";
                            *wat += "\n";
                        }
                        Op::Sub(n) => {
                            *wat += "local.get $pointer\n";
                            writeln!(wat, "i32.const {n}").unwrap();
                            *wat += "call $sub\n";
                            *wat += "\n";
                        }
                        Op::PtrAdd(n) => {
                            *wat += "local.get $pointer\n";
                            writeln!(wat, "i32.const {n}").unwrap();
                            *wat += "i32.add\n";
                            *wat += "local.set $pointer\n";
                            *wat += "\n";
                        }
                        Op::PtrSub(n) => {
                            *wat += "local.get $pointer\n";
                            writeln!(wat, "i32.const {n}").unwrap();
                            *wat += "i32.sub\n";
                            *wat += "local.set $pointer\n";
                            *wat += "\n";
                        }
                        Op::Clear => {
                            *wat += "local.get $pointer\n";
                            *wat += "i32.const 0\n";
                            *wat += "i32.store8\n";
                            *wat += "\n";
                        }
                        Op::Out => {
                            *wat += "local.get $pointer\n";
                            *wat += "i32.load8_u\n";
                            *wat += "call $print_char\n";
                            *wat += "\n";
                        }
                        Op::Input => {
                            *wat += "local.get $pointer\n";
                            *wat += "call $input_char\n";
                            *wat += "\n";
                        }
                    },
                    BlockItem::Loop(block) => {
                        loop_stack.push(loop_count);

                        let loop_label = format!("loop_{loop_count}");
                        let block_label = format!("block_{loop_count}");
                        writeln!(
                            wat,
                            "(block ${block_label}
                                    (loop ${loop_label}
                                        i32.const 0
                                        local.get $pointer
                                        i32.load8_u

                                        (br_if ${block_label} (i32.eq))\n"
                        )
                        .unwrap();

                        loop_count += 1;
                        block_to_wat(block, wat, loop_stack, loop_count);

                        loop_stack.pop().unwrap();

                        writeln!(wat, "(br ${loop_label})").unwrap();
                        *wat += "))";
                    }
                }
            }
        }

        let mut wat = String::new();

        writeln!(
            wat,
            "(local $pointer i32) i32.const {memory_base_address} local.set $pointer"
        )
        .unwrap();

        let mut loop_stack = vec![];

        block_to_wat(self, &mut wat, &mut loop_stack, 0);

        wat
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
    let block = bf_to_block(bf);
    let block = opt::merge(block);
    let block = opt::clear(block);
    let body = block.to_wat(40);

    // Base Wasmer
    // https://github.com/wasmerio/wasmer/blob/75a98ab171bee010b9a7cd0f836919dc4519dcaf/lib/wasi/tests/stdio.rs
    format!(
        r#"(module
    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_unstable" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (memory (export "memory") 1 1000)
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
    (func $add (param $pointer i32) (param $value i32)
        local.get $pointer
        local.get $pointer
        i32.load8_u
        local.get $value
        i32.add

        i32.store8
    )
    (func $sub (param $pointer i32) (param $value i32)
        local.get $pointer
        local.get $pointer
        i32.load8_u
        local.get $value
        i32.sub

        i32.store8
    )

    (func $main (export "_start") {body})
)
"#,
    )
}

// pub fn run_bf(bf: &str) -> anyhow::Result<()> {
//     let wat = bf_wat(bf);

//     let engine = Engine::default();

//     let mut linker = Linker::new(&engine);
//     wasmtime_wasi::add_to_linker(&mut linker, |a| a)?;

//     let wasi = WasiCtxBuilder::new().inherit_stdout().build();
//     let mut store = Store::new(&engine, wasi);

//     let module = Module::new(&engine, wat)?;

//     linker
//         .module(&mut store, "", &module)?
//         .get_default(&mut store, "")?
//         .typed::<(), (), _>(&store)?
//         .call(&mut store, ())?;
//     Ok(())
// }

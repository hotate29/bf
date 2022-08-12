pub mod wasm;

pub mod c {
    use std::fmt::Write;

    use crate::transpile::wasm::{BlockItem, Op};

    use super::wasm::Block;

    const PTR_NAME: &str = "p";

    pub fn to_c(block: &Block, memory_len: usize) -> String {
        fn inner(block: &Block, c_code: &mut String) {
            for item in &block.items {
                match item {
                    BlockItem::Loop(loop_block) => {
                        write!(c_code, "while(*{PTR_NAME}){{").unwrap();
                        inner(loop_block, c_code);
                        c_code.push('}');
                    }
                    BlockItem::If(if_block) => {
                        write!(c_code, "if(*{PTR_NAME}!=0){{").unwrap();
                        inner(if_block, c_code);
                        c_code.push('}');
                    }
                    BlockItem::Op(instruction) => match instruction {
                        Op::Add(x, offset) => {
                            write!(c_code, "*({PTR_NAME}+{offset})+={x};").unwrap()
                        }
                        Op::Sub(x, offset) => {
                            write!(c_code, "*({PTR_NAME}+{offset})-={x};").unwrap()
                        }
                        Op::PtrAdd(x) => write!(c_code, "{PTR_NAME}+={x};").unwrap(),
                        Op::PtrSub(x) => write!(c_code, "{PTR_NAME}-={x};").unwrap(),
                        Op::Mul(to, x, offset) => write!(
                            c_code,
                            "*({PTR_NAME}+{offset}+{to})+=*({PTR_NAME}+{offset})*{x};",
                        )
                        .unwrap(),
                        Op::Set(x, offset) => {
                            write!(c_code, "*({PTR_NAME}+{offset})={x};",).unwrap()
                        }
                        Op::Out(offset) => {
                            write!(c_code, "putchar(*({PTR_NAME}+{offset}));",).unwrap()
                        }
                        Op::Input(offset) => {
                            write!(c_code, "*({PTR_NAME}+{offset})=getchar();",).unwrap()
                        }
                    },
                }
            }
        }

        let mut a = String::new();
        inner(block, &mut a);

        format!("#include <stdio.h>\n#include <stdint.h>\nint main(void){{uint8_t mem[{memory_len}]={{0}};uint8_t*{PTR_NAME}=mem;{a}}}")
    }
}

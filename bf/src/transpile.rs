pub mod wasm;

pub mod c {
    use std::fmt::Write;

    use crate::transpile::wasm::{BlockItem, Op};

    use super::wasm::Block;

    const PTR_NAME: &str = "ptr";

    pub fn to_c(root_node: &Block, memory_len: usize) -> String {
        fn inner(nodes: &Block, c_code: &mut String) {
            for node in &nodes.items {
                match node {
                    BlockItem::Loop(loop_nodes) => {
                        write!(c_code, "while(*{PTR_NAME}){{").unwrap();
                        inner(loop_nodes, c_code);
                        c_code.push('}');
                    }
                    BlockItem::If(if_block) => {
                        write!(c_code, "if(*{PTR_NAME}!=0){{").unwrap();
                        inner(if_block, c_code);
                        c_code.push('}');
                    }
                    BlockItem::Op(instruction) => match instruction {
                        Op::Add(x, offset) => write!(c_code, "*(ptr+{offset})+={x};").unwrap(),
                        Op::Sub(x, offset) => write!(c_code, "*(ptr+{offset})-={x};").unwrap(),
                        Op::PtrAdd(x) => write!(c_code, "ptr+={x};").unwrap(),
                        Op::PtrSub(x) => write!(c_code, "ptr-={x};").unwrap(),
                        Op::Mul(to, x, offset) => {
                            write!(c_code, "*(ptr+{offset}+{to})+=*(ptr+{offset})*{x};",).unwrap()
                        }
                        Op::Set(x, offset) => write!(c_code, "*(ptr+{offset})={x};",).unwrap(),
                        Op::Out(offset) => write!(c_code, "putchar(*(ptr+{offset}));",).unwrap(),
                        Op::Input(offset) => write!(c_code, "*(ptr+{offset})=getchar();",).unwrap(),
                    },
                }
            }
        }

        let mut a = String::new();
        inner(root_node, &mut a);

        let mut c_code = format!("#include <stdio.h>\n#include <stdint.h>\nint main(void){{uint8_t mem[{memory_len}]={{0}};uint8_t* {PTR_NAME} = mem;");
        c_code += &a;
        c_code += "}";

        c_code
    }
}

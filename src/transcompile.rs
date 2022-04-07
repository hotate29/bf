use crate::instruction::Instruction;
use crate::parse::Nodes;

pub fn to_c2(root_node: &Nodes) -> String {
    fn inner(nodes: &Nodes, c_code: &mut String) {
        for node in nodes {
            match node {
                crate::parse::Node::Loop(loop_nodes) => {
                    c_code.push_str("while(*ptr){");
                    inner(loop_nodes, c_code);
                    c_code.push('}');
                }
                crate::parse::Node::Instruction(instruction) => match instruction {
                    Instruction::PtrIncrement(n) => c_code.push_str(&format!("ptr+={n};")),
                    Instruction::PtrDecrement(n) => c_code.push_str(&format!("ptr-={n};")),
                    Instruction::AddOffset(offset, value) => {
                        c_code.push_str(&format!("*(ptr+{offset})+={value};"))
                    }
                    Instruction::AddTo(to_offset, offset) if *offset >= 0 => {
                        c_code.push_str(&format!("*(ptr+{to_offset})+=ptr[{offset}];"))
                    }
                    Instruction::AddTo(to_offset, offset) if *offset < 0 => c_code.push_str(
                        &format!("if(*(ptr+{offset})!=0){{*(ptr+{to_offset})+=*(ptr+{offset});}}"),
                    ),
                    Instruction::SubOffset(offset, value) => {
                        c_code.push_str(&format!("*(ptr+{offset})-={value};"))
                    }
                    Instruction::SubTo(to_offset, offset) if *offset >= 0 => {
                        c_code.push_str(&format!("*(ptr+{to_offset})-=ptr[{offset}];"))
                    }
                    Instruction::SubTo(to_offset, offset) if *offset < 0 => c_code.push_str(
                        &format!("if(*(ptr+{offset})!=0){{*(ptr+{to_offset})-=*(ptr+{offset});}}"),
                    ),
                    Instruction::OutputOffset(offset, repeat) => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("putchar(*(ptr+{offset}));"))
                        }
                    }
                    Instruction::Input(n) => {
                        for _ in 0..*n {
                            c_code.push_str("ptr[0]=getchar();");
                        }
                    }
                    Instruction::MulAdd(to_offset, offset, value) if *offset >= 0 => {
                        c_code.push_str(&format!("*(ptr+{to_offset})+={value}*ptr[{offset}];"));
                    }
                    Instruction::MulAdd(to_offset, offset, value) if *offset < 0 => {
                        c_code.push_str(&format!(
                            "if(*(ptr+{offset})!=0){{*(ptr+{to_offset})+={value}**(ptr+{offset});}}"
                        ));
                    }
                    Instruction::MulSub(to_offset, offset, value) if *offset >= 0 => {
                        c_code.push_str(&format!("*(ptr+{to_offset})-={value}*ptr[{offset}];"));
                    }
                    Instruction::MulSub(to_offset, offset, value) if *offset < 0 => {
                        c_code.push_str(&format!(
                            "if(*(ptr+{offset})!=0){{*(ptr+{to_offset})-={value}**(ptr+{offset});}}"
                        ));
                    }
                    Instruction::ZeroSet => {
                        c_code.push_str("*ptr=0;");
                    }
                    Instruction::InputOffset(offset, repeat) => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("*(ptr+{offset})=getchar();"))
                        }
                    }
                    Instruction::ZeroSetOffset(offset) => {
                        c_code.push_str(&format!("*(ptr+{offset})=0;"))
                    }
                    ins => panic!("unimplemented instruction. {ins:?}"),
                    // Instruction::Copy(_) => todo!(),
                },
            }
        }
    }

    let mut a = String::new();
    inner(root_node, &mut a);

    let mut c_code = String::from("#include <stdio.h>\n#include <stdint.h>\nint main(void){uint8_t mem[30000]={0};uint8_t* ptr = mem;");
    c_code += &a;
    c_code += "}";

    c_code
}

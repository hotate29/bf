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
                    Instruction::Add(n) => c_code.push_str(&format!("*ptr+={n};")),
                    Instruction::AddTo(offset) if *offset >= 0 => {
                        c_code.push_str(&format!("ptr[{offset}]+=ptr[0];"))
                    }
                    Instruction::AddTo(offset) if *offset < 0 => {
                        c_code.push_str(&format!("if(ptr[0]!=0){{*(ptr{offset})+=ptr[0];}}"))
                    }
                    Instruction::Sub(n) => c_code.push_str(&format!("*ptr-={n};")),
                    Instruction::SubTo(offset) if *offset >= 0 => {
                        c_code.push_str(&format!("ptr[{offset}]-=ptr[0];"))
                    }
                    Instruction::SubTo(offset) if *offset < 0 => {
                        c_code.push_str(&format!("if(ptr[0]!=0){{*(ptr{offset})-=ptr[0];}}"))
                    }
                    Instruction::Output(n) => {
                        for _ in 0..*n {
                            c_code.push_str("putchar(ptr[0]);")
                        }
                    }
                    Instruction::Input(n) => {
                        for _ in 0..*n {
                            c_code.push_str("ptr[0]=getchar();");
                        }
                    }
                    Instruction::MulAdd(to_offset, offset, value) if *offset >= 0 => {
                        c_code.push_str(&format!("ptr[{offset}]+={value}*ptr[0];"));
                    }
                    Instruction::MulAdd(to_offset, offset, value) if *offset < 0 => {
                        c_code.push_str(&format!(
                            "if(*ptr!=0){{*(ptr{to_offset})+={value}*ptr[{offset}];}}"
                        ));
                        unimplemented!()
                    }
                    Instruction::MulSub(to_offset, offset, value) if *offset >= 0 => {
                        c_code.push_str(&format!("ptr[{offset}]-={value}*ptr[0];"));
                        unimplemented!()
                    }
                    Instruction::MulSub(to_offset, offset, value) if *offset < 0 => {
                        c_code.push_str(&format!("if(*ptr!=0){{*(ptr{offset})-={value}*ptr[0];}}"));
                        unimplemented!()
                    }
                    Instruction::ZeroSet => {
                        c_code.push_str("*ptr=0;");
                    }
                    Instruction::AddOffset(offset, value) if *offset < 0 => {
                        c_code.push_str(&format!("if(*ptr!=0){{*(ptr{offset})+={value};}}"))
                    }
                    Instruction::AddOffset(offset, value) if *offset >= 0 => {
                        c_code.push_str(&format!("ptr[{offset}]+={value};"))
                    }
                    Instruction::SubOffset(offset, value) if *offset < 0 => {
                        c_code.push_str(&format!("if(*ptr!=0){{*(ptr{offset})-={value};}}"))
                    }
                    Instruction::SubOffset(offset, value) if *offset >= 0 => {
                        c_code.push_str(&format!("ptr[{offset}]-={value};"))
                    }
                    Instruction::OutputOffset(offset, repeat) if *offset < 0 => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("if(*ptr!=0){{putchar(*(ptr{offset}));}}"))
                        }
                    }
                    Instruction::OutputOffset(offset, repeat) if *offset >= 0 => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("putchar(ptr[{offset}]);"))
                        }
                    }
                    Instruction::InputOffset(offset, repeat) if *offset >= 0 => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("ptr[{offset}]=getchar();"))
                        }
                    }
                    Instruction::InputOffset(offset, repeat) if *offset < 0 => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("*(ptr+{offset})=getchar();"))
                        }
                    }
                    Instruction::ZeroSetOffset(_) => todo!(),
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

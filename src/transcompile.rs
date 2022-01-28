use crate::optimize::{ExprKind, Node};
use crate::token::Instruction;

pub fn to_c(root_node: &Node) -> String {
    fn inner(node: &Node, c_code: &mut String) {
        for expr in &node.0 {
            match expr {
                ExprKind::Instructions(instructions) => {
                    for instruction in instructions {
                        match instruction {
                            Instruction::PtrIncrement(n) => {
                                c_code.push_str(&format!("ptr+={};", n))
                            }
                            Instruction::PtrDecrement(n) => {
                                c_code.push_str(&format!("ptr-={};", n))
                            }
                            Instruction::Add(n) => c_code.push_str(&format!("ptr[0]+={};", n)),
                            Instruction::MoveAdd(n) => {
                                c_code.push_str(&format!("ptr[{}]+=ptr[0];ptr[0]=0;", n))
                            }
                            Instruction::MoveAddRev(n) => c_code.push_str(&format!(
                                "if(*ptr!=0){{*(ptr-{})+=ptr[0];ptr[0]=0;}}",
                                n
                            )),
                            Instruction::MoveSub(n) => {
                                c_code.push_str(&format!("ptr[{}]-=ptr[0];ptr[0]=0;", n))
                            }
                            Instruction::MoveSubRev(n) => c_code.push_str(&format!(
                                "if(*ptr!=0){{*(ptr-{})-=ptr[0];ptr[0]=0;}}",
                                n
                            )),
                            Instruction::Sub(n) => c_code.push_str(&format!("ptr[0]-={};", n)),
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
                            Instruction::SetValue(offset, value) => {
                                c_code.push_str(&format!("ptr[{}]={};", offset, value));
                            }
                        }
                    }
                }
                ExprKind::While(while_node) => {
                    c_code.push_str("while(*ptr){");
                    inner(while_node, c_code);
                    c_code.push('}');
                }
            }
        }
    }

    let mut a = String::new();
    inner(root_node, &mut a);

    let mut c_code = String::from("#include <stdio.h>\n#include <stdint.h>\n\nint main(void){\nuint8_t mem[30000] = {0};\nuint8_t* ptr = mem;");
    c_code += &a;
    c_code += "}";

    c_code
}

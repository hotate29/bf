use crate::instruction::{Instruction, Value};
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
                    Instruction::Add(offset, value @ Value::Const(_)) => {
                        let value = value_to_string("ptr", *value);
                        c_code.push_str(&format!("*(ptr+{offset})+={value};"))
                    }
                    Instruction::Add(to_offset, Value::Memory(offset)) if *offset >= 0 => {
                        c_code.push_str(&format!("*(ptr+{to_offset})+=ptr[{offset}];"))
                    }
                    Instruction::Add(to_offset, Value::Memory(offset)) if *offset < 0 => c_code
                        .push_str(&format!(
                            "if(*(ptr+{offset})!=0){{*(ptr+{to_offset})+=*(ptr+{offset});}}"
                        )),
                    Instruction::Sub(offset, Value::Const(value)) => {
                        c_code.push_str(&format!("*(ptr+{offset})-={value};"))
                    }
                    Instruction::Sub(to_offset, Value::Memory(offset)) if *offset >= 0 => {
                        c_code.push_str(&format!("*(ptr+{to_offset})-=ptr[{offset}];"))
                    }
                    Instruction::Sub(to_offset, Value::Memory(offset)) if *offset < 0 => c_code
                        .push_str(&format!(
                            "if(*(ptr+{offset})!=0){{*(ptr+{to_offset})-=*(ptr+{offset});}}"
                        )),
                    Instruction::Output(offset, repeat) => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("putchar(*(ptr+{offset}));"))
                        }
                    }
                    Instruction::Input(offset, repeat) => {
                        for _ in 0..*repeat {
                            c_code.push_str(&format!("*(ptr+{offset})=getchar();"))
                        }
                    }
                    Instruction::MulAdd(to_offset, Value::Memory(offset), value)
                        if *offset >= 0 =>
                    {
                        let value = value_to_string("ptr", *value);
                        c_code.push_str(&format!("*(ptr+{to_offset})+={value}*ptr[{offset}];"));
                    }
                    Instruction::MulAdd(to_offset, Value::Memory(offset), value) if *offset < 0 => {
                        let value = value_to_string("ptr", *value);
                        c_code.push_str(&format!(
                        "if(*(ptr+{offset})!=0){{*(ptr+{to_offset})+={value}**(ptr+{offset});}}"
                        ));
                    }
                    Instruction::MulSub(to_offset, Value::Memory(offset), value)
                        if *offset >= 0 =>
                    {
                        let value = value_to_string("ptr", *value);
                        c_code.push_str(&format!("*(ptr+{to_offset})-={value}*ptr[{offset}];"));
                    }
                    Instruction::MulSub(to_offset, Value::Memory(offset), value) if *offset < 0 => {
                        let value = value_to_string("ptr", *value);
                        c_code.push_str(&format!(
                        "if(*(ptr+{offset})!=0){{*(ptr+{to_offset})-={value}**(ptr+{offset});}}"
                        ));
                    }
                    Instruction::SetValue(offset, value) => {
                        let value = value_to_string("ptr", *value);
                        c_code.push_str(&format!("*(ptr+{offset})={value};"))
                    }
                    ins => panic!("unimplemented instruction. {ins:?}"),
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

fn value_to_string(ptr_name: &str, value: Value) -> String {
    match value {
        Value::Const(value) => value.to_string(),
        Value::Memory(offset) => format!("*({ptr_name}+{offset})"),
    }
}

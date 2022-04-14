use crate::instruction::{Instruction, Value};
use crate::parse::Nodes;

const PTR_NAME: &str = "ptr";

pub fn to_c(root_node: &Nodes, memory_len: usize) -> String {
    fn inner(nodes: &Nodes, c_code: &mut String) {
        for node in nodes {
            match node {
                crate::parse::Node::Loop(loop_nodes) => {
                    c_code.push_str(&format!("while(*{PTR_NAME}){{"));
                    inner(loop_nodes, c_code);
                    c_code.push('}');
                }
                crate::parse::Node::Instruction(instruction) => match instruction {
                    Instruction::PtrIncrement(n) => c_code.push_str(&format!("{PTR_NAME}+={n};")),
                    Instruction::PtrDecrement(n) => c_code.push_str(&format!("{PTR_NAME}-={n};")),
                    Instruction::Add(offset, value @ Value::Const(_)) => {
                        let value = value_to_string(PTR_NAME, *value);
                        c_code.push_str(&format!("*({PTR_NAME}+{offset})+={value};"))
                    }
                    Instruction::Add(to_offset, Value::Memory(offset)) if *offset >= 0 => c_code
                        .push_str(&format!("*({PTR_NAME}+{to_offset})+={PTR_NAME}[{offset}];")),
                    Instruction::Add(to_offset, Value::Memory(offset)) if *offset < 0 => {
                        let check = check_zero(PTR_NAME, *offset);
                        c_code.push_str(&format!(
                            "{check}{{*({PTR_NAME}+{to_offset})+=*({PTR_NAME}+{offset});}}"
                        ))
                    }
                    Instruction::Sub(offset, Value::Const(value)) => {
                        c_code.push_str(&format!("*({PTR_NAME}+{offset})-={value};"))
                    }
                    Instruction::Sub(to_offset, Value::Memory(offset)) if *offset >= 0 => c_code
                        .push_str(&format!("*({PTR_NAME}+{to_offset})-={PTR_NAME}[{offset}];")),
                    Instruction::Sub(to_offset, Value::Memory(offset)) if *offset < 0 => {
                        let check = check_zero(PTR_NAME, *offset);
                        c_code.push_str(&format!(
                            "{check}{{*({PTR_NAME}+{to_offset})-=*({PTR_NAME}+{offset});}}"
                        ))
                    }
                    Instruction::Output(offset) => {
                        c_code.push_str(&format!("putchar(*({PTR_NAME}+{offset}));"))
                    }
                    Instruction::Input(offset) => {
                        c_code.push_str(&format!("*({PTR_NAME}+{offset})=getchar();"))
                    }
                    Instruction::MulAdd(to_offset, Value::Memory(offset), value)
                        if *offset >= 0 =>
                    {
                        let value = value_to_string(PTR_NAME, *value);
                        c_code.push_str(&format!(
                            "*({PTR_NAME}+{to_offset})+={value}*{PTR_NAME}[{offset}];"
                        ));
                    }
                    Instruction::MulAdd(to_offset, Value::Memory(offset), value) if *offset < 0 => {
                        let value = value_to_string(PTR_NAME, *value);
                        let check = check_zero(PTR_NAME, *offset);
                        c_code.push_str(&format!(
                            "{check}{{*({PTR_NAME}+{to_offset})+={value}**({PTR_NAME}+{offset});}}"
                        ));
                    }
                    Instruction::MulSub(to_offset, Value::Memory(offset), value)
                        if *offset >= 0 =>
                    {
                        let value = value_to_string(PTR_NAME, *value);
                        c_code.push_str(&format!(
                            "*({PTR_NAME}+{to_offset})-={value}*{PTR_NAME}[{offset}];"
                        ));
                    }
                    Instruction::MulSub(to_offset, Value::Memory(offset), value) if *offset < 0 => {
                        let value = value_to_string(PTR_NAME, *value);
                        let check = check_zero(PTR_NAME, *offset);
                        c_code.push_str(&format!(
                            "{check}{{*({PTR_NAME}+{to_offset})-={value}**({PTR_NAME}+{offset});}}"
                        ));
                    }
                    Instruction::SetValue(offset, value) => {
                        let value = value_to_string(PTR_NAME, *value);
                        c_code.push_str(&format!("*({PTR_NAME}+{offset})={value};"))
                    }
                    ins => panic!("unimplemented instruction. {ins:?}"),
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

fn value_to_string(ptr_name: &str, value: Value) -> String {
    match value {
        Value::Const(value) => value.to_string(),
        Value::Memory(offset) => format!("*({ptr_name}+{offset})"),
    }
}

fn check_zero(ptr_name: &str, offset: isize) -> String {
    format!("if(*({ptr_name}+{offset})!=0)")
}

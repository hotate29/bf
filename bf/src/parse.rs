use std::str::Chars;

use anyhow::ensure;

fn validate_bf(bf: &str) -> anyhow::Result<()> {
    // バリテーション
    let mut loop_depth = 0;

    for ci in bf.chars() {
        match ci {
            '[' => {
                loop_depth += 1;
            }
            ']' => loop_depth -= 1,
            _ => (),
        }

        ensure!(
            loop_depth >= 0,
            "invalid syntax: `]` not corresponding to `[`"
        )
    }
    ensure!(
        loop_depth == 0,
        "invalid syntax: `[` not corresponding to `]`"
    );
    Ok(())
}

pub enum Op {
    Add,
    Sub,
    PtrAdd,
    PtrSub,
    Output,
    Input,
}
pub enum Item {
    Op(Op),
    Loop(Ast),
}

pub struct Ast {
    items: Vec<Item>,
}
impl Ast {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn push(&mut self, item: Item) {
        self.items.push(item);
    }
    pub fn inner(&self) -> &Vec<Item> {
        &self.items
    }
    pub fn from_bf(bf: &str) -> anyhow::Result<Ast> {
        fn inner(ast: &mut Ast, chars: &mut Chars) {
            while let Some(char) = chars.next() {
                match char {
                    '+' => ast.push(Item::Op(Op::Add)),
                    '-' => ast.push(Item::Op(Op::Sub)),
                    '>' => ast.push(Item::Op(Op::PtrAdd)),
                    '<' => ast.push(Item::Op(Op::PtrSub)),
                    '.' => ast.push(Item::Op(Op::Output)),
                    ',' => ast.push(Item::Op(Op::Input)),
                    '[' => {
                        let mut s = Ast::new();
                        inner(&mut s, chars);
                        ast.push(Item::Loop(s));
                    }
                    ']' => return,
                    _ => (),
                }
            }
        }

        validate_bf(bf)?;

        let mut block = Ast::new();
        let mut bf_chars = bf.chars();

        inner(&mut block, &mut bf_chars);

        Ok(block)
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

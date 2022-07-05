use std::ops::Add;

use super::{Block, BlockItem, Op};

impl Add for Op {
    type Output = Option<Op>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Op::Add(n), Op::Add(m)) => Some(Op::Add(n + m)),
            (Op::Sub(n), Op::Sub(m)) => Some(Op::Sub(n + m)),
            (Op::PtrAdd(n), Op::PtrAdd(m)) => Some(Op::PtrAdd(n + m)),
            (Op::PtrSub(n), Op::PtrSub(m)) => Some(Op::PtrSub(n + m)),
            (Op::Add(_), Op::Sub(_)) => None,
            (Op::Sub(_), Op::Add(_)) => None,
            (Op::PtrAdd(_), Op::PtrSub(_)) => None,
            (Op::PtrSub(_), Op::PtrAdd(_)) => None,
            (_, _) => None,
        }
    }
}

pub(super) fn merge(block: Block) -> Block {
    let mut merged_block = Block::new();

    for item in block.items {
        let last_item = merged_block.items.last();
        match (last_item, item) {
            (Some(BlockItem::Op(lhs)), BlockItem::Op(rhs)) if (*lhs + rhs).is_some() => {
                // 無念
                let op = (*lhs + rhs).unwrap();
                merged_block.items.pop().unwrap();
                merged_block.push_item(BlockItem::Op(op))
            }
            (_, BlockItem::Loop(loop_item)) => {
                merged_block.push_item(BlockItem::Loop(merge(loop_item)))
            }
            (_, item) => (merged_block.push_item(item)),
        }
    }

    merged_block
}

pub(super) fn clear(block: Block) -> Block {
    let mut optimized_block = Block::new();

    for item in block.items {
        if let BlockItem::Loop(block) = item {
            if let [BlockItem::Op(Op::Sub(1))] = block.items.as_slice() {
                optimized_block.push_item(BlockItem::Op(Op::Clear));
            } else {
                let item = clear(block);
                optimized_block.push_item(BlockItem::Loop(item));
            }
        } else {
            optimized_block.push_item(item)
        }
    }

    optimized_block
}

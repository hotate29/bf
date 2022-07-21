use std::{collections::BTreeMap, ops::Add};

use super::{Block, BlockItem, Op};

impl Add for Op {
    type Output = Option<Op>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Op::Add(n), Op::Add(m)) => Some(Op::Add(n + m)),
            (Op::Sub(n), Op::Sub(m)) => Some(Op::Sub(n + m)),
            (Op::PtrAdd(n), Op::PtrAdd(m)) => Some(Op::PtrAdd(n + m)),
            (Op::PtrSub(n), Op::PtrSub(m)) => Some(Op::PtrSub(n + m)),
            (Op::Add(_) | Op::Sub(_), Op::Clear) => Some(Op::Clear),
            (Op::Clear, Op::Mul(_, _)) => Some(Op::Clear),
            (Op::Clear, Op::Clear) => Some(Op::Clear),
            (Op::Sub(_), Op::Add(_)) => rhs + self,
            (Op::PtrSub(_), Op::PtrAdd(_)) => rhs + self,
            (Op::Add(_), Op::Sub(_)) => None,
            (Op::PtrAdd(_), Op::PtrSub(_)) => None,
            (_, _) => None,
        }
    }
}

pub(super) fn merge(block: &Block) -> Block {
    let mut merged_block = Block::new();

    for item in &block.items {
        match item {
            BlockItem::Loop(loop_block) => {
                merged_block.push_item(BlockItem::Loop(merge(loop_block)))
            }
            // BlockItem::Opのコピーは軽いのでおｋ
            item => merged_block.push_item(item.clone()),
        };
        loop {
            if merged_block.items.len() < 2 {
                break;
            }
            let last2 = merged_block.items.iter().nth_back(1).unwrap();
            let last = merged_block.items.last().unwrap();

            match (last2, last) {
                (BlockItem::Op(lhs), BlockItem::Op(rhs)) if (*lhs + *rhs).is_some() => {
                    let op = (*lhs + *rhs).unwrap();
                    merged_block.items.pop().unwrap();
                    merged_block.items.pop().unwrap();
                    merged_block.push_item(BlockItem::Op(op))
                }
                (_, _) => break,
            }
        }
    }
    merged_block
}

pub(super) fn clear(block: &Block) -> Block {
    let mut optimized_block = Block::new();

    for item in &block.items {
        if let BlockItem::Loop(block) = item {
            if let [BlockItem::Op(Op::Add(1) | Op::Sub(1))] = block.items.as_slice() {
                optimized_block.push_item(BlockItem::Op(Op::Clear));
            } else {
                let item = clear(block);
                optimized_block.push_item(BlockItem::Loop(item));
            }
        } else {
            optimized_block.push_item(item.clone())
        }
    }

    optimized_block
}

pub(super) fn unwrap(block: &mut Block) {
    fn inner(item: &mut BlockItem) -> bool {
        if let BlockItem::Loop(loop_block) = item {
            if loop_block.items.len() == 1 {
                match loop_block.items.pop().unwrap() {
                    BlockItem::Loop(deep_loop_block) => *loop_block = deep_loop_block,
                    item => loop_block.push_item(item),
                }
            }
        }
        false
    }
    block.items.iter_mut().for_each(|item| {
        while inner(item) {}
        if let BlockItem::Loop(loop_block) = item {
            unwrap(loop_block)
        }
    });
}

pub(super) fn mul(block: Block) -> Block {
    #[derive(Debug, PartialEq, Eq)]
    enum OpType {
        Mul(i32),
        Clear,
    }
    impl OpType {
        fn mul(&mut self, x: i32) {
            match self {
                OpType::Mul(y) => *y += x,
                OpType::Clear => *self = OpType::Mul(x),
            }
        }
    }

    let is_optimizable = block.items.iter().all(|item| {
        if let BlockItem::Loop(loop_block) = item {
            let optimizable = loop_block.items.iter().any(|item| {
                matches!(
                    item,
                    BlockItem::Op(Op::Clear | Op::Mul(_, _) | Op::Out | Op::Input)
                        | BlockItem::Loop(_)
                )
            });
            !optimizable
        } else {
            true
        }
    });

    if is_optimizable {
        let mut optimized_block = Block::new();

        for item in &block.items {
            match item {
                // こっちだったら最適化
                BlockItem::Loop(loop_block) => {
                    let mut offset_op = BTreeMap::<_, OpType>::new();
                    let mut ptr_offset = 0;

                    for item in &loop_block.items {
                        match item {
                            BlockItem::Loop(_) => unreachable!(),
                            BlockItem::Op(op) => match op {
                                Op::Add(v) => {
                                    offset_op
                                        .entry(ptr_offset)
                                        .and_modify(|x| x.mul(*v as i32))
                                        .or_insert(OpType::Mul(*v as i32));
                                }
                                Op::Sub(v) => {
                                    offset_op
                                        .entry(ptr_offset)
                                        .and_modify(|x| x.mul(-(*v as i32)))
                                        .or_insert(OpType::Mul(-(*v as i32)));
                                }

                                Op::PtrAdd(of) => ptr_offset += *of as i32,
                                Op::PtrSub(of) => ptr_offset -= *of as i32,
                                // Op::Clear => {
                                //     offset_op.insert(ptr_offset, OpType::Clear);
                                // }
                                Op::Clear | Op::Mul(_, _) | Op::Out | Op::Input => unreachable!(),
                            },
                        };
                    }

                    // eprintln!("{offset_op:?}");

                    let clear_plus = offset_op.get(&0) == Some(&OpType::Mul(1));
                    let clear_minus = offset_op.get(&0) == Some(&OpType::Mul(-1));

                    let is_clear_loop = offset_op.len() == 1 && (clear_minus || clear_plus);

                    if is_clear_loop {
                        optimized_block.push_item(BlockItem::Op(Op::Clear));
                        continue;
                    }
                    if ptr_offset != 0 || !clear_minus {
                        eprintln!("失敗, {ptr_offset}, {offset_op:?}");
                        return block;
                    }

                    for (offset, value) in offset_op {
                        // 0は後で処理
                        if offset == 0 {
                            continue;
                        }
                        match value {
                            OpType::Mul(value) => {
                                optimized_block.push_item(BlockItem::Op(Op::Mul(offset, value)))
                            }
                            OpType::Clear => {
                                optimized_block.push_item(BlockItem::Op(Op::ptr(offset)));
                                optimized_block.push_item(BlockItem::Op(Op::Clear));
                                optimized_block.push_item(BlockItem::Op(Op::ptr(-offset)));
                            }
                        };
                    }

                    optimized_block.push_item(BlockItem::Op(Op::Clear))
                }
                BlockItem::Op(op) => optimized_block.push_item(BlockItem::Op(*op)),
            };
        }

        optimized_block
    } else {
        let mut optimized_block = Block::new();
        for item in block.items {
            match item {
                BlockItem::Loop(loop_block) => {
                    optimized_block.push_item(BlockItem::Loop(mul(loop_block)))
                }
                item => optimized_block.push_item(item),
            }
        }

        optimized_block
    }
}

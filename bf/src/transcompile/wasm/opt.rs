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
            (Op::Add(x), Op::Sub(y)) => {
                let z = x as i32 - y as i32;

                if z.is_positive() {
                    Some(Op::Add(z as u32))
                } else {
                    Some(Op::Sub(-z as u32))
                }
            }
            // 0を足し引きするのは無駄なので、適当な機会に消滅してほしい。
            (op, Op::Add(0) | Op::Sub(0)) => Some(op),
            (op, Op::PtrAdd(0) | Op::PtrSub(0)) => Some(op),
            (Op::PtrAdd(x), Op::PtrSub(y)) => {
                let z = x as i32 - y as i32;

                if z.is_positive() {
                    Some(Op::PtrAdd(z as u32))
                } else {
                    Some(Op::PtrSub(-z as u32))
                }
            }
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
        while merged_block.items.len() >= 2 {
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

pub(super) fn mul(block: &Block) -> Block {
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

    fn is_optimizable_loop(loop_block: &Block) -> Option<BTreeMap<i32, OpType>> {
        let mut offset_op = BTreeMap::<_, OpType>::new();
        let mut ptr_offset = 0;

        for item in &loop_block.items {
            match item {
                // 最適化できないものが混じっていたらreturn
                BlockItem::Loop(_)
                | BlockItem::If(_)
                | BlockItem::Op(Op::Clear | Op::Mul(_, _) | Op::Out | Op::Input) => return None,

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

        let clear_minus = offset_op.get(&0) == Some(&OpType::Mul(-1));

        // bool.then_some()にできる
        if ptr_offset == 0 && clear_minus {
            Some(offset_op)
        } else {
            None
        }
    }

    let mut optimized_block = Block::new();

    for item in &block.items {
        match item {
            BlockItem::Loop(loop_block) => {
                let offset_ops = is_optimizable_loop(loop_block);

                match offset_ops {
                    // こっちだったら最適化
                    Some(offset_ops) => {
                        let mut mul_ops = Block::new();
                        for (offset, value) in offset_ops {
                            // 0は最後に処理
                            if offset == 0 {
                                continue;
                            }
                            match value {
                                OpType::Mul(value) => {
                                    mul_ops.push_item(BlockItem::Op(Op::Mul(offset, value)))
                                }
                                OpType::Clear => {
                                    mul_ops.push_item(BlockItem::Op(Op::ptr(offset)));
                                    mul_ops.push_item(BlockItem::Op(Op::Clear));
                                    mul_ops.push_item(BlockItem::Op(Op::ptr(-offset)));
                                }
                            };
                        }

                        mul_ops.push_item(BlockItem::Op(Op::Clear));

                        optimized_block.push_item(BlockItem::If(mul_ops));
                    }
                    None => {
                        let b = mul(loop_block);
                        optimized_block.push_item(BlockItem::Loop(b));
                    }
                }
            }
            BlockItem::Op(op) => optimized_block.push_item(BlockItem::Op(*op)),
            // うむむむむ...
            BlockItem::If(if_block) => optimized_block.push_item(BlockItem::If(if_block.clone())),
        };
    }

    optimized_block
}

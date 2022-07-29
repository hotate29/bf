use std::{collections::BTreeMap, ops::Add};

use super::{Block, BlockItem, Op};

impl Add for Op<u32> {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Op::Add(n, o), Op::Add(m, f)) if o == f => Some(Op::Add(n + m, o)),
            (Op::Sub(n, o), Op::Sub(m, f)) if o == f => Some(Op::Sub(n + m, o)),
            (Op::PtrAdd(n), Op::PtrAdd(m)) => Some(Op::PtrAdd(n + m)),
            (Op::PtrSub(n), Op::PtrSub(m)) => Some(Op::PtrSub(n + m)),
            (Op::Add(_, o) | Op::Sub(_, o), Op::Set(_, f)) if o == f => Some(rhs),
            (Op::Set(0, o), Op::Mul(_, _, f)) if o == f => Some(Op::Set(0, o)),
            (Op::Set(_, o), Op::Set(_, f)) if o == f => Some(rhs),
            (Op::Sub(_, _), Op::Add(_, _)) => rhs + self,
            (Op::PtrSub(_), Op::PtrAdd(_)) => rhs + self,
            (Op::Set(x, o), Op::Add(y, f)) if o == f => Some(Op::Set(x + y as i32, o)),
            (Op::Set(x, o), Op::Sub(y, f)) if o == f => Some(Op::Set(x - y as i32, o)),
            (Op::Add(x, o), Op::Sub(y, f)) if o == f => {
                let z = x as i32 - y as i32;

                if z.is_positive() {
                    Some(Op::Add(z as u32, o))
                } else {
                    Some(Op::Sub(-z as u32, o))
                }
            }
            (Op::PtrAdd(x), Op::PtrSub(y)) => {
                let z = x as i32 - y as i32;

                if z.is_positive() {
                    Some(Op::PtrAdd(z as u32))
                } else {
                    Some(Op::PtrSub(-z as u32))
                }
            }
            // 0を足し引きするのは無駄なので、適当な機会に消滅してほしい。
            (op, Op::Add(0, _) | Op::Sub(0, _)) => Some(op),
            (op, Op::PtrAdd(0) | Op::PtrSub(0)) => Some(op),
            (op, Op::Mul(_, 0, _)) => Some(op),
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

pub(super) fn clear(block: &mut Block) {
    for item in &mut block.items {
        if let BlockItem::Loop(block) = item {
            if let [BlockItem::Op(Op::Add(1, 0) | Op::Sub(1, 0))] = block.items.as_slice() {
                *item = BlockItem::Op(Op::Set(0, 0));
            } else {
                clear(block);
            }
        }
    }
}

pub(super) fn unwrap(block: &mut Block) {
    fn inner(item: &mut BlockItem) -> bool {
        if let BlockItem::Loop(loop_block) = item {
            if loop_block.items.len() == 1 {
                match loop_block.items.pop().unwrap() {
                    BlockItem::Loop(deep_loop_block) => {
                        *loop_block = deep_loop_block;
                        return true;
                    }
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

pub(super) fn mul(block: &mut Block) {
    #[derive(Debug, PartialEq, Eq)]
    enum OpType {
        Mul(i32),
        Set(i32),
    }
    impl OpType {
        fn mul(&mut self, x: i32) {
            match self {
                OpType::Mul(y) => *y += x,
                OpType::Set(y) => *y += x,
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
                | BlockItem::Op(Op::Mul(_, _, _) | Op::Out(_) | Op::Input(_)) => return None,

                BlockItem::Op(op) => match op {
                    Op::Add(v, of) => {
                        offset_op
                            .entry(ptr_offset + *of as i32)
                            .and_modify(|x| x.mul(*v as i32))
                            .or_insert(OpType::Mul(*v as i32));
                    }
                    Op::Sub(v, of) => {
                        offset_op
                            .entry(ptr_offset - *of as i32)
                            .and_modify(|x| x.mul(-(*v as i32)))
                            .or_insert(OpType::Mul(-(*v as i32)));
                    }

                    Op::PtrAdd(of) => ptr_offset += *of as i32,
                    Op::PtrSub(of) => ptr_offset -= *of as i32,
                    Op::Set(v, offset) => {
                        offset_op.insert(ptr_offset + *offset as i32, OpType::Set(*v));
                    }
                    Op::Mul(_, _, _) | Op::Out(_) | Op::Input(_) => {
                        unreachable!()
                    }
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

    for item in &mut block.items {
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
                                    mul_ops.push_item(BlockItem::Op(Op::Mul(offset, value, 0)))
                                }
                                OpType::Set(value) => {
                                    mul_ops.push_item(BlockItem::Op(Op::ptr(offset)));
                                    mul_ops.push_item(BlockItem::Op(Op::Set(value, 0)));
                                    mul_ops.push_item(BlockItem::Op(Op::ptr(-offset)));
                                }
                            };
                        }
                        mul_ops.push_item(BlockItem::Op(Op::Set(0, 0)));

                        *item = BlockItem::If(mul_ops);
                    }
                    None => {
                        mul(loop_block);
                    }
                }
            }
            BlockItem::If(if_block) => mul(if_block),
            BlockItem::Op(_) => (),
        };
    }
}

pub(crate) fn offset_opt(block: &Block) -> Block {
    // 先に命令列の固まりから処理する
    // Loop | Ifでsplitする事によって、命令列を良い感じに抽出する事ができる。
    // Loop | Ifは後々処理することが可能
    let mut optimized_ops = Vec::new();

    for item_slice in block
        .items
        .split(|item| matches!(item, BlockItem::Loop(_) | BlockItem::If(_)))
    {
        // eprintln!("{item_slice:?}");

        let mut offset_ops = Vec::new();
        let mut offset = 0;

        for item in item_slice {
            match item {
                BlockItem::Op(op) => match op {
                    Op::Add(value, of) => offset_ops.push(Op::Add(*value, offset + *of as i32)),
                    Op::Sub(value, of) => offset_ops.push(Op::Sub(*value, offset + *of as i32)),
                    Op::PtrAdd(x) => offset += *x as i32,
                    Op::PtrSub(x) => offset -= *x as i32,
                    Op::Mul(x, y, of) => offset_ops.push(Op::Mul(*x, *y, offset + *of as i32)),
                    Op::Set(value, of) => offset_ops.push(Op::Set(*value, offset + *of as i32)),
                    Op::Out(of) => offset_ops.push(Op::Out(offset + *of as i32)),
                    Op::Input(of) => offset_ops.push(Op::Input(offset + *of as i32)),
                },
                BlockItem::Loop(_) | BlockItem::If(_) => unreachable!(),
            }
        }

        // offsetの最小値を計算
        let min_offset = offset_ops
            .iter()
            .filter_map(|op| match op {
                Op::Add(_, offset)
                | Op::Sub(_, offset)
                | Op::Out(offset)
                | Op::Input(offset)
                | Op::Set(_, offset)
                | Op::Mul(_, _, offset) => Some(*offset),
                Op::PtrAdd(_) => todo!(),
                Op::PtrSub(_) => todo!(),
            })
            .min();

        let mut ops = Vec::new();

        if let Some(min_offset) = min_offset {
            if min_offset != 0 {
                ops.push(BlockItem::Op(Op::ptr(min_offset)));
            }

            ops.extend(
                offset_ops
                    .into_iter()
                    .map(|op| match op {
                        Op::Add(value, offset) => Op::Add(value, (offset - min_offset) as u32),
                        Op::Sub(value, offset) => Op::Sub(value, (offset - min_offset) as u32),
                        Op::PtrAdd(_) => todo!(),
                        Op::PtrSub(_) => todo!(),
                        Op::Mul(x, y, offset) => Op::Mul(x, y, (offset - min_offset) as u32),
                        Op::Set(value, offset) => Op::Set(value, dbg!(offset - min_offset) as u32),
                        Op::Out(offset) => Op::Out((offset - min_offset) as u32),
                        Op::Input(offset) => Op::Input((offset - min_offset) as u32),
                    })
                    .map(BlockItem::Op),
            );

            // 謎の命名
            let of = dbg!(offset - min_offset);
            if of != 0 {
                ops.push(BlockItem::Op(Op::ptr(of)));
            }

            eprintln!("{ops:?}");
        } else if offset != 0 {
            ops.push(BlockItem::Op(Op::ptr(offset)));
        }

        optimized_ops.push(ops);
    }

    let mut optimized_block = Block::new();

    let mut optimized_ops = optimized_ops.into_iter();

    // Loop | Ifを処理する
    let mut optimized_loops = block.items.iter().filter_map(|item| match item {
        BlockItem::Op(_) => None,
        BlockItem::Loop(b) => Some(BlockItem::Loop(offset_opt(b))),
        BlockItem::If(b) => Some(BlockItem::If(offset_opt(b))),
    });

    // eprintln!("{optimized_ops:?}");
    // eprintln!("{optimized_loops:?}");

    loop {
        match (optimized_ops.next(), optimized_loops.next()) {
            (Some(mut ops), Some(loops)) => {
                optimized_block.items.append(&mut ops);
                optimized_block.push_item(loops);
            }
            (Some(mut ops), None) => {
                optimized_block.items.append(&mut ops);
            }
            (_, _) => break,
        }
    }

    optimized_block
}

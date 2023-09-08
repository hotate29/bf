use std::{collections::BTreeMap, ops::Add};

use crate::ir::{Block, BlockItem, Op};

pub fn optimize(block: &Block, is_top_level: bool, non_negative_offset: bool) -> Block {
    let mut block = merge(block, is_top_level);

    unwrap(&mut block);
    clear(&mut block);
    mul(&mut block);
    let mut block = merge(&block, is_top_level);
    if_opt(&mut block);
    let mut block = offset_opt(&block);

    if non_negative_offset {
        block = to_not_negative_offset(&block);
    }

    let mut block = merge(&block, is_top_level);
    remove_nop(&mut block);

    block
}

pub fn optimize_for_interpreter(block: &mut Block) {
    opt_lick(block)
}

impl Add for Op {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Op::Add(n, o), Op::Add(m, f)) if o == f => Some(Op::Add(n + m, o)),
            (Op::MovePtr(n), Op::MovePtr(m)) => Some(Op::ptr(n + m)),
            (Op::Set(0, o), Op::Mul(_, _, f)) if o == f => Some(Op::Set(0, o)),
            (Op::Set(_, o), Op::Set(_, f)) if o == f => Some(rhs),
            (Op::Set(x, o), Op::Add(y, f)) if o == f => Some(Op::Set(x + y, o)),
            // 0を足し引きするのは無駄なので、適当な機会に消滅してほしい。
            (op, rhs) if rhs.is_nop() => Some(op),
            (_, _) => None,
        }
    }
}

impl Add for &Op {
    type Output = Option<Op>;

    fn add(self, rhs: Self) -> Self::Output {
        (*self) + (*rhs)
    }
}

fn remove_nop(block: &mut Block) {
    block
        .items
        .retain(|item| !matches!(item, BlockItem::Op(op) if op.is_nop()));

    block.items.iter_mut().for_each(|item| {
        if let BlockItem::Loop(block) | BlockItem::If(block) = item {
            remove_nop(block)
        }
    });
}

// 2回以上適用すると壊れる！（ポインターが負になる？）
fn to_not_negative_offset(block: &Block) -> Block {
    fn map_ops(ops: &mut Vec<Op>, negative_offset: i32) {
        ops.iter_mut().for_each(|op| {
            *op = op
                .map_offset(|offset| offset - negative_offset)
                .unwrap_or(*op);
        });

        ops.insert(0, Op::ptr(negative_offset));
        ops.push(Op::ptr(-negative_offset));
    }
    let mut ops = vec![];

    let mut offset = 0;
    let mut min_offset = 0;

    let mut new_block = Block::new();

    for item in &block.items {
        match item {
            BlockItem::Op(op) => {
                if let Some(op_offset) = op.offset() {
                    min_offset = min_offset.min(op_offset + offset);
                }
                if let Op::MovePtr(moving) = op {
                    offset += moving;
                }
                ops.push(*op)
            }
            item @ (BlockItem::Loop(_) | BlockItem::If(_)) => {
                if min_offset.is_negative() {
                    map_ops(&mut ops, min_offset);
                }
                new_block
                    .items
                    .extend(ops.iter().copied().map(BlockItem::Op));

                new_block.push_item(item.map_block(to_not_negative_offset).unwrap());

                ops.clear();

                offset = 0;
                min_offset = 0;
            }
        }
    }

    if min_offset.is_negative() {
        map_ops(&mut ops, min_offset);
    }
    new_block
        .items
        .extend(ops.iter().copied().map(BlockItem::Op));

    new_block
}

/// `block`から合体可能な命令を見つけて合体する。
/// `is_top_level`を`true`にした場合、先頭に`Set(0, 0)`を追加して処理する。
pub(crate) fn merge(block: &Block, is_top_level: bool) -> Block {
    let mut merged_block = Block::new();

    if is_top_level {
        merged_block.items.push(BlockItem::Op(Op::Set(0, 0)));
    }

    for item in &block.items {
        let item = match item {
            BlockItem::Loop(loop_block) => BlockItem::Loop(merge(loop_block, false)),
            BlockItem::If(if_block) => BlockItem::If(merge(if_block, false)),
            BlockItem::Op(op) => BlockItem::Op(*op),
        };
        merged_block.push_item(item);

        // 連鎖的に消えるかもしれないのでwhile
        while let Some(merged) = {
            let lhs = merged_block
                .items
                .iter()
                .nth_back(1)
                .and_then(BlockItem::op);
            let rhs = merged_block.items.last().and_then(BlockItem::op);

            lhs.zip(rhs).and_then(|(lhs, rhs)| lhs + rhs)
        } {
            merged_block.items.pop().unwrap();
            merged_block.items.pop().unwrap();
            merged_block.push_item(BlockItem::Op(merged))
        }
    }

    if is_top_level && Some(&BlockItem::Op(Op::Set(0, 0))) == merged_block.items.first() {
        merged_block.items.remove(0);
    }

    merged_block
}

pub(crate) fn clear(block: &mut Block) {
    for item in &mut block.items {
        if let BlockItem::Loop(block) = item {
            if let [BlockItem::Op(Op::Add(1, 0)) | BlockItem::Op(Op::Add(-1, 0))] =
                block.items.as_slice()
            {
                *item = BlockItem::Op(Op::Set(0, 0));
            } else {
                clear(block);
            }
        }
    }
}

pub(crate) fn unwrap(block: &mut Block) {
    fn inner(item: &mut BlockItem) -> bool {
        if let BlockItem::Loop(loop_block) = item {
            if let [BlockItem::Loop(deep_loop_block)] = loop_block.items.as_slice() {
                *loop_block = deep_loop_block.clone();
                return true;
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

pub(crate) fn mul(block: &mut Block) {
    #[derive(Debug, PartialEq, Eq)]
    enum OpType {
        Mul(i32),
        Set(i32),
    }
    impl OpType {
        fn mul(&mut self, x: i32) {
            match self {
                OpType::Mul(y) | OpType::Set(y) => *y += x,
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
                | BlockItem::Op(Op::Mul(_, _, _) | Op::Lick(_) | Op::Out(_) | Op::Input(_)) => {
                    return None
                }

                BlockItem::Op(op) => match op {
                    Op::Add(v, of) => {
                        offset_op
                            .entry(ptr_offset + *of)
                            .and_modify(|x| x.mul(*v))
                            .or_insert(OpType::Mul(*v));
                    }
                    Op::MovePtr(of) => ptr_offset += *of,
                    Op::Set(v, offset) => {
                        offset_op.insert(ptr_offset + *offset, OpType::Set(*v));
                    }
                    Op::Mul(_, _, _) | Op::Lick(_) | Op::Out(_) | Op::Input(_) => {
                        unreachable!()
                    }
                },
            };
        }

        let clear_minus = offset_op.get(&0) == Some(&OpType::Mul(-1));

        (ptr_offset == 0 && clear_minus).then_some(offset_op)
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
        let mut offset_ops = Vec::new();
        let mut offset = 0;

        for item in item_slice {
            match item {
                BlockItem::Op(Op::MovePtr(x)) => {
                    offset += *x;
                }
                BlockItem::Op(op) => offset_ops.push(op.map_offset(|of| of + offset).unwrap()),
                BlockItem::Loop(_) | BlockItem::If(_) => unreachable!(),
            }
        }

        // 帳尻を合わせる
        offset_ops.push(Op::ptr(offset));

        let items = offset_ops
            .into_iter()
            .map(BlockItem::Op)
            .collect::<Vec<_>>();
        optimized_ops.push(items);
    }

    let mut optimized_block = Block::new();

    let mut optimized_ops = optimized_ops.into_iter();

    // Loop | Ifを処理する
    let mut optimized_loops = block
        .items
        .iter()
        .filter_map(|item| item.map_block(offset_opt));

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

pub fn if_opt(block: &mut Block) {
    fn inner(loop_item: &mut BlockItem) {
        match loop_item {
            BlockItem::Loop(block) => {
                if Some(&BlockItem::Op(Op::Set(0, 0))) == block.items.last() {
                    if block.items.len() == 1 {
                        *loop_item = BlockItem::Op(Op::Set(0, 0));
                    } else {
                        let if_items = block.items.clone();
                        let mut if_block = Block::from_items(if_items);
                        if_opt(&mut if_block);
                        *loop_item = BlockItem::If(if_block);
                    }
                } else {
                    if_opt(block);
                }
            }
            BlockItem::If(block) => if_opt(block),
            BlockItem::Op(_) => (),
        }
    }
    block.items.iter_mut().for_each(inner);
}

fn opt_lick(block: &mut Block) {
    for block_item in &mut block.items {
        // 中がポインタ移動のみか判定する
        fn is_only_move_ptr(block: &Block) -> i32 {
            if let [BlockItem::Op(Op::MovePtr(x))] = block.items.as_slice() {
                *x
            } else {
                0
            }
        }

        if let BlockItem::Loop(loop_block) = block_item {
            let offset = is_only_move_ptr(loop_block);
            if offset != 0 {
                eprintln!("lick: {}", offset);
                *block_item = BlockItem::Op(Op::Lick(offset));
            }
        }

        if let BlockItem::Loop(block) | BlockItem::If(block) = block_item {
            opt_lick(block)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{utils::bf_to_block, InterPreter};

    use super::*;

    fn run(block: &Block) -> (Vec<u8>, usize) {
        let mut interpreter = InterPreter::builder()
            .memory(vec![0u8; 300000])
            .input(io::empty())
            .output(io::sink())
            .root_node(block)
            .build();

        interpreter.run().unwrap();
        let memory = interpreter.memory().to_vec();
        let pointer = interpreter.pointer();
        (memory, pointer)
    }

    #[test]
    fn same_state() {
        let block = bf_to_block("+++[>+++<-]>.").unwrap();
        let optimized_block = optimize(&block, true, false);
        assert_eq!(run(&block), run(&optimized_block));

        let block = bf_to_block("+++++++>>>>>>>>>--------<<<<<<<<<++++++").unwrap();
        let optimized_block = optimize(&block, true, false);
        assert_eq!(run(&block), run(&optimized_block));

        let block = bf_to_block("+[-]-[-]+[+]").unwrap();
        let optimized_block = optimize(&block, true, false);
        assert_eq!(run(&block), run(&optimized_block));

        let block = bf_to_block("+++[[[[[>+++<-]]]]]>.").unwrap();
        let optimized_block = optimize(&block, true, false);
        assert_eq!(run(&block), run(&optimized_block));

        let block = bf_to_block("+++++[[-]>++++++<]").unwrap();
        let optimized_block = optimize(&block, true, false);
        assert_eq!(run(&block), run(&optimized_block));

        let block = bf_to_block(">>>+++>>>+++[-<+++>]").unwrap();
        let optimized_block = optimize(&block, true, false);
        assert_eq!(run(&block), run(&optimized_block));

        // let block = bf_to_block("[-]>[<+>>+<-]>[-]<<[>>+<+<-]>[<+>-]>>++++[<<+++++>>-]<[-<[>>+>+<<<-]>>>[<<<+>>>-]+<[<<->>>-<[-]]>[<<[-]>>-]<<]<[>+<[-]]>").unwrap();
        // let optimized_block = optimize(&block, true, false);
        // assert_eq!(run(&block), run(&optimized_block));
    }

    #[test]
    fn test_offset_opt() {
        let block = bf_to_block("+>+>+>+[-]<<<->>>").unwrap();
        let mut optimized_block = block.clone();
        clear(&mut optimized_block);
        let optimized_block = offset_opt(&optimized_block);
        assert_eq!(
            &[
                BlockItem::Op(Op::Add(1, 0)),
                BlockItem::Op(Op::Add(1, 1)),
                BlockItem::Op(Op::Add(1, 2)),
                BlockItem::Op(Op::Add(1, 3)),
                BlockItem::Op(Op::Set(0, 3)),
                BlockItem::Op(Op::Add(-1, 0)),
                BlockItem::Op(Op::ptr(3)),
            ],
            optimized_block.items.as_slice()
        );
    }

    #[test]
    fn test_unwrap() {
        let mut block = bf_to_block("[[[[[-]]]]]").unwrap();
        unwrap(&mut block);

        assert_eq!(block, bf_to_block("[-]").unwrap());

        let mut block = bf_to_block("[[[+][[[-]]]]]").unwrap();
        unwrap(&mut block);

        assert_eq!(block, bf_to_block("[[+][-]]").unwrap());
    }

    #[test]
    fn test_to_not_negative_offset() {
        let block = Block::from_items(vec![
            BlockItem::Op(Op::Add(1, -5)),
            BlockItem::Op(Op::ptr(-5)),
        ]);
        let block = to_not_negative_offset(&block);

        assert_eq!(
            block.items,
            vec![
                BlockItem::Op(Op::ptr(-5)),
                BlockItem::Op(Op::Add(1, 0)),
                BlockItem::Op(Op::ptr(-5)),
                BlockItem::Op(Op::ptr(5)),
            ]
        );
    }
}

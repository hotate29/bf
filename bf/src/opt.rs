use crate::transpile::{
    wasm::{opt, BlockItem, Op},
    Block,
};

pub fn optimize(mut block: Block, is_top_level: bool) -> Block {
    if is_top_level {
        block.items.insert(0, BlockItem::Op(Op::Set(0, 0)));
    }

    let mut block = opt::merge(&block);

    opt::unwrap(&mut block);
    opt::clear(&mut block);
    opt::mul(&mut block);
    let mut block = opt::merge(&block);
    opt::if_opt(&mut block);
    opt::offset_opt(&block)
}

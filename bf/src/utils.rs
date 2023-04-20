use ir::Block;

use crate::{ir, opt, parse, Error};

pub fn bf_to_block(bf: &str, optimize: bool) -> Result<ir::Block, Error> {
    let ast = parse::parse(bf)?;
    let block = Block::from_ast(&ast);
    if optimize {
        Ok(opt::optimize(block, true))
    } else {
        Ok(block)
    }
}

use ir::Block;

use crate::{ir, parse, Error};

pub fn bf_to_block(bf: &str) -> Result<ir::Block, Error> {
    let ast = parse::parse(bf)?;
    let block = Block::from_ast(&ast);

    Ok(block)
}

pub mod error;
pub mod interpreter;
pub mod ir;
pub mod opt;
pub mod parse;
pub mod transpile;
pub mod utils;

pub use error::Error;
pub use interpreter::InterPreter;
use ir::Block;
pub use transpile::{
    c::block_to_c,
    wasm::{block_to_wasm, block_to_wat},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn bf_to_wasm(bf: &str, optimize: bool) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    let ast = parse::parse(bf).map_err(|e| e.to_string())?;
    let mut block = Block::from_ast(&ast);

    if optimize {
        block = opt::optimize(&block, true, true);
    }

    block_to_wasm(&block, &mut buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}

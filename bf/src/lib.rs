pub mod error;
pub mod interpreter;
pub mod ir;
pub mod opt;
pub mod parse;
pub mod transpile;
pub mod utils;

pub use error::Error;
pub use interpreter::InterPreter;
pub use transpile::{
    c::block_to_c,
    wasm::{block_to_wasm, block_to_wat},
};

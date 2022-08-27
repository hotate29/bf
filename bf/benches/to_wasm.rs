#![feature(test)]
extern crate test;

use bf::transpile::wasm::{to_wasm, Block};

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_block_to_wasm(bencher: &mut test::Bencher) {
    let block = Block::from_bf(MANDELBROT).unwrap();
    let optimized_block = block.optimize(true);
    bencher.iter(|| {
        let mut buffer = Vec::new();
        to_wasm(&optimized_block, &mut buffer).unwrap();
    })
}

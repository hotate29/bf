#![feature(test)]
extern crate test;

use bf::transpile::wasm::Block;

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let block = Block::from_bf(MANDELBROT).unwrap();

    bencher.iter(|| block.optimize(true))
}

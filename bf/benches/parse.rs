#![feature(test)]
extern crate test;

use bf::transpile::wasm::Block;

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_parse_mandelbrot(bencher: &mut test::Bencher) {
    bencher.iter(|| Block::from_bf(MANDELBROT).unwrap())
}

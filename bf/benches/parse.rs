#![feature(test)]
extern crate test;

use bf::{parse::Ast, transpile::wasm::Block};

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_ast_to_block_mandelbrot(bencher: &mut test::Bencher) {
    bencher.iter(|| Block::from_bf(MANDELBROT).unwrap())
}

#[bench]
fn bench_parse_ast_mandelbrot(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        MANDELBROT.parse::<Ast>().unwrap();
    })
}

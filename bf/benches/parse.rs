#![feature(test)]
extern crate test;

use bf::{
    parse::{tokenize, Node},
    transpile::wasm::Block,
};

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_parse_mandelbrot(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let source = tokenize(MANDELBROT);
        Node::from_tokens(source).unwrap()
    })
}

#[bench]
fn bench_parse_transpile_mandelbrot(bencher: &mut test::Bencher) {
    bencher.iter(|| Block::from_bf(MANDELBROT).unwrap())
}

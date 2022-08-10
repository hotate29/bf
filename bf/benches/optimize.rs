#![feature(test)]
extern crate test;

use bf::optimize::optimize;
use bf::parse::{tokenize, Node};
use bf::transpile::wasm::Block;

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let tokens = tokenize(MANDELBROT);
    let root_node = Node::from_tokens(tokens).unwrap();

    bencher.iter(|| optimize(&root_node))
}

#[bench]
fn bench_optimizing_transpile_mandelbrot(bencher: &mut test::Bencher) {
    let block = Block::from_bf(MANDELBROT).unwrap();

    bencher.iter(|| block.optimize(true))
}

#![feature(test)]
extern crate test;

use bf::optimize::optimize;
use bf::parse::{tokenize, Node};
use bf::transpile;

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let source = include_str!("../../bf_codes/mandelbrot.bf");

    let tokens = tokenize(source);
    let root_node = Node::from_tokens(tokens).unwrap();

    bencher.iter(|| optimize(&root_node))
}

#[bench]
fn bench_optimizing_transpile_mandelbrot(bencher: &mut test::Bencher) {
    let source = include_str!("../../bf_codes/mandelbrot.bf");

    let block = transpile::wasm::bf_to_block(source);

    bencher.iter(|| block.optimize(true))
}

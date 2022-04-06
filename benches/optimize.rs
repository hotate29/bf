#![feature(test)]
extern crate test;

use bf::optimize::optimize;
use bf::parse::{tokenize, Node};

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let source = include_str!("mandelbrot.bf");

    let tokens = tokenize(source);
    let root_node = Node::from_tokens(tokens).unwrap();

    bencher.iter(|| optimize(&root_node))
}

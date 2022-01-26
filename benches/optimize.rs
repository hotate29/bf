#![feature(test)]
extern crate test;

use std::fs;

use bf::optimize::optimize;
use bf::token::Node;

#[bench]
fn bench_optimizing(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let root_node = Node::from_source(&source).unwrap();

    bencher.iter(|| optimize(root_node.clone()))
}
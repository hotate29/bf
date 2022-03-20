#![feature(test)]
extern crate test;

use std::fs;

use bf::optimize::{all_optimizer, optimize};
use bf::parse::Node;

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let root_node = Node::from_source(&source).unwrap();

    bencher.iter(|| optimize(root_node.clone(), &all_optimizer()))
}

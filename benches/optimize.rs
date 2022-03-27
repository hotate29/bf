#![feature(test)]
extern crate test;

use std::fs;

use bf::optimize_new_node;
use bf::parse::{tokenize, Nod};

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let tokens = tokenize(&source);
    let root_node = Nod::from_tokens(tokens).unwrap();

    bencher.iter(|| optimize_new_node::optimize(root_node.clone()))
}

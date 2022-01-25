#![feature(test)]
extern crate test;

use std::fs;

use bf::optimize::optimize;
use bf::token::{middle_token, node, tokenize};

#[bench]
fn bench_optimizing(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let tokens = tokenize(&source);
    let middle_tokens = middle_token(&tokens);
    let root_node = node(&middle_tokens);

    bencher.iter(|| optimize(root_node.clone()))
}

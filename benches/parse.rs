#![feature(test)]
extern crate test;

use std::fs;

use bf::parse::{tokenize, Node};

#[bench]
fn bench_parse_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();
    bencher.iter(|| {
        let source = tokenize(&source);
        Node::from_tokens(source).unwrap()
    })
}

#[bench]
fn bench_parse_new_node(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    bencher.iter(|| {
        let tokens = tokenize(&source);
        Node::from_tokens(tokens).unwrap();
    })
}

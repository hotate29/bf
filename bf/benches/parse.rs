#![feature(test)]
extern crate test;

use bf::parse::{tokenize, Node};

#[bench]
fn bench_parse_mandelbrot(bencher: &mut test::Bencher) {
    let source = include_str!("../../bf_codes/mandelbrot.bf");

    bencher.iter(|| {
        let source = tokenize(source);
        Node::from_tokens(source).unwrap()
    })
}

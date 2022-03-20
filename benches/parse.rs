#![feature(test)]
extern crate test;

use std::fs;

use bf::parse::Node;

#[bench]
fn bench_parse_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    bencher.iter(|| Node::from_source(&source).unwrap())
}

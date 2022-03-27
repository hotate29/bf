#![feature(test)]
extern crate test;

use std::fs;

use bf::parse::{tokenize, Nod};

#[bench]
fn bench_parse_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();
    bencher.iter(|| {
        let source = tokenize(&source);
        Nod::from_tokens(source).unwrap()
    })
}

#[bench]
fn bench_parse_new_node(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    bencher.iter(|| {
        let tokens = tokenize(&source);
        Nod::from_tokens(tokens).unwrap();
    })
}

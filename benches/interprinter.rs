#![feature(test)]
extern crate test;

use std::fs;
use std::io;

use bf::interprinter::InterPrinter;
use bf::optimize::optimize;
use bf::token::{middle_token, node, tokenize};

#[bench]
fn test_not_optimize_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let tokens = tokenize(&source);
    let middle_tokens = middle_token(&tokens);
    let root_node = node(&middle_tokens);

    bencher.iter(|| {
        InterPrinter::new(root_node.clone(), io::empty(), io::sink()).start();
    })
}

#[bench]
fn test_optimized_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let tokens = tokenize(&source);
    let middle_tokens = middle_token(&tokens);
    let root_node = node(&middle_tokens);

    let optimized_node = optimize(root_node);

    bencher.iter(|| {
        InterPrinter::new(optimized_node.clone(), io::empty(), io::sink()).start();
    })
}

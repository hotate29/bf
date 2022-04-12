#![feature(test)]
extern crate test;

use std::io;

use bf::interprinter::InterPrinter;
use bf::optimize::optimize;
use bf::parse::tokenize;
use bf::parse::Node;

#[bench]
fn bench_not_optimize_mandelbrot(bencher: &mut test::Bencher) {
    let source = include_str!("../../bf_codes/mandelbrot.bf");

    let tokens = tokenize(source);
    let root_node = Node::from_tokens(tokens).unwrap();

    bencher.iter(|| {
        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
    })
}

#[bench]
fn bench_optimized_mandelbrot(bencher: &mut test::Bencher) {
    let source = include_str!("../../bf_codes/mandelbrot.bf");

    let tokens = tokenize(source);
    let root_node = Node::from_tokens(tokens).unwrap();

    let optimized_node = optimize(&root_node);

    bencher.iter(|| {
        InterPrinter::builder()
            .root_node(&optimized_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
    })
}

#[bench]
fn bench_hello_world(bencher: &mut test::Bencher) {
    let hello_world = include_str!("../../bf_codes/hello_world.bf");

    let tokens = tokenize(hello_world);
    let root_node = Node::from_tokens(tokens).unwrap();

    bencher.iter(|| {
        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
    })
}

#[bench]
fn bench_optimized_hello_world(bencher: &mut test::Bencher) {
    let hello_world = include_str!("../../bf_codes/hello_world.bf");

    let tokens = tokenize(hello_world);
    let root_node = Node::from_tokens(tokens).unwrap();

    let optimized_node = optimize(&root_node);

    bencher.iter(|| {
        InterPrinter::builder()
            .root_node(&optimized_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
    })
}
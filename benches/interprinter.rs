#![feature(test)]
extern crate test;

use std::fs;
use std::io;

use bf::interprinter::InterPrinter;
use bf::optimize::all_optimizer;
use bf::optimize::{optimize, Node};

#[bench]
fn bench_not_optimize_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let root_node = Node::from_source(&source).unwrap();

    bencher.iter(|| {
        InterPrinter::new(&root_node, io::empty(), io::sink()).start();
    })
}

#[bench]
fn bench_optimized_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let root_node = Node::from_source(&source).unwrap();

    let optimized_node = optimize(root_node, &all_optimizer());

    bencher.iter(|| {
        InterPrinter::new(&optimized_node, io::empty(), io::sink()).start();
    })
}

#[bench]
fn bench_hello_world(bencher: &mut test::Bencher) {
    let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

    let root_node = Node::from_source(hello_world).unwrap();

    bencher.iter(|| {
        InterPrinter::new(&root_node, io::empty(), io::sink()).start();
    })
}

#[bench]
fn bench_optimized_hello_world(bencher: &mut test::Bencher) {
    let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

    let root_node = Node::from_source(hello_world).unwrap();
    let root_node = optimize(root_node, &all_optimizer());

    bencher.iter(|| {
        InterPrinter::new(&root_node, io::empty(), io::sink()).start();
    })
}

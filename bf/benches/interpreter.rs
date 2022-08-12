#![feature(test)]
extern crate test;

use std::io;

use bf::{interpreter::InterPreter, transpile::wasm::Block};

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_not_optimize_mandelbrot(bencher: &mut test::Bencher) {
    let block = Block::from_bf(MANDELBROT).unwrap();

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_optimized_mandelbrot(bencher: &mut test::Bencher) {
    let block = Block::from_bf(MANDELBROT).unwrap().optimize(true);

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_hello_world(bencher: &mut test::Bencher) {
    let hello_world = include_str!("../../bf_codes/hello_world.bf");

    let block = Block::from_bf(hello_world).unwrap();

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_optimized_hello_world(bencher: &mut test::Bencher) {
    let hello_world = include_str!("../../bf_codes/hello_world.bf");

    let block = Block::from_bf(hello_world).unwrap().optimize(true);

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_optimized_pi16(bencher: &mut test::Bencher) {
    let pi16 = include_str!("../../bf_codes/pi16.bf");

    let block = Block::from_bf(pi16).unwrap().optimize(true);

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .build()
            .iter()
            .count();
    })
}

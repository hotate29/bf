#![feature(test)]
extern crate test;

use std::io;

use bf::{
    interpreter::{AutoExtendMemory, InterPreter},
    opt, utils,
};

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_not_optimize_mandelbrot(bencher: &mut test::Bencher) {
    let block = utils::bf_to_block(MANDELBROT).unwrap();

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .memory(AutoExtendMemory::new(vec![0]))
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_optimized_mandelbrot(bencher: &mut test::Bencher) {
    let block = utils::bf_to_block(MANDELBROT).unwrap();
    let block = opt::optimize(&block, true, false);

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .memory(AutoExtendMemory::new(vec![0]))
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_hello_world(bencher: &mut test::Bencher) {
    let hello_world = include_str!("../../bf_codes/hello_world.bf");
    let block = utils::bf_to_block(hello_world).unwrap();

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .memory(AutoExtendMemory::new(vec![0]))
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_optimized_hello_world(bencher: &mut test::Bencher) {
    let hello_world = include_str!("../../bf_codes/hello_world.bf");

    let block = utils::bf_to_block(hello_world).unwrap();
    let block = opt::optimize(&block, true, false);

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .memory(AutoExtendMemory::new(vec![0]))
            .build()
            .iter()
            .count();
    })
}

#[bench]
fn bench_optimized_pi16(bencher: &mut test::Bencher) {
    let pi16 = include_str!("../../bf_codes/pi16.bf");

    let block = utils::bf_to_block(pi16).unwrap();
    let block = opt::optimize(&block, true, false);

    bencher.iter(|| {
        InterPreter::builder()
            .root_node(&block)
            .input(io::empty())
            .output(io::sink())
            .memory(AutoExtendMemory::new(vec![0]))
            .build()
            .iter()
            .count();
    })
}

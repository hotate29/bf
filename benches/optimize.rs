#![feature(test)]
extern crate test;

use bf::{opt::optimize, utils};

const MANDELBROT: &str = include_str!("../bf_codes/mandelbrot.bf");

#[bench]
fn bench_optimizing_mandelbrot(bencher: &mut test::Bencher) {
    let block = utils::bf_to_block(MANDELBROT).unwrap();
    bencher.iter(|| optimize(&block, true, false))
}

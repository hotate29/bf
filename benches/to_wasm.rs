#![feature(test)]
extern crate test;

use std::io;

use bf::{transpile::wasm::block_to_wasm, utils};

const MANDELBROT: &str = include_str!("../bf_codes/mandelbrot.bf");

#[bench]
fn bench_block_to_wasm(bencher: &mut test::Bencher) {
    // BlockからWasmへの変換速度を計測するので、最適化はしない。
    let block = utils::bf_to_block(MANDELBROT).unwrap();

    let mut sink = io::sink();

    bencher.iter(|| {
        block_to_wasm(&block, &mut sink).unwrap();
    })
}

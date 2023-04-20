#![feature(test)]
extern crate test;

use bf::parse::parse;

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_parse_mandelbrot_ast(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let _ = parse(MANDELBROT);
    })
}

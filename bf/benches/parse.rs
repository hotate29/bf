#![feature(test)]
extern crate test;

use bf::parse::bf_parser;
use chumsky::Parser;

const MANDELBROT: &str = include_str!("../../bf_codes/mandelbrot.bf");

#[bench]
fn bench_parse_mandelbrot_ast(bencher: &mut test::Bencher) {
    let parser = bf_parser();
    bencher.iter(|| {
        parser.parse(MANDELBROT);
    })
}

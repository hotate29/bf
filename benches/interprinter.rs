#![feature(test)]
extern crate test;

use std::fs;
use std::io;

use bf::interprinter::InterPrinter;
use bf::parse::tokenize;
use bf::parse::Nod;

#[bench]
fn bench_not_optimize_mandelbrot(bencher: &mut test::Bencher) {
    let source = fs::read_to_string("mandelbrot.bf").unwrap();

    let tokens = tokenize(&source);
    let root_node = Nod::from_tokens(tokens).unwrap();

    bencher.iter(|| {
        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
    })
}

// #[bench]
// fn bench_optimized_mandelbrot(bencher: &mut test::Bencher) {
//     let source = fs::read_to_string("mandelbrot.bf").unwrap();

//     let tokens = tokenize(&source);
//     let root_node = Nod::from_tokens(tokens).unwrap();

//     let optimized_node = optimize(root_node, &all_optimizer());

//     bencher.iter(|| {
//         InterPrinter::builder()
//             .root_node(&optimized_node)
//             .input(io::empty())
//             .output(io::sink())
//             .build()
//             .count();
//     })
// }

#[bench]
fn bench_hello_world(bencher: &mut test::Bencher) {
    let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

    let tokens = tokenize(hello_world);
    let root_node = Nod::from_tokens(tokens).unwrap();

    bencher.iter(|| {
        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::empty())
            .output(io::sink())
            .build()
            .count();
    })
}

// #[bench]
// fn bench_optimized_hello_world(bencher: &mut test::Bencher) {
//     let hello_world = ">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]>++++++++[<++++>-]<.>+++++++++++[<+++++>-]<.>++++++++[<+++>-]<.+++.------.--------.[-]>++++++++[<++++>-]<+.[-]++++++++++.";

//     let tokens = tokenize(hello_world);
//     let root_node = Nod::from_tokens(tokens).unwrap();

//     let optimized_node = optimize(root_node, &all_optimizer());

//     bencher.iter(|| {
//         InterPrinter::builder()
//             .root_node(&optimized_node)
//             .input(io::empty())
//             .output(io::sink())
//             .build()
//             .count();
//     })
// }

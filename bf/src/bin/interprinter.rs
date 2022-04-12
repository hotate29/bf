use std::{
    env::args,
    fs, io,
    time::{Duration, Instant},
};

use bf::{
    interprinter::InterPrinter,
    optimize::optimize,
    parse::{tokenize, Node},
};
use log::info;

fn time_keisoku<F, T>(func: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let instant = Instant::now();
    let func_out = func();
    (func_out, instant.elapsed())
}

fn main() {
    env_logger::init();
    let args = args().collect::<Vec<String>>();

    let code = fs::read_to_string(&args[1]).unwrap();

    let optimize_flag = args.get(2).map_or(false, |arg| arg == "O");

    let tokens = tokenize(&code);
    let mut root_node = Node::from_tokens(tokens).unwrap();

    if optimize_flag {
        let (optimized_node, dur) = time_keisoku(|| optimize(&root_node));
        root_node = optimized_node;

        info!("optimize: {}ms", dur.as_millis());
    }

    let (step_count, dur) = time_keisoku(|| {
        InterPrinter::builder()
            .root_node(&root_node)
            .input(io::stdin())
            .output(io::stdout())
            .build()
            .count()
    });

    info!("run: {}ms", dur.as_millis());
    eprintln!("step: {step_count}");
}

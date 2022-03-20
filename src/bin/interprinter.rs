use std::{env::args, fs, io};

use bf::{
    interprinter::InterPrinter,
    optimize::{all_optimizer, optimize},
    parse::Node,
};

fn main() {
    env_logger::init();
    let args = args().collect::<Vec<String>>();

    let code = fs::read_to_string(&args[1]).unwrap();

    let optimize_flag = args.get(2).map_or(false, |arg| arg == "O");

    let mut root_node = Node::from_source(&code).unwrap();
    if optimize_flag {
        root_node = optimize(root_node, &all_optimizer());
    }

    let step_count = InterPrinter::builder()
        .root_node(&root_node)
        .input(io::stdin())
        .output(io::stdout())
        .build()
        .count();

    eprintln!("step: {step_count}");
}

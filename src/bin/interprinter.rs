use std::{env::args, fs, io};

use bf::{
    interprinter::InterPrinter,
    optimize::{all_optimizer, optimize, Node},
};

fn main() {
    env_logger::init();
    let args = args().collect::<Vec<String>>();

    let code = fs::read_to_string(&args[1]).unwrap();

    let optimize_flag = args.get(2) == Some(&"O".to_string());

    let mut root_node = Node::from_source(&code).unwrap();
    if optimize_flag {
        root_node = optimize(root_node, &all_optimizer());
    }

    InterPrinter::new(root_node, io::stdin(), io::stdout()).start();
}

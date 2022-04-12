use std::{fs, io, path::PathBuf};

use bf::{
    interprinter::InterPrinter,
    optimize::optimize,
    parse::{tokenize, Node},
};
use clap::StructOpt;

#[derive(Debug, clap::Parser)]
struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
    Run(RunArg),
}

#[derive(Debug, clap::Parser)]
struct RunArg {
    file: PathBuf,
    #[clap(short, long)]
    optimize: bool,
}

macro_rules! time {
    ($e:expr) => {{
        let instant = std::time::Instant::now();
        let e_ret = $e;
        let dur = instant.elapsed();
        log::info!("line:{} {} {}ms", line!(), stringify!($e), dur.as_millis());
        e_ret
    }};
}

fn main() {
    env_logger::init();

    let arg = Command::parse();

    match arg.subcommand {
        SubCommand::Run(arg) => {
            let code = fs::read_to_string(arg.file).unwrap();

            let tokens = tokenize(&code);

            let mut root_node = Node::from_tokens(tokens).unwrap();

            if arg.optimize {
                root_node = time!(optimize(&root_node))
            }

            let interpreter = InterPrinter::builder()
                .input(io::stdin())
                .output(io::stdout())
                .root_node(&root_node)
                .memory_len(30000)
                .build();

            time!(interpreter.count());
        }
    }
}

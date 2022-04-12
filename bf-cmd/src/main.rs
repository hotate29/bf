use std::{fs, io, path::PathBuf};

use bf::{
    interprinter::InterPrinter,
    optimize::optimize,
    parse::{tokenize, Node},
    transcompile,
};
use clap::StructOpt;
use log::info;

#[derive(Debug, clap::Parser)]
struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
    Run(RunArg),
    Trans(TransArg),
}

#[derive(Debug, clap::Parser)]
struct RunArg {
    file: PathBuf,
    #[clap(short, long)]
    optimize: bool,
}

#[derive(Debug, clap::Parser)]
struct TransArg {
    file: PathBuf,
    #[clap(short, long)]
    optimize: bool,
    out: Option<PathBuf>,
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

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let arg = Command::parse();

    match arg.subcommand {
        SubCommand::Run(arg) => {
            let code = fs::read_to_string(arg.file)?;

            let tokens = tokenize(&code);

            let mut root_node = Node::from_tokens(tokens)?;

            if arg.optimize {
                root_node = time!(optimize(&root_node))
            }

            let interpreter = InterPrinter::builder()
                .input(io::stdin())
                .output(io::stdout())
                .root_node(&root_node)
                .memory_len(30000)
                .build();

            let step_count = time!(interpreter.count());
            info!("step: {step_count}");
        }
        SubCommand::Trans(arg) => {
            let code = fs::read_to_string(&arg.file)?;

            let tokens = tokenize(&code);

            let mut root_node = Node::from_tokens(tokens)?;

            if arg.optimize {
                root_node = time!(optimize(&root_node))
            }

            let code = transcompile::to_c2(&root_node);

            let filename = arg.out.unwrap_or_else(|| PathBuf::from("a.c"));

            fs::write(filename, code)?;
        }
    }
    Ok(())
}

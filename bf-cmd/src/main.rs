use std::{
    fs,
    io::{self, Read},
    num::NonZeroUsize,
    path::PathBuf,
};

use bf::{
    interpreter::InterPreter,
    optimize::optimize,
    parse::{tokenize, Node},
    transcompile,
};
use clap::{ArgEnum, StructOpt};
use log::{info, Level};

#[derive(Debug, clap::Parser)]
struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
    #[clap(long, env = "RUST_LOG", default_value_t = Level::Warn)]
    log_level: Level,
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
    #[clap(long, default_value_t = NonZeroUsize::try_from(30000).unwrap())]
    initial_memory_len: NonZeroUsize,
}

#[derive(Debug, clap::Parser)]
struct TransArg {
    file: Option<PathBuf>,
    #[clap(arg_enum, value_parser)]
    target: TransTarget,
    #[clap(short, long)]
    optimize: bool,
    out: Option<PathBuf>,
    #[clap(short, long, default_value_t = 30000)]
    memory_len: usize,
}

#[derive(Debug, Clone, Copy, ArgEnum)]
enum TransTarget {
    C,
    Wat,
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
    let arg = Command::parse();

    env_logger::builder()
        .filter_level(arg.log_level.to_level_filter())
        .init();

    match arg.subcommand {
        SubCommand::Run(arg) => {
            let code = fs::read_to_string(arg.file)?;

            let tokens = tokenize(&code);

            let mut root_node = Node::from_tokens(tokens)?;

            if arg.optimize {
                root_node = time!(optimize(&root_node))
            }

            let mut interpreter = InterPreter::builder()
                .input(io::stdin())
                .output(io::stdout())
                .root_node(&root_node)
                .memory_len(arg.initial_memory_len.get())
                .build();

            let step_count = time!(interpreter.iter().count());
            info!("step: {step_count}");
        }
        SubCommand::Trans(arg) => {
            let code = match arg.file {
                Some(path) => fs::read_to_string(path)?,
                None => {
                    let mut code = String::new();
                    io::stdin().read_to_string(&mut code)?;
                    code
                }
            };

            let output = match arg.target {
                TransTarget::C => {
                    let tokens = tokenize(&code);

                    let mut root_node = Node::from_tokens(tokens)?;

                    if arg.optimize {
                        root_node = time!(optimize(&root_node))
                    }

                    transcompile::c::to_c(&root_node, arg.memory_len)
                }
                TransTarget::Wat => transcompile::wasm::bf_to_wat(&code),
            };

            match arg.out {
                Some(path) => fs::write(path, output)?,
                None => println!("{output}"),
            }
        }
    }
    Ok(())
}

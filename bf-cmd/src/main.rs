use std::{
    fs::{self, File},
    io::{self, Write},
    num::NonZeroUsize,
    path::PathBuf,
};

use bf::{interpreter::AutoExtendMemory, transpile, utils::bf_to_block, InterPreter};
use clap::{ArgEnum, StructOpt};
use log::{info, Level};

#[derive(Debug, clap::Parser)]
struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
    #[clap(long, env = "RUST_LOG", default_value_t = Level::Info)]
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
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, clap::Parser)]
struct TransArg {
    #[clap(value_parser)]
    file: PathBuf,
    #[clap(arg_enum, default_value_t = TransTarget::Wasm)]
    target: TransTarget,
    #[clap(short, long)]
    optimize: bool,
    out: Option<PathBuf>,
    #[clap(short, long, default_value_t = 30000)]
    memory_len: usize,
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, ArgEnum)]
enum TransTarget {
    C,
    Wat,
    Wasm,
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

            let block = bf_to_block(&code, arg.optimize)?;
            if arg.verbose {
                info!("block: {:#?}", block);
            }

            let mut interpreter = InterPreter::builder()
                .input(io::stdin())
                .output(io::stdout())
                .root_node(&block)
                .memory(AutoExtendMemory::new(vec![0; arg.initial_memory_len.get()]))
                .build();

            let step_count = time!(interpreter.iter().count());
            info!("step: {step_count}");
        }
        SubCommand::Trans(arg) => {
            let code = fs::read_to_string(arg.file)?;

            let block = bf_to_block(&code, arg.optimize)?;
            if arg.verbose {
                info!("block: {:#?}", block);
            }

            let output_path = arg.out.unwrap_or_else(|| {
                match arg.target {
                    TransTarget::C => "a.c",
                    TransTarget::Wat => "a.wat",
                    TransTarget::Wasm => "a.wasm",
                }
                .into()
            });

            let mut output = File::create(&output_path)?;

            match arg.target {
                TransTarget::C => {
                    let c_code = transpile::block_to_c(&block, arg.memory_len);
                    output.write_all(c_code.as_bytes())?;
                }
                TransTarget::Wat => {
                    transpile::block_to_wat(&block, &mut output)?;
                }
                TransTarget::Wasm => {
                    transpile::block_to_wasm(&block, &mut output)?;
                }
            };

            info!("Done {:?}", output_path);
        }
    }
    Ok(())
}

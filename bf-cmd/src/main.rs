use std::{
    fs::{self, File},
    io::{self, Write},
    num::NonZeroIsize,
    path::PathBuf,
};

use anyhow::Context;
use bf::{
    interpreter::AutoExtendMemory, opt::optimize_for_interpreter, transpile, utils::bf_to_block,
    InterPreter,
};
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
    #[clap(long, default_value_t = NonZeroIsize::try_from(30000).unwrap())]
    memory_len: NonZeroIsize,
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Debug, clap::Parser)]
struct TransArg {
    #[clap(value_parser)]
    file: PathBuf,
    #[clap(long, short, arg_enum)]
    target: Option<TransTarget>,
    #[clap(short, long)]
    optimize: bool,
    out: PathBuf,
    #[clap(short, long, default_value_t = 30000)]
    memory_len: usize,
    #[clap(short, long)]
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

            let mut block = bf_to_block(&code)?;
            if arg.optimize {
                block = bf::opt::optimize(&block, true, false);
                optimize_for_interpreter(&mut block);
            }

            if arg.verbose {
                info!("block: {:#?}", block);
            }
            let step_count = match arg.memory_len.get().cmp(&0) {
                std::cmp::Ordering::Less => {
                    let interpreter = InterPreter::builder()
                        .input(io::stdin())
                        .output(io::stdout())
                        .root_node(&block)
                        // .memory(AutoExtendMemory::new(vec![0; arg.initial_memory_len.get()]))
                        .memory(AutoExtendMemory::new(vec![0; 300000]))
                        .build();

                    time!(interpreter.run()?)
                }
                std::cmp::Ordering::Equal => unreachable!(),
                std::cmp::Ordering::Greater => {
                    let interpreter = InterPreter::builder()
                        .input(io::stdin())
                        .output(io::stdout())
                        .root_node(&block)
                        .memory(vec![0; arg.memory_len.get() as usize])
                        .build();

                    time!(interpreter.run()?)
                }
            };
            info!("step: {step_count}");
        }
        SubCommand::Trans(arg) => {
            let code = fs::read_to_string(&arg.file)?;

            let mut block = bf_to_block(&code)?;

            if arg.verbose {
                info!("block: {:#?}", block);
            }

            let target = arg
                .out
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| match ext {
                    "c" => Some(TransTarget::C),
                    "wasm" => Some(TransTarget::Wasm),
                    "wat" => Some(TransTarget::Wat),
                    _ => None,
                })
                .or(arg.target)
                .context(
                    "出力形式が不明: --target(-t) 引数か, 出力パスの拡張子で出力形式(wasm, wat, c)を指定する",
                )?;

            let mut output = File::create(&arg.out)?;

            match target {
                TransTarget::C => {
                    if arg.optimize {
                        block = bf::opt::optimize(&block, true, false);
                    }
                    let c_code = transpile::block_to_c(&block, arg.memory_len);
                    output.write_all(c_code.as_bytes())?;
                }
                TransTarget::Wat => {
                    if arg.optimize {
                        block = bf::opt::optimize(&block, true, true);
                    }
                    transpile::block_to_wat(&block, &mut output)?;
                }
                TransTarget::Wasm => {
                    if arg.optimize {
                        block = bf::opt::optimize(&block, true, true);
                    }
                    transpile::block_to_wasm(&block, &mut output)?;
                }
            };

            info!("Done {:?}", arg.out);
        }
    }
    Ok(())
}

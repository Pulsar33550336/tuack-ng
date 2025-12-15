use crate::parse::ParseArgs;
use crate::ren::RenArgs;
use clap::command;
use clap::{Parser, Subcommand};
use clap_i18n_richformatter::clap_i18n;
use log::info;

mod config;
mod context;
mod init;
mod parse;
mod ren;

#[derive(Debug, Parser)]
#[clap_i18n]
#[command(version, about = "Tuack-NG", disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(short, long, global = true)]
    /// 详细模式
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 渲染题面
    Ren(RenArgs),
    Parse(ParseArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_i18n_or_exit();
    // let cli = Cli::parse();

    init::init(&cli.verbose)?;

    info!("booting up");

    let result = match cli.command {
        Commands::Ren(args) => ren::main(args),
        Commands::Parse(args) => parse::main(args),
    }
    .unwrap();

    // if result.is_err() {
    //     std::process::exit(1);
    // }

    Ok(())
}

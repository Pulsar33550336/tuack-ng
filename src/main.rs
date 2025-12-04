use clap::command;
use clap::{Parser, Subcommand};
use clap_i18n_richformatter::clap_i18n;

use crate::ren::RenArgs;

mod ren;

#[derive(Debug, Parser)]
#[clap_i18n]
#[command(version, about = "Tuack-NG", disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 渲染题面
    Ren(RenArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_i18n_or_exit();
    // let cli = Cli::parse();

    match cli.command {
        Commands::Ren(args) => {
            ren::main(args)?;
        }
    }
    Ok(())
}

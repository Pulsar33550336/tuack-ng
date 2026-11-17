//! Tuack-NG 的构建与 CI 辅助任务。

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::dist::Platform;
use crate::target::Target;

mod checkers;
mod dist;
mod target;
mod workspace;

#[derive(Debug, Parser)]
#[command(version, about = "Tuack-NG 的构建与 CI 辅助工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// 构建 Checker
    BuildCheckers(BuildCheckersArgs),
    /// 构建发布产物
    Dist(DistArgs),
}

#[derive(Debug, Args)]
struct BuildCheckersArgs {
    /// 静态链接 checker
    #[arg(long = "static")]
    static_link: bool,
}

#[derive(Debug, Args)]
struct DistArgs {
    /// 构建目标三元组，例如 x86_64-unknown-linux-gnu
    #[arg(long)]
    target: Target,
    /// 打包平台
    #[arg(long, value_enum)]
    platform: Platform,
    /// glibc 版本，仅 gnu 目标生效
    #[arg(long, default_value = "2.28")]
    glibc: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::BuildCheckers(args) => checkers::build(args.static_link),
        Commands::Dist(args) => dist::run(args.target, args.platform, &args.glibc),
    }
}

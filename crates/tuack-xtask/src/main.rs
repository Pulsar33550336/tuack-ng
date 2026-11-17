//! Tuack-NG 的构建与 CI 辅助任务。

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::dist::Platform;
use crate::target::LibcChoice;

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
    /// 打包平台
    #[arg(long, value_enum)]
    platform: Platform,
    /// 目标 C 库
    #[cfg_attr(not(windows), arg(long, value_enum, default_value = "gnu"))]
    #[cfg_attr(windows, arg(skip))]
    libc: Option<LibcChoice>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::BuildCheckers(args) => checkers::build(args.static_link),
        Commands::Dist(args) => dist::run(args.platform, args.libc),
    }
}

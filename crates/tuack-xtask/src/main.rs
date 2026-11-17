//! Tuack-NG 的构建与 CI 辅助任务。

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};

use crate::dist::Platform;
use crate::target::LibcChoice;

mod checkers;
mod commit_msg;
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
    /// 校验提交信息
    CommitMsg(CommitMsgArgs),
}

#[derive(Debug, Args)]
struct BuildCheckersArgs {
    /// 静态链接 checker
    #[arg(long = "static")]
    static_link: bool,
}

#[derive(Debug, Args)]
struct CommitMsgArgs {
    /// 提交信息文件（钩子使用）
    #[arg(required_unless_present = "range", conflicts_with = "range")]
    path: Option<PathBuf>,
    /// 提交范围，如 `origin/master...HEAD`（CI 使用）
    #[arg(long, value_name = "RANGE")]
    range: Option<String>,
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
        Commands::CommitMsg(args) => match (args.path, args.range) {
            (Some(path), _) => commit_msg::run_file(&path),
            (None, Some(range)) => commit_msg::run_range(&range),
            (None, None) => bail!("需要给出提交信息文件或 --range"),
        },
    }
}

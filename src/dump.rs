use crate::prelude::*;
use clap::Args;
use clap::ValueEnum;

mod arbiter;
mod lemon;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Target {
    Lemon,
    Arbiter,
}

#[derive(Args, Debug)]
#[command(version)]
pub struct DumpArgs {
    /// 导出目标
    #[arg(required = true)]
    pub target: Target,
}

pub fn dump_main(day: &ContestDayConfig, target: Target, day_num: usize) -> Result<()> {
    let dump_dir = day.path.join("dump");
    if !dump_dir.exists() {
        fs::create_dir_all(dump_dir)?;
    }

    match target {
        Target::Lemon => lemon::main(day),
        Target::Arbiter => arbiter::main(day, day_num),
    }
}

pub fn main(args: DumpArgs) -> Result<()> {
    let config = get_context().config.as_ref().context("没有有效的配置文件")?;

    match &config.1 {
        CurrentLocation::None => bail!("此命令必须在工程下执行"),
        CurrentLocation::Problem(_, _) => bail!("此命令不能在题目下执行"),
        CurrentLocation::Day(day_name) => {
            let day_config = config.0.subconfig.get(day_name).context("找不到天配置")?;
            // Find index of day
            let day_num = config.0.subconfig.keys().position(|k| k == day_name).unwrap() + 1;
            dump_main(day_config, args.target, day_num)?;
        }
        CurrentLocation::Root => {
            for (i, (_, day_config)) in config.0.subconfig.iter().enumerate() {
                dump_main(day_config, args.target, i + 1)?;
            }
        }
    }

    Ok(())
}

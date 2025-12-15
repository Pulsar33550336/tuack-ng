use clap::Args;

use crate::config::ContestConfig;
use log::debug;

use crate::config;

#[derive(Args, Debug)]
#[command(version)]
pub struct ParseArgs {}

// will remove later

pub fn main(args: ParseArgs) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::current_dir().unwrap();

    let config: ContestConfig = config::load_config(&path)?;

    debug!("解析到的配置文件：{:?}", config);

    Ok(())
}

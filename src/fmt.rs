use crate::prelude::*;
use crate::config::{CONFIG_FILE_NAME, save_problem_config};
use clap::Args;
use markdown_ppp::parser::*;
use markdown_ppp::printer::config::Config;
use markdown_ppp::printer::render_markdown;

#[derive(Args, Debug)]
#[command(version)]
pub struct FmtArgs {}

pub fn main(_args: FmtArgs) -> Result<()> {
    let (config, location) = get_context().config.as_ref().context("找不到配置文件")?;

    match location {
        CurrentLocation::Problem(day_name, problem_name) => {
            let day_config = config.subconfig.get(day_name).unwrap();
            let problem_config = day_config.subconfig.get(problem_name).unwrap();
            fmt_problem(problem_config)?;
        }
        CurrentLocation::Day(day_name) => {
            let day_config = config.subconfig.get(day_name).unwrap();
            for problem_config in day_config.subconfig.values() {
                fmt_problem(problem_config)?;
            }
        }
        CurrentLocation::Root | CurrentLocation::None => {
            for day_config in config.subconfig.values() {
                for problem_config in day_config.subconfig.values() {
                    fmt_problem(problem_config)?;
                }
            }
        }
    }

    Ok(())
}

fn fmt_problem(prob: &ProblemConfig) -> Result<()> {
    info!("格式化题目: {}", prob.name);
    let statement_path = prob.path.join("statement.md");
    if !statement_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&statement_path)?;
    let state = MarkdownParserState::new();

    match parse_markdown(state, &content) {
        Ok(ast) => {
            let output = render_markdown(&ast, Config::default().with_width(10000000));

            let mut final_output = String::new();
            if !output.contains("{{ s.title() }}") && !output.contains("{{ s('title') }}") {
                final_output.push_str("{{ s.title() }}\n\n");
            }
            final_output.push_str(&output);

            fs::write(&statement_path, final_output)?;
        },
        Err(e) => {
            warn!("无法解析题面 {} 以进行格式化: {:?}", statement_path.display(), e);
        }
    }

    // Save config
    let config_path = prob.path.join(CONFIG_FILE_NAME);
    fs::write(config_path, save_problem_config(prob)?)?;

    Ok(())
}

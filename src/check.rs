use crate::prelude::*;
use clap::Args;
use markdown_ppp::parser::*;
use markdown_ppp::ast::*;

#[derive(Args, Debug)]
#[command(version)]
pub struct CheckArgs {}

pub fn main(_args: CheckArgs) -> Result<()> {
    let config = get_context().config.as_ref().context("找不到配置文件")?;

    info!("检查配置文件...");
    // The loading logic already validates most fields and versions.
    // If it loaded successfully, the structure is largely correct.
    // We should ensure all warnings from the loading process are visible.

    match &config.1 {
        CurrentLocation::Problem(day_name, problem_name) => {
            let day_config = config.0.subconfig.get(day_name).unwrap();
            let problem_config = day_config.subconfig.get(problem_name).unwrap();
            check_problem(problem_config)?;
        }
        CurrentLocation::Day(day_name) => {
            let day_config = config.0.subconfig.get(day_name).unwrap();
            check_day(day_config)?;
        }
        CurrentLocation::Root | CurrentLocation::None => {
            check_contest(&config.0)?;
        }
    }

    info!("检查完成！");
    Ok(())
}

fn check_problem(prob: &ProblemConfig) -> Result<()> {
    info!("检查题目: {}", prob.name);

    if prob.time_limit <= 0.0 {
        warn!("题目 {} 的时间限制无效: {}", prob.name, prob.time_limit);
    }

    if prob.memory_limit.as_u64() == 0 {
        warn!("题目 {} 的内存限制无效", prob.name);
    }

    let statement_path = prob.path.join("statement.md");
    if !statement_path.exists() {
        warn!("题目 {} 缺少题面文件", prob.name);
    } else {
        check_statement(&statement_path)?;
    }

    Ok(())
}

fn check_day(day: &ContestDayConfig) -> Result<()> {
    info!("检查比赛日: {}", day.name);
    for prob in day.subconfig.values() {
        check_problem(prob)?;
    }
    Ok(())
}

fn check_contest(contest: &ContestConfig) -> Result<()> {
    info!("检查全场比赛: {}", contest.name);
    for day in contest.subconfig.values() {
        check_day(day)?;
    }
    Ok(())
}

fn check_statement(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let state = MarkdownParserState::new();

    match parse_markdown(state, &content) {
        Ok(ast) => {
            // AST-based checks
            for block in ast.blocks {
                match block {
                    Block::Paragraph(inlines) => {
                        // Check for common issues in text, like missing spaces between Chinese and English
                        // This mirrors some logic from Tuack's doc.py
                        let text = format!("{:?}", inlines); // Simplified check
                        if text.contains("TODO") {
                            warn!("题面 {} 包含 TODO", path.display());
                        }
                    }
                    _ => {}
                }
            }
        },
        Err(e) => {
            warn!("题面 {} 解析失败: {:?}", path.display(), e);
        }
    }

    // Check for Jinja2 syntax
    if content.contains("{{ _('") {
        warn!("题面 {} 使用了旧的 Tuack 翻译语法 {{ _('...' ) }}, 请迁移到 Tuack-NG 语法", path.display());
    }

    Ok(())
}

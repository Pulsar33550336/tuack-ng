use clap::Args;
use log::{debug, error, info, warn};
use markdown_ppp::parser::*;
use markdown_ppp::typst_printer::config::Config;
use markdown_ppp::typst_printer::render_typst;
use std::path::Path;
use std::process::Command;
use std::{fs, path::PathBuf};

use crate::{
    config::{
        ContestConfig, ContestDayConfig, DataJson, DateInfo, Problem, ProblemConfig,
        SupportLanguage, load_config,
    },
    context,
};

#[derive(Args, Debug)]
#[command(version)]
pub struct RenArgs {
    /// 渲染目标模板
    #[arg(default_value = "template")]
    target: String,

    /// 要渲染的天的名称（可选，如果不指定则渲染所有天）
    #[arg(short, long)]
    day: Option<String>,
}

pub fn main(args: RenArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("当前目录: {}", Path::new(".").to_string_lossy());
    let config = load_config(Path::new("."))?;

    let template_dir = context::get_context().template_dirs.iter().find(|dir| {
        let subdir = dir.join(&args.target);
        subdir.exists() && subdir.is_dir()
    });

    let template_dir = match template_dir {
        Some(dir) => {
            info!("找到模板目录: {}", dir.join(&args.target).to_string_lossy());
            dir.join(&args.target)
        }
        None => {
            error!("没有找到模板 {}", args.target);
            return Err("没有找到模板".into());
        }
    };

    debug!("检查Typst编译环境");
    let typst_check = Command::new("typst").arg("--version").output();

    match typst_check {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                debug!("Typst 版本: {}", version.trim());
            } else {
                error!("Typst 命令执行失败，请检查是否已安装");
                return Err("Typst 命令执行失败，请检查是否已安装".into());
            }
        }
        Err(_) => {
            return Err("未找到 typst 命令，请确保已安装并添加到PATH".into());
        }
    }

    let template_required_files = ["main.typ", "utils.typ"];
    for file in template_required_files {
        if !template_dir.join(file).exists() {
            error!("模板缺少必要文件: {}", file);
            return Err(format!("模板缺少必要文件: {}", file).into());
        }
        info!("文件存在: {}", file);
    }

    let statements_dir = config.path.join("statements/");
    info!("{}", &statements_dir.to_string_lossy());
    if !statements_dir.exists() {
        fs::create_dir(&statements_dir)?;
        info!("创建题面输出目录: {}", statements_dir.display());
    }

    // 过滤要渲染的天
    let days_to_render: Vec<&ContestDayConfig> = if let Some(day_name) = &args.day {
        match config.subconfig.iter().find(|d| d.name == *day_name) {
            Some(day) => {
                info!("渲染指定天: {}", day_name);
                vec![day]
            }
            None => {
                error!("未找到天: {}", day_name);
                return Err(format!("未找到天: {}", day_name).into());
            }
        }
    } else {
        info!("渲染所有天（共{}个）", config.subconfig.len());
        config.subconfig.iter().collect()
    };

    for day in days_to_render {
        info!("开始渲染天: {}", day.name);
        render_day(&config, day, &template_dir, &statements_dir)?;
    }

    info!("所有天的题面渲染完成！");
    Ok(())
}

fn generate_data_json(
    contest_config: &ContestConfig,
    day_config: &ContestDayConfig,
) -> Result<DataJson, Box<dyn std::error::Error>> {
    // 构建问题列表
    let mut problems = Vec::new();

    for problem_config in &day_config.subconfig {
        // 注意：这里根据你的Problem结构进行调整
        // 由于ProblemConfig和DataJson::Problem结构不同，需要转换
        // 这里我创建一个简化的转换，你可能需要根据实际需求调整

        let problem = Problem {
            name: problem_config.name.clone(),
            title: problem_config.title.clone(),
            dir: problem_config.name.clone(), // 假设目录名就是问题名
            exec: problem_config.name.clone(), // 默认值，你可能需要从配置文件读取
            input: problem_config.name.clone() + ".in",
            output: problem_config.name.clone() + ".out",
            // problem_type: problem_config.problem_type.clone(),
            problem_type: match problem_config.problem_type.as_str() {
                "program" => "传统型",
                "output" => "提交答案型",
                "interactive" => "交互型",
                _ => {
                    warn!(
                        "未知的题目类型 {} , 使用默认值 传统型",
                        problem_config.problem_type
                    );
                    "传统型"
                }
            }
            .to_string(),
            time_limit: format!("{:.1} 秒", problem_config.time_limit),
            memory_limit: problem_config.memory_limit.clone(),
            testcase: problem_config.data.len().to_string(),
            point_equal: "是".to_string(),
            submit_filename: vec![format!("{}.cpp", problem_config.name)], // 默认值
        };
        problems.push(problem);
    }

    // 构建支持的语言列表
    // 注意：ContestConfig中没有support_languages字段，这里使用默认值
    let support_languages = vec![
        SupportLanguage {
            name: "C++".to_string(),
            compile_options: day_config.compile.cpp.clone(),
        },
        // SupportLanguage {
        //     name: "C".to_string(),
        //     compile_options: if !day_config.compile.c.is_empty() {
        //         day_config.compile.c.clone()
        //     } else {
        //         "gcc -std=c11 -O2 -static -lm".to_string()
        //     },
        // },
    ];

    // 创建日期信息
    let date = DateInfo {
        start: day_config.start_time,
        end: day_config.end_time,
    };

    Ok(DataJson {
        title: contest_config.title.clone(),
        subtitle: contest_config.short_title.clone(),
        dayname: day_config.title.clone(),
        date,
        use_pretest: false, // 默认值，你可能需要从配置文件读取
        noi_style: true,    // 默认值，你可能需要从配置文件读取
        file_io: true,      // 默认值，你可能需要从配置文件读取
        support_languages,
        problems,
        images: Vec::new(),
    })
}

fn render_day(
    contest_config: &ContestConfig,
    day_config: &ContestDayConfig,
    template_dir: &PathBuf,
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let day_output_dir = output_dir.join(&day_config.name);
    if !day_output_dir.exists() {
        fs::create_dir(&day_output_dir)?;
        info!("创建天输出目录: {}", day_output_dir.display());
    }

    let tmp_dir = day_output_dir.join("tmp");
    if tmp_dir.exists() {
        info!("清理已存在的临时目录: {}", tmp_dir.display());
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir(&tmp_dir)?;
    info!("创建临时目录: {}", tmp_dir.display());

    info!("复制模板文件到临时目录");
    copy_dir_recursive(template_dir, &tmp_dir)?;

    // 生成并写入 data.json
    let data_json = generate_data_json(contest_config, day_config)?;
    let data_json_str = serde_json::to_string_pretty(&data_json)?;
    fs::write(tmp_dir.join("data.json"), data_json_str)?;
    info!("生成 data.json");

    // 复制问题资源和题面
    for (idx, problem) in day_config.subconfig.iter().enumerate() {
        info!(
            "处理问题 {}/{}: {}",
            idx + 1,
            day_config.subconfig.len(),
            problem.name
        );

        // 题面文件路径
        let problem_dir = &problem.path;
        let statement_path = problem_dir.join("statement.md");

        if !statement_path.exists() {
            error!("未找到题面文件: {}", statement_path.display());
            return Err(format!("未找到题面文件: {}", statement_path.display()).into());
        }

        // 解析题面
        let content = fs::read_to_string(&statement_path)?;
        let state = MarkdownParserState::new();
        let ast = match parse_markdown(state, &content) {
            Ok(ast) => ast,
            Err(e) => {
                error!("解析题面文件 {} 失败: {:?}", statement_path.display(), e);
                return Err("解析题面文件失败".into());
            }
        };

        // 生成Typst内容
        let typst_output = render_typst(&ast, Config::default().with_width(1000000));
        let typst_output = format!("#import \"utils.typ\": *\n{}", typst_output);

        // 写入Typst文件
        let typst_filename = format!("problem-{}.typ", idx);
        fs::write(tmp_dir.join(&typst_filename), typst_output)?;
        info!("生成: {}", typst_filename);

        // 复制资源文件
        // let resource_dirs = ["img", "sample", "pretest", "data"];
        // for resource_dir in &resource_dirs {
        //     let src_dir = problem_dir.join(resource_dir);
        //     if src_dir.exists() && src_dir.is_dir() {
        //         let dst_dir = tmp_dir.join(resource_dir);
        //         if dst_dir.exists() {
        //             fs::remove_dir_all(&dst_dir)?;
        //         }
        //         copy_dir_recursive(&src_dir, &dst_dir)?;
        //         info!(
        //             "复制资源目录: {} -> {}",
        //             src_dir.display(),
        //             dst_dir.display()
        //         );
        //     }
        // }
    }

    // 处理注意事项文件
    let precaution_path = contest_config.path.join("precaution.md");
    info!("{}", precaution_path.to_string_lossy());
    if precaution_path.exists() {
        info!("处理注意事项文件: {}", precaution_path.display());
        let content = fs::read_to_string(&precaution_path)?;
        let state = MarkdownParserState::new();
        match parse_markdown(state, &content) {
            Ok(ast) => {
                let typst_output = render_typst(&ast, Config::default().with_width(1000000));
                let typst_output = format!("#import \"utils.typ\": *\n{}", typst_output);
                fs::write(tmp_dir.join("precaution.typ"), typst_output)?;
                info!("生成: precaution.typ");
            }
            Err(e) => {
                warn!("解析注意事项文件失败: {:?}", e);
            }
        }
    } else {
        warn!("未找到注意事项文件");
    }

    // 编译PDF
    info!("开始编译PDF...");
    let compile_result = Command::new("typst")
        .arg("compile")
        .arg("--font-path=fonts")
        .arg("main.typ")
        .arg(format!("{}.pdf", day_config.name))
        .current_dir(&tmp_dir)
        .output()?;

    if compile_result.status.success() {
        info!("编译成功！");

        // 复制PDF到输出目录
        let pdf_source = tmp_dir.join(format!("{}.pdf", day_config.name));
        let pdf_target = day_output_dir.join(format!("{}.pdf", day_config.name));
        fs::copy(&pdf_source, &pdf_target)?;
        info!("PDF已保存到: {}", pdf_target.display());

        // 清理临时目录
        fs::remove_dir_all(&tmp_dir)?;
        info!("清理临时目录");
    } else {
        let error_output = String::from_utf8_lossy(&compile_result.stderr);
        error!("编译失败:");
        error!("{}", error_output);

        // 保留临时目录以供调试
        warn!("保留临时目录以供调试: {}", tmp_dir.display());
        return Err("编译过程出错".into());
    }

    Ok(())
}

// 递归复制目录的辅助函数
fn copy_dir_recursive<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

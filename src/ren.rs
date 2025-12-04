use clap::Args;
use markdown_ppp::parser::*;
use markdown_ppp::typst_printer::config::Config;
use markdown_ppp::typst_printer::render_typst;
use serde_json;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Args, Debug)]
#[command(version)]
pub struct RenArgs {}

// Todo: improve render
// 现在纯瞎写

pub fn main(args: RenArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("[I] 检查Typst编译环境...");

    // 检查typst命令是否可用
    let typst_check = Command::new("typst").arg("--version").output();

    match typst_check {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("[I] Typst 版本: {}", version.trim());
            } else {
                return Err("[E] Typst 命令执行失败，请检查是否已安装".into());
            }
        }
        Err(_) => {
            return Err("[E] 未找到 typst 命令，请确保已安装并添加到PATH".into());
        }
    }

    // 检查必要文件是否存在
    let required_files = ["data.json", "precaution.typ"];

    for file in required_files {
        if !Path::new(file).exists() {
            return Err(format!("[E] 缺少必要文件: {}", file).into());
        }
        println!("[I] 文件存在: {}", file);
    }

    // 定义要处理的Markdown文件列表（在这里写死）
    let markdown_files = [
        "problem-0.md",
        "problem-1.md",
        "problem-2.md",
        "problem-3.md",
    ];

    // 设置计数器用于编号
    let mut problem_count = 0;

    // 处理每个Markdown文件
    for &markdown_file in &markdown_files {
        if Path::new(markdown_file).exists() {
            println!("[I] 处理文件: {}", markdown_file);

            // 读取Markdown文件
            let content = match fs::read_to_string(markdown_file) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("[W] 无法读取文件 {}: {}, 跳过", markdown_file, e);
                    continue;
                }
            };

            // 使用 markdown-ppp 解析
            let state = MarkdownParserState::new();
            let ast = match parse_markdown(state, &content) {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("[W] 解析文件 {} 失败: {:?}, 跳过", markdown_file, e);
                    continue;
                }
            };

            // 导出完整AST为JSON（可选，可以注释掉以减少文件生成）
            // let json_full = match serde_json::to_string_pretty(&ast) {
            //     Ok(json) => json,
            //     Err(e) => {
            //         eprintln!("[W] 无法序列化AST为JSON: {}, 跳过", e);
            //         continue;
            //     }
            // };

            // 为每个文件生成单独的AST JSON文件
            // let ast_filename = format!("ast-{}.json", problem_count);
            // if let Err(e) = fs::write(&ast_filename, json_full) {
            //     eprintln!("[W] 无法写入AST文件 {}: {}, 跳过", ast_filename, e);
            // } else {
            //     println!("[I] 完整AST已导出到: {}", ast_filename);
            // }

            // 转换为Typst（包含数学公式处理）
            println!("[I] 转换为Typst...");

            let typst_output = render_typst(&ast, Config::default().with_width(1000000));

            let typst_output = format!("#import \"utils.typ\": *\n{}", typst_output);

            // 为每个文件生成单独的Typst文件
            let typst_filename = format!("problem-{}.typ", problem_count);
            if let Err(e) = fs::write(&typst_filename, typst_output.clone()) {
                eprintln!("[W] 无法写入Typst文件 {}: {}, 跳过", typst_filename, e);
            } else {
                println!("[I] Typst文件已生成: {}", typst_filename);
            }

            // 添加到主Typst文件中
            // main_typst_content.push_str(&format!("\n=== 问题 {} ===\n", problem_count + 1));
            // main_typst_content.push_str(&typst_output);
            // main_typst_content.push_str("\n");

            problem_count += 1;
        } else {
            println!("[I] 未找到 {}, 跳过", markdown_file);
        }
    }

    if problem_count == 0 {
        return Err("[E] 未找到任何有效的Markdown文件进行处理".into());
    }

    // 写入主Typst文件
    // fs::write("all-problems.typ", main_typst_content)?;
    // println!("[I] 主Typst文件已生成: all-problems.typ，包含 {} 个问题", problem_count);

    // 读取并验证data.json格式
    let data_content = fs::read_to_string("data.json")?;
    let _data: serde_json::Value = serde_json::from_str(&data_content)
        .map_err(|e| format!("[E] data.json 格式错误: {}", e))?;
    println!("[I] data.json 格式验证通过");

    // 尝试编译
    println!("[I] 开始编译...");
    let compile_result = Command::new("typst")
        .arg("compile")
        .arg("--font-path=fonts")
        .arg("main.typ")
        .arg("output.pdf")
        .output()?;

    if compile_result.status.success() {
        println!("[I] 编译成功！生成 output.pdf");
        println!("[I] 处理了 {} 个Markdown文件", problem_count);
    } else {
        let error_output = String::from_utf8_lossy(&compile_result.stderr);
        eprintln!("[E] 编译失败:");
        eprintln!("{}", error_output);
        return Err("编译过程出错".into());
    }

    Ok(())
}

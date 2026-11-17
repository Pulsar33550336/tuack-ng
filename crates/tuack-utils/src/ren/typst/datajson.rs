use crate::prelude::*;

/// Typst 模板 `data.json` 的结构
///
/// `Problem` 中多数文本字段为供模板直接展示的字符串（如 `1.0 秒`，或 `是` / `否`）。
#[derive(Debug, Serialize, Deserialize)]
pub struct DataJson {
    pub title: String,
    pub subtitle: String,
    pub dayname: String,
    pub date: Option<DateInfo>,
    pub use_pretest: bool,
    pub noi_style: bool,
    pub file_io: bool,
    pub support_languages: Vec<SupportLanguage>,
    pub problems: Vec<Problem>,
}

/// 单个题目的模板字段
#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub name: String,
    pub title: String,
    /// 题目类型（`传统型` / `提交答案型` / `交互型`）
    #[serde(rename = "type")]
    pub problem_type: String,
    /// 题目目录名
    pub dir: String,
    /// 可执行文件名
    pub exec: String,
    pub input: String,
    pub output: String,
    /// 已格式化的时间限制（含单位）
    pub time_limit: String,
    /// 已格式化的内存限制（含单位）
    pub memory_limit: String,
    /// 测试点或子任务数目
    pub testcase: String,
    /// 各测试点分数是否相等（`是` / `否`）
    pub point_equal: String,
    /// 允许提交的文件名
    pub submit_filename: Vec<String>,
}

/// 支持的语言
#[derive(Debug, Serialize, Deserialize)]
pub struct SupportLanguage {
    pub name: String,
    pub compile_options: String,
}

/// 比赛起止时间
#[derive(Debug, Serialize, Deserialize)]
pub struct DateInfo {
    /// `[年，月，日，时，分，秒]`
    pub start: [u32; 6],
    /// `[年，月，日，时，分，秒]`
    pub end: [u32; 6],
}

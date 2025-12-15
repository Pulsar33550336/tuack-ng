use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub problem_type: String,
    pub dir: String,
    pub exec: String,
    pub input: String,
    pub output: String,
    pub time_limit: String,
    pub memory_limit: String,
    pub testcase: String,
    pub point_equal: String,
    pub submit_filename: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportLanguage {
    pub name: String,
    pub compile_options: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateInfo {
    pub start: [u32; 6],
    pub end: [u32; 6],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateManifest {
    #[serde(default = "default_use_pretest")]
    pub use_pretest: bool,
    #[serde(default = "default_noi_style")]
    pub noi_style: bool,
    #[serde(default = "default_file_io")]
    pub file_io: bool,
}

fn default_use_pretest() -> bool {
    false
}

fn default_noi_style() -> bool {
    true
}

fn default_file_io() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataJson {
    pub title: String,
    pub subtitle: String,
    pub dayname: String,
    pub date: DateInfo,
    pub use_pretest: bool,
    pub noi_style: bool,
    pub file_io: bool,
    pub support_languages: Vec<SupportLanguage>,
    pub problems: Vec<Problem>,
    pub images: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContestConfig {
    pub version: u32,
    pub folder: String,
    pub name: String,
    pub subdir: Vec<String>,
    pub title: String,
    #[serde(rename = "short title")]
    pub short_title: String,
    #[serde(rename = "use-pretest")]
    #[serde(default)]
    pub use_pretest: Option<bool>,
    #[serde(rename = "noi-style")]
    #[serde(default)]
    pub noi_style: Option<bool>,
    #[serde(rename = "file-io")]
    #[serde(default)]
    pub file_io: Option<bool>,
    #[serde(skip)]
    pub subconfig: Vec<ContestDayConfig>,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContestDayConfig {
    pub version: u32,
    pub folder: String,
    pub name: String,
    pub subdir: Vec<String>,
    pub title: String,
    pub compile: CompileConfig,
    #[serde(rename = "start time")]
    pub start_time: [u32; 6],
    #[serde(rename = "end time")]
    pub end_time: [u32; 6],
    #[serde(rename = "use-pretest")]
    #[serde(default)]
    pub use_pretest: Option<bool>,
    #[serde(rename = "noi-style")]
    #[serde(default)]
    pub noi_style: Option<bool>,
    #[serde(rename = "file-io")]
    #[serde(default)]
    pub file_io: Option<bool>,
    #[serde(skip)]
    pub subconfig: Vec<ProblemConfig>,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProblemConfig {
    pub version: u32,
    pub folder: String,
    #[serde(rename = "type")]
    pub problem_type: String,
    pub name: String,
    pub title: String,
    #[serde(rename = "time limit")]
    pub time_limit: f64,
    #[serde(rename = "memory limit")]
    pub memory_limit: String,
    #[serde(rename = "partial score")]
    pub partial_score: bool,
    #[serde(skip)]
    pub path: PathBuf,
    pub samples: Vec<SampleItem>,
    // pub args: HashMap<String, serde_json::Value>,
    pub data: Vec<DataItem>,
    // pub pretest: Vec<PreItem>,
    // pub tests: HashMap<String, serde_json::Value>,
}

impl ProblemConfig {
    pub fn finalize(mut self) -> Self {
        // 初始化 samples 的默认文件名
        self.samples = self.samples.into_iter().map(|s| s.finalize()).collect();

        // 初始化 data 的默认文件名
        self.data = self.data.into_iter().map(|d| d.finalize()).collect();

        self
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct LocalizedString {
//     #[serde(rename = "zh-cn")]
//     pub zh_cn: String,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CompileConfig {
    pub cpp: String,
    #[serde(default)]
    pub c: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Sample {
//     pub samples: Vec<SampleItem>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleItem {
    pub id: u32,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
}

impl SampleItem {
    pub fn finalize(mut self) -> Self {
        if self.input.as_ref().map_or(true, |s| s.is_empty()) {
            self.input = Some(format!("{}.in", self.id));
        }
        if self.output.as_ref().map_or(true, |s| s.is_empty()) {
            self.output = Some(format!("{}.ans", self.id));
        }
        self
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Data {
//     pub datas: Vec<DataItem>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    pub id: u32,
    pub score: u32,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
}

impl DataItem {
    pub fn finalize(mut self) -> Self {
        if self.input.as_ref().map_or(true, |s| s.is_empty()) {
            self.input = Some(format!("{}.in", self.id));
        }
        if self.output.as_ref().map_or(true, |s| s.is_empty()) {
            self.output = Some(format!("{}.ans", self.id));
        }
        self
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PreItem {
//     #[serde(flatten)]
//     pub data: HashMap<String, serde_json::Value>,
// }

fn find_contest_config(start_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut current_path = start_path.to_path_buf().canonicalize()?;

    loop {
        debug!("path: {}", current_path.to_string_lossy());
        // 检查配置文件并判断类型
        let possible_file = "conf.json";
        let file_path = current_path.join(possible_file);
        if file_path.exists() && is_contest_config(&file_path)? {
            return Ok(file_path);
        }

        if !current_path.pop() {
            error!("未找到contest配置文件");
            return Err("未找到contest配置文件".into());
        }
    }
}
fn is_contest_config(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let json_value: serde_json::Value = serde_json::from_str(&content)?;

    // 通过字段判断是否是contest配置
    if let Some(version) = json_value.get("version").and_then(|v| v.as_u64()) {
        if version >= 3 {
            if let Some(folder) = json_value.get("folder").and_then(|v| v.as_str()) {
                if folder == "contest" {
                    return Ok(true);
                }
            }
        } else {
            error!(
                "配置文件版本过低，可能是 tuack 的配置文件。请迁移到 tuack-ng 配置文件格式再使用。"
            );
            return Err("配置文件版本过低".into());
        }
    }

    Ok(false)
}

pub fn load_config(path: &Path) -> Result<ContestConfig, Box<dyn std::error::Error>> {
    let config_path = find_contest_config(path)?;

    // 读取并验证主配置文件
    let main_content = fs::read_to_string(&config_path)?;
    let main_json_value: serde_json::Value = serde_json::from_str(&main_content)?;

    // 检查版本
    if let Some(version) = main_json_value.get("version").and_then(|v| v.as_u64()) {
        if version < 3 {
            error!(
                "配置文件版本过低，可能是 tuack 的配置文件。请迁移到 tuack-ng 配置文件格式再使用。"
            );
            return Err("配置文件版本过低".into());
        }
    }

    // 反序列化主配置
    let mut config: ContestConfig = serde_json::from_str(&main_content)?;

    config.path = config_path.clone().parent().unwrap().to_path_buf();

    config.subconfig = Vec::new();

    let parent_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or("无法获取配置文件父目录")?;

    for dayconfig_name in &config.subdir {
        let dayconfig_path = parent_dir.join(dayconfig_name).join("conf.json");

        // 读取并验证每日配置文件
        let day_content = fs::read_to_string(&dayconfig_path)?;
        let day_json_value: serde_json::Value = serde_json::from_str(&day_content)?;

        // 检查版本
        if let Some(version) = day_json_value.get("version").and_then(|v| v.as_u64()) {
            if version < 3 {
                error!(
                    "配置文件版本过低，可能是 tuack 的配置文件。请迁移到 tuack-ng 配置文件格式再使用。"
                );
                return Err("配置文件版本过低".into());
            }
        }

        let mut dayconfig: ContestDayConfig = serde_json::from_str(&day_content)?;

        dayconfig.path = config_path
            .join(dayconfig_name)
            .parent()
            .unwrap()
            .to_path_buf();

        dayconfig.subconfig = Vec::new();

        let day_parent_dir = dayconfig_path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or("无法获取配置文件父目录")?;

        for problemconfig_name in &dayconfig.subdir {
            let problemconfig_path = day_parent_dir.join(problemconfig_name).join("conf.json");

            // 读取并验证问题配置文件
            let problem_content = fs::read_to_string(&problemconfig_path)?;
            let problem_json_value: serde_json::Value = serde_json::from_str(&problem_content)?;

            // 检查版本
            if let Some(version) = problem_json_value.get("version").and_then(|v| v.as_u64()) {
                if version < 3 {
                    error!(
                        "配置文件版本过低，可能是 tuack 的配置文件。请迁移到 tuack-ng 配置文件格式再使用。"
                    );
                    return Err("配置文件版本过低".into());
                }
            }

            let mut problemconfig: ProblemConfig = serde_json::from_str(&problem_content)?;

            problemconfig.path = problemconfig_path
                .parent()
                .map(|p| p.to_path_buf())
                .ok_or("无法获取配置文件父目录")?;

            problemconfig = problemconfig.finalize();

            dayconfig.subconfig.push(problemconfig);
        }

        config.subconfig.push(dayconfig);
    }

    Ok(config)
}

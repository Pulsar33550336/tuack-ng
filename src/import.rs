use crate::prelude::*;
use clap::Args;
use crate::config::{CONFIG_FILE_NAME, save_problem_config, save_day_config, save_contest_config, BundleDataItem};

#[derive(Args, Debug)]
#[command(version)]
pub struct ImportArgs {
    /// 原 Tuack 工程路径
    #[arg(required = true)]
    pub path: PathBuf,
}

pub fn main(args: ImportArgs) -> Result<()> {
    info!("从 {} 导入原 Tuack 工程", args.path.display());

    let old_conf_path = if args.path.join("conf.yaml").exists() {
        args.path.join("conf.yaml")
    } else if args.path.join("conf.json").exists() {
        args.path.join("conf.json")
    } else {
        bail!("找不到原 Tuack 配置文件 (conf.yaml 或 conf.json)");
    };

    let content = fs::read_to_string(&old_conf_path)?;
    let old_conf: serde_json::Value = if old_conf_path.extension().unwrap_or_default() == "yaml" {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };

    let folder = old_conf.get("folder").and_then(|v| v.as_str()).unwrap_or("problem");

    match folder {
        "contest" => import_contest(&args.path, &old_conf)?,
        "day" => import_day(&args.path, &old_conf)?,
        "problem" => import_problem(&args.path, &old_conf)?,
        "extend" => {
            let base_path = args.path.join(old_conf.get("base path").and_then(|v| v.as_str()).context("extend 类型缺少 base path")?);
            let content = fs::read_to_string(&base_path)?;
            let base_conf: serde_json::Value = if base_path.extension().unwrap_or_default() == "yaml" {
                serde_yaml::from_str(&content)?
            } else {
                serde_json::from_str(&content)?
            };
            // Merge old_conf into base_conf?
            // For now, just import base_conf but in the current directory
            import_any(&args.path, &base_conf)?;
        }
        _ => bail!("不支持的 folder 类型: {}", folder),
    }

    info!("导入完成！");
    Ok(())
}

fn import_any(path: &Path, old_conf: &serde_json::Value) -> Result<()> {
    let folder = old_conf.get("folder").and_then(|v| v.as_str()).unwrap_or("problem");
    match folder {
        "contest" => import_contest(path, old_conf),
        "day" => import_day(path, old_conf),
        "problem" => import_problem(path, old_conf),
        _ => bail!("不支持的 folder 类型: {}", folder),
    }
}

fn import_contest(path: &Path, old_conf: &serde_json::Value) -> Result<()> {
    info!("导入比赛: {}", path.display());

    let subdir: Vec<String> = old_conf.get("subdir").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let config = ContestConfig {
        version: 3,
        folder: "contest".to_string(),
        name: old_conf.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string(),
        subdir: subdir.clone(),
        title: old_conf.get("title").and_then(|v| if v.is_string() { v.as_str() } else { v.get("zh-cn").and_then(|v| v.as_str()) }).unwrap_or("").to_string(),
        short_title: old_conf.get("short title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        use_pretest: old_conf.get("use-pretest").and_then(|v| v.as_bool()),
        noi_style: old_conf.get("noi-style").and_then(|v| v.as_bool()),
        file_io: old_conf.get("file-io").and_then(|v| v.as_bool()),
        subconfig: indexmap::IndexMap::new(),
        path: path.to_path_buf(),
    };

    fs::write(path.join(CONFIG_FILE_NAME), save_contest_config(&config)?)?;

    for sub in subdir {
        let sub_path = path.join(&sub);
        let sub_old_conf_path = if sub_path.join("conf.yaml").exists() {
            sub_path.join("conf.yaml")
        } else {
            sub_path.join("conf.json")
        };
        if sub_old_conf_path.exists() {
            let content = fs::read_to_string(&sub_old_conf_path)?;
            let sub_old_conf: serde_json::Value = if sub_old_conf_path.extension().unwrap_or_default() == "yaml" {
                serde_yaml::from_str(&content)?
            } else {
                serde_json::from_str(&content)?
            };
            import_day(&sub_path, &sub_old_conf)?;
        }
    }
    Ok(())
}

fn import_day(path: &Path, old_conf: &serde_json::Value) -> Result<()> {
    info!("导入比赛日: {}", path.display());

    let subdir: Vec<String> = old_conf.get("subdir").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let compile: HashMap<String, String> = old_conf.get("compile").and_then(|v| v.as_object())
        .map(|o| o.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
        .unwrap_or_default();

    let config = ContestDayConfig {
        version: 3,
        folder: "day".to_string(),
        name: old_conf.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string(),
        subdir: subdir.clone(),
        title: old_conf.get("title").and_then(|v| if v.is_string() { v.as_str() } else { v.get("zh-cn").and_then(|v| v.as_str()) }).unwrap_or("").to_string(),
        compile,
        start_time: old_conf.get("start time").and_then(|v| v.as_array())
            .map(|a| [a[0].as_u64().unwrap_or(0) as u32, a[1].as_u64().unwrap_or(0) as u32, a[2].as_u64().unwrap_or(0) as u32, a[3].as_u64().unwrap_or(0) as u32, a[4].as_u64().unwrap_or(0) as u32, a[5].as_u64().unwrap_or(0) as u32])
            .unwrap_or([2024, 1, 1, 8, 0, 0]),
        end_time: old_conf.get("end time").and_then(|v| v.as_array())
            .map(|a| [a[0].as_u64().unwrap_or(0) as u32, a[1].as_u64().unwrap_or(0) as u32, a[2].as_u64().unwrap_or(0) as u32, a[3].as_u64().unwrap_or(0) as u32, a[4].as_u64().unwrap_or(0) as u32, a[5].as_u64().unwrap_or(0) as u32])
            .unwrap_or([2024, 1, 1, 13, 0, 0]),
        use_pretest: old_conf.get("use-pretest").and_then(|v| v.as_bool()),
        noi_style: old_conf.get("noi-style").and_then(|v| v.as_bool()),
        file_io: old_conf.get("file-io").and_then(|v| v.as_bool()),
        subconfig: indexmap::IndexMap::new(),
        path: path.to_path_buf(),
    };

    fs::write(path.join(CONFIG_FILE_NAME), save_day_config(&config)?)?;

    for sub in subdir {
        let sub_path = path.join(&sub);
        let sub_old_conf_path = if sub_path.join("conf.yaml").exists() {
            sub_path.join("conf.yaml")
        } else {
            sub_path.join("conf.json")
        };
        if sub_old_conf_path.exists() {
            let content = fs::read_to_string(&sub_old_conf_path)?;
            let sub_old_conf: serde_json::Value = if sub_old_conf_path.extension().unwrap_or_default() == "yaml" {
                serde_yaml::from_str(&content)?
            } else {
                serde_json::from_str(&content)?
            };
            import_problem(&sub_path, &sub_old_conf)?;
        }
    }
    Ok(())
}

fn import_problem(path: &Path, old_conf: &serde_json::Value) -> Result<()> {
    info!("导入题目: {}", path.display());

    let mut samples = Vec::new();
    if let Some(old_samples) = old_conf.get("samples").and_then(|v| v.as_array()) {
        for s in old_samples {
            if let Some(cases) = s.get("cases").and_then(|v| v.as_array()) {
                for case in cases {
                    if let Some(id_str) = case.as_str() {
                        let id = id_str.trim_start_matches(|c: char| !c.is_numeric()).parse().unwrap_or(0);
                        samples.push(SampleItem {
                            id,
                            input: crate::utils::optional::Optional::uninitialized(),
                            output: crate::utils::optional::Optional::uninitialized(),
                            args: HashMap::new(),
                            manual: None,
                        }.finalize());
                    }
                }
            }
        }
    }

    let mut data_items = Vec::new();
    let mut subtasks_map = BTreeMap::new();

    if let Some(old_data) = old_conf.get("data").and_then(|v| v.as_array()) {
        for d in old_data {
            if let Some(cases) = d.get("cases").and_then(|v| v.as_array()) {
                let mut ids = Vec::new();
                for case in cases {
                    if let Some(id_str) = case.as_str() {
                        let id: i32 = id_str.trim_start_matches(|c: char| !c.is_numeric()).parse().unwrap_or(0);
                        ids.push(id);
                    } else if let Some(id) = case.as_i64() {
                        ids.push(id as i32);
                    }
                }
                let subtask_id = d.get("subtask").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                data_items.push(DataItem::Bundle(BundleDataItem {
                    id: ids,
                    score: d.get("score").and_then(|v| v.as_u64()).unwrap_or(10) as u32,
                    subtask: subtask_id,
                    args: HashMap::new(),
                    manual: None,
                }));
                subtasks_map.entry(subtask_id).or_insert(ScorePolicy::Sum);
            }
        }
    }

    let config = ProblemConfig {
        version: 3,
        folder: "problem".to_string(),
        problem_type: match old_conf.get("type").and_then(|v| v.as_str()).unwrap_or("program") {
            "output" => ProblemType::Output,
            "interactive" => ProblemType::Interactive,
            _ => ProblemType::Program,
        },
        name: old_conf.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string(),
        title: old_conf.get("title").and_then(|v| if v.is_string() { v.as_str() } else { v.get("zh-cn").and_then(|v| v.as_str()) }).unwrap_or("").to_string(),
        time_limit: old_conf.get("time limit").and_then(|v| v.as_f64()).unwrap_or(1.0),
        memory_limit: old_conf.get("memory limit").and_then(|v| v.as_str()).unwrap_or("512 MB").parse().unwrap_or(bytesize::ByteSize::mb(512)),
        partial_score: old_conf.get("partial score").and_then(|v| v.as_bool()).unwrap_or(false),
        args: HashMap::new(),
        samples,
        orig_data: data_items,
        orig_subtasks: subtasks_map,
        tests: indexmap::IndexMap::new(),
        use_chk: old_conf.get("use-chk").and_then(|v| v.as_bool()),
        use_pretest: old_conf.get("use-pretest").and_then(|v| v.as_bool()),
        noi_style: old_conf.get("noi-style").and_then(|v| v.as_bool()),
        file_io: old_conf.get("file-io").and_then(|v| v.as_bool()),
        path: path.to_path_buf(),
        data: Vec::new(),
        subtasks: BTreeMap::new(),
    }.finalize();

    fs::write(path.join(CONFIG_FILE_NAME), save_problem_config(&config)?)?;
    Ok(())
}

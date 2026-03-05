use std::process::Command;
use crate::prelude::*;
use crate::utils::filesystem::copy_dir_recursive;
use indicatif::ProgressBar;

fn arbiter_info(info: &BTreeMap<String, String>, filename: &Path) -> Result<()> {
    let mut content = String::new();
    for (key, val) in info {
        content.push_str(key);
        content.push_str(val);
        content.push('\n');
    }
    fs::write(filename, content)?;
    Ok(())
}

pub fn main(day: &ContestDayConfig, day_num: usize) -> Result<()> {
    let arbiter_dir = day.path.join("dump/arbiter");
    if arbiter_dir.exists() {
        fs::remove_dir_all(&arbiter_dir)?;
    }

    let main_dir = arbiter_dir.join("main");
    fs::create_dir_all(&main_dir)?;
    fs::create_dir(main_dir.join("data"))?;
    fs::create_dir(main_dir.join("final"))?;
    fs::create_dir(main_dir.join("players"))?;
    fs::create_dir(main_dir.join("result"))?;
    fs::create_dir(main_dir.join("filter"))?;
    fs::create_dir(main_dir.join("tmp"))?;

    let pb = get_context().multiprogress.add(ProgressBar::new(day.subconfig.len() as u64));
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_prefix("Arbiter");

    // day info
    let mut dayinfo = BTreeMap::new();
    dayinfo.insert("NAME=".to_string(), day.name.clone());
    dayinfo.insert("PLAYERDIR=".to_string(), "".to_string());
    dayinfo.insert("CASEDIR=".to_string(), "".to_string());
    dayinfo.insert("BASESCORE=".to_string(), "0".to_string());
    dayinfo.insert("TASKNUM=".to_string(), day.subconfig.len().to_string());
    arbiter_info(&dayinfo, &main_dir.join(format!("day{}.info", day_num)))?;

    for (prob_num, (_, prob)) in day.subconfig.iter().enumerate() {
        let prob_num = prob_num + 1;
        pb.set_message(format!("Processing problem: {}", prob.name));

        let mut probinfo = BTreeMap::new();
        probinfo.insert("TITLE=".to_string(), "".to_string());
        probinfo.insert("NAME=".to_string(), prob.name.clone());
        probinfo.insert("RUN=".to_string(), "".to_string());
        probinfo.insert("INFILESUFFIX=".to_string(), "in".to_string());
        probinfo.insert("ANSFILESUFFIX=".to_string(), "ans".to_string());
        probinfo.insert("PLUG=".to_string(), format!("{}_e", prob.name));

        match prob.problem_type {
            ProblemType::Program => {
                probinfo.insert("TYPE=".to_string(), "SOURCE".to_string());
            }
            _ => {
                warn!("暂时只支持非交互式程序题: {}", prob.name);
                probinfo.insert("TYPE=".to_string(), "SOURCE".to_string());
            }
        }

        probinfo.insert("LIMIT=".to_string(), prob.time_limit.to_string());
        probinfo.insert("MEMLIMITS=".to_string(), prob.memory_limit.as_mib().to_string());

        let mut total_cases = 0;
        for task in prob.subtasks.values() {
            total_cases += task.items.len();
        }
        probinfo.insert("SAMPLES=".to_string(), total_cases.to_string());

        // Compiler options
        if let Some(opt) = day.compile.get("c") {
            probinfo.insert("CCL=c@gcc".to_string(), format!(" -o %o %i {}", opt));
        }
        if let Some(opt) = day.compile.get("cpp") {
            probinfo.insert("CCL=cpp@g++".to_string(), format!(" -o %o %i {}", opt));
        }
        if let Some(opt) = day.compile.get("pas") {
            probinfo.insert("CCL=pas@fpc".to_string(), format!(" %i {}", opt));
        }

        let mut case_idx = 0;
        for task in prob.subtasks.values() {
            let score_per_case = if task.items.len() > 0 {
                (task.max_score as f64) / (task.items.len() as f64)
            } else {
                0.0
            };

            for item in &task.items {
                case_idx += 1;
                let case_data = prob.data.iter().find(|d| d.id == item.id).context("找不到数据点")?;

                fs::copy(
                    prob.path.join("data").join(&case_data.input),
                    main_dir.join("data").join(format!("{}{}.in", prob.name, case_idx))
                )?;
                fs::copy(
                    prob.path.join("data").join(&case_data.output),
                    main_dir.join("data").join(format!("{}{}.ans", prob.name, case_idx))
                )?;

                probinfo.insert(
                    format!("MARK={}@", case_idx),
                    (score_per_case as u32).to_string()
                );
            }
        }

        if prob.use_chk.unwrap_or(false) {
            info!("尝试编译 SPJ: {}", prob.name);
            let chk_src = prob.path.join("data/chk/chk.cpp");
            if !chk_src.exists() {
                bail!("chk 文件不存在: {}", chk_src.display());
            }
            let status = Command::new("g++")
                .arg(&chk_src)
                .arg("-o")
                .arg(main_dir.join("filter").join(format!("{}_e", prob.name)))
                .arg("-O2")
                .arg("-std=c++23")
                .status()?;
            if !status.success() {
                bail!("编译 chk 失败: {}", prob.name);
            }
        } else {
            let checker_sample = get_context().assets_dirs.iter().find_map(|dir| {
                let path = dir.join("sample/arbiter_e.sample");
                if path.exists() { Some(path) } else { None }
            });
            if let Some(sample) = checker_sample {
                fs::copy(sample, main_dir.join("filter").join(format!("{}_e", prob.name)))?;
            }
        }

        arbiter_info(&probinfo, &main_dir.join(format!("task{}_{}.info", day_num, prob_num)))?;
        pb.inc(1);
    }

    // setup.cfg
    let mut cfg = BTreeMap::new();
    cfg.insert("NAME=".to_string(), day.name.clone());
    cfg.insert("DAYNUM=".to_string(), day_num.to_string());
    cfg.insert("ENV=".to_string(), "env.info".to_string());
    cfg.insert("PLAYER=".to_string(), "player.info".to_string());
    cfg.insert("TEAM=".to_string(), "team.info".to_string());
    cfg.insert("MISC=".to_string(), "misc.info".to_string());
    arbiter_info(&cfg, &main_dir.join("setup.cfg"))?;

    // empty files
    fs::write(main_dir.join("team.info"), "")?;
    fs::write(main_dir.join("player.info"), "")?;

    // Copy data to evaldata
    copy_dir_recursive(main_dir.join("data"), main_dir.join("evaldata"))?;

    // Arbiter down
    let down_dir = arbiter_dir.join("down");
    fs::create_dir_all(&down_dir)?;

    for (_, prob) in &day.subconfig {
        let prob_down = down_dir.join(&prob.name);
        fs::create_dir_all(&prob_down)?;

        for (idx, sample) in prob.samples.iter().enumerate() {
            let idx = idx + 1;
            if let Some(input) = sample.input.get() {
                let src = prob.path.join("sample").join(input);
                if src.exists() {
                    fs::copy(src, prob_down.join(format!("{}{}.in", prob.name, idx)))?;
                }
            }
            if let Some(output) = sample.output.get() {
                let src = prob.path.join("sample").join(output);
                if src.exists() {
                    fs::copy(src, prob_down.join(format!("{}{}.ans", prob.name, idx)))?;
                }
            }
        }

        let prob_down_src = prob.path.join("down");
        if prob_down_src.exists() {
            for entry in fs::read_dir(prob_down_src)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::copy(&path, prob_down.join(path.file_name().unwrap()))?;
                }
            }
        }
    }

    pb.finish_with_message("Arbiter dump complete");
    Ok(())
}

//! 发布产物。

use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::ValueEnum;
use xshell::{Shell, cmd};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::checkers;
use crate::target::{LibcChoice, Target};
use crate::workspace;

/// 发布产物的打包平台。
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Platform {
    Zip,
    Deb,
    Nix,
}

/// 构建发布产物，并按平台打包。
pub fn run(platform: Platform, libc: Option<LibcChoice>) -> Result<()> {
    let target = Target::from_host(libc)?;
    let sh = Shell::new()?;
    let checkers = build_checkers(target, platform)?;
    build_binary(&sh, target, platform)?;

    // Nix 的 testlib 由 Nix 自行符号链接
    let testlib = match platform {
        Platform::Nix => None,
        Platform::Zip | Platform::Deb => Some(checkers::testlib_source()),
    };
    let dist = stage(target, &sh, testlib.as_deref(), &checkers)?;

    match platform {
        Platform::Nix => {
            // 由 Nix 处理打包
            Ok(())
        }
        Platform::Zip => package_zip(&dist, target),
        Platform::Deb => package_deb(&sh, target),
    }
}

/// 把 checker 源码复制到 `target/checkers` 后编译，返回该目录。
fn build_checkers(target: Target, platform: Platform) -> Result<PathBuf> {
    let root = workspace::root();
    let checkers_dir = root.join("target/checkers");
    if checkers_dir.exists() {
        fs::remove_dir_all(&checkers_dir)
            .with_context(|| format!("无法清空 {}", checkers_dir.display()))?;
    }
    fs::create_dir_all(&checkers_dir)
        .with_context(|| format!("无法创建 {}", checkers_dir.display()))?;
    for entry in fs::read_dir(root.join("assets/checkers")).context("无法读取 assets/checkers")?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            fs::copy(&path, checkers_dir.join(entry.file_name()))
                .with_context(|| format!("无法复制 {}", path.display()))?;
        }
    }

    let testlib = checkers::testlib_source();
    let compiler: Vec<String> = if target.is_musl() {
        vec![
            "zig".to_owned(),
            "c++".to_owned(),
            "-target".to_owned(),
            target.musl_zig_target(),
        ]
    } else if target.is_windows() {
        vec![
            "zig".to_owned(),
            "c++".to_owned(),
            "-target".to_owned(),
            format!(
                "{}-windows-gnu",
                if target.is_aarch64() {
                    "aarch64"
                } else {
                    "x86_64"
                }
            ),
        ]
    } else {
        vec!["g++".to_owned()]
    };
    checkers::build_with(
        &checkers_dir,
        &testlib,
        &compiler,
        // Nix 下不应也无须静态链接
        !matches!(platform, Platform::Nix),
    )?;
    Ok(checkers_dir)
}

fn build_binary(sh: &Shell, target: Target, platform: Platform) -> Result<()> {
    let nix = matches!(platform, Platform::Nix);
    let build_target = if target.is_gnu() && !nix {
        format!("{target}.2.28")
    } else {
        target.to_string()
    };
    let subcommand = if target.is_windows() || nix {
        "build"
    } else {
        "zigbuild"
    };
    let mut cmd = sh
        .cmd("cargo")
        .arg(subcommand)
        .arg("--release")
        .arg("--locked")
        .arg("-p")
        .arg("tuack-ng");
    if nix {
        cmd = cmd.arg("--no-default-features").arg("--features=nix");
    }
    cmd.arg("--target").arg(build_target).run()?;
    Ok(())
}

fn stage(target: Target, sh: &Shell, testlib: Option<&Path>, checkers: &Path) -> Result<PathBuf> {
    let root = workspace::root();
    let dist = root.join("target/dist");

    if dist.exists() {
        fs::remove_dir_all(&dist).with_context(|| format!("无法清空 {}", dist.display()))?;
    }
    fs::create_dir_all(&dist).with_context(|| format!("无法创建 {}", dist.display()))?;

    let exe = format!("tuack-ng{}", env::consts::EXE_SUFFIX);
    let bin = root.join(format!("target/{target}/release/{exe}"));
    fs::copy(&bin, dist.join(&exe)).with_context(|| format!("无法复制 {}", bin.display()))?;

    copy_dir_all(&root.join("assets"), &dist.join("assets"))?;

    let checkers_dest = dist.join("assets/checkers");
    if checkers_dest.exists() {
        fs::remove_dir_all(&checkers_dest)
            .with_context(|| format!("无法删除 {}", checkers_dest.display()))?;
    }

    copy_dir_all(checkers, &checkers_dest)?;

    for entry in fs::read_dir(&checkers_dest)
        .with_context(|| format!("无法读取 {}", checkers_dest.display()))?
    {
        let path = entry?.path();
        // Windows 编译会留下 pdb 调试符号，产物里不需要
        if path.extension().is_some_and(|ext| ext == "pdb") {
            fs::remove_file(&path).with_context(|| format!("无法删除 {}", path.display()))?;
        }
    }

    let testlib_dest = dist.join("assets/checkers/testlib.h");
    match testlib {
        Some(testlib) => checkers::copy_testlib(testlib, &testlib_dest)?,
        // Nix 的 testlib 由 Nix 自行符号链接，产物里不留
        None if testlib_dest.exists() => fs::remove_file(&testlib_dest)
            .with_context(|| format!("无法删除 {}", testlib_dest.display()))?,
        None => {}
    }

    let license = root.join("LICENSE");
    fs::copy(&license, dist.join("LICENSE"))
        .with_context(|| format!("无法复制 {}", license.display()))?;
    let notice = root.join("packaging/NOTICE.md");
    fs::copy(&notice, dist.join("NOTICE.md"))
        .with_context(|| format!("无法复制 {}", notice.display()))?;

    let completions = dist.join("completions");
    fs::create_dir_all(&completions)
        .with_context(|| format!("无法创建 {}", completions.display()))?;
    for shell in ["bash", "zsh", "fish"] {
        let script = cmd!(sh, "{bin} gen complete {shell}").read()?;
        let path = completions.join(shell);
        fs::write(&path, script).with_context(|| format!("无法写入 {}", path.display()))?;
    }
    Ok(dist)
}

fn package_deb(sh: &Shell, target: Target) -> Result<()> {
    let triple = target.to_string();
    let output = workspace::root()
        .join("target/debian")
        .join(format!("tuack-ng-{}.deb", target.artifact_suffix()));
    let mut cmd = sh
        .cmd("cargo")
        .arg("deb")
        .arg("--no-build")
        .arg("--no-strip")
        .arg("-p")
        .arg("tuack-ng")
        .arg("--target")
        .arg(&triple)
        .arg("--output")
        .arg(&output);
    if target.is_musl() {
        cmd = cmd.arg("--variant=musl");
    }
    cmd.run()?;
    Ok(())
}

fn package_zip(dist: &Path, target: Target) -> Result<()> {
    let archive =
        workspace::root().join(format!("target/tuack-ng-{}.zip", target.artifact_suffix()));

    let mut files = Vec::new();
    collect_files(dist, &mut files)?;
    files.sort();

    let mut writer = ZipWriter::new(
        File::create(&archive).with_context(|| format!("无法创建 {}", archive.display()))?,
    );

    for path in files {
        let name = path.strip_prefix(dist).expect("文件应位于 dist 下");
        let options = file_options(&fs::metadata(&path)?);

        writer
            .start_file_from_path(name, options)
            .with_context(|| format!("无法写入 {}", path.display()))?;

        let mut file = File::open(&path).with_context(|| format!("无法打开 {}", path.display()))?;
        io::copy(&mut file, &mut writer).with_context(|| format!("无法写入 {}", path.display()))?;
    }
    writer.finish().context("无法完成 zip")?;
    Ok(())
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("无法读取 {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn file_options(meta: &fs::Metadata) -> SimpleFileOptions {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        options.unix_permissions(meta.permissions().mode())
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        options
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("无法创建 {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("无法读取 {}", src.display()))? {
        let entry = entry?;
        // 子模块的 .git 只是一个指向仓库的指针，打进产物没有意义。
        if entry.file_name().as_os_str() == ".git" {
            continue;
        }
        let dst = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst)?;
        } else {
            fs::copy(entry.path(), &dst)
                .with_context(|| format!("无法复制 {}", entry.path().display()))?;
        }
    }
    Ok(())
}

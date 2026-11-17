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
use crate::target::Target;
use crate::workspace;

/// 发布产物的打包平台。
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Platform {
    Zip,
    Deb,
    Nix,
}

/// 构建发布产物，并按平台打包。
pub fn run(target: Target, platform: Platform) -> Result<()> {
    let sh = Shell::new()?;
    build_checkers(target, platform)?;
    build_binary(&sh, target)?;

    let testlib = match platform {
        Platform::Nix => None,
        Platform::Zip | Platform::Deb => Some(checkers::testlib_source()),
    };
    let dist = stage(target, testlib.as_deref())?;

    match platform {
        Platform::Nix => Ok(()),
        Platform::Zip => package_zip(&dist, target),
        Platform::Deb => package_deb(&sh, target),
    }
}

fn build_checkers(target: Target, platform: Platform) -> Result<()> {
    let checkers_dir = workspace::root().join("assets/checkers");
    let testlib = checkers::testlib_source();
    let compiler: Vec<String> = if target.is_musl() {
        vec![
            "zig".to_owned(),
            "c++".to_owned(),
            "-target".to_owned(),
            target.musl_zig_target(),
        ]
    } else {
        vec!["g++".to_owned()]
    };
    let static_link = !matches!(platform, Platform::Nix);
    checkers::build_with(&checkers_dir, &testlib, &compiler, static_link)
}

fn build_binary(sh: &Shell, target: Target) -> Result<()> {
    let build_target = if target.is_gnu() {
        format!("{target}.2.28")
    } else {
        target.to_string()
    };
    let subcommand = if target.is_windows() {
        "build"
    } else {
        "zigbuild"
    };
    cmd!(
        sh,
        "cargo {subcommand} --release --locked -p tuack-ng --target {build_target}"
    )
    .run()?;
    Ok(())
}

fn stage(target: Target, testlib: Option<&Path>) -> Result<PathBuf> {
    let root = workspace::root();
    let dist = root.join("target").join("dist");

    if dist.exists() {
        fs::remove_dir_all(&dist).with_context(|| format!("无法清空 {}", dist.display()))?;
    }
    fs::create_dir_all(&dist).with_context(|| format!("无法创建 {}", dist.display()))?;

    let exe = format!("tuack-ng{}", env::consts::EXE_SUFFIX);
    let bin = root
        .join("target")
        .join(target.to_string())
        .join("release")
        .join(&exe);
    fs::copy(&bin, dist.join(&exe)).with_context(|| format!("无法复制 {}", bin.display()))?;

    copy_dir_all(&root.join("assets"), &dist.join("assets"))?;

    let testlib_dest = dist.join("assets").join("checkers").join("testlib.h");
    if let Some(testlib) = testlib {
        fs::create_dir_all(testlib_dest.parent().expect("testlib.h 应有父目录"))?;
        fs::copy(testlib, &testlib_dest)
            .with_context(|| format!("无法复制 {}", testlib.display()))?;
    } else if testlib_dest.exists() {
        fs::remove_file(&testlib_dest)
            .with_context(|| format!("无法删除 {}", testlib_dest.display()))?;
    }
    Ok(dist)
}

fn package_deb(sh: &Shell, target: Target) -> Result<()> {
    let triple = target.to_string();
    cmd!(
        sh,
        "cargo deb --no-build --no-strip -p tuack-ng --target {triple}"
    )
    .run()?;
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

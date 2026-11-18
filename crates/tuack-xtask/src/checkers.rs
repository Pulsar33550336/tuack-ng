//! 构建 `assets/checkers` 下的内置 checker。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

use crate::workspace;

/// 构建 `assets/checkers` 下的 checker，使用 `g++` 与默认 testlib。
pub fn build(static_link: bool) -> Result<()> {
    let checkers_dir = workspace::root().join("assets/checkers");
    let compiler = ["g++".to_owned()];
    build_with(&checkers_dir, &testlib_source(), &compiler, static_link)
}

/// 构建 `checkers_dir` 下所有 checker。
///
/// testlib.h 不复制到 `checkers_dir`，编译时用 `-I` 指向 `testlib` 所在目录。
/// `compiler` 的首元素是程序，其余是基础参数；`static_link` 为真时给编译器加
/// `-static`。
///
/// # Errors
///
/// testlib.h 缺失、目录不可读、编译器无法启动或编译失败时返回错误。
pub fn build_with(
    checkers_dir: &Path,
    testlib: &Path,
    compiler: &[String],
    static_link: bool,
) -> Result<()> {
    let testlib = std::path::absolute(testlib).context("无法解析 testlib.h 路径")?;
    if !testlib.is_file() {
        bail!("找不到 testlib.h：{}", testlib.display());
    }
    let include_dir = testlib.parent().context("testlib.h 没有父目录")?;

    let entries = fs::read_dir(checkers_dir)
        .with_context(|| format!("无法读取 {}", checkers_dir.display()))?;
    let sh = Shell::new()?;
    for entry in entries {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "cpp") {
            compile(&sh, &path, compiler, include_dir, static_link)?;
        }
    }
    Ok(())
}

/// 把 `source` 处的 testlib.h 复制到 `dest`，内容一致时跳过。
pub fn copy_testlib(source: &Path, dest: &Path) -> Result<()> {
    if !source.is_file() {
        bail!("找不到 testlib.h：{}", source.display());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("无法创建 {}", parent.display()))?;
    }

    if dest.is_file() && fs::read(source)? == fs::read(dest)? {
        return Ok(());
    }

    fs::copy(source, dest)
        .with_context(|| format!("无法复制 {} 到 {}", source.display(), dest.display()))?;
    Ok(())
}

/// 返回 testlib.h 的来源：优先 `TESTLIB_PATH`，否则 `vendor/testlib/testlib.h`。
pub fn testlib_source() -> PathBuf {
    env::var_os("TESTLIB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace::root().join("vendor/testlib/testlib.h"))
}

fn compile(
    sh: &Shell,
    cpp: &Path,
    compiler: &[String],
    include_dir: &Path,
    static_link: bool,
) -> Result<()> {
    let exe = cpp.with_extension(env::consts::EXE_EXTENSION);
    if !needs_compile(cpp, &exe) {
        return Ok(());
    }

    // 首元素是程序，其余是基础参数。
    let (program, base_args) = compiler.split_first().context("编译器命令为空")?;
    let cpp_name = cpp.file_name().context("checker 源文件没有文件名")?;
    let exe_name = exe.file_name().context("checker 产物没有文件名")?;
    let static_arg = static_link.then_some("-static");
    let _dir = sh.push_dir(cpp.parent().context("checker 源文件没有父目录")?);
    cmd!(
        sh,
        "{program} {base_args...} -std=c++17 -O2 -I {include_dir} {cpp_name} -o {exe_name} {static_arg...}"
    )
    .run()?;
    Ok(())
}

fn needs_compile(cpp: &Path, exe: &Path) -> bool {
    let (Ok(cpp_meta), Ok(exe_meta)) = (fs::metadata(cpp), fs::metadata(exe)) else {
        return true;
    };
    let (Ok(src_time), Ok(exe_time)) = (cpp_meta.modified(), exe_meta.modified()) else {
        return true;
    };
    src_time > exe_time
}

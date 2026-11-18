//! 构建 `assets/checkers` 下的内置 checker。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

use crate::workspace;

/// 构建 `assets/checkers` 下的 checker，使用 `g++` 与默认 testlib。
pub fn build() -> Result<()> {
    let checkers_dir = workspace::root().join("assets/checkers");
    build_with(&checkers_dir, &testlib_source(), Compiler::Host)
}

/// C++ 编译器。
pub enum Compiler {
    /// 宿主 `g++`，本地开发与 Nix 构建用。
    Host,
    /// zig 交叉编译到指定的 C++ 三元组。
    Zig {
        /// zig 接收的 `-target {target}`
        target: String,
    },
    /// MSVC 的 `cl.exe`，用 `/MT` 静态链接 CRT。
    Msvc,
}

/// 构建 `checkers_dir` 下所有 checker。
///
/// testlib.h 不复制到 `checkers_dir`，编译时用 `-I` 指向 `testlib` 所在目录。
///
/// # Errors
///
/// testlib.h 缺失、目录不可读、编译器无法启动或编译失败时返回错误。
pub fn build_with(checkers_dir: &Path, testlib: &Path, compiler: Compiler) -> Result<()> {
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
            compile(&sh, &path, &compiler, include_dir)?;
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

fn compile(sh: &Shell, cpp: &Path, compiler: &Compiler, include_dir: &Path) -> Result<()> {
    let exe = cpp.with_extension(env::consts::EXE_EXTENSION);
    if !needs_compile(cpp, &exe) {
        return Ok(());
    }

    let cpp_name = cpp.file_name().context("checker 源文件没有文件名")?;
    let exe_name = exe.file_name().context("checker 产物没有文件名")?;
    let _dir = sh.push_dir(cpp.parent().context("checker 源文件没有父目录")?);
    match compiler {
        Compiler::Host => {
            cmd!(
                sh,
                "g++ -std=c++17 -O2 -I {include_dir} {cpp_name} -o {exe_name}"
            )
            .run()?;
        }
        Compiler::Zig { target } => {
            cmd!(
                sh,
                "zig c++ -target {target} -std=c++17 -O2 -I {include_dir} {cpp_name} -o {exe_name}"
            )
            .run()?;
        }
        Compiler::Msvc => {
            let include = format!("/I{}", include_dir.display());
            let output = format!("/Fe:{}", exe_name.to_string_lossy());
            cmd!(
                sh,
                "cl /nologo /std:c++17 /O2 /EHsc /MT {include} {cpp_name} {output}"
            )
            .run()?;
        }
    }
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

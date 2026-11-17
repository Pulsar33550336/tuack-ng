//! 构建目标。

use std::env;
use std::fmt;

use anyhow::{Result, bail};
use clap::ValueEnum;

/// 发布构建目标。
#[derive(Clone, Copy, Debug)]
pub struct Target {
    platform: Platform,
    arch: Arch,
    libc: Libc,
}

/// 命令行选择的 C 库。
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LibcChoice {
    Gnu,
    Musl,
}

impl Target {
    /// 从当前运行平台构造目标，`libc` 省略时取平台默认值。
    pub fn from_host(libc: Option<LibcChoice>) -> Result<Self> {
        let platform = match env::consts::OS {
            "linux" => Platform::Linux,
            "windows" => Platform::Windows,
            other => bail!("不支持的操作系统：{other}"),
        };
        let arch = match env::consts::ARCH {
            "x86_64" => Arch::X86_64,
            "aarch64" => Arch::Aarch64,
            other => bail!("不支持的架构：{other}"),
        };
        let libc = match (platform, libc) {
            (Platform::Linux, None | Some(LibcChoice::Gnu)) => Libc::Gnu,
            (Platform::Linux, Some(LibcChoice::Musl)) => Libc::Musl,
            (Platform::Windows, None) => Libc::Msvc,
            (Platform::Windows, Some(_)) => bail!("Windows 只能使用 msvc"),
        };
        Ok(Self {
            platform,
            arch,
            libc,
        })
    }

    /// 是否使用 glibc。
    pub fn is_gnu(self) -> bool {
        matches!(self.libc, Libc::Gnu)
    }

    /// 是否使用 musl。
    pub fn is_musl(self) -> bool {
        matches!(self.libc, Libc::Musl)
    }

    /// 是否为 Windows 目标。
    pub fn is_windows(self) -> bool {
        matches!(self.platform, Platform::Windows)
    }

    /// 是否为 aarch64 目标。
    pub fn is_aarch64(self) -> bool {
        matches!(self.arch, Arch::Aarch64)
    }

    /// 产物名中使用的平台与架构，例如 `linux-x86_64`；musl 目标附加 `-musl`。
    pub fn artifact_suffix(self) -> String {
        let platform = match self.platform {
            Platform::Linux => "linux",
            Platform::Windows => "windows",
        };
        let arch = match self.arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "arm64",
        };
        let libc = match self.libc {
            Libc::Musl => "-musl",
            _ => "",
        };
        format!("{platform}-{arch}{libc}")
    }

    /// zig 编译 C++ 时使用的 musl 目标，例如 `x86_64-linux-musl`。
    ///
    /// # Panics
    ///
    /// 目标不是 musl 时 panic。
    pub fn musl_zig_target(self) -> String {
        assert!(self.is_musl(), "musl_zig_target 仅适用于 musl 目标");
        let arch = match self.arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
        };
        format!("{arch}-linux-musl")
    }
}

/// 目标平台。
#[derive(Clone, Copy, Debug)]
enum Platform {
    Linux,
    Windows,
}

/// 目标指令集。
#[derive(Clone, Copy, Debug)]
enum Arch {
    X86_64,
    Aarch64,
}

/// 目标 C 库。
#[derive(Clone, Copy, Debug)]
enum Libc {
    Gnu,
    Musl,
    Msvc,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.arch, self.platform, self.libc)
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Linux => "unknown-linux",
            Self::Windows => "pc-windows",
        })
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        })
    }
}

impl fmt::Display for Libc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Gnu => "gnu",
            Self::Musl => "musl",
            Self::Msvc => "msvc",
        })
    }
}

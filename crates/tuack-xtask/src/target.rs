//! 构建目标。

use std::fmt;
use std::str::FromStr;

/// 发布构建目标。
#[derive(Clone, Copy, Debug)]
pub struct Target {
    platform: Platform,
    arch: Arch,
    libc: Libc,
}

impl Target {
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
            Libc::Gnu | Libc::Msvc => "",
        };
        format!("{platform}-{arch}{libc}")
    }

    /// zig 编译 C++ 时使用的 musl 目标，例如 `x86_64-linux-musl`。
    pub fn musl_zig_target(self) -> String {
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

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (platform, arch, libc) = match s {
            "x86_64-unknown-linux-gnu" => (Platform::Linux, Arch::X86_64, Libc::Gnu),
            "x86_64-unknown-linux-musl" => (Platform::Linux, Arch::X86_64, Libc::Musl),
            "aarch64-unknown-linux-gnu" => (Platform::Linux, Arch::Aarch64, Libc::Gnu),
            "aarch64-unknown-linux-musl" => (Platform::Linux, Arch::Aarch64, Libc::Musl),
            "x86_64-pc-windows-msvc" => (Platform::Windows, Arch::X86_64, Libc::Msvc),
            "aarch64-pc-windows-msvc" => (Platform::Windows, Arch::Aarch64, Libc::Msvc),
            _ => return Err(format!("不支持的目标：{s}")),
        };
        Ok(Self {
            platform,
            arch,
            libc,
        })
    }
}

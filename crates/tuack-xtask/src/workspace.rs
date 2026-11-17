//! 工作区路径。

use std::path::{Path, PathBuf};

/// 返回 workspace 根目录。
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tuack-xtask 应位于 crates/tuack-xtask")
        .to_path_buf()
}

//! Tuack-NG 处理器 / 渲染器 / 导出器插件的 Rust SDK。
//!
//! - [`Processor`] / [`processor!`](crate::processor!)：处理题目 AST
//! - [`Renderer`] / [`renderer!`](crate::renderer!)：渲染题面
//! - [`Dumper`] / [`dumper!`](crate::dumper!)：导出到评测系统
//!
//! 实现对应 trait 并用宏注册后，编解码、内存与错误处理均由 SDK 接管。
//!
//! # Examples
//!
//! 一个最小处理器插件（编译目标 `wasm32-wasip1`）：
//! ```ignore
//! #![no_main]
//! use tuack_plugin_sdk::{processor, Document, Error, Processor, ProcessorOutput};
//!
//! struct MyProcessor;
//!
//! impl Processor for MyProcessor {
//!     fn new() -> Self {
//!         MyProcessor
//!     }
//!
//!     fn process(&self, doc: Document) -> Result<ProcessorOutput, Error> {
//!         Ok(ProcessorOutput {
//!             ast: doc,
//!             warnings: Vec::new(),
//!         })
//!     }
//! }
//!
//! processor!(MyProcessor);
//! ```

pub use extism_pdk;
pub use extism_pdk::{Error, Json, host_fn};
pub use log;
pub use tuack_lib::dump::{
    DumpCase, DumpChecker, DumpConfig, DumpDocument, DumpFile, DumpProblem, DumpSample,
    DumpSubtask, ScorePolicy,
};
pub use tuack_lib::plugin::{
    AssetChunk, AssetError, CommandError, CommandResult, DumperOutput, OutputSpec, PathError,
    RendererOutput,
};
pub use tuack_lib::ren::{
    DateInfo, Problem, ProblemMeta, ProblemType, ProcessorOutput, RenConfig, RenParams,
    RenderDocument, SupportLanguage,
};
pub use tuack_lib::utils::output::OutputFile;
pub use tuack_ng_parser::ast::Document;

mod dumper;
mod host;
mod logger;
mod output;
mod processor;
mod renderer;

pub use dumper::Dumper;
pub use host::{AssetReader, command, get_path};
pub use logger::__init_logger;
pub use output::__to_specs;
pub use processor::Processor;
pub use renderer::Renderer;

/// 将错误回传给宿主（供 [`processor!`](crate::processor!) /
/// [`renderer!`](crate::renderer!) / [`dumper!`](crate::dumper!) 宏内部使用）。
#[doc(hidden)]
pub fn __report_error(e: &extism_pdk::Error) {
    let err = format!("{:?}", e);
    let mem = extism_pdk::Memory::from_bytes(&err).unwrap();
    unsafe {
        extism_pdk::extism::error_set(mem.offset());
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    /// SDK 版本与宿主插件 API 版本须 major.minor 一致（patch 可独立升）：插件清单里的
    /// `pluginapi` 写 SDK 版本，宿主判定兼容性时忽略 patch。
    #[test]
    fn sdk_version_matches_plugin_api() {
        let sdk = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let api = Version::parse(tuack_lib::plugin::PLUGIN_API_VERSION).unwrap();
        assert_eq!((sdk.major, sdk.minor), (api.major, api.minor));
    }
}

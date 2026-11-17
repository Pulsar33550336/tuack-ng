//! Tuack-NG 下一代 Markdown 解析器。
//!
//! 基于 `rushdown` 解析，产出可遍历的 AST。
//!
//! - [`parse`]：Markdown 文本到 [`Document`]
//! - [`ast`]：AST 节点类型
//! - [`visitor`] / [`transform`]：只读遍历与就地变换
//! - [`printers`]：输出 Markdown / Typst
//! - [`span`]：源码位置
//!
//! # Examples
//!
//! ```
//! let doc = tuack_ng_parser::parse("# 标题\n");
//! assert_eq!(doc.blocks.len(), 1);
//! ```

pub mod ast;
pub mod parser;
pub mod printers;
pub mod span;
pub mod transform;
pub mod visitor;

pub use ast::{Block, Document, Inline, TableCell};
pub use parser::parse;
pub use span::{Span, Spanned};

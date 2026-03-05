use markdown_ppp::ast::{Document, Block, HeadingKind, SetextHeading};

pub trait Quirks {
    fn apply(&self, ast: Document) -> Document;
}

pub struct UojQuirks;
impl Quirks for UojQuirks {
    fn apply(&self, ast: Document) -> Document {
        let mut new_blocks = Vec::new();
        for block in ast.blocks {
            if let Block::Heading(mut heading) = block {
                match heading.kind {
                    HeadingKind::Atx(level) => {
                        heading.kind = HeadingKind::Atx(level + 1);
                    }
                    HeadingKind::Setext(level) => {
                        heading.kind = match level {
                            SetextHeading::Level1 => HeadingKind::Setext(SetextHeading::Level2),
                            SetextHeading::Level2 => HeadingKind::Atx(3),
                        };
                    }
                }
                new_blocks.push(Block::Heading(heading));
            } else {
                new_blocks.push(block);
            }
        }
        Document { blocks: new_blocks }
    }
}

pub struct LojQuirks;
impl Quirks for LojQuirks {
    fn apply(&self, ast: Document) -> Document {
        ast
    }
}

pub fn get_quirks(target: &str) -> Vec<Box<dyn Quirks>> {
    let mut quirks: Vec<Box<dyn Quirks>> = Vec::new();
    let target_lower = target.to_lowercase();
    if target_lower.contains("uoj") {
        quirks.push(Box::new(UojQuirks));
    }
    if target_lower.contains("loj") || target_lower.contains("ipuoj") {
        quirks.push(Box::new(LojQuirks));
    }
    quirks
}

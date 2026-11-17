//! 校验提交信息，规则见 CONTRIBUTING.md 的「提交」一节。

use std::fs;
use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use xshell::{Shell, cmd};

use crate::workspace;

/// 允许的提交类型。
const TYPES: &[&str] = &[
    "feat", "fix", "perf", "refactor", "docs", "test", "ci", "build", "chore", "revert", "release",
];

/// 允许的顶层范围。
const SCOPES: &[&str] = &[
    "cli",
    "lib",
    "config",
    "utils",
    "parser",
    "plugin-sdk",
    "assets",
    "nix",
    "ci",
    "release",
    "deps",
];

/// 无需校验的标题前缀：合并、回滚与 fixup/squash。
const SKIPPED: &[&str] = &["Merge ", "Revert ", "revert:", "fixup!", "squash!"];

/// 校验 `path` 里的提交信息，不合规时打印原因并以状态码 1 退出。
///
/// 空提交与 [`SKIPPED`] 中的标题直接放行。
pub fn run_file(path: &Path) -> Result<()> {
    let message =
        fs::read_to_string(path).with_context(|| format!("无法读取 {}", path.display()))?;
    let Some(subject) = subject(&message) else {
        return Ok(());
    };
    if let Err(reason) = check(subject, &message) {
        eprint!("{}", report(&reason));
        process::exit(1);
    }
    Ok(())
}

/// 校验 `range` 内每个提交的信息，跳过 merge 提交。
///
/// `range` 接受 git 的 `A..B` 与 `A...B` 语法。左端是全 0 的占位提交时视为新建
/// 分支，直接放行；有提交不合规时逐个打印原因，最后以状态码 1 退出。
pub fn run_range(range: &str) -> Result<()> {
    if is_null_revision(range) {
        return Ok(());
    }
    let root = workspace::root();
    let sh = Shell::new()?;
    let left = range.split("..").next().unwrap_or_default();
    let shas = if left.is_empty() || exists(&sh, &root, left)? {
        cmd!(sh, "git -C {root} rev-list --no-merges {range}")
            .read()
            .with_context(|| format!("无法解析提交范围 {range}"))?
    } else {
        // 强制推送后旧提交可能不在克隆里，退化为只校验 HEAD
        eprintln!("提交 {left} 不在本地仓库中，改为只校验 HEAD");
        cmd!(sh, "git -C {root} rev-list --no-merges -n 1 HEAD").read()?
    };

    let mut failed = false;
    for sha in shas.split_whitespace() {
        let message = cmd!(sh, "git -C {root} log -1 --format=%B {sha}")
            .read()
            .with_context(|| format!("无法读取提交 {sha} 的信息"))?;
        let Some(subject) = subject(&message) else {
            continue;
        };
        if let Err(reason) = check(subject, &message) {
            eprintln!("{}", report(&reason));
            eprintln!("  提交：{sha}");
            failed = true;
        }
    }
    if failed {
        process::exit(1);
    }
    Ok(())
}

/// 判断 `rev` 在仓库里能否解析成提交。
fn exists(sh: &Shell, root: &Path, rev: &str) -> Result<bool> {
    let spec = format!("{rev}^{{commit}}");
    let output = sh
        .cmd("git")
        .arg("-C")
        .arg(root)
        .arg("cat-file")
        .arg("-e")
        .arg(&spec)
        .ignore_status()
        .output()?;
    Ok(output.status.success())
}

/// 判断范围左端是否为全 0 的占位提交。
fn is_null_revision(range: &str) -> bool {
    let left = range.split("..").next().unwrap_or_default();
    !left.is_empty() && left.chars().all(|c| c == '0')
}

/// 取标题：第一个非空且不是注释的行。
fn subject(message: &str) -> Option<&str> {
    message
        .lines()
        .find(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

/// 校验标题，返回不合规的原因。
fn check(subject: &str, message: &str) -> Result<(), String> {
    if SKIPPED.iter().any(|prefix| subject.starts_with(prefix)) {
        return Ok(());
    }

    let (head, description) = subject
        .split_once(": ")
        .ok_or_else(|| "缺少「: 」分隔符".to_owned())?;
    let (head, breaking) = match head.strip_suffix('!') {
        Some(head) => (head, true),
        None => (head, false),
    };
    let (kind, scope) = split_scope(head)?;

    if kind.is_empty() {
        return Err("缺少类型".to_owned());
    }
    if !TYPES.contains(&kind) {
        return Err(format!("未知类型：{kind}"));
    }
    if let Some(scope) = scope {
        check_scope(scope)?;
    }
    if description.is_empty() {
        return Err("描述为空".to_owned());
    }
    if description.ends_with(['。', '.']) {
        return Err("描述句末不要加句号".to_owned());
    }
    if breaking && !message.lines().any(is_breaking_note) {
        return Err("破坏性变更需要在正文写 `BREAKING CHANGE: ` 描述破坏性".to_owned());
    }
    Ok(())
}

/// 从标题的头部拆出类型与范围，范围要求以 `)` 收尾。
fn split_scope(head: &str) -> Result<(&str, Option<&str>), String> {
    match head.split_once('(') {
        Some((kind, rest)) => {
            let scope = rest
                .strip_suffix(')')
                .ok_or_else(|| "范围的括号没有闭合".to_owned())?;
            Ok((kind, Some(scope)))
        }
        None => Ok((head, None)),
    }
}

/// 校验范围：用小写字母、数字、下划线、减号与斜杠书写，顶层必须是已知的 crate 或仓库级事务。
fn check_scope(scope: &str) -> Result<(), String> {
    if scope.is_empty() {
        return Err("范围为空".to_owned());
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '/');
    if !scope.chars().all(allowed) {
        return Err(format!(
            "范围只能用小写字母、数字、下划线、减号与斜杠：{scope}"
        ));
    }
    if scope.starts_with('/') || scope.ends_with('/') || scope.contains("//") {
        return Err(format!("范围路径不合法：{scope}"));
    }
    let top = scope.split('/').next().unwrap_or(scope);
    if !SCOPES.contains(&top) {
        return Err(format!("未知的顶层范围：{top}"));
    }
    Ok(())
}

/// 判断正文中是否写了破坏性变更的迁移说明。
fn is_breaking_note(line: &str) -> bool {
    line.starts_with("BREAKING CHANGE: ") || line.starts_with("BREAKING-CHANGE: ")
}

/// 拼出完整的错误报告。
fn report(reason: &str) -> String {
    format!("提交信息不符合规范：{reason}")
}

#[cfg(test)]
mod tests {
    use super::*;

    //autocorrect-disable

    fn violation(subject: &str) -> Option<String> {
        check(subject, subject).err()
    }

    #[test]
    fn accepts_conventional_subjects() {
        for subject in [
            "feat: 基于 Rust 的构建管线",
            "fix(cli/generate): gen code 跳过 interactive 目录",
            "ci(nix/templates): 补全模板里的 checker",
            "release: 1.2.0",
        ] {
            assert_eq!(violation(subject), None, "{subject}");
        }
    }

    #[test]
    fn skips_merge_and_fixup() {
        for subject in [
            "Merge branch 'master' into rpc",
            "Revert \"feat: x\"",
            "revert: feat: x",
            "fixup! feat: x",
            "squash! feat: x",
        ] {
            assert_eq!(violation(subject), None, "{subject}");
        }
    }

    #[test]
    fn rejects_broken_subjects() {
        let cases = [
            ("feat 缺少分隔符", "缺少「: 」分隔符"),
            (": 空类型", "缺少类型"),
            ("frobnicate: 未知类型", "未知类型：frobnicate"),
            ("fix(): 空范围", "范围为空"),
            ("fix(cli: 括号未闭合", "范围的括号没有闭合"),
            ("fix(CLI): 大写范围", "范围只能用小写字母"),
            ("fix(gone): 未知顶层", "未知的顶层范围：gone"),
            ("fix(cli/): 尾斜杠", "范围路径不合法"),
            ("fix(cli//doc): 双斜杠", "范围路径不合法"),
            ("fix: 句号。", "描述句末不要加句号"),
            ("fix: 句号.", "描述句末不要加句号"),
            ("fix: 句号。", "描述句末不要加句号"),
            ("fix: 句号.", "描述句末不要加句号"),
            ("fix(cli)!: 破坏性变更", "破坏性变更需要在正文写"),
        ];
        for (subject, expected) in cases {
            let reason = violation(subject).unwrap_or_else(|| panic!("{subject} 应被拒绝"));
            assert!(reason.contains(expected), "{subject} -> {reason}");
        }
    }

    #[test]
    fn breaking_change_needs_note() {
        let subject = "feat(cli)!: 换掉入口";
        assert!(
            check(
                subject,
                "feat(cli)!: 换掉入口\n\nBREAKING CHANGE: 改用 `tuack-ng run`\n"
            )
            .is_ok()
        );
        assert!(check(subject, subject).is_err());
    }

    #[test]
    fn detects_null_revision() {
        assert!(is_null_revision(
            "0000000000000000000000000000000000000000..HEAD"
        ));
        assert!(!is_null_revision("d4d70c1..HEAD"));
        assert!(!is_null_revision("..HEAD"));
    }

    #[test]
    fn subject_skips_comments_and_blank_lines() {
        assert_eq!(subject("\n# 注释\n\nfeat: x\n"), Some("feat: x"));
        assert_eq!(subject("\n# 只有注释\n\n"), None);
    }
    //autocorrect-enable
}

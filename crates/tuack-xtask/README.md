# xtask for Tuack-NG

## Why it?

1. 天下苦 GitHub CI 难写久矣，因此我尝试将主要的 CI 流程通过一个 xtask 来解决；
2. Tuack-NG 并不止是一个简单的 Rust 包，它有 Checker 构建逻辑，打包，等等。

引用 Nuke 的一句话：

> 稳定且可扩展的 CI/CD 流水线是保持竞争力和打造优秀产品的重要支柱。但为什么我们大多数人都对接触 YAML 文件有些畏惧，甚至不敢看一眼构建脚本？这在很大程度上是因为 C# 开发者被优秀的语言和智能 IDE 宠坏了，他们不喜欢失去代码补全、便捷调试、重构和代码格式化方面的好伙伴。

## 打算做什么

1. 把 `tuack-ng` 的 Checker 构建逻辑移出来。
2. 把打包逻辑从脚本迁移回 Rust；
3. 重写 pre-commit-hook。
4. 增加 `zigbuild` 等更加高级的构建逻辑

## WIP

没错，我才画完饼。

## 命令思路

- `xtask build-checkers`：构建 Checker。
- `xtask ci`：待定
- `xtask commit-hook`：Git 的钩子，通过什么方法安装。

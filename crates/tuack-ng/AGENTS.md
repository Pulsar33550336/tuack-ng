# Tuack-NG

命令行入口：命令解析、编排、进度条、错误输出与退出码。`gctx()` 是本 crate 的运行期全局上下文。题面检查/格式化规则在 `tuack-utils/doc`，不在本 crate。

## 退出码

- 0 正常；1 用户问题；2 未处理的自身问题。
- 用户问题在抛出点就地输出，并以 `Reported` 上抛；普通 `anyhow` 错误按自身问题处理。
- 批量命令：用户问题记录后继续，自身问题立即中止。

## 与其他 crate 的边界

- 改契约去 `tuack-lib`，改实现去 `tuack-utils`，改配置模型去 `tuack-config`，改解析/AST 去 `tuack-ng-parser`。
- 只在本 crate 写：CLI 参数、`gctx()`、用户可见消息、进度条、退出码、产物的展示与落盘决策。
- 新能力顺序：`tuack-lib` 契约 -> `tuack-utils` 实现 -> 这里接线。

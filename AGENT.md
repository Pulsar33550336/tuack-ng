作为一个 AI，你的任务是：完成 Tuack-NG 项目的迁移。

你有以下目录可用：

- tests/: 存储着我平时用于测试 tuack-ng 功能的“沙盒”
- tuack/: 原 Tuack，供参考。

这个项目的现状如下：

gen: 完成。
ren: Markdown部分需要你参考原Tuack，进行一些修改。这可能需要你修改 markdown-ppp 的代码，你可以自己克隆并修改，改好后随源代码一同附上。
test：完成，但是可能有点功能差异，比如倍率什么的
conf：完成。
dmk: 完成。
dump: 没写完，但是仅迁移 arbiter 即可。
doc: 没写，对于它的要求在下面。

doc需要拆分为以下内容：
tuack-ng fmt --> tuack doc format
tuack-ng check --> tuack doc check，以及检查所有配置文件的正确性，并展示序列化过程中的警告。
tuack-ng import --> 从原tuack导入，你量力而行。

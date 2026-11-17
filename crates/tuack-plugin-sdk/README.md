# tuack-plugin-sdk

Tuack-NG 插件的 Rust SDK：实现 `Processor` / `Renderer` / `Dumper`，用宏注册为 extism 导出函数，
JSON 编解码（字节载荷走 msgpack）、内存与错误处理由 SDK 接管。

编译目标为 `wasm32-wasip1`，产物作为 Tuack-NG 插件包的 `entry`。

完整示例见 [tuack-ng-plugin-example](https://github.com/tuackng/tuack-ng-plugin-example)。

许可证：MIT。

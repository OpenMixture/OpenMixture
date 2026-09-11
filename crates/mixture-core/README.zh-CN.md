# mixture-core

[English](./README.md) | 简体中文

纯 Rust `.mix` v1 解码、验证及确定性 `RenderPlan` 编译。本 crate 不依赖 GPU、CLI 或图像库。完整公开 API 示例见 [src/lib.rs](./src/lib.rs) 或生成的 Rustdoc。

本地 0.1.0 包处于 pre-alpha，仍禁用发布。已实现十一种带版本的节点契约；文档版本和计划版本仍为 1。明确拒绝不支持的输入。见仓库[兼容性策略](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.zh-CN.md)与[本地包验证](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.zh-CN.md)。

包包含源码、两种 README 语言及 [MIT](./LICENSE-MIT)／[Apache-2.0](./LICENSE-APACHE) 许可。仓库集成夹具和材质证据不是消费者运行时资源。在仓库运行 `cargo xtask package-check` 会于隔离本地工作区验证真实归档；它不发布包，也不声称包在注册表可用。

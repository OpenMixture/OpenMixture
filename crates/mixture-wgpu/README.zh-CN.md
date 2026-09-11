# mixture-wgpu

[English](./README.md) | 简体中文

Mixture 唯一像素执行器：显式 `GpuContext` 所有权、类型化 `RenderPlan` 执行、九个嵌入 WGSL 内核、自有 RGBA8 输出及结构化 GPU 诊断。公开示例见 [src/lib.rs](./src/lib.rs)；获取成功不代表 compute／readback 探针健康。

本地 0.1.0 包仍处于 pre-alpha，禁用发布。它需要匹配版本的 `mixture-core`，并通过已文档化的高级句柄及 limits 暴露 wgpu 30 类型。设备丢失不触发恢复；OOM 根据类型化错误分类。调度和 CPU 输出聚合保留由调用者负责。见仓库[兼容性策略](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.zh-CN.md)。

全部运行时 shader 位于 [shaders/nodes](./shaders/nodes)，在编译时嵌入；运行时不需要仓库路径。[src/testdata](./src/testdata) 中三个小型单元测试输入由 `package-check` 对照规范仓库夹具检查。本地归档包含源码、shader、源码内测试、两种 README 语言和 [MIT](./LICENSE-MIT)／[Apache-2.0](./LICENSE-APACHE) 许可。仓库集成测试及材质基准不纳入。

在仓库运行 `cargo xtask package-check` 完成隔离 CPU／包检查，显式 `cargo xtask gpu-smoke` 验证包的 GPU／CLI 消费。默认测试不获取 GPU。见[本地包验证](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.zh-CN.md)；这些命令不会发布包。

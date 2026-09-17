# mixture-wgpu

[English](./README.md) | 简体中文

Mixture 唯一像素执行器：显式 `GpuContext` 所有权、类型化 `RenderPlan` 执行、九个嵌入 WGSL 内核、自有 RGBA8 输出及结构化 GPU 诊断。公开示例见 [src/lib.rs](./src/lib.rs)；获取成功不代表 compute／readback 探针健康。

本地 0.1.0 包仍处于 pre-alpha，禁用发布。它需要匹配版本的 `mixture-core`，并通过已文档化的高级句柄及 limits 暴露 wgpu 30 类型。设备丢失不触发恢复；OOM 根据类型化错误分类。调度和 CPU 输出聚合保留由调用者负责。见仓库[兼容性策略](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.zh-CN.md)。

运行时节点 shader 位于 [shaders/nodes](./shaders/nodes)，共用[半精度存储转换](./shaders/precision.wgsl)，在编译时嵌入；运行时不需要仓库路径。[src/testdata](./src/testdata) 中三个小型单元测试输入由 `package-check` 对照规范仓库夹具检查。本地归档包含源码、shader、源码内测试、两种 README 语言和 [MIT](./LICENSE-MIT)／[Apache-2.0](./LICENSE-APACHE) 许可。仓库集成测试及材质基准不纳入。

在仓库运行 `cargo xtask package-check` 完成隔离 CPU／包检查，显式 `cargo xtask gpu-smoke` 验证包的 GPU／CLI 消费。默认测试不获取 GPU。见[本地包验证](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.zh-CN.md)；这些命令不会发布包。

`wasm32-unknown-unknown` 目标通过 `wgpu/webgpu` 仅启用浏览器 WebGPU 执行；该目标的 `BackendPreference::Auto` 选择 `BROWSER_WEBGPU`。原生 Auto 保持 Vulkan／Metal／Dx12 选择。不启用 WebGL、noop 或其他像素执行器。浏览器包负责安全上下文与 API 预检；缺失或被隐藏的适配器身份保持后端实际提供的证据。

浏览器提交与回读等待队列／映射回调及成对的异步错误作用域，期间让出事件循环。原生轮询及单次 30 秒等待保持不变。浏览器操作不新增超时或取消承诺：标签页终止或平台事件未送达可能阻止完成。墙钟计时通过 `web-time` 使用 `performance.now()`，包含浏览器调度与计时精度取整，不是 GPU 时间戳查询。浏览器 wgpu 错误保留类型化 OOM 分类及驱动／原因文本，不声称拥有原生 `Send + Sync` 错误来源链。

安装相应 Rust 目标后，使用 `cargo check --locked -p mixture-wgpu --target wasm32-unknown-unknown` 编译该执行器。这仅验证编译；真实浏览器渲染、奇数宽度回读、结果生命周期及设备丢失测试属于独立消费者与 M5 证据。当前已实现边界及剩余门槛见仓库[浏览器 SDK 契约](https://github.com/OpenMixture/OpenMixture/blob/main/docs/browser-sdk.zh-CN.md)。

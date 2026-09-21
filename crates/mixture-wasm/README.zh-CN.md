# mixture-wasm

[English](./README.md) | 简体中文

OpenMixture M5 的浏览器编译和传输边界。此未发布 crate 调用 `mixture-core` 完成严格原始源码解析、校验、目录和不可变计划编译，调用 `mixture-wgpu` 完成显式浏览器 WebGPU 获取和异步渲染。不增加像素代码或产品状态。

公开消费者安装独立构建的 `@openmixture/runtime` npm 归档。其 facade 负责安全 JS 参数捕获和单渲染/destroy 调度。Rust 返回独立目录/绑定元数据及 JS 自有 RGBA8 副本，并保留 core/GPU 诊断证据。u64 投影为 bigint，已验证数值参数保持 number。生成的 wasm-bindgen 胶水属于实现细节。

```sh
cargo check --locked -p mixture-wasm --target wasm32-unknown-unknown
npm ci --prefix packages/runtime --ignore-scripts
npm test --prefix packages/runtime
node scripts/browser-runtime/build.mjs
```

从引擎仓库执行。构建脚本需要固定 Rust WASM target 和 wasm-bindgen CLI 0.2.128，将同一源码构建身份注入 JS、类型和 WASM。直接 Cargo 构建标记为 unpackaged，不可替代经过检查的包产物。构建不等于浏览器渲染验收：使用独立产品生产消费测试并保留实际浏览器/适配器证据。完整 M5 发布门槛仍需另行完成。

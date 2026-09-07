# levels v1 夹具

[English](./README.md) | 简体中文

PR-007 的 [input.mix](./input.mix) 和 [cases.json](./cases.json)覆盖 `levels` 图契约。用例：`defaults`, `gamma`, `inverted`, `lower-clamp`, `upper-clamp`, `min-gamma`, `max-gamma`, `tiny-interval`。清单包含 8 个有效 GPU 用例和 4 个无效源文件／覆盖用例。样本为固定预期 RGBA8 哨点，显式使用 0／1 码值容差，不通过 CPU 图求值器生成。无效用例必须在获取 GPU 前失败。

[契约](../../../docs/node-contracts.zh-CN.md)定义参数／默认值／公式；[图渲染](../../../docs/graph-rendering.zh-CN.md)定义 f32／f16 精度、通道编码及证据。像素路径使用 [levels.wgsl](../../../crates/mixture-wgpu/shaders/nodes/levels.wgsl)。`material-output` 自身仅映射资源，生成的默认值使用常量 kernel。

```bash
cargo xtask test-node levels
```

此命令先在无 GPU 环境验证各夹具，再显式使用选定适配器运行该节点。渲染指南链接本地 Metal 和固定 SwiftShader 结果。像素基准仍受保护，夹具命令不会更新基准。

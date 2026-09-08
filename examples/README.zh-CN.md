# 示例

[English](./README.md) | 简体中文

PR-007 提供三个可读 `.mix v1` 文档，可经同一个 `wgpu` 执行器完成验证、编译和渲染：

| 示例 | 图结构与常用输出 | 暴露控制 |
| --- | --- | --- |
| [checker.mix](./checker.mix) | Checker → baseColor；可选通道使用版本化默认值 | `frequency` → cellsX |
| [levels.mix](./levels.mix) | Scalar → levels → roughness，另有白色 baseColor | `input`、`gamma`、输入／输出上下限 |
| [blend.mix](./blend.mix) | Checker + tint，scalar → levels → blend mask，blend → baseColor；scalar → roughness | `frequency`、`contrast`、`strength` |

```bash
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
cargo run --locked -p mixture-cli -- inspect examples/blend.mix --plan --json
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render examples/levels.mix --size 256 \
  --output baseColor,roughness --set 'gamma=2' --out ./tmp/levels --json
cargo run --locked -p mixture-cli -- render examples/blend.mix --size 256 \
  --output baseColor,roughness,normal --out ./tmp/blend --json
```

文件使用确定的 `<channel>.png` 名称。颜色输出为 sRGB，标量及编码法线输出为线性。Blend 仅请求 roughness 时裁剪颜色／levels 分支，只执行一个常量 pass。示例用于证明图执行，不宣称真实感材质质量。

独立的 `render-builtin checker`／doctor 探针不读取 `.mix`，但共享棋盘格着色器与分发路径。见[图渲染](../docs/graph-rendering.zh-CN.md)、[格式](../docs/file-format.zh-CN.md)、[契约](../docs/node-contracts.zh-CN.md)及[开发指南](../docs/development.zh-CN.md)。

[独立 Rust 消费者](./native-consumer/README.zh-CN.md)拥有单独 Cargo 工作区及输入。运行 `cargo xtask test-consumer` 检查 CPU／公开 API；`gpu-smoke` 还会执行其显式 GPU 路径，在 renderer 销毁后消费返回像素。

PR-012 为该消费者夹具增加 [CLI 进程契约测试](./native-consumer/tests/cli_contract.rs)，使用自有源码并解码 PNG。CPU 用例通过 `test-consumer` 运行；真实 GPU 及部分写入用例仍在 `gpu-smoke` 中显式执行。

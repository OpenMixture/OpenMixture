# 示例

[English](./README.md) | 简体中文

[checker.mix](./checker.mix) 是首个可执行的 `.mix v1` 验证示例：默认棋盘格连接必填的 `baseColor`，`frequency` 暴露使用默认值的 `cellsX` 参数。可选材质通道使用版本化默认值。PR-005 验证此源文件；PR-006 [编译和检查计划](../docs/render-plan.zh-CN.md)。图像素执行仍属于 PR-007。

```bash
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
```

[全部 M2 夹具](../fixtures/format/valid/all-m2.mix)连接六个初始契约。独立的 `render-builtin checker` 命令渲染 PR-004 固定探针，不读取 `.mix` 文档。

参阅[文件格式](../docs/file-format.zh-CN.md)、[节点契约](../docs/node-contracts.zh-CN.md)、[开发命令](../docs/development.zh-CN.md)和[实施计划](../INITIAL_PRS.zh-CN.md)。

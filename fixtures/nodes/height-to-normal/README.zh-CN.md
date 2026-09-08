# height-to-normal 夹具

[English](./README.md) | 简体中文

平坦／非平坦输入、零／最大强度、单像素尺寸及无效强度。实际着色器的字面量坡面探针确认矩形 UV 导数、重复边界及切线 Y 方向。

```bash
cargo xtask test-node height-to-normal
```

参阅[契约](../../../docs/node-contracts.zh-CN.md)、[用例](./cases.json)及[输入](./input.mix)。测试在 GPU 工作前验证无效用例，保留基准文件，并在 `tmp/node-tests/<backend>/` 保存实际适配器／计划证据。着色器是唯一像素实现。

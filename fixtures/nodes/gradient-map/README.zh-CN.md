# gradient-map 夹具

[English](./README.md) | 简体中文

默认渐变、端点、内部插值、非预乘 alpha 颜色和无效端点颜色。固定采样点验证线性到 sRGB 编码边界。

```bash
cargo xtask test-node gradient-map
```

参阅[契约](../../../docs/node-contracts.zh-CN.md)、[用例](./cases.json)及[输入](./input.mix)。测试在 GPU 工作前验证无效用例，保留基准文件，并在 `tmp/node-tests/<backend>/` 保存实际适配器／计划证据。着色器是唯一像素实现。

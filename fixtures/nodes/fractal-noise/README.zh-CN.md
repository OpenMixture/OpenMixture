# fractal-noise 夹具

[English](./README.md) | 简体中文

显式源种子、两种周期基底、完整 u32 边界、octave／persistence 等价性、不同种子及热缓存重复稳定性。两份 129×65 全图 RGBA 基准来自固定 SwiftShader；硬件最大允许一字节误差。

```bash
cargo xtask test-node fractal-noise
```

参阅[契约](../../../docs/node-contracts.zh-CN.md)、[用例](./cases.json)及[输入](./input.mix)。测试在 GPU 工作前验证无效用例，保留基准文件，并在 `tmp/node-tests/<backend>/` 保存实际适配器／计划证据。着色器是唯一像素实现。

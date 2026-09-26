# fractal-noise 夹具

[English](./README.md) | 简体中文

显式源种子、两种周期基底、完整 u32 边界、octave／persistence 等价性、不同种子及热缓存重复稳定性。两份 129×65 全图 RGBA 基准来自固定 SwiftShader；硬件最大允许一字节误差。

```bash
cargo xtask test-node fractal-noise
```

参阅[契约](../../../docs/node-contracts.zh-CN.md)、[用例](./cases.json)及[输入](./input.mix)。测试在 GPU 工作前验证无效用例，保留基准文件，并在 `tmp/node-tests/<backend>/` 保存实际适配器／计划证据。着色器是唯一像素实现。

## 版本 2

[NUM-01](../../../docs/stable-noise.zh-CN.md)增加 `stable.mix`、小矩形/边界用例及独立精确基线 `stable-defaults-129x65.rgba`。新基线捕获于 Windows GT 1030 Vulkan，在所有适配器（包括固定软件 CI）上以零容差检查。原 v1 基线不变。生产算术对照独立 u64/f64 运算，两个版本均保留 seed/cache/octave 不变量。

[涂漆金属验收指南](../../materials/painted-metal/README.zh-CN.md)负责所选 v2 宏观／细节／压力输入的跨周期原点探针。该探针增加精确周期采样证据，不扩大 v1 或 cellular 的验收范围。

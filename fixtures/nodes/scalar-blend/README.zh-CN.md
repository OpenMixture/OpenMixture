# Scalar 组合夹具

[English](./README.md) | 简体中文

`input.mix` 提供固定 Scalar 输入；`cases.json` 覆盖默认值、端点、相同／反向输入、范围边界、65×3 及单像素轴。生产 WGSL 探针注入有限越界样本验证钳制。运行 `cargo xtask test-node scalar-blend`。

`two-noise.mix` 组合种子 11／29、尺度 4／32，暴露 `detailWeight`。高度与法线来自同一组合场。独立原生及浏览器消费者在 1K 下测试权重 0、0.25、0.5、1，检查端点与直接输入场完全一致（含法线）、像素变化与高度平铺边界连续性。验收前固定门槛：高度范围超过 20 个 RGBA8 级别，相邻权重变化字节超过 10%，边界平均步长／内部平均步长小于 2。它们是此夹具的结构门槛，不是通用材质质量阈值。

已有材质 golden 保持不变。源码需要 ENG-04 runtime；已发布 npm 0.1.0-alpha.0 明确拒绝未知类型。[契约及兼容性](../../../docs/eng-04-scalar-blend.zh-CN.md)。

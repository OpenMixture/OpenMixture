# warp 夹具

[English](./README.md) | 简体中文

版本 1 要求 `in: Scalar` 和 `displacement: Scalar` 输入，输出 `value: Scalar`。浮点参数 `strengthX`、`strengthY` 范围均为 `[-1,1]`，默认分别为 `0.05`、`0`，表示各轴独立的带符号 UV 位移。

在每个输出像素中心，读取该输出纹素位置的位移场。令 `d=2*clamp(field,0,1)-1`，在 `uv+d*strength` 采样输入；原点在左上角，v 向下。位移场 `0.5` 为中性，`1` 应用正强度，`0` 应用负强度。正 X／Y 位移向右／下采样，使视觉图案向左／上移动。零强度或中性场直接读取输入，精确保留标量像素。位移场不是在位移后的源位置采样。

输入在 `fract(sampleUV)*dimensions-0.5` 对四个环绕纹素做双线性插值，包含负邻居。没有 sampler、mip 链或抗锯齿。周期源和周期位移场保持平铺契约，但不会修复输入已有的接缝。相邻边界像素仍是不同的像素中心采样。大强度或快速变化的位移场可能折叠图案并产生欠采样。计算使用 f32，中间存储为 f16；标量导出时将红通道复制到 RGB。节点明确只处理标量；应先扭曲高度，再派生颜色／法线。

[cases.json](./cases.json) 和 [input.mix](./input.mix) 覆盖既有噪声基准的中性场恒等输出、正负半纹素位移、垂直位移、零和最大强度、单纹素轴、标量范围端点、无效参数和缺少位移场。[constant.mix](./constant.mix) 提供精确常量哨兵。只读引用既有噪声基准，不复制或改写基准。

[字面量 GPU 探针](../../../crates/mixture-wgpu/tests/support/resampling_probe.rs) 将手工指定的半精度标量、位移场和预期值送入唯一生产 WGSL，验证中性和零强度恒等、位移钳制／极性、负强度、X／Y 双线性接缝环绕、单纹素行为和在输出坐标读取的空间变化场。完整图测试验证有意义的位移变化与精确热缓存重复性。没有 CPU 像素渲染器。

```bash
cargo xtask test-node warp
```

按 [GPU 指南](../../../docs/gpu-context.zh-CN.md) 显式选择适配器。证据写入 `tmp/node-tests/<backend>/`，包括 `warp-literal-probes.json`。参阅[契约](../../../docs/node-contracts.zh-CN.md)和唯一 [WGSL 实现](../../../crates/mixture-wgpu/shaders/nodes/warp.wgsl)。木纹材质将变换后的纹理源与单独显式设种子的周期位移场组合。

实现使用整数输出像素加局部纹素位移来计算等价坐标：环绕邻居来自 `pixel+floor(delta)`，权重为 `fract(delta)`，避免先舍入绝对 UV 再恢复纹素坐标。同一计划的全部纹理使用渲染分辨率。新增字面值探针以解析预期覆盖 1024 像素上的 2^-26 UV 位移、奇数尺寸正负位移和负整周期。本次数值修正不修改 golden 或容差。

warp 插值在每行及两行之间使用显式 `fma(b-a,t,a)`，以减少已测后端的中间舍入。WGSL 允许非融合 fma，因此这不保证跨后端逐位一致。候选仍需满足冻结材质 golden，不修改容差或基线。字面值回归覆盖双轴不同权重、正负方向跨接缝、3x2 纹理的大位移及完整周期、细长奇数高度纹理和固定的 half 舍入临界点。

显式插值候选的验证结果保留于[日期化证据](../../../docs/evidence/warp-explicit-interpolation/README.zh-CN.md)。字面值回归通过不能替代仍失败的软件材质门槛。

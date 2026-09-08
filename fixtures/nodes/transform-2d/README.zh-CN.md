# transform-2d 夹具

[English](./README.md) | 简体中文

版本 1 要求 `in: Scalar` 输入，输出 `value: Scalar`。参数：整数 `scaleX`／`scaleY` 范围 `[1,64]`，默认 `1`；整数 `quarterTurns` 范围 `[0,3]`，默认 `0`；浮点 `offsetX`／`offsetY` 范围 `[-1,1]`，默认 `0`。缩放表示每个输出 UV 图块采样的源周期数；为保持平铺契约，明确拒绝小数缩放。

以左上角为原点、v 向下，在像素中心计算 UV，令 `p=uv-0.5`。先逆向旋转：旋转次数 0–3 分别得到 `p`、`(p.y,-p.x)`、`-p`、`(-p.y,p.x)`；然后在 `rotated * scale + 0.5 + offset` 采样。`quarterTurns` 使图案及其各向异性缩放围绕图块中心顺时针旋转。旋转发生在归一化 UV 空间，矩形输出的宽高不交换。正偏移使采样点向右／下移动，视觉图案则向左／上移动。

在 `fract(sampleUV)*dimensions-0.5` 对四个环绕纹素做双线性插值；负邻居坐标在读取前环绕。没有 sampler、mip 链或抗锯齿。整数缩放和四分之一圈旋转保持周期输入的平铺契约，但不会修复输入已有的接缝。两侧边界像素是不同的像素中心采样，不要求相等。高缩放可能欠采样细节。计算使用 f32，中间存储为 f16，标量导出时将红通道复制到 RGB。默认恒等变换直接读取纹素，精确保留标量像素。

[cases.json](./cases.json) 和 [input.mix](./input.mix) 覆盖既有噪声基准的恒等输出、各向异性、顺时针旋转、小数偏移、最大缩放、负偏移、单纹素轴、标量范围端点，以及无效参数／缺少输入。[constant.mix](./constant.mix) 提供精确常量哨兵。只读引用既有噪声基准，不复制或改写基准。

[字面量 GPU 探针](../../../crates/mixture-wgpu/tests/support/resampling_probe.rs) 将手工指定的半精度行和矩形直接送入生产 WGSL。精确预期值验证正负方向、X／Y 半纹素插值、边界环绕、全部四分之一圈旋转、先旋转后缩放、整周期偏移与单纹素行为。完整图检查还验证热缓存重复性和有意义的参数变化。这些是夹具预期值，不是 CPU 变换实现。

```bash
cargo xtask test-node transform-2d
```

按 [GPU 指南](../../../docs/gpu-context.zh-CN.md) 显式选择适配器。证据写入 `tmp/node-tests/<backend>/`，包括 `transform-2d-literal-probes.json`。参阅[契约](../../../docs/node-contracts.zh-CN.md)和唯一 [WGSL 实现](../../../crates/mixture-wgpu/shaders/nodes/transform-2d.wgsl)。木纹材质是方向性消费者；该节点在颜色和法线派生前变换标量数据。

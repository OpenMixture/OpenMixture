# 砖块图案 v1

[English](./brick-pattern.md) | 简体中文

## 有界用例与归属

维护者选择生成可平铺、带砖缝和边缘过渡的砖块高度场。现有 checker 只输出颜色，没有砖缝与浮雕。本增量仅增加 Scalar 生成器 `brick-pattern@1`。Core 负责验证与降级，wgpu 负责唯一着色器。不增加随机性、crate、文件格式、解码器、编辑器或第二执行器。分支实现不等于验收、集成或发布。

## 契约

无输入；输出 `value: Scalar`，范围 [0,1]。

| 参数 | 类型 | 默认 | 合法值 |
|---|---|---|---|
| columns | Integer | 4 | 1–256 |
| rows | Integer | 8 | 1–256；半砖错行时必须为偶数 |
| layout | Enum | half-offset | aligned、half-offset |
| gap | Float | 0.08 | 0–0.5 |
| bevel | Float | 0.08 | 0–0.25 |

左上角为原点，以像素中心采样。整数除法与余数确定单元；半砖错行模式将奇数行偏移半个单元。该模式下奇数行数返回 `MIX_PARAMETER_INVALID_VALUE`，包括默认值及未使用节点的覆盖。对齐模式允许奇数行。

各轴到最近单元边界的距离以该轴单元尺寸为单位，取两轴较小值 d，令 t = d - gap/2。t <= 0 时高度为 0；否则 bevel 为 0 时高度为 1，不为 0 时为 min(t/bevel,1)。gap 表示完整砖缝宽度，bevel 从砖缝边缘向内延伸。两轴分别按单元尺寸归一化，因此矩形砖的横纵缝宽不保证像素数相同。最大参数仍保留非负平台宽度。gap、bevel 都为 0 时，恰好落在边界的样本仍为砖缝。这是无抗锯齿的点采样；亚像素缝与过渡可能消失，极小图像仅验证正确性。参数由 f64 降为 f32，中间结果显式半精度舍入，不增加跨任意硬件逐位一致保证。

两种排列均按完整 UV 图块重复；错行要求偶数行以保持纵向周期。本节点不执行纹理采样或隐式缩放。

## 兼容性

Rust 0.6.0、浏览器 0.6.0-alpha.0 是未发布候选。新增穷尽枚举 `KernelId::BrickPattern` 与 `KernelInvocation::BrickPattern`，下游匹配须检查。显式目录更新为十四种类型、十二个像素内核，保留两个噪声版本。`.mix v1`、plan v2/哈希域、API schema 2、`.mixpack v1` 不变，不迁移旧文档或 golden。旧运行时必须拒绝新类型。候选消费仅调整生产者拥有的临时宿主目录和版本预期，registry 仍单独验证已发布 0.3.0-alpha.0。

## 复现与验收

[完整图](../fixtures/nodes/brick-pattern/input.mix) 使用已有 normal、gradient-map、levels 输出高度、法线、颜色和粗糙度。默认半砖错行；覆盖 layout 为 aligned 得到对齐铺砖，覆盖 gap 为 0.2 得到宽缝砖墙。这是一个图的三个变体。

```sh
cargo xtask test-core
cargo xtask test-plan
cargo xtask shader-check
cargo xtask test-node brick-pattern
cargo run --locked -p mixture-cli -- render fixtures/nodes/brick-pattern/input.mix --size 1024 --output baseColor,height,normal,roughness --out tmp/brick-preview
cargo xtask check
```

必须验证砖缝/平台/线性过渡的字面样本、硬边、单像素轴、非方形控制、非法参数、确定性哈希、完整 2x2 像素重复、半砖行偏移以及缝宽/过渡的单调变化。Native/browser 公共消费者使用同一图和变体，以不变的 <=1 通道分量门槛比较高度/法线；检查 1K 高度/法线与平铺接触图。原始及 noise-v2 材质、六项必需检查保持有效。本地检查不自动验收归档或平台；证据与发布状态分别报告，不重置旧 golden。

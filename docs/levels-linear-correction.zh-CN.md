# gamma=1 的 levels 修正

[English](./levels-linear-correction.md) | 简体中文

状态：源码 76e8039 通过[限定的软件与 GT 1030 验收](./evidence/levels-linear-correction/README.zh-CN.md)；集成及完整 MAT-02 验收分别处理。冻结的 MAT-02 涂漆金属图在 1K、2K 未通过软件 Native／浏览器法线一致性门槛。[阶段定位](./evidence/perf-mat-numerics/README.zh-CN.md)首先在 `detail` 观察到差异：它是 gamma=1、[0,1] → [0.5,1] 映射的 `levels@1`，输入噪声阶段相同。此限定前置修正与物理纹理复用分开审查，不关闭 MAT-02，也不启动 MAT-03/04。

唯一的生产 WGSL 现在对 gamma=1 使用 `curved = t`，随后执行既有仿射输出映射。其他有效 f32 gamma 保持 `pow(t, 1/gamma)`、端点钳制及半精度转换。`pow(t,1)=t` 是已声明的数学恒等式，但近似幂运算可能让数值跨过 binary16 舍入中点。冻结的失败请求现已通过：软件 Native／浏览器全部通道完全一致，硬件法线完全一致，颜色保持在 1/255 内。

## 兼容性决定

保留 `levels@1`：修正既有 gamma=1 恒等式，没有引入不同的函数、范围、默认值或坐标约定。Core 合同／降级、内核 ABI、`.mix`／`.mixpack`、计划／API／报告版本及计划哈希不变；修正归 wgpu 所有。明确不将近似误差保留为第二个节点版本。禁止文档迁移、自动重写和 golden 重置。

新旧构建可能在舍入边界产生不同像素。[兼容性记录](./compatibility.zh-CN.md)要求像素缓存绑定实现／着色器构建；计划哈希不标识像素。未发布的 0.8 候选需要重新验收归档。已发布的 0.3 包及历史失败不变。不承诺任意 gamma、仿射边界、cellular、warp 或所有硬件的精确一致性。

## 验证

修改着色器前，原有八组 levels 用例在源码 `594279a1960e5bdc56ed05040f32b4f82880a194` 的 Windows 自带 SwiftShader 上通过。Linux [运行 35875437605](https://github.com/OpenMixture/OpenMixture/actions/runs/35875437605)再次未通过材质一致性；最初十二个字面量抽样的误差均为零。这些抽样没有复现失败，也不能证明全部舍入中点一致。

新增八组精确 [levels 用例](../fixtures/nodes/levels/cases.json)覆盖观察到 RGBA8 差异的数值区间内的 binary16 输入，包括两种舍入奇偶性。预期字节来自二进制有理数仿射结果的 binary16 最近偶数舍入及既有 RGBA8 转换。用例通过生产图执行器在奇数尺寸运行，检查 dispatch 两端样本。已有默认、反向、端点、极小区间及非单位 gamma 用例保持不变。此数值参考不是 CPU 渲染 API。

必需验证：`cargo xtask shader-check`、`cargo xtask test-node levels`、已有材质回归及 `cargo xtask check`；独立的新 Native／浏览器归档消费；涂漆金属 1K／2K 对照；原始及显式 noise-v2 材质矩阵；六项检查及记录的 Vulkan／DX12 路径。像素变化时检查留存材质前后对照。全部冻结阈值不变。留存记录证明源码 76e8039 的这些检查；后续源码和未测试平台需要各自的证据。

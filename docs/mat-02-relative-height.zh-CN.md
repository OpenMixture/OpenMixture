# MAT-02 配方修订 2——相对高度

[English](./mat-02-relative-height.md) | 简体中文

状态：候选配方补充契约，不代表材质验收。此为 MAT-02 下由仓库验收工具负责的夹具／调用方修改。Core 语义、唯一 wgpu 执行器、节点版本、`.mix`／`.mixpack`、公开 API 和源码包版本均不变。现有文档保留精确含义，不自动迁移。

修订 1 图将基底设为 0.2，涂层设为 0.2+厚度，锈层设为 0.2+起伏量。每个中间值存储为 half，因此正偏移占用了细小高度变化所需的精度。Windows GT 1030 Vulkan 的 1K 诊断捕获实际 W/R 和最终高度／法线：仅移除偏移后，高度有效值从 394 增至 4592；相对于同一遮罩的未舍入标量合成，中心邻域高度差 RMS 误差从约 4.74e-5 降至 5.98e-6。PNG 编码前原始法线分量差最大为 0.0234375。这些本地诊断测量支持候选方案，不代表外观或跨平台验收。

## 选定修正与兼容性

[新图](../fixtures/materials/painted-metal/material.mix)恰好修改三个默认值：coatingHeight.outputMin 从 0.25 改为 0.05，coatingHeight.outputMax 从 0.2 改为 0，rustHeight.value 从 0.225 改为 0.025。调用方覆盖值采用 paintHeight=paintThickness、rustHeight=rustRelief。最终高度为 mix(paintThickness*(1-W),rustRelief,R)，仍遵循现有中间 half 舍入；法线继续从同一个最终高度派生，强度不变。颜色、金属度、粗糙度、遮罩、种子、拓扑和 pass 数不变。

完整涂层／裸露／全锈端点高度分别变为 paintThickness、0、rustRelief。绝对高度及其量化法线像素会有意改变。派生法线前不加回常量。这是显式新夹具修订，不重新解释现有节点，也不静默改写已保存文档。需要原高度基准的消费者应保留原图；采用新夹具必须显式进行。

保留[修订 1 计划](../fixtures/materials/painted-metal/qualification-plan-v1.json)、[设计](../fixtures/materials/painted-metal/graph-design-v1.json)、[性能图](./evidence/perf-mat-before/material.mix)、全部旧记录和黄金基线。性能复用消费者继续渲染原图。当前涂漆材质请求携带 recipeRevision=2，绑定新图／计划的精确哈希。CPU 回归限定图只能修改三个常量，并拒绝其他冻结门槛变化。

## 所需验证

通过公开 Native 和浏览器消费者重跑全部七预设／四尺寸／五通道、精确重复／包等价、端点参考、标量合成及法线重放／周期测试、原始及迁移材质回归、冻结内存／耗时和默认降采样门槛、压力测量、PBR 对比及人工评审，并通过全部六项 CI。修订 1 的通过记录不能验收修订 2 像素。保留前后图像及各自独立的生产身份。

此修正针对实测偏移敏感性，本身不能修复宽而圆滑的磨损边界、较弱锈覆盖、遮罩量化或欠采样。不得声称这些问题已修复、降低法线强度掩盖它们，或仅凭精度指标宣布 MAT-02 已验收。MAT-03/04 继续保留在路线图中。

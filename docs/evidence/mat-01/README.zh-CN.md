# MAT-01 验收 — 在记录范围内接受

[English](./README.md) | 简体中文

本记录保留 2026-09-23 开发主机对干净源码 `7a92b82e8b60fd37bacc8304eaf72d1cfa407216` 的结果，结合后来的[人工决定](./human-decision.json)与[集成检查](./integration/README.zh-CN.md)，在下述限制内关闭 MAT-01。MAT-02～04 及整个路线图仍未完成。[PR #52](https://github.com/OpenMixture/OpenMixture/pull/52)负责验收工具。维护者已于 2026-09-23 接受提交的 PBR 视图。工具现已集成；独立 [CI／集成记录](./integration/README.zh-CN.md)绑定后来的被测源码，不改变本次 Windows 结果。未发布任何包。

## 精确候选与实测范围

- [归档回执](./archive-receipt.json)：未发布 `0.6.0-alpha.0`，SHA-256 为 `7db498db0397d7252906a56ebab25a8d924326050ec631e7996c20dec705388d`，构建为 `sha256:b987c0ba4794989e818787897855a545809ec38901c900edf83d0960c88e6f52`。[独立 Chrome 验收](./browser-qualification.json)在记录的干净源码上通过全部十七项测试，不认证更早或更晚的归档。
- 五个用例 × 四种尺寸 × 四通道：[Vulkan](./native-vulkan.json) 相对 [Chrome](./browser-matrix.json) 的八十组 RGBA8 通道比较最大分量差为 **0**；[DX12](./native-dx12.json) 相对同一 Chrome 候选的最大差为 **1**。冻结 ≤1 门槛未改变。两条 Native 路径使用记录的 NVIDIA GeForce GT 1030，不推广到其他硬件。
- 重复渲染精确一致；包往返保留源码原始字节、计划哈希及全部通道像素。所有用例使用十个 pass，峰值 **369,099,024 描述符字节**，低于 448 MiB 门槛。返回结果时每次调用的存活描述符字节为零。这是描述符计费测量，不是进程 RSS。
- Release 1K／2K 热渲染墙钟中位数（含回读）为 **Vulkan 129.42／578.17 ms**、**DX12 138.66／625.83 ms**；冷／热预算均通过，不构成公开 SLA。此前 debug 测量不作为 release 性能证据。
- 默认 256² 相对箱式降采样 1024² 的平均误差为**高度 0.2934/255**、**baseColor 最大 0.1849/255**，低于 4/255。独立布局、灰缝、倒角、高度／颜色变化及种子检查通过，法线从最终高度生成。
- [Vulkan 周期探针](./seams-vulkan.json)和 [DX12 周期探针](./seams-dx12.json)分别通过 48 组生产 WGSL 的原点移位比较，包括奇数矩形及单像素轴。原始 f16 分量有限且有界。原点注入仅存在于测试着色器副本，不进入公开节点契约。
- 记录源码的 `cargo xtask check` 已通过，包括隔离 Rust／CLI 包消费。当前 PR CI 和最终集成须单独核对，历史检查不认证新 head。

## 实测质量限制

**64×64 格渲染为 256²** 时，压力用例的高度平均降采样误差为 **14.1268/255**，baseColor 红通道误差为 **7.9928/255**，均超过默认 4/255 标准。按冻结契约，该设置处于默认质量保证之外；不静默修改门槛、golden 或输入。压力像素与 Native 矩阵一起留存。

## 视觉评审

固定 WebGPU 消费者视图使用平面及球体、1×／3× 平铺，相机、中性光照和介电 GGX 参数保持相同，不进行几何位移。它们显示已生成的贴图，不增加材质执行器。

[默认](./review/default-pbr.png) · [规则](./review/regular-pbr.png) · [错行](./review/staggered-pbr.png) · [变化](./review/varied-pbr.png) · [第二种子](./review/second-seed-pbr.png) · [总览](./review/overview.png)。

代理检查认为砖块／灰缝结构清楚、通道对齐，布局／边缘／种子变化可辨，视图中未见明显新增平铺断缝。这**仅是代理评审**。后续[人工决定](./human-decision.json)记录维护者回复，按满足本阶段目标理解，并绑定未改变的评审字节。重新生成的五张图与提交评审的图逐字节一致。[评审绑定](./review-binding.json)记录被测源码和留存文件摘要；[预览回执](./review/preview.json)记录浏览器、预览源码及输入 PNG 摘要。原代理评审绑定作为历史证据保持不变，独立人工回执记录后来的决定。

## 复现与留存

遵循[夹具指南](../../../fixtures/materials/brick-paving/README.zh-CN.md)及 [MAT-01 契约](../../mat-01-structured-materials.zh-CN.md)。构建干净归档，经 `consumer.mjs candidate` 消费，再以显式 Vulkan／DX12 和预期适配器运行 `check-brick.mjs`。`cargo xtask test-node brick-pattern` 运行周期探针，`cargo xtask test-material brick-paving` 运行 Native 矩阵，`brick-material-preview.mjs` 生成评审图。每次使用新输出目录。

Git 保留精确输入／资产、八十张浏览器通道 PNG、两个 Native 矩阵及压力 PNG、六张评审图、关键机器回执与摘要，可在普通产物过期后重新检查所选 Windows 结果的全部像素。编译产物、npm 归档字节、重复日志及未选尝试保留在忽略目录或 CI 产物中，不承诺永久外部归档或注册表发布。原有三材质金图、迁移材质矩阵及历史 v1 失败保持不变。

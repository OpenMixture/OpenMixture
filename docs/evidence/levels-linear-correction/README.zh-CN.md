# gamma=1 修正与分配对照验收

[English](./README.md) | 简体中文

留存复核：2026-09-26。实测源码为 `76e8039500ceaa849f7c3792c1712a1310ce824b`，CI 实测合并源码为 `430bea6ee5c3ece895f3b89a4542a2c24032d4a1`。[记录](./receipt.json)绑定精确归档、报告及选定图片。此处验收限定的 [gamma=1 修正](../../levels-linear-correction.zh-CN.md)和 PERF-MAT 四组请求，不验收 MAT-02 完整七预设矩阵或美术质量。发布、集成分别处理。

## 结果

- [Linux Native／浏览器](./ci-reuse-native.json)：涂漆金属全部 20 组通道对照完全一致，包括 1K／2K 法线。此前 4/255、8/255 失败保留在[原始记录](../perf-mat-numerics/README.zh-CN.md)，未放宽门槛。
- [GT 1030 Vulkan](./vulkan-native.json)、[DX12](./dx12-native.json) 对干净 [Chrome 候选](./windows-browser.json)：每后端 20 组通过。法线、高度、金属度和粗糙度完全一致，baseColor 最大差异 1/255。公开的重复／裁剪输出、包往返及销毁检查通过。两后端 Scalar／资源／砖材质对照也通过，六份摘要保留于本目录。
- 冻结的冷／热预算通过。软件 1K 冷／热中位数为 1924.52／1155.30 ms，2K 为 5958.83／5208.72 ms；Vulkan 为 231.54／191.08 ms、1013.20／840.35 ms；DX12 为 857.65／201.95 ms、1760.06／818.54 ms。这是记录适配器上的实测墙钟时间，不是通用性能承诺。
- [原始](./ci-original-materials.json)及[显式 noise-v2](./ci-noise-v2-materials.json)材质矩阵通过。实测 PR 源码的六项检查全部通过：[CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528637)、[GPU／包](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528839)、[WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528708)、[Chromium](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528870)。不把后续文档提交冒充为实测源码。

本地默认 Chromium 在浏览器测试运行前以 `spawn UNKNOWN` 启动失败；显式选择已安装 Chrome 后通过，不将该启动失败记作运行时通过。Windows 自带 SwiftShader 不替代固定 Linux golden 门槛，后者已由 CI 通过。

## 区分数值变化与分配变化

对比原 GT 1030 复用前记录，修正候选仅有一个 baseColor 像素改变一个码值，其余四通道完全一致。不能把精度修正归因于存储复用，也不覆盖原始基线。

独立对照以原复用前源码 `e7b25c4e36f6d7a2052fdba11d8cdfffad762a9d` 构建，**仅**应用[相同 levels 着色器补丁](./retain-all.patch)。记录明确给出 dirty 状态、变更文件列表和着色器哈希，不冒充干净历史构建。[release 构建日志](./retain-all-build.log)、[1K Vulkan 报告](./retain-all-render.json)和五张 `control-*.png` 保留于此。GT 1030 Vulkan 上，五个解码通道均与复用候选完全相同。对照没有修改限制或分配器。这在修正着色器下隔离了复用影响，同时保留早期实现的原始前后证据。

复现时新建该基准提交的 detached 检出，应用留存补丁，执行 `cargo build --release --locked -p mixture-cli`，再以 1024、`--output baseColor,normal,roughness,metallic,height --backend vulkan` 渲染不变的[材质](../perf-mat-before/material.mix)。将解码 PNG 与候选公开 Native default-1k 输出比较。候选／归档消费及对照使用 [PERF-MAT 命令](../../perf-mat-texture-reuse.zh-CN.md)，每次使用新证据目录。

## 审查及留存

[前后／差异对照页](./review.html)使用留存的 Native 法线图；[指标](./review-metrics.json)区分构建前后变化与跨运行时门槛。1K／2K Native 法线分别改变 98／358 个像素（最大 4／8），修正后的 Native／浏览器差异则为零。智能体检查 1K 图片后确认整体表面结构一致；这是数值审查，不是人工材质／PBR 决策。

两个精确候选归档及构建记录均保留。完整普通输出位于忽略的本地目录和 CI 产物 `10763790398`（213,836,291 字节，2026-10-23T16:12:39Z 到期）。选定报告／图片／对照像素在此绑定，不承诺未留存的重复输出永久可用。历史 v1 硬件失败、任意 gamma／warp／cellular 的限制，以及 MAT-02 尚待完成的因果、接缝、金属 PBR、人工审查要求均不变。

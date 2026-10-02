# MAT-02 验收——在记录范围内通过

[English](./README.md) | 简体中文

本记录针对干净 `main` 源码 `6b82a8a525937d8e6e4ba72972f696dee3b88d94`（[PR #77](https://github.com/OpenMixture/OpenMixture/pull/77) 的合并提交）关闭 MAT-02 分层风化（涂漆金属，[配方修订 2](../../mat-02-relative-height.zh-CN.md)）。它合并了合并后 CI 矩阵、同一源码的本地 GT 1030 Native／Chrome 矩阵，以及维护者 2026-10-02 的[人工决定](./human-decision.json)。未发布的 Rust 0.8.0／browser 0.8.0-alpha.0 候选仍未发布。MAT-03/04 尚未开始。

## 精确源码与门槛

- 合并后 `6b82a8a` 通过全部六项必需检查：三平台 CPU、固定 SwiftShader GPU／材质、WASM/npm 以及 [Chromium 材质运行 36878078015](https://github.com/OpenMixture/OpenMixture/actions/runs/36878078015)。冻结的[验证计划](../../../fixtures/materials/painted-metal/qualification-plan.json)未修改，其 SHA-256 `fddad1ca…42a4` 与每份回执一致。
- **软件：** SwiftShader Device (LLVM 10.0.0) 对 CI Chromium 的 [CI 对照](./ci-6b82a8a/comparison.json)覆盖 28 个冻结行和 5 个压力行：165 次通道比较的最大差异均为 **0**。[Native 回执](./ci-6b82a8a/native-swiftshader.json)；[浏览器候选](./ci-6b82a8a/browser-qualification.json)。
- **硬件：** 干净的本地候选（构建 `sha256:16a40a92…5678`，归档 `f5faeb14…cce8`）在 NVIDIA GeForce GT 1030 上通过全部 [21 项 Chrome 消费者测试](./gt1030/browser-qualification.json)。[Vulkan](./gt1030/native-vulkan.json) 与 [DX12](./gt1030/native-dx12.json) 对同一 [Chrome 矩阵](./gt1030/browser-matrix.json)各通过 165 次比较：160 次精确，五个底色通道（默认 1K/2K、宏观种子 1K/2K、细节种子 1K）差异 **1/255**。≤1 门槛不变。这不验收其他硬件。
- 两个宿主均检查精确重复、松散／包等价、切片高度、销毁后自有输出、对独立常量图的 60 个端点通道、十一项参数隔离和物理分配统计（14 个纹理；2K 峰值 **503,316,944** 描述符字节，低于 512 MiB；live 字节为零）。
- 各后端默认 256² 与 1024² 盒式降采样的平均误差为高度 **0.0990/255**、底色最大 **0.1860/255**，低于 4/255。
- release 计时（五次热样本，含回读），冷启动 1K／热中位 1K／2K：GT 1030 Vulkan **307 / 197 / 881 ms**，DX12 **834 / 207 / 867 ms**；SwiftShader **1457 / 892 / 4018 ms**。均在冻结的硬件（10 s / 1 s / 4 s）与软件（60 s / 20 s / 80 s）预算内。这不是公开 SLA。
- 原始 half 遮罩关系、标量合成与法线重放、最终高度法线周期边界及所选噪声输入周期性，由 `cargo xtask gpu-smoke` 运行并要求通过的 ignored `mixture-wgpu` 测试覆盖；它们在合并后 SwiftShader GPU 检查和本地 GT 1030 Vulkan 上均通过。各项范围见 [fixture 指南](../../../fixtures/materials/painted-metal/README.zh-CN.md)。

## 纹理复用（PERF-MAT）

[Vulkan](./gt1030/reuse-vulkan.json) 与 [DX12](./gt1030/reuse-dx12.json) 公开复用检查各通过 20 次 Native／Chrome 比较，计时预算匹配。保留的复用前像素早于 [gamma-one 修正](../levels-linear-correction/README.zh-CN.md)，因此同适配器对照使用修正后的 retain-all 像素：[像素对照](./gt1030/reuse-control.json)解码全部五个默认 1K Vulkan 通道，结果**精确一致**。与原始修正前记录相比，一个底色分量差 1/255，重现的是已记录的修正，而非复用效应。原始记录保持不变。

## 测得的压力限制

[压力](../../../fixtures/materials/painted-metal/README.zh-CN.md#高频与亚像素宽度压力)降采样测量在 SwiftShader 与 GT 1030 Vulkan/DX12 上数值完全相同。`high-frequency`（exposure scale 64、edge width 1）底色 256²/1024² 平均误差为 **3.439 / 5.984 / 7.811**（R/G/B），超出默认质量保证范围；高度为 0.551。`subpixel-width` 在指标内（高度 0.107，底色 ≤1.217）。选定的[压力 PNG](./gt1030/stress/) 已保留。未修改任何门槛、golden 或输入。

## 视觉评审与决定

受控的金属 GGX 消费者从 [review/inputs](./review/inputs/) 中保留的 CI SwiftShader 1K 贴图渲染七张[评审图](./review/)；[preview.json](./review/preview.json) 绑定生产构建、浏览器（Chrome 153）、适配器厂商、输入与截图哈希以及金属度着色检查。每张图包含平面／球体、1×/3× 平铺、4× 近景及通道缩略图；高度不做位移。

[默认](./review/default-pbr.png) · [完好](./review/intact-pbr.png) · [裸露](./review/exposed-pbr.png) · [锈蚀](./review/rusted-pbr.png) · [边缘锈](./review/edge-rust-pbr.png) · [宏观种子](./review/macro-seed-pbr.png) · [细节种子](./review/detail-seed-pbr.png)。

决定前，代理检查记录了清晰的磨损／裸露层次且无可见平铺接缝，但指出默认设置下锈色可见性弱、近景磨损偏圆滑／块状。维护者接受了所展示的图（"接受验收"）。[人工决定](./human-decision.json)绑定所展示的精确字节。GT 1030 Native 贴图与评审用 SwiftShader 贴图在五个通道中最多差 1/255；决定绑定评审贴图，而非每个后端的字节。

## 复现与保留

按 [fixture 指南](../../../fixtures/materials/painted-metal/README.zh-CN.md)生成请求并运行 Native 矩阵。硬件部分依次运行 `node scripts/browser-runtime/build.mjs`、使用已安装 Chrome 的 `node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh>`，再以显式 `MIXTURE_GPU_BACKEND=vulkan|dx12`、`MIXTURE_GPU_SOFTWARE=0` 及预期适配器运行 `check-painted.mjs` 与 `check-reuse.mjs`。用 `painted-material-preview.mjs` 生成评审图。每次使用新的输出目录。

Git 保留评审图及其 35 张输入贴图、选定压力 PNG、全部机器回执与[绑定](./binding.json)。完整通道 PNG 集（每个运行时 165 张）、候选归档、日志以及 CI 产物 `chromium-material-matrix`（376,181,194 字节）不保留；该产物按仓库 CI 保留期过期。回执中的哈希可识别但不能保存这些被省略的像素。

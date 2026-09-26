# 术语表

[English](./glossary.md) | 简体中文

OpenMixture 文档中常用标识符与证据术语的简短定义。每一条都链接到负责细节的文档；本页不记录当前版本或验收状态，相关内容见[发布状态](./release.zh-CN.md)和[路线图](../ROADMAP.zh-CN.md)。

## 工作标识

工作项 ID 是规划标签，不是 GitHub PR 编号。真实 PR 以链接或 `#N` 表示。

| ID | 含义 | 负责文档 |
|---|---|---|
| `PR-001`–`PR-015` | M0–M4 的历史实施批次，命名早于使用 GitHub PR | [初始实施计划](../INITIAL_PRS.zh-CN.md)、[M4 实施计划](../M4_PRS.zh-CN.md) |
| M0–M3 | 基础设施、无界面 wgpu 纵向切片、`.mix` v1 图 MVP，以及三种材质质量门槛 | [路线图](../ROADMAP.zh-CN.md) |
| M4、M4.1 | 稳定原生 SDK；仓库治理与必需检查 | [发布状态](./release.zh-CN.md)、[治理](./governance.zh-CN.md) |
| M5、`M5-01`–`M5-05` | WebAssembly 绑定、npm 运行时与浏览器验收 | [M5 实施计划](../M5_PRS.zh-CN.md)、[浏览器运行时](./browser-runtime.zh-CN.md) |
| `ALPHA-01`–`ALPHA-07` | 首个浏览器 Alpha：候选验收、必需检查与 npm 交付 | [浏览器 Alpha](./browser-alpha.zh-CN.md) |
| ENG-01–04 | Alpha 之后的工程工作项；ENG-02 以用例准入取代节点数量门槛，ENG-04 新增 `scalar-blend@1` | [路线图](../ROADMAP.zh-CN.md) |
| M6-A、`M6A-01`–`05` | 调用方提供的外部图像资源与 plan v2 | [资源契约](./m6a-resource-contract.zh-CN.md) |
| M6-B、`M6B-01`–`05` | 可移植 `.mixpack` 资产与 `mixture-asset` 编解码器 | [可移植资产](./m6b-portable-assets.zh-CN.md)、[包格式](./m6b-package-format.zh-CN.md) |
| NUM-01 | 带版本的稳定 value noise（`fractal-noise@2`，Q0.24 运算） | [稳定噪声](./stable-noise.zh-CN.md) |
| MAT-01…MAT-04 | 分阶段的材质表达增量：结构、分层风化、编织表面、可复用配方。字母后缀表示子步骤：`a` 契约，`b`/`c` 实现，`d` 验收 | [路线图](../ROADMAP.zh-CN.md) |
| PERF-MAT（`a`/`b`/`c`） | 由实测触发的成本工作：证据与契约、实现、验收 | [ADR 0009](./decisions/0009-transient-texture-reuse.zh-CN.md) |

## 引擎术语

| 术语 | 含义 |
|---|---|
| `.mix` | 带版本的 JSON 材质图，是唯一事实来源。见[文件格式](./file-format.zh-CN.md)。 |
| `.mixpack` | 规范的无压缩 USTAR 包，包含精确 `.mix` 字节与原始图像。见[包格式](./m6b-package-format.zh-CN.md)。 |
| `RenderPlan` | 仅由 Core 生成、与后端无关且哈希稳定的编译计划。见[渲染计划](./render-plan.zh-CN.md)。 |
| `PreparedRender` | 不可变计划加同步捕获的资源快照，可直接交给 wgpu。 |
| `KernelId` | 计划中一个 pass 执行的像素操作；每个都只对应一个 WGSL 文件。 |
| 节点身份 `type@version` | 节点按显式类型与版本解析；多个版本共存，含义永不改变。见[节点契约](./node-contracts.zh-CN.md)。 |
| golden（基准） | 固定软件适配器产生并已接受的预期输出。见[材质基准](./material-goldens.zh-CN.md)。 |
| 固定软件适配器 | 经源码核验的 SwiftShader Vulkan 构建，用于可复现的 GPU 检查。 |
| Studio | 独立负责的产品仓库；其固定副本仅作为 CI 测试宿主。 |

## 交付与证据术语

| 术语 | 含义 |
|---|---|
| 候选（candidate） | 验收或发布之前的源码版本及其构建归档。 |
| 集成（integration） | 通过 PR 合入 `main`，且必需检查通过。 |
| 验收（qualification） | 针对精确候选、在指定适配器上记录的机器门槛，以及必要时的人工审查。不会推广到记录范围之外。 |
| 发布（publication） | 单独明确授权的注册表发行，发布的是精确的归档字节。 |
| 六项必需检查 | 规则集中的 CPU（三个系统）、SwiftShader GPU、WASM/npm 及 Chromium 材质任务。见[治理](./governance.zh-CN.md)。 |
| 绑定源码（source-bound） | 与完整提交 SHA、工作区是否干净及相关输入哈希绑定的证据。见[证据保留](./evidence-policy.zh-CN.md)。 |
| 回执（receipt） | 将结果与其源码、运行和产物绑定的小型机器可读记录。 |
| 记录范围（recorded scope） | 结果所覆盖的精确主机、适配器、用例与版本；不声明任何其他范围。 |

# 架构

[English](./ARCHITECTURE.md) | 简体中文

**状态：** 从零实现 Mixture 的架构契约。

本文定义 Mixture 的职责、数据在系统中的流向，以及初期明确排除的设计。它是初始路线图的规范依据。

## 1. 架构目标

Mixture 通过唯一执行路径，将小型、带版本的材质图转换为纹理输出：

```text
untrusted .mix bytes
  -> parse
  -> validate
  -> normalize
  -> compile requested outputs
  -> deterministic RenderPlan
  -> wgpu compute execution
  -> texture readback
  -> caller-selected encoding
```

架构优先满足：

- 统一的语义归属；
- 每个节点只有一份像素实现；
- 原生无界面渲染优先；
- 未来浏览器 WebGPU 复用同一图运行时；
- 显式状态和可观测的失败；
- 小型、可审查且便于编码智能体安全修改的模块；
- 以材质质量证据为导向，而非追求功能数量。

架构不以编辑器功能、最大节点数量、多渲染后端或广泛引擎集成为优化目标。

## 2. 系统上下文

```text
                         +----------------------+
                         |  .mix author/tool    |
                         |  human or consumer   |
                         +----------+-----------+
                                    |
                                    | JSON bytes + overrides + requested outputs
                                    v
+-------------------+      +--------+---------+      +----------------------+
| mixture-cli       | ---> | mixture-core     | ---> | mixture-wgpu         |
| file I/O          |      | parse / validate |      | explicit GPU context |
| report formatting |      | compile / hash   |      | compute / readback   |
| PNG encoding      | <--- | RenderPlan       | <--- | metrics / outputs    |
+-------------------+      +------------------+      +----------------------+

Browser:
+-------------------+
| mixture-wasm      |  thin browser binding over the same core and wgpu crates
+-------------------+
```

使用方可以自行决定如何展示、打包或导出纹理。Mixture 核心不负责 3D 预览场景、目标引擎命名约定、ZIP 打包或编辑器状态。

## 3. 工作区边界

初始仓库包含三个产品 crate 和一个私有工具 crate。

### 3.1 `mixture-core`

`mixture-core` 是不依赖 GPU 或平台的纯 Rust 库。

负责：

- `.mix` 的版本化数据结构；
- 解码与确定性序列化辅助功能；
- 图、端口、参数和预算验证；
- 内置节点契约及版本；
- 暴露参数的覆盖验证；
- 按请求输出裁剪依赖；
- 稳定的拓扑排序；
- 编译为 `RenderPlan`；
- 执行计划哈希和非 GPU 诊断。

不负责：

- `wgpu` 类型；
- 适配器选择；
- 着色器编译；
- 纹理分配；
- 图像文件编码；
- CLI 行为；
- 浏览器绑定。

### 3.2 `mixture-wgpu`

`mixture-wgpu` 是唯一的像素执行实现。

负责：

- 显式获取 `GpuContext` 并管理其生命周期；
- 适配器／设备／队列诊断；
- 从 `KernelId` 到 WGSL 的穷尽映射；
- 计算管线构建和缓存；
- 绑定组和参数上传；
- 逻辑资源到物理纹理的映射；
- M3 确有需要时，在最后一个使用者完成后释放资源并复用兼容纹理；
- 命令提交；
- 输出回读；
- GPU 执行指标和强类型 GPU 错误。

不负责：

- `.mix` 解析；
- 图验证或修复；
- 节点默认值或公共参数范围；
- 重复的节点目录；
- 特定引擎的输出配置；
- CPU 渲染回退。

### 3.3 `mixture-cli`

`mixture-cli` 是公共库 API 之上的轻量可执行程序。

负责：

- 文件与目录 I/O；
- CLI 参数解析；
- JSON 及面向人类的报告格式化；
- PNG 编码；
- 进程退出码。

不得重新实现格式、图、编译器或渲染语义。

### 3.4 `xtask`

`xtask` 是私有的仓库自动化工具，不是产品 crate。

可以编排：

- 格式、Clippy、测试和文档检查；
- 专项节点和材质测试；
- 着色器验证；
- 基准比较及受保护的更新；
- GPU 冒烟测试；
- 可复现且可审查的夹具生成。

`xtask` 必须调用公共 API 或测试支持 API，不得成为隐藏的第二套文档或渲染实现。

### 3.5 `mixture-wasm`

M5 引入 `mixture-wasm`，作为浏览器绑定 crate。

它必须保持为轻量绑定层，负责：

- 接收 `.mix` 字节或字符串以及结构化选项；
- 调用 `mixture-core` 解析和编译；
- 调用 `mixture-wgpu` 执行浏览器 WebGPU；
- 返回原始像素、可直接用于图像的缓冲区、指标和结构化诊断；
- 根据 Rust 公共 API 生成或提供 TypeScript 声明。

不得维护 TypeScript 节点注册表、图验证器、编译器或着色器实现。

[M5 计划](./M5_PRS.zh-CN.md)选择单一浏览器分发包 `@openmixture/runtime`，在本仓库由同一次 JS 接口、声明及 WASM 构建生成。独立产品仓库消费该包，先实现 Player，后实现 Studio。产品控件、请求新鲜度、预览、文件导出及编辑器布局仍归消费者。不引入独立 SDK 仓库或额外语义执行器。

[浏览器 SDK 契约](./docs/browser-sdk.zh-CN.md)定义显式 WASM／GPU 初始化、无需 GPU 的验证／目录访问、异步完成、自有 RGBA8 输出及销毁。浏览器适配必须保持原生语义，并适配平台等待和错误交付；把原生路径编译成 WASM 不代表浏览器验收。[首个运行时实现](./docs/browser-runtime.zh-CN.md)已提供该绑定和打包加载路径。[有界 M5 浏览器验收](./docs/evidence/m5-05/README.zh-CN.md)已完成；原生 API 契约不变。当前工作遵循 [Post-Alpha 路线图](./ROADMAP.zh-CN.md)。

## 4. 依赖方向

```text
mixture-core
    ^
    |
mixture-wgpu
    ^
    |
mixture-cli

xtask -> public/test-support APIs from all three crates
mixture-wasm -> mixture-core + mixture-wgpu
```

具体规则如下：

- `mixture-core` 只依赖通用 Rust 库。
- `mixture-wgpu` 依赖 `mixture-core`。
- `mixture-cli` 依赖这两个库。
- 库不得依赖 CLI。
- 核心模块不得导入平台适配层。
- 适配层不得承载实质性的运行时逻辑。

产品 crate 之间禁止出现循环依赖。

## 5. `.mix` v1 文档模型

PR-005 在[格式指南](./docs/file-format.zh-CN.md)中实现首个严格源文件结构。文档字段为 `version`、`nodes`、`edges` 和可选 `exposedParameters`；节点要求 `id`、`type`、`version`，可选 `parameters`。端点使用 `nodeId`／`portId`，暴露绑定使用 `id`／`nodeId`／`parameterId`。ID 遵循文档中的 ASCII 语法。拒绝重复键及未声明字段，当前不允许元数据字段。这替代的是 README 的前瞻性示例，不是已发布格式。

### 5.1 唯一事实来源规则

`.mix` v1 使用 UTF-8 JSON，仅包含编译和渲染图所需的材质语义。

包含：

- 顶层格式版本；
- 具有稳定 ID、类型 ID、节点版本和参数的节点；
- 命名端口之间的有向边；
- 暴露参数的绑定；
- schema 明确允许时，可选的不影响执行的材质元数据。

不包含：

- 节点坐标或视口状态；
- 以 UI 控件表示的注释；
- 缩略图或渲染预览；
- 预设；
- 特定引擎的导出目标；
- 内嵌二进制资源；
- 子图或函数图；
- 任意用户 WGSL。

未来编辑器应单独存储布局，例如使用 `material.mix.layout.json` 或使用方自己的本地状态。

### 5.2 稳定标识符

- 节点 ID 是文档内的局部字符串。
- 节点类型 ID 使用小写 kebab-case，例如 `fractal-noise`。
- 端口和参数 ID 使用稳定的 camelCase 字符串。
- PBR 输出 ID 使用稳定的 camelCase 字符串，例如 `baseColor` 和 `ambientOcclusion`。
- 节点版本独立于顶层格式版本。

改变现有节点或字段的含义，必须修改版本并制定迁移策略。

### 5.3 端口类型与材质通道

v1 图刻意保持少量数据类型：

- `Scalar`：每像素一个逻辑值，用于遮罩、高度、粗糙度、金属度、AO 和不透明度；
- `Color`：四分量颜色数据；
- `Normal`：编码后的切线空间法线数据。

通道含义由 `material-output` 赋予，不通过大量近似的灰度／高度／粗糙度端口类型层级来表达。连接要求类型完全兼容。转换使用显式节点，例如 `gradient-map` 将 `Scalar` 转为 `Color`，`height-to-normal` 将 `Scalar` 转为 `Normal`。

v1 的 `material-output` 契约如下：

| 通道 | 端口类型 | 连接策略 | 未连接时的默认值 |
|---|---|---|---|
| `baseColor` | `Color` | 必需 | 无 |
| `normal` | `Normal` | 可选 | 中性切线空间法线 |
| `roughness` | `Scalar` | 可选 | `1.0` |
| `metallic` | `Scalar` | 可选 | `0.0` |
| `height` | `Scalar` | 可选 | `0.0` |
| `ambientOcclusion` | `Scalar` | 可选 | `1.0` |
| `opacity` | `Scalar` | 可选 | `1.0` |
| `emissive` | `Color` | 可选 | 黑色 |

渲染和检查报告必须说明：每个请求输出是来自连接，还是由文档规定的默认值提供。

### 5.4 初始限制

初始实现应执行保守的默认限制：

| 限制 | v1 默认值 |
|---|---:|
| 解码后的 JSON 字节数 | 2 MiB |
| 节点数 | 128 |
| 边数 | 512 |
| 暴露参数数 | 64 |
| 请求输出尺寸 | 每轴 2048 |
| 请求材质输出数 | 8 |
| 估算的临时 GPU 字节数 | 512 MiB |

PR-002 提供显式 `SafetyLimits` 对象，包含上述默认值和上限检查。不得隐式提高限制。每次因超限而拒绝请求，都报告配置上限和实际观测值。PR-004 检查棋盘格请求下限，PR-005 执行字节／集合限制并验证源图。PR-006 检查编译请求及估算的峰值分配；见[安全限制契约](./docs/diagnostics.zh-CN.md)。

v1 不支持内嵌资源，因此其预算为零。

## 6. 验证流水线

所有验证都在 GPU 工作之前进行。

### 6.1 解码验证

检查：

- UTF-8 和 JSON 有效；
- 解码大小受限；
- 顶层版本受支持；
- 必需字段及字段类型正确；
- 前向兼容策略没有明确允许的未知字段应被拒绝；
- 数值为有限值。

### 6.2 节点验证

检查：

- 节点 ID 唯一；
- 节点类型已知且节点版本受支持；
- 参数完整且类型正确；
- 参数范围和枚举值有效；
- 随机节点显式提供必需的种子。

### 6.3 边验证

检查：

- 源节点和目标节点已知；
- 源端口和目标端口已知；
- 方向为输出到输入；
- 端口类型兼容；
- 单输入端口最多只有一个连接；
- 边标识不重复；
- 图中没有环。

### 6.4 材质验证

检查：

- v1 中恰好有一个 `material-output` 节点；
- `baseColor` 已连接；
- 请求输出已连接，或具有文档规定的材质输出默认值；
- 暴露参数指向真实、可变的参数；
- 覆盖使用已声明的公共 ID 和有效值；
- 图预算和估算资源预算未超限。

验证应尽可能按稳定顺序返回所有能够安全、独立发现的诊断，不静默修复文档。

## 7. 节点契约模型

内置节点契约定义：

- 稳定的类型 ID 和节点版本；
- 可读名称与简短说明；
- 强类型输入和输出端口；
- 参数类型、默认值、范围和枚举值；
- 是否要求显式种子；
- 编译计划使用的 `KernelId`；
- 平铺与坐标约定；
- 精度和范围说明。

使用显式 Rust 模块和静态数据。证明存在真实、稳定的重复模式后才提出过程宏；目录规模本身不足以作为理由。

PR-005 通过小型静态模块注册六个 [M2 契约](./docs/node-contracts.zh-CN.md)，不添加像素执行器或推测性的 KernelId 桩。`ValidatedDocument` 解析版本化默认值，并提供连接／默认输入来源，不修改源文档。PR-006 实现参数覆盖及类型化 kernel 降级。PR-007 通过共享[图执行器](./docs/graph-rendering.zh-CN.md)实现穷尽 WGSL 映射及六个节点的像素夹具。固定棋盘格与图棋盘格使用同一着色器及分发路径。

### 7.1 唯一的像素实现

不维护节点像素的 CPU 镜像实现。

对于每个生成像素的 `KernelId`，`mixture-wgpu` 中恰好存在一个 WGSL 实现。Rust 定义契约和强类型调用；WGSL 定义像素计算。映射必须穷尽，使新增 `KernelId` 在缺少执行器映射时无法编译。

测试、夹具和基准材质构成 Rust 节点定义与 WGSL 内核之间的行为契约。

### 7.2 经审查的节点目录与准入

ENG-02 依据 [M3 材质评审](./docs/m3-review.zh-CN.md)及[后续远端验收](./docs/evidence/remote-ci/README.zh-CN.md)，结束 M3 前的十二节点预算。这明确替代评审中保留门槛的历史决定，不改变已接受像素或运行时语义。

当前经审查的目录保持如下：

1. `constant-scalar`
2. `constant-color`
3. `checker`
4. `levels`
5. `blend`
6. `material-output`
7. `fractal-noise`
8. `gradient-map`
9. `transform-2d`
10. `warp`
11. `height-to-normal`

节点数量既不是扩张目标，也不是永久上限。新增类型需要批准的有界引擎用例、版本／兼容性决定、契约、产生像素时唯一的 WGSL 实现、聚焦夹具、配对文档及适用的 Native／浏览器证据。已有材质回归继续必需。在同一经审查 PR 中更新[注册表测试](./crates/mixture-core/tests/registry.rs)的显式类型／版本预期，不得通过删除这些断言绕过目录审查。[路线图](./ROADMAP.zh-CN.md)和[节点工作流](./AGENTS.zh-CN.md#添加内置节点)约束准入。ENG-02 不添加节点，也不授权 ENG-04 实现。

PR-009 以 `fractal-noise`、`gradient-map` 和 `height-to-normal` 将目录增量扩展到九种节点。源结构及文档／节点版本 1 不变。噪声种子必须显式填写：`ParameterContract::default: Option<ParameterDefault>` 以 `None` 表示必填参数。未使用分支缺失种子也会导致源验证失败。类型化计划保留全部 u32 种子位；唯一 WGSL 映射拥有各新增像素公式。[节点约定](./docs/node-contracts.zh-CN.md)及[皮革夹具](./fixtures/materials/leather/README.zh-CN.md)定义坐标、精度和消费者证据。

## 8. 编译模型

编译器接收：

- 已验证的 `MaterialDocument`；
- 输出尺寸；
- 请求的材质通道；
- 暴露参数的覆盖值；
- 安全限制。

编译器生成具有确定性的 `RenderPlan`。

### 8.1 编译步骤

1. 将已验证的覆盖值应用到不可变、规范化的文档视图。
2. 解析 `material-output` 节点上的请求输出端口。
3. 反向遍历依赖，剔除无关节点。
4. 生成稳定的拓扑顺序。
5. 将每个必需节点转换为一个强类型计算通道。
6. 分配逻辑纹理资源及其描述。
7. 记录输出与资源之间的映射。
8. 估算累计和峰值资源压力。
9. 计算稳定的计划哈希。

初版编译器刻意保持简单，不做通用表达式融合、SSA 构建、全局精度推导或着色器 AST 优化。

### 8.2 RenderPlan 的结构

PR-006 实现不可变的 `RenderPlan`，通过共享引用访问内容。具体类型定义于 [plan.rs](./crates/mixture-core/src/plan.rs)，可运行的使用示例由[核心 rustdoc](./crates/mixture-core/src/lib.rs)测试。

```rust
let request = CompileRequest::default();
let plan = mixture_core::compile(&validated_document, &request)?;
let passes: &[ComputePass] = plan.passes();
let outputs: &[PlanOutput] = plan.outputs();
let estimates: &PlanEstimates = plan.estimates();
let hash: &PlanHash = plan.hash();
```

`KernelInvocation` 是携带输入资源绑定的强类型枚举。pass 同时保留源节点／默认值来源，输出将连接／默认来源映射到真实资源。可选默认值降级为常量 pass。计划版本 1 使用 rgba16float 纹理、f32 参数、8×8 工作组及不含资源池的保守分配模型。见[精确计划与哈希契约](./docs/render-plan.zh-CN.md)。

### 8.3 计划哈希

PR-006 使用 SHA-256 对带域分隔的紧凑计划主体计算哈希。仅请求依赖切片及有效参数进入哈希，不包含未使用分支／暴露元数据或策略上限。源顺序、显式默认值、无效应覆盖和请求顺序规范后相同；保留的节点 ID 有意义。

计划哈希包含以下语义输入：

- 格式版本和计划版本；
- 节点类型和节点版本；
- 规范化参数与覆盖值；
- 请求输出；
- 输出尺寸；
- 计算通道顺序；
- 逻辑资源描述。

不包含：

- 绝对文件路径；
- 时间戳；
- 适配器名称；
- 日志格式；
- 墙钟时间指标。

## 9. `wgpu` 执行模型

### 9.1 显式上下文

GPU 初始化必须显式进行：

```rust
let context = GpuContext::request(options).await?;
let mut renderer = Renderer::new(context);
let result = renderer.render(&plan).await?;
```

`GpuContext` 持有 `wgpu::Instance`、所选适配器、设备和队列。`Renderer` 持有与该上下文关联的执行缓存和可复用资源。

任何公共渲染 API 都不得初始化隐藏的全局设备。

PR-013 在每个上下文内部记录首个原生设备丢失通知，与获取快照分开。已观察到的丢失阻止后续 GPU 工作并清除该渲染器的流水线，不触发恢复。类型化 OOM／丢失诊断保留操作阶段及首个失败，受保护的回读清理保留次生 unmap 错误。失败报告包含已跟踪描述符的释放证据。见 [GPU 失败与生命周期](./docs/gpu-failures.zh-CN.md)。

PR-003 实现上下文获取，其不可变快照保持 `unverified`。PR-004 添加 `GpuContext::render_checker` 和验证型 doctor 探针：实际计算／回读及像素检查通过后为 `healthy`，显式跳过为 `unverified`，失败为 `unhealthy` 并保留准确阶段。PR-007 实现了接受不可变核心计划的共享 `Renderer`；固定棋盘格与图路径使用同一个执行器。见 [GPU 所有权](./docs/gpu-context.zh-CN.md)和[棋盘格契约](./docs/builtin-checker.zh-CN.md)。

### 9.2 仅使用无界面计算

初始渲染只使用计算管线和离屏纹理。

核心渲染器不创建：

- 窗口表面；
- 交换链配置；
- 帧循环；
- 网格、相机、灯光或 PBR 场景；
- UI 事件处理。

每个计算通道读取采样或存储输入，上传强类型参数，并写入一个存储纹理。

### 9.3 基础纹理表示

初始中间表示应优先使用一种可移植格式，预计为 `rgba16float`，同时承载标量和颜色数据。

约定：

- 标量／灰度值使用红色分量，其余分量采用规定的填充值；
- 颜色使用 RGBA；
- 法线使用文档规定的切线空间编码约定；
- 输出色彩空间元数据与中间存储分离。

只有性能分析证明格式特化能显著改善基准材质或使用方工作负载时，才引入更紧凑的格式。

### 9.4 资源生命周期

M1 和早期 M2 可以采用直观的分配方式，以保持清晰。M3 退出前，渲染器必须测量，并在需要时实现：

- 基于执行计划分析最后一个使用者；
- 释放不再使用的逻辑中间资源；
- 复用兼容的物理纹理；
- 显式保留输出资源，直到回读完成；
- 有界的管线和资源缓存。

在 2K 基准材质跟踪结果证明有需求之前，不要实现通用分配器。

### 9.5 管线缓存

管线缓存键只包含 GPU 语义输入，例如内核、着色器版本、纹理格式和相关设备能力。

缓存属于 `Renderer`，不得放在全局状态中。PR-010 以 `KernelId` 为键最多保留九条管线，每个渲染器固定设备／着色器 ABI／格式／工作组策略。显式清理及渲染器释放会释放管线，请求尺寸与参数不会扩展缓存。

### 9.6 适配器策略与回退

调用方可通过 `GpuContextOptions` 表达适配器／后端偏好。始终报告所选适配器和原生后端。

获取或执行失败时，Mixture 返回强类型错误，不切换到 CPU 渲染器或其他图运行时。

`wgpu` 可以根据显式策略在可用的原生 API 适配器之间选择。这属于统一 `wgpu` 执行模型内部的适配器选择，不是语义回退。

## 10. 结果与输出职责

渲染结果包含：

- 原始纹理输出或回读像素缓冲区；
- 逐通道尺寸、编码元数据，以及 `connected`／`default` 来源状态；
- 实际适配器／后端证据；
- 计算通道数和可用的执行耗时；
- 分配与峰值字节数指标；
- 计划哈希；
- 未使结果失效的警告。

PNG 编码初期归 CLI 或小型、无材质语义的共享辅助模块负责。核心编译器不关心文件名、ZIP 布局、Godot ORM 通道打包或浏览器下载行为。

PR-007 执行遵循 PR-006 简单分配模型，不使用资源池。[图输出编码](./docs/graph-rendering.zh-CN.md)将线性颜色 RGB 转为 sRGB，alpha 保持线性，将标量红分量复制为灰度，并使编码法线保持线性。CLI 写入逐通道 PNG，报告来源、实际适配器、计划哈希、pass、估算及缓存／耗时证据。

## 11. 确定性与数值一致性

Mixture 区分语义确定性和逐字节一致的浮点输出。

### 11.1 确定的语义输入

项目保证以下内容稳定：

- 给定版本的文档解码行为；
- 验证顺序；
- 依赖裁剪；
- 拓扑顺序；
- 种子约定；
- 相同语义输入对应的 RenderPlan 内容和哈希。

### 11.2 像素证据

在可行时，使用固定版本的软件适配器进行逐字节或极严格容差的基准比较。

不同硬件、驱动、原生 API 和浏览器 WebGPU 实现之间，可以采用基于容差的比较。跨适配器测试应测量：

- 最大和平均绝对误差；
- 超过小阈值的像素不匹配比例；
- 非有限像素；
- 输出范围；
- 平铺接缝误差；
- 有帮助时，增加特定材质的结构度量。

不要宣称所有 GPU 都能产生逐字节一致的输出。

## 12. 错误与诊断模型

对外可见的失败使用稳定的 Mixture 错误码。

PR-002 在 `mixture-core` 中实现 `Diagnostic`、`DiagnosticReport` 和原生源错误链。报告对诊断进行确定性排序，并根据严重程度推导 `ok`。源错误对象仍可通过 `std::error::Error::source` 访问，不自动序列化。准确的 JSON 字段、排序、预留错误码和共享 CLI 退出码策略见[诊断契约](./docs/diagnostics.zh-CN.md)。

示例：

```json
{
  "ok": false,
  "diagnostics": [
    {
      "code": "MIX_PORT_TYPE_MISMATCH",
      "stage": "validation",
      "severity": "error",
      "message": "Cannot connect a color output to a scalar input used as height.",
      "nodeId": "normal",
      "portId": "height",
      "evidence": {
        "sourceKind": "color",
        "targetKind": "scalar"
      },
      "suggestion": "Connect a scalar output or add an explicit supported conversion node."
    }
  ]
}
```

初始错误码族应包含：

- `MIX_PARSE_*`
- `MIX_FORMAT_*`
- `MIX_LIMIT_*`
- `MIX_NODE_*`
- `MIX_PORT_*`
- `MIX_GRAPH_*`
- `MIX_PARAMETER_*`
- `MIX_COMPILE_*`
- `MIX_GPU_ADAPTER_*`
- `MIX_GPU_DEVICE_*`
- `MIX_GPU_OUT_OF_MEMORY`
- `MIX_GPU_SHADER_*`
- `MIX_GPU_EXECUTION_*`
- `MIX_READBACK_*`
- `MIX_ENCODING_*`

保留驱动消息作为证据，但它们不是唯一的 API 契约。

## 13. 测试架构

### 13.1 核心测试

无需 GPU，覆盖：

- 解码和拒绝输入案例；
- 格式版本；
- 图中的环和端口不匹配；
- 参数范围和覆盖；
- 预算；
- 依赖裁剪；
- 稳定的计划顺序和哈希。

### 13.2 着色器检查

尽可能在无需真实适配器的情况下解析和验证每个 WGSL 模块。测试绑定和布局约定是否与 Rust 侧参数编码一致。

### 13.3 节点夹具

每个节点都包含以下专项案例：

- 默认值；
- 最小值与最大值；
- 无效输入；
- 适用时，确定性的种子行为；
- 声明支持平铺时，对应的平铺行为；
- 一个有代表性的非平凡输出。

### 13.4 基准材质

每种材质夹具包含：

```text
fixtures/materials/<id>/
├── material.mix
├── README.md
├── acceptance.json
├── variants/
├── expected/
└── reports/
```

验收包括固定软件适配器输出、具有容差的硬件检查、平铺度量、非退化性、参数因果关系和人工审查。

### 13.5 GPU 冒烟测试

独立的小型 CI 任务安装或使用固定版本的软件 GPU 栈，并验证：

- 适配器获取；
- 内置计算通道；
- 纹理回读；
- 一次图渲染；
- 结构化诊断。

非 GPU 单元测试任务中没有 GPU，不得被误报为图错误。

### 13.6 外部使用方测试

M4 退出前，独立使用方必须仅通过公共 CLI 或 Rust API 完成使用。相对源代码路径导入或使用工作区私有内部实现，不算满足要求。

## 14. 安全与信任边界

`.mix` 文件是不可信输入。

系统必须：

- 分配资源前执行解码与图预算限制；
- 拒绝非有限值和越界参数；
- 对格式错误的文档避免 panic；
- 防止 CLI 输出名称中的路径穿越；
- 限制回读和图像编码的内存分配；
- 保留源错误，同时不泄露秘密或无关环境数据；
- v1 不执行任意用户着色器源码；
- 在 CI 中锁定依赖版本。

## 15. 初期拒绝的设计

以下设计明确不纳入初始路线图。

### 多像素后端

拒绝原因：会重新引入语义漂移，并成倍增加节点维护成本。

### 将 CPU 渲染器作为正确性参照

拒绝原因：它会成为第二套像素实现。固定版本的软件 GPU 适配器可以直接验证真实 WGSL 路径。

### TypeScript 负责图运行时

拒绝原因：原生与 WebAssembly 必须共享同一个编译器和同一套节点契约。

### 通用编译器 IR 或着色器 AST

在实际性能分析证明简单的强类型计算通道无法满足基准材质要求之前，不接受这种设计。

### v1 使用二进制 `.mixar`

拒绝原因：可读 JSON 更容易被人类和智能体检查、版本化、迁移、测试和修改。打包推迟到真实资源分发需求出现之后。

### 编辑器优先开发

拒绝原因：UI 可能掩盖不稳定的引擎契约，并在渲染质量得到证明之前消耗大部分项目能力。

## 16. 架构变更流程

引入或改变以下内容时，需要架构决策记录：

- 产品 crate；
- 依赖方向；
- 公共文件格式字段或版本；
- 第二条运行时或像素执行路径；
- 全局缓存或后台执行模型；
- 任意着色器执行；
- 重大资源生命周期策略；
- 兼容性承诺。

ADR 必须包含背景、决策、备选方案、影响、迁移和验证。已接受决策改变规范规则时，还必须更新本文。

## PR-010 实测重采样实现

目录现有十一种节点与九个 kernel。标量 `transform-2d` 和 `warp` 在颜色／法线派生前使用显式循环双线性纹理读取；文档与节点版本仍为 1。新增类型化载荷见[节点契约](./docs/node-contracts.zh-CN.md)。`RenderReport.allocations` 在核心估算之外独立记录成功创建的资源描述符字节数。继续保留全部 pass 纹理的朴素生命周期；`trace-2k` 按峰值估算对三种材质的所有案例排序，再测量最大项是否符合既有 512 MiB 预算。计数不含驱动开销与 CPU 缓冲区；销毁不代表物理内存立即归还。见[开发与追踪语义](./docs/development.zh-CN.md)。不引入新 crate、格式、依赖、缓存、优化器或渲染器。

PR-014 仅在独立消费者中引入应用自有新鲜度状态：一个活跃渲染、一个可替换待执行输入及一个保留展示，替换时最多两个 CPU 输出。未添加 core 调度器、全局 worker、取消 API 或资源池。见[消费者生命周期边界](./docs/stale-results.zh-CN.md)。

PR-015 仅在仓库工具中添加本地归档验证。独立消费者与规范化公开包内容暂存于一次性外部工作区，本地版本 patch 解析到解包 crate，不解析到生产方私有模块。包许可／README／单元资源自包含，已提交锁不变，发布仍禁用。[兼容性记录](./docs/compatibility.zh-CN.md)汇总既有边界；[M4 发布状态](./docs/release.zh-CN.md)记录已完成的本地及远端验收、剩余分发决策和未测试硬件限制。

M5 浏览器启动增加实际的 `mixture-wasm` 编译边界，npm 资源留在本引擎仓库。`mixture-wgpu` 现已适配浏览器回调完成和计时；原生等待保持独立。[浏览器运行时指南](./docs/browser-runtime.zh-CN.md)标识首个已实现切片。上方图已包含实现的浏览器边界；未增加第二套编译器或像素执行器。

浏览器运行时资格采用 [ADR 0006](./docs/decisions/0006-browser-quality-gates.zh-CN.md) 定义的有界纹理一致性规则。原生固定软件金图继续作为精确回归证据；跨浏览器接近逐字节一致不是支持承诺。当前比较收据分开语义、结构、数值和历史回归判定。

## ENG-04 目录扩展

ENG-04 增加 scalar-blend v1：十二种节点映射到十个 WGSL 核心。Renderer 自有缓存现有十种 kernel 身份。既有源码语义和计划序列化不变。见[契约](./docs/eng-04-scalar-blend.zh-CN.md)。

## M6A-01 已选定资源设计 — 尚未实现

[ADR 0007](./docs/decisions/0007-external-image-resources.zh-CN.md)及[最小资源合同](./docs/m6a-resource-contract.zh-CN.md)选定既有节点参数中的逻辑资源引用、调用方提供线性 RGBA8 输入、Core 拥有不可变捕获及内容身份、wgpu 拥有上传和生命周期。集成接受设计；运行时实现仍待完成。这有限扩展 ADR 0003 对资源引用的排除，不增加内嵌资源或改变唯一像素执行器。实现将引入 image-input v1、计划／哈希 v2 和 API schema 2；上方架构说明仍是当前已实现基线，直至对应实现变更落地。

M6A-02 已实现 Core 资源语义和计划 v2。[M6A-03](./docs/m6a-03-native-resources.zh-CN.md)通过唯一 wgpu 执行器增加公开准备资源执行、每次渲染 RGBA8 上传及计数。现有十二种节点像素语义不变。浏览器图像捕获及跨平台图像资格仍待 M6A-04／05。

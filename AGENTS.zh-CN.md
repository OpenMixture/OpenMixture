# 贡献者与智能体指南

[English](./AGENTS.md) | 简体中文

本文是参与 Mixture 开发的编码智能体和贡献者的操作契约。

调整边界前阅读[架构文档](./ARCHITECTURE.zh-CN.md)，增加范围前阅读[路线图](./ROADMAP.zh-CN.md)。[初始 PR 实施计划](./INITIAL_PRS.zh-CN.md)和 [M4 实施计划](./M4_PRS.zh-CN.md)保留已完成的实施批次。[M5 实施计划](./M5_PRS.zh-CN.md)保留已完成的浏览器实施批次，[浏览器 SDK 契约](./docs/browser-sdk.zh-CN.md)定义其公开契约。当前工作遵循 Post-Alpha 路线图；区分规划、实现和已接受证据。新分支、真实 GitHub PR 及必需检查遵循[仓库治理](./docs/governance.zh-CN.md)，记录结果时遵循[证据保留](./docs/evidence-policy.zh-CN.md)。仅提交规则配置文件，不代表远端保护已启用。

## 当前实现要点

基线核对于 2026-09-22：M6-A、M6-B 和 NUM-01 已集成。[发布状态](./docs/release.zh-CN.md)负责当前发布及平台验收声明，源码清单负责构建版本。工作中的 MAT-02b 候选为未发布且尚未验收的 Rust 0.7.0 / browser 0.7.0-alpha.0；最近已验收的 MAT-01 基线为记录的软件与 GT 1030 范围内的 0.6；记录的浏览器发布版为 0.3.0-alpha.0。集成、验收、发布是不同状态，不得把历史里程碑计划当作当前待办。

- **版本边界：** `.mix` 保持 v1，`RenderPlan` 与浏览器 API schema 为 v2。计划哈希域为 `mixture-render-plan-v2\0`，包含选中外部图像的身份。图 `render` 和 `inspect --plan` 报告为 schema 2；doctor、固定 checker、asset 外层报告为 schema 1；`validate` 保持无版本外层结构。见[兼容性](./docs/compatibility.zh-CN.md)。
- **节点语义：** 十七种节点类型降级为十五种像素内核；`scalar-blend@1` 和 `image-input@1` 已实现。必须按显式类型/版本解析，`fractal-noise@1` 与 `@2` 共存。数量仅描述当前快照，不是准入目标或永久上限。
- **外部资源（M6-A）：** 调用方提供紧密排列的 `rgba8-linear` 图像字节；Core 验证身份、尺寸、预算，同步捕获选中像素并返回不可变 `PreparedRender`；wgpu 负责上传和执行。禁止隐式路径/URL 查找、PNG 解码、重采样及保留调用方可变缓冲区。见[资源契约](./docs/m6a-resource-contract.zh-CN.md)。
- **可移植资产（M6-B）：** 可选 `mixture-asset` 负责共享纯 CPU `.mixpack v1` 读写，采用严格规范化的无压缩 USTAR 子集。保留精确 `.mix` 字节和节点版本；在裁剪前验证哈希与完整资源闭包，不将归档路径提取到文件系统。包 v1 对所有 `resourceRef` 覆盖（包括相同值）返回 `MIX_PACKAGE_RESOURCE_OVERRIDE`，普通覆盖保持 Core 语义。见[格式/所有权](./docs/m6b-package-format.zh-CN.md)及 [codec](./docs/m6b-03-cpu-assets.zh-CN.md)。
- **包内存：** 默认上限为归档 67 MiB、manifest 64 KiB、计费字节缓冲区 202 MiB，调用方只能降低。计入 Rust 实际保留容量、源码/manifest 临时缓冲区、Core 选中快照及适配层副本。浏览器同时计入同步 JS 快照与 Rust 副本；异步 GPU 执行前释放传输缓冲区。这是字节缓冲区策略，不是进程 RSS 承诺。[适配规则](./docs/m6b-04-adapters.zh-CN.md)必须与可执行限制同步。
- **浏览器边界：** `mixture-wasm` 与 `packages/runtime` 是相同 Rust 逻辑的轻量绑定。导入无副作用，`loadRuntime` 和 GPU 创建显式执行；让出执行权前捕获接受的字节/选项，复制前拒绝 busy/closing 调用；散装/包渲染共享一个 busy 槽，幂等销毁后返回输出仍由调用方拥有。`inspectPackage` 仅使用 CPU，`renderPackage` 使用唯一 wgpu 执行器。
- **数值验收（NUM-01）：** 显式 v2 **value** noise 采用 Q0.24 运算。记录的 GT 1030 Vulkan/DX12 对 Chrome 资源/Scalar 高度及法线最大差为 0；冻结 v1 资源/资产夹具仍保留法线差 8/255、原门槛 ≤1/255 的失败。不得自动迁移文档、重置 golden，或将该修复泛化为 cellular、warp、任意完整图及所有硬件保证。见[稳定噪声](./docs/stable-noise.zh-CN.md)及其绑定源码的证据。
- **交付门禁：** main 必需六项检查：Linux/macOS/Windows CPU、固定 SwiftShader GPU/包消费、WASM/npm 打包、Chromium 材质验收。保留原始与显式迁移两套材质矩阵。候选归档与精确注册表消费分别留证；仅 CPU/接口/软件检查通过不认证 Windows 硬件像素。遵循[治理](./docs/governance.zh-CN.md)及[证据保留](./docs/evidence-policy.zh-CN.md)。

## 当前材质规划规则

[材质能力路线图](./ROADMAP.zh-CN.md)记录 MAT-01 砖墙／铺地砖结构已接受，当前为 MAT-02b 分层风化节点实现，随后是 MAT-03 编织表面及 MAT-04 图复用。后续能力仍是规划，不是已实现目录。每阶段实现前须冻结有界契约、目录／版本审查及验收用例；只增加选定材质证明需要的原语。PERF-MAT 从实测开销失败启动。该顺序补全表达缺口，同时保持一个活动功能增量。

[MAT-01a 契约](./docs/mat-01-structured-materials.zh-CN.md)选定 brick-pattern@1 与 scalar-mask-blend@1，并冻结[验收用例／预算](./fixtures/materials/brick-paving/qualification-plan.json)。工作候选已实现 brick-pattern 与 scalar-mask-blend，四通道砖材质夹具已在冻结范围内接受；人工决定与合并后六项检查完成 MAT-01 材质验收。[夹具指南](./fixtures/materials/brick-paving/README.zh-CN.md)记录候选 Native 矩阵工具，候选浏览器 CI 现要求二十组用例／八十通道对照。工具也检查包往返和匹配适配器的耗时预算；`test-node brick-pattern` 检查周期原点移位及原始 f16 范围；`test-material brick-paving` 运行 Native 矩阵并测量 64×64 格混叠限制。两个砖材质入口显式将 Cargo 构建输出放入 `target/native-consumer`；验收不能依赖私有 Git 排除配置或削弱干净源码检查。见 [CI 输出路径失败](./docs/evidence/mat-01/ci-failure-4bf/README.zh-CN.md)及隔离回归。固定 WebGPU PBR 评审工具仅负责消费者可视化；[Windows 候选证据](./docs/evidence/mat-01/README.zh-CN.md)留存所选完整像素及绑定源码的回执；独立[人工决定](./docs/evidence/mat-01/human-decision.json)已接受这些视图。节点与验收工具已通过 PR #50–52 集成；[集成记录](./docs/evidence/mat-01/integration/README.zh-CN.md)分别记录通过的合并前检查及合并后验证。人工决定与记录的机器门槛共同关闭本有界阶段；不能单凭集成声明接受。Core 负责语义／降级，wgpu 负责像素，适配层保持轻量；Studio 工作继续独立归属。契约因 Rust 内核穷举匹配变化而计划首次实现推进 0.6 候选；本设计本身不改变格式、节点行为、源码清单或迁移策略。各阶段须提供路线图要求的多分辨率、参数因果、接缝、PBR 视觉及 Native／浏览器公开消费证据，并通过已有材质回归和全部六项必需检查。保留历史失败，只认证实测硬件。发布继续单独处理。

[MAT-02a 选定契约](./docs/mat-02-layered-weathering.zh-CN.md)为有界涂漆金属用例选定 scalar-morphology@1 与 scalar-subtract@1，两个身份均已有工作候选实现。[减法夹具](./fixtures/nodes/scalar-subtract/README.zh-CN.md)验证必需 Scalar 输入、饱和端点及精确半精度可表示差值，使用一个保留全零的 16 字节 uniform；Core 负责契约和降级，wgpu 负责唯一像素操作。[记录的减法证据](./docs/evidence/mat-02-subtract-node/README.zh-CN.md)在 Windows Vulkan／DX12 各有 64 组 Native／浏览器精确一致用例；最终软件及集成检查仍为必需。[节点夹具](./fixtures/nodes/scalar-morphology/README.zh-CN.md)覆盖类型验证、原始半精度极值、周期支撑集合、退化尺寸、轴顺序交换及半径单调性；[记录的 Windows 公开消费证据](./docs/evidence/mat-02-morphology/README.zh-CN.md)在 Vulkan／DX12 各有 192 组 Native／浏览器精确一致用例；PR #56 已通过六项必需检查，并以 313074451cd6ddb5e5f82cce933c3d4fe3b4ed38 合并；合并后检查单独记录。Core 负责语义／降级，wgpu 负责唯一内核。[冻结设计与计划](./fixtures/materials/painted-metal/README.zh-CN.md)规定调用方控制、计划 23 pass、七预设／四尺寸／五通道、精确探针场／ABI，以及不变的 4/255 降采样、24 pass 和 512 MiB 目标。[既有输入可行性](./docs/evidence/mat-02-input-feasibility/README.zh-CN.md)及[减法反例](./docs/evidence/mat-02-subtraction/README.zh-CN.md)支持设计，不验收新节点。[真实 PERF-MAT 进入证据](./docs/evidence/perf-mat-before/README.zh-CN.md)保留 2K 拒绝（805,306,832 > 536,870,912 字节）及有效的 23 pass／五通道 1K 渲染。[ADR 0009](./docs/decisions/0009-transient-texture-reuse.zh-CN.md)选定 Core 的确定性最后使用／物理槽规划和 wgpu 的单次渲染复用，不改变内核或 pass 顺序。设计尚未实现：首次实现必须原子更新 Core 估算与执行器，选定 plan／hash／API／图报告 v3 及未发布 0.8 候选，保持 .mix／.mixpack v1 与预算不变，从源请求重编译并使缓存失效，证明同适配器像素、2K 实测成本、清理和六项检查。本设计 PR 不改变版本或运行时。首次节点实现因内核穷举变化选定 Rust 0.7.0／browser 0.7.0-alpha.0，下游穷举匹配须处理 ScalarMorphology 与 ScalarSubtract。不改变格式、自动迁移、唯一执行器或发布规则。契约已通过 [PR #55](https://github.com/OpenMixture/OpenMixture/pull/55) 集成，保留六项门槛及配对文档。

## 本指南的强制维护要求

每次重大调整**必须在同一 PR 中更新 `AGENTS.md` 和 `AGENTS.zh-CN.md`**，这是完成条件，不是可选后续工作。重大调整包括：架构/职责边界；公共 API、命令、格式、节点语义、版本或迁移策略；资源/内存/生命周期规则；必需检查或验收范围；改变工作基线的里程碑集成/发布；跨项目或交付策略。

直接更新对应的当前规则、职责地图、命令或测试条目，避免堆积互相矛盾的状态横幅。记录改了什么、为什么改、负责边界、兼容/迁移影响，以及所需验证或证据链接。详细设计留在专项指南/ADR，当前发布状态留在发布指南，绑定源码的结果留在证据记录。不得把历史失败改成通过，也不得把计划标成实现。日常重构、错别字修正和重复运行，若不改变上述操作规则，无需新增里程碑记录。

将 PR 标为就绪前，必须明确检查本指南的影响：重大调整在 PR 描述中列出更新章节和依据链接；其他工作说明未改变操作规则。代码或已接受决策与本指南冲突时，遵循下方依据优先级，并在同次修改中修正漂移。重大调整若指南陈旧或中英文不同步，任务不算完成。

## 使命

Mixture 是基于 Rust 的材质图编译器与无界面纹理渲染器，只有一条像素执行路径：`wgpu` 计算着色器。

关键路径如下：

```text
.mix -> parse -> validate -> compile -> RenderPlan -> wgpu -> texture outputs
```

优先选择能够让这条路径更正确、更可观测、对真实使用方更有用的最小改动。

## 不可破坏的约束

没有已接受的架构决策记录和明确的路线图变更，不得违反以下规则。

1. `mixture-core` 不得依赖 `wgpu`、浏览器 API、CLI 库或 UI 框架。
2. `mixture-wgpu` 是唯一的像素执行器。不得新增 CPU 渲染器或第二套 TypeScript/WebGL 渲染器。
3. 不得静默回退到其他语义执行器。应返回结构化错误，并报告所选适配器。
4. GPU 状态必须显式管理。不得引入进程级全局适配器、设备、队列、渲染器或缓存。
5. `.mix` 是材质语义的唯一事实来源。不得在 v1 文档中放入编辑器布局、缩略图、临时 UI 状态或目标引擎配置。
6. 图遍历和序列化必须具有确定性。对外可见行为不得依赖哈希表的迭代顺序。
7. 每个随机节点都必须显式指定种子。
8. 每个内置节点都必须具备契约、唯一的 WGSL 实现、专项测试夹具、文档和针对性测试。
9. 不得仅为了让失败测试通过而覆盖基准输出。
10. 生成的绑定和适配层必须保持轻量。运行时行为应放在常规的强类型 Rust 模块中。
11. 只有实际的编译、发布、运行时或依赖边界需要时，才能新增 crate。
12. 每个新增内置节点都需要批准的有界引擎用例及明确的目录／版本审查。M3 已验收，ENG-02 据此结束初期数量门槛；节点数量既不是目标，也不是永久上限。已有材质门槛继续必需。

## 依据的优先级

来源相互冲突时，按以下顺序处理：

1. 可执行测试与稳定的公共行为；
2. `ARCHITECTURE.md` 中的不可变约束；
3. 已接受的架构决策记录；
4. `ROADMAP.md` 中的范围和停止规则；
5. issue 或 PR 描述；
6. 注释和历史说明。

不要仅因为旧 Mixture 仓库存在某种偶然行为，就保留它。旧代码是研究材料，不是兼容性契约。

## 速查

当前已实现的仓库命令见[开发指南](./docs/development.zh-CN.md)。`cargo xtask check`、`fmt`、`clippy`、`test`、`test-core`、`test-format`、`test-plan`、`test-consumer`、`package-check`、`test-node <id>`、`test-material <id>`、`golden check`、受保护的 `golden update <id> --accept`、`doc`、`deps`、`links`、`trace-2k`、`shader-check` 和显式 `gpu-smoke` 目前可用。CLI 已实现纯 CPU `validate` 和 `inspect --plan`、支持 `--skip-probe` 的验证型 `doctor` 及图 `render`／`render-builtin checker`，均提供人类可读与 JSON 模式。

常用已实现命令如下：

```bash
# Fast repository checks used during normal development
cargo xtask check

# Focused checks
cargo xtask test-core
cargo xtask test-node <node-id>
cargo xtask test-material <material-id>
cargo xtask test-format
cargo xtask test-plan
cargo xtask test-consumer
cargo xtask package-check
cargo xtask shader-check
cargo xtask golden check
cargo xtask gpu-smoke

# CLI workflows
cargo run -p mixture-cli -- doctor --json
cargo run -p mixture-cli -- validate <file.mix> --json
cargo run -p mixture-cli -- inspect <file.mix> --plan --json
cargo run -p mixture-cli -- render <file.mix> --size 512 --out ./out

# Portable asset workflows (raw tightly packed rgba8-linear input)
cargo run -p mixture-cli -- asset pack material.mix --size 65x3 --image Input input.rgba --out material.mixpack --json
cargo run -p mixture-cli -- asset inspect material.mixpack --json
cargo run -p mixture-cli -- asset render material.mixpack --size 65x3 --output height --out ./out --json

# Shared codec and public browser package checks
cargo test --locked -p mixture-asset
npm test --prefix packages/runtime
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh-evidence-directory>
```

保持已引入命令的含义稳定。未来命令提案在实际存在前须标记为未实现。不得用无关命令静默替换文档中的命令。

## 仓库职责地图

下面列出当前运行时职责位置。历史 PR-001–015 的实现批次见 [INITIAL_PRS](./INITIAL_PRS.zh-CN.md) 与 [M4_PRS](./M4_PRS.zh-CN.md)，浏览器实施见 [M5_PRS](./M5_PRS.zh-CN.md)。当前资源/资产/噪声规则见上方要点；模块按实际职责引入，不创建空桩。

```text
crates/mixture-core/
  src/document.rs       .mix data model and decoding
  src/validation.rs     format, graph, parameter, and budget validation
  src/registry.rs       built-in node contracts
  src/compiler.rs       document -> RenderPlan
  src/resources.rs      image identities, resource limits and PreparedRender
  src/plan.rs           backend-neutral execution plan
  src/error.rs          stable diagnostic codes and structured errors
  src/limits.rs         explicit safety limits and measured limit failures
  src/nodes/            one small module per built-in node contract

crates/mixture-asset/
  src/archive.rs        strict byte-only USTAR loading
  src/writer.rs         deterministic package writing
  src/manifest.rs       manifest/closure validation
  src/limits.rs         explicit byte-buffer accounting

crates/mixture-wgpu/
  src/context.rs        explicit adapter/device/queue acquisition
  src/executor.rs       RenderPlan execution
  src/resources.rs      textures, buffers, and lifetime management
  src/readback.rs       GPU texture readback
  src/diagnostics.rs    adapter and execution evidence
  shaders/nodes/        one WGSL implementation per pixel kernel

crates/mixture-cli/
  src/main.rs           argument parsing and command dispatch
  src/commands/doctor.rs
  src/commands/validate.rs
  src/commands/inspect.rs
  src/commands/render.rs
  src/commands/asset.rs

crates/mixture-wasm/    thin Rust browser bindings; src/assets.rs package adapter
packages/runtime/      public ESM loader, TypeScript API and lifecycle bridge
scripts/browser-runtime/  candidate/registry and material qualification
fixtures/packages/     portable asset acceptance and rejection corpus

fixtures/nodes/         focused node input/output fixtures
fixtures/materials/     golden materials and acceptance evidence
examples/               small readable user examples
xtask/                   repository automation only
```

## 主要职责规则

### `mixture-core`

负责：

- 文档解码和版本检查；
- 图与预算验证；
- 节点 ID、端口、参数、默认值和版本；
- 按输出裁剪依赖以及稳定的拓扑排序；
- 参数覆盖验证；
- 编译为 `RenderPlan`；
- 稳定的执行计划哈希；
- 外部图像身份、资源预算、同步像素捕获及不可变 `PreparedRender`；
- 不依赖 GPU 的结构化诊断。

不得负责：

- 适配器选择；
- GPU 资源分配；
- WGSL 模块编译；
- PNG 文件 I/O；
- 命令行输出格式化；
- 浏览器或 Node.js 绑定。

### `mixture-wgpu`

负责：

- `wgpu::Instance`、适配器、设备和队列的生命周期；
- 将每个 `KernelId` 映射到唯一的 WGSL 实现；
- 管线创建与缓存；
- 计算通道编码；
- 纹理分配、复用与释放；
- 回读和执行指标；
- GPU 专用的结构化诊断。

不得负责：

- `.mix` 解析；
- 图修复；
- 节点默认值；
- 参数覆盖语义；
- 第二份节点目录；
- 特定引擎的纹理命名或打包。

### `mixture-asset`

负责纯字节包格式、确定性写入、借用/自有装载、完整资源闭包与包预算；复用 Core 的验证、资源身份与准备接口。不负责文件/网络 I/O、GPU、浏览器对象、像素执行或第二套节点语义。

### `mixture-wasm` 与 `packages/runtime`

负责薄绑定、显式加载、同步输入捕获、类型/错误传输和公开生命周期；包预算调用共享 Rust codec。不得在 JS/绑定中重新实现图语义、包解析器或渲染器。

### `mixture-cli`

只负责：

- 文件和目录 I/O；
- 命令行解析；
- 调用 `mixture-core`、`mixture-asset` 与 `mixture-wgpu` 的公共 API；
- PNG 编码以及 JSON／面向人类的报告；
- 进程退出码。

命令模块应保持轻量。如果某段逻辑在 CLI 之外也有用，或需要直接进行单元测试，应移到对应的库 crate。

## 跨项目任务边界

OpenMixture 任务负责引擎代码、公开运行时契约、包、生产者侧验收及发行文档。Studio 负责自身依赖升级、产品验收、部署和用户试用。依赖关系、共享证据或能够访问另一个检出目录，不构成接管该项目工作的授权。除非用户明确分配跨项目工作，引擎任务不得修改 Studio、执行其产品交付任务或管理其 issue／PR。

Studio 怀疑运行时存在缺陷时，应在 OpenMixture 提交上游 issue，包含精确包版本／构建身份、最小 `.mix`、请求／覆盖参数、浏览器／OS／适配器、复现步骤、预期与实际行为，以及首个结构化错误或失败的当前门槛。实现前先判断归属；引擎缺陷在本仓库复现，添加聚焦回归验证，通过引擎 PR 和已验收发行交付。在上游 issue 中反馈修复／版本及引擎验证结果，由 Studio 独立升级和验收产品。仅属于产品的问题留在 Studio。本地发现的引擎回归和引擎检查失败仍可直接触发工作，不要求 Studio issue。

现有生产者侧 CI 可以使用固定、可丢弃的 Studio 消费者作为测试宿主，但不授权修改 Studio 仓库或线上产品。保留必需检查和历史跨仓证据；它们用于验收，不代表跨项目任务所有权。

工作也可来自[路线图](./ROADMAP.zh-CN.md)中的批准里程碑、维护者定义的引擎用例和测量。外部 issue 是规划输入之一；OpenMixture 自行决定优先级与验收。这不扩大修改消费者项目的权限。

## 标准工作流程

每项任务都应：

1. 编辑前阅读相关文档和附近实现。
2. 确认负责该逻辑的 crate，避免绕过职责边界。
3. 添加或更新能够表达预期行为的最小专项测试。
4. 实施最小且完整一致的改动。
5. 先运行针对性检查。
6. 行为涉及视觉时，检查生成产物或渲染图像。
7. 宣布改动就绪前运行 `cargo xtask check`。
8. 行为或命令发生变化时，在同一 PR 中更新公共文档。
9. 按强制维护要求评估重大调整，同一 PR 更新两份 Agent Guide，并在 PR 描述中记录指南影响。

在同一 PR 中同步维护英文文档及对应的 `*.zh-CN.md` 中文版。语言和原始文档包约定见[文档维护说明](./docs/README.zh-CN.md)。

本地修改一行解析代码，不必先运行完整 GPU 矩阵。涉及 GPU 或格式的改动，也不能只运行一个狭窄的单元测试就结束。

## 任务与测试映射

| 改动区域 | 最低限度的针对性验证 |
|---|---|
| 节点注册表之前的固定内置棋盘格 | `cargo xtask shader-check`、定向 `checker`／`readback` 测试及 `cargo xtask gpu-smoke` |
| `.mix` 解码或版本管理 | `cargo xtask test-format` |
| 图验证 | `cargo xtask test-core` 加专项验证测试 |
| 编译器或计划哈希 | `cargo xtask test-plan` |
| 图像素执行前的 M2 契约（PR-005） | `cargo test --locked -p mixture-core --test registry` 和 `cargo xtask test-format` |
| 已有图像素执行器的节点契约（PR-007 起） | `cargo xtask test-node <id>` |
| WGSL 内核 | `cargo xtask shader-check` 与 `cargo xtask test-node <id>` |
| 资源生命周期或回读 | 专项 `mixture-wgpu` 测试与 `cargo xtask gpu-smoke` |
| CLI 命令 | 命令快照／集成测试；公开报告／退出码契约还运行 `test-consumer` |
| 基准材质 | `cargo xtask test-material <id>` 与视觉证据审查 |
| 仓库自动化 | 直接受影响的 `xtask` 测试与 `cargo xtask check` |
| 资源身份／准备 | `cargo xtask test-core`、`test-plan`、独立 Native／浏览器资源消费；像素变化补相应跨端比较 |
| 共享资产 codec／包格式 | `cargo test --locked -p mixture-asset`、`test-consumer`、`package-check`；像素路径变化补 `gpu-smoke` 与包跨端比较 |
| WASM／JS 生命周期或包适配 | `npm test --prefix packages/runtime`、干净 WASM/npm 构建、独立 candidate 消费及受影响资源／Scalar／资产像素比较 |
| 文档／Agent Guide | 核对源码与证据、中英文同步、`cargo xtask links`、`cargo xtask check`；Windows 上串行运行 xtask，避免重写运行中的 launcher |
| 公共 API | 下游示例或外部使用方夹具 |

完整的平台与 GPU 矩阵由 CI 负责。本地开发应优先选择最小且能够得出明确结论的命令。

浏览器命令的环境/适配器设置和跨端像素比较，分别遵循[浏览器资源](./docs/m6a-04-browser-resources.zh-CN.md)、[资产验收](./docs/m6b-05-qualification.zh-CN.md)和[稳定噪声](./docs/stable-noise.zh-CN.md)中的完整复现步骤。每次使用新的证据目录，不以接口测试代替像素验收。

## 添加内置节点

新增节点需要打通完整纵向实现，不能只添加注册表项。

必需步骤：

1. 关联批准的路线图工作项或有界引擎用例及验收标准。修改经审查的目录前明确兼容性与版本行为；下游产品工作不是前置。
2. 在 `mixture-core/src/nodes/` 下添加小型节点契约模块。
3. 定义稳定类型 ID、节点版本、端口、参数类型、默认值、范围和验证规则。
4. 添加或扩展穷尽的 `KernelId` 映射。
5. 在 `mixture-wgpu/shaders/nodes/` 下添加且仅添加一份 WGSL 实现。
6. 为默认值、边界、无效参数及至少一个非平凡案例添加专项夹具。
7. 添加智能体可读的节点文档，说明输入、输出、参数、平铺行为和已知精度限制。
8. 在固定的软件适配器上运行着色器验证和节点测试，并执行已有材质回归及相关 Native／浏览器公开消费者检查。
9. 验证节点解决了获准的材质表达用例或已记录的消费者失败。同一 PR 更新显式类型／版本目录预期，不得改为仅检查数量或由被测目录自行推导的断言。

节点能够编译不代表完成。契约、着色器、夹具、诊断和视觉证据相互一致，才算完成。

优先使用直观的静态 Rust 数据和显式匹配。目录变大本身不构成引入过程宏的理由；提出抽象前需证明存在重复模式。

## 修改 `.mix`

格式变更必须经过明确设计并进行版本管理。

必需步骤：

1. 说明变更是增量扩展、可迁移变更，还是破坏性变更。
2. 更新 `document.rs` 和验证规则。
3. 更新 `ARCHITECTURE.md` 及专项文件格式指南。
4. 添加解码、拒绝输入、往返转换和确定性序列化测试。
5. 仅通过显式迁移更新示例与夹具。
6. 路线图承诺兼容时保留旧版解码；否则返回明确的“不支持该版本”错误。

不得在不修改版本的情况下重新解释已有字段。不得让 CLI 静默修复无效图，再将不同的语义保存回文件。

## 修改着色器

着色器变更就是语义变更。

必需步骤：

1. 阅读节点契约和全部专项夹具。
2. 编辑前运行现有节点基准，并保留结果用于比较。
3. 做出最小的着色器修改。
4. GPU 执行前先进行着色器解析／验证。
5. 比较默认参数和边界参数变体。
6. 检查平铺接缝、非有限值和输出范围。
7. 基准像素变化时生成修改前／修改后对照图。
8. 在 PR 中解释新结果为何更正确。

不得在执行渲染的同一个命令中更新基准。更新路径必须要求显式接受标志，并在 CI 中禁用。

## 基准更新规则

已实现的命令如下：

```bash
cargo xtask golden check
cargo xtask golden update <material-id> --accept
```

`golden update` 必须：

- 拒绝在 CI 中运行；
- 要求 `--accept`；
- 先把新文件写入可审查的位置；
- 报告逐通道指标和变化像素比例；
- 生成修改前／修改后／差异对照图；
- 不提交文件，也不将文件加入 Git 暂存区；
- 留下机器可读的验收报告。

审查者必须能够区分正确的视觉变更和意外的基准重置。

## GPU 调试流程

GPU 执行失败时：

1. 运行 `mixture doctor --json` 并保存完整输出。
2. 记录请求的适配器策略及实际选择的适配器／后端。
3. 用最小的内置示例或节点夹具复现。
4. 区分适配器获取、设备创建、着色器编译、管线创建、执行和回读阶段。
5. 保留第一个结构化 GPU 错误，不得替换为泛化的包装错误。
6. 在固定版本软件适配器上运行，以区分硬件／驱动行为与图语义。
7. 大范围重构前，添加专项回归测试或诊断探针。

回退成功不算修复，因为 Mixture 没有其他语义渲染器。

## 错误与诊断规则

库错误必须结构化，并具有足够稳定性，供 CLI、WebAssembly 和智能体使用。

每个对外可见的失败，在适用时应提供：

- 稳定的 `code`；
- `stage`；
- 简洁的 `message`；
- 文档路径或节点／端口／参数 ID；
- 证据或实测值；
- 可执行的 `suggestion`；
- 供人工诊断使用的源错误链。

库 crate 使用强类型错误。`anyhow` 可用于 CLI 编排边界，但不能作为公共库的错误模型。

不得仅以原始驱动消息作为错误契约。应将其保留为稳定 Mixture 错误码之下的证据。

## Rust 与依赖规则

- 优先使用安全 Rust。任何 `unsafe` 块都需要安全注释、专项测试和架构理由。
- 在不可信文档或设备行为可触达的库路径中，避免 `unwrap`、`expect` 和索引引发的 panic。
- 依赖应尽量少，并具有明确用途。
- 标准库或现有依赖足够时，不新增依赖。
- 不在无关任务中顺带更新锁文件。
- 将 `Cargo.lock` 纳入版本控制，并在 CI 中使用 `--locked`。
- 将着色器解析器、图像解码器和序列化依赖视为输入边界上的安全依赖。
- 避免隐藏的后台线程和进程级全局缓存。

## 确定性规则

- 节点、边、输出、诊断和序列化执行计划采用稳定顺序。
- 计划哈希包含文档版本、节点版本、请求输出、分辨率、参数覆盖和相关资源标识。
- 计划哈希不包含适配器名称、时间戳、绝对路径或无关语义的日志字段。
- 拒绝文档参数中的 NaN 和无穷值，除非节点明确规定其含义；v1 节点不应允许这些值。
- 随机行为必须由显式整数种子和已记录的坐标约定推导。

## 性能工作

先测量，再优化。

性能改动必须从以下内容开始：

- 可复现的夹具；
- 适配器与后端证据；
- 分辨率和请求输出；
- 墙钟耗时、可用时的 GPU 耗时、估算峰值字节数和计算通道数；
- 正确性基准。

按以下顺序优先处理：

1. 裁剪依赖，使未使用输出不参与编译；
2. 在最后一个使用者完成后释放中间资源；
3. 复用兼容的物理纹理；
4. 缓存管线；
5. 减少回读和编码工作；
6. 最后才考虑计算通道融合或更复杂的编译。

不要为了单个未经测量的案例引入通用优化器、SSA、着色器 AST 或精度推导系统。

## PR 规则

- 新变更按照[仓库治理](./docs/governance.zh-CN.md)通过真实 GitHub PR 合入。里程碑工作项 ID 与 GitHub PR 编号分开记录；历史 `PR-001` 至 `PR-015` 标识实施批次。
- 每个 PR 聚焦一个架构衔接点。
- 适配层和生成的绑定保持轻量。
- 避免无关重命名和大范围格式修改。
- 在行为变更之前或同时添加测试。
- 每个 PR 必须说明刻意不纳入范围的事项。
- 大改动按依赖顺序堆叠，并保持可独立审查。
- 不绕过必需检查。
- 不合并会使文档中主要命令路径失效的 PR。
- 按[证据规则](./docs/evidence-policy.zh-CN.md)保留已接受结果及其必需审查内容；保留历史记录，普通重复输出放入 CI 产物或被忽略的本地目录。

## 完成定义

只有满足以下条件，任务才算完成：

- 职责边界保持完整；
- 针对性测试通过；
- 能得出明确结论的仓库检查通过；
- 结构化诊断仍然有效；
- 公共行为与文档一致；
- 重大调整已同步记录在两份 Agent Guide 中，包含理由、负责边界、兼容影响及验证链接；
- 视觉变更具有可审查证据；
- 没有引入隐藏回退或全局状态；
- 没有夹带范围之外的工作；
- 下一个智能体仅凭仓库文件，就能发现如何复现并验证结果。

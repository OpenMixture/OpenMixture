# 初始 PR 实施计划

[English](./INITIAL_PRS.md) | 简体中文

本文定义从零构建 Mixture 仓库的第一轮实施顺序。每个 PR 都应可独立审查、保持检查通过，范围足够小，使编码智能体能够执行而不自行扩展相邻需求。

本轮覆盖 M0 至 M3。只有这些 PR 产生真实证据后，才能确定 M4 及后续工作的范围。

## 本轮实施规则

- 按依赖顺序合入 PR。
- 优先使用堆叠分支，但在前置依赖合并后，对后续分支执行 rebase。
- 每个 PR 聚焦一个架构衔接点。
- 每个 PR 必须为引入的行为提供测试和文档。
- 每份描述都必须包含明确的**不在范围内（Out of scope）**部分。
- 不将依赖升级、格式整理或无关重构与功能工作混合。
- 使用下列符合 Conventional Commits 风格的标题，除非仓库策略选择了其他明确标准。
- 审查发现一个 PR 包含两个独立职责边界时，可以拆分。不要仅为减少数量而合并相邻 PR。

## 依赖图

```text
PR-001 Foundation
  ├── PR-002 Diagnostics contract
  └── PR-003 Explicit GpuContext
         └── PR-004 Built-in checker execution

PR-002 + PR-004
  └── PR-005 .mix v1 decoding and validation
         └── PR-006 RenderPlan and graph inspection
                └── PR-007 Graph MVP node execution
                       └── PR-008 Golden harness + ceramic
                              └── PR-009 Leather material pipeline
                                     └── PR-010 Wood material + 2K evidence
```

---

## PR-001 — `chore(repo): establish the greenfield Rust workspace`

**仓库状态：** 已实现初始基础工程。本地验证使用 `cargo xtask check`；关闭 M0 前仍需远端跨平台 CI 证据。见[开发指南](./docs/development.zh-CN.md)。

### 目标

建立清晰、如实反映实现状态的仓库，使其在没有运行时行为前就可以被克隆并验证。

### 建议分支

```text
chore/foundation
```

### 必需改动

- 添加工作区成员：
  - `crates/mixture-core`；
  - `crates/mixture-wgpu`；
  - `crates/mixture-cli`；
  - `xtask`。
- 添加能编译的最小 crate 根模块，不宣称未实现行为已经可用。
- 添加：
  - `README.md`；
  - `AGENTS.md`；
  - `ARCHITECTURE.md`；
  - `ROADMAP.md`；
  - `INITIAL_PRS.md`。
- 添加 `.cargo/config.toml`，提供仓库本地的 `cargo xtask` 别名。
- 添加 `rust-toolchain.toml`、工作区 lint 策略、格式配置，以及纳入版本控制的 `Cargo.lock`。
- 添加 `LICENSE-MIT` 和 `LICENSE-APACHE`。
- 添加以下目录及简短说明：
  - `fixtures/nodes`；
  - `fixtures/materials`；
  - `examples`；
  - `docs/decisions`。
- 添加初始 ADR：
  - Rust 负责图语义；
  - `wgpu` 是唯一的像素后端；
  - `.mix` v1 是单个可读 DAG；
  - 不进行隐式语义回退。
- 实现 `cargo xtask check`，至少运行：
  - 格式检查；
  - 将警告视为错误的 Clippy；
  - 工作区测试；
  - rustdoc 构建；
  - 基础仓库文档链接检查。
- 添加使用 `--locked` 的 Linux、macOS 和 Windows 非 GPU CI。

### 测试与验证

```bash
cargo xtask check
```

### 验收标准

- 干净克隆能够通过上述命令。
- 所有文档中的路径存在。
- 不请求 `wgpu` 适配器。
- 没有占位命令输出虚假的成功结果。
- crate 依赖方向符合 `ARCHITECTURE.md`。

### 不在范围内

- GPU 依赖和适配器获取；
- `.mix` 数据结构；
- 着色器；
- 图像编码；
- 发行版本或软件包发布。

---

## PR-002 — `feat(core): define stable diagnostics and safety limits`

**仓库状态：** 已实现公共 Rust API、JSON 快照、确定性报告、原生源错误链和七类显式上限。见[诊断契约](./docs/diagnostics.zh-CN.md)。M0 远端 CI 验收仍待完成；不宣称已经实现 GPU 或运行时 CLI 功能。

### 目标

在解析或 GPU 代码开始返回临时拼凑的错误字符串之前，建立结构化错误和限制术语。

### 建议分支

```text
feat/core-diagnostics
```

### 必需改动

- 在 `mixture-core` 中添加强类型诊断模型：
  - 稳定错误码；
  - 阶段；
  - 严重程度；
  - 消息；
  - 可选的节点／端口／参数 ID；
  - 证据映射或强类型证据；
  - 建议；
  - 面向人类报告的源错误链。
- 定义 `ARCHITECTURE.md` 中的初始错误码族。
- 定义 `SafetyLimits`，采用保守的 v1 默认值：
  - 解码字节数；
  - 节点数；
  - 边数；
  - 暴露参数数；
  - 输出尺寸；
  - 请求输出数；
  - 估算的临时资源字节数。
- 定义多条诊断的确定性排序规则。
- 添加供后续 CLI 和 WebAssembly 使用的 JSON 序列化。
- 记录退出码映射策略，不实现全部 CLI 命令。

### 测试与验证

```bash
cargo test -p mixture-core diagnostics
cargo test -p mixture-core limits
cargo xtask check
```

### 验收标准

- 诊断 JSON 具有快照测试覆盖。
- 证据顺序具有确定性。
- 限制报告同时包含配置值和观测值。
- 公共错误不依赖 `wgpu` 或 CLI 类型。

### 不在范围内

- `.mix` 解码；
- 图验证；
- 除预留稳定错误码族外的 GPU 错误实现；
- 自动修复。

---

## PR-003 — `feat(gpu): add an explicit headless GpuContext and doctor command`

**仓库状态：** 已实现显式上下文所有权、稳定获取错误、人类可读／JSON doctor 和显式 GPU 冒烟检查。本地 Apple M5／Metal 证据已通过；固定 SwiftShader CI 任务仍待远端运行。见 [GPU 上下文文档](./docs/gpu-context.zh-CN.md)。计算／回读及 `healthy` 仍属于 PR-004。

### 目标

显式获取并报告 `wgpu` 适配器／设备，不渲染，也不使用全局状态。

### 建议分支

```text
feat/gpu-context-doctor
```

### 必需改动

- 仅向 `mixture-wgpu` 添加 `wgpu` 依赖。
- 实现 `GpuContextOptions`，包含显式适配器／后端偏好字段。
- 实现 `GpuContext::request`，持有 instance、适配器、设备和队列。
- 记录：
  - 适配器名称；
  - 设备类型；
  - 后端；
  - 支持的限制；
  - 所选特性；
  - 请求的策略。
- 使用 PR-002 错误码添加强类型 GPU 诊断。
- 添加轻量 CLI `doctor` 命令，支持面向人类和 `--json` 两种输出。
- 定义初步状态结论，例如：
  - 适配器／设备获取成功，但尚未执行计算／回读探针时，返回 `unverified`；
  - 获取失败时，返回 `unhealthy`。
- 将 `healthy` 保留到 PR-004，只有能证明计算执行与回读后才使用。
- 添加独立 GPU 冒烟 CI 任务，使用固定版本软件适配器环境。
- 确保普通核心测试不需要 GPU。

### 测试与验证

```bash
cargo test -p mixture-wgpu context
cargo test -p mixture-cli doctor
cargo run -p mixture-cli -- doctor --json
cargo xtask gpu-smoke
```

### 验收标准

- `doctor --json` 报告请求与实际适配器证据。
- 缺少适配器时的行为结构化且便于处理。
- 不存在全局设备或缓存。
- 不创建渲染通道或窗口表面。

### 不在范围内

- 着色器；
- 回读；
- `.mix`；
- 回退到 CPU 或其他运行时。

---

## PR-004 — `feat(gpu): render and read back a built-in checker`

**仓库状态：** 已实现单次 rgba16float 棋盘格计算、对齐回读、RGBA8 输出、CLI PNG 编码、支持显式跳过的验证型 doctor、已审查 SwiftShader 基准，以及阶段／清理回归测试。本地 Metal 与固定 SwiftShader Vulkan 冒烟测试通过，远端 CI 仍待运行。见[棋盘格契约与证据](./docs/builtin-checker.zh-CN.md)。

### 目标

引入图的复杂性之前，证明完整的无界面计算路径可用。

### 建议分支

```text
feat/builtin-checker
```

### 必需改动

- 添加一个 WGSL 棋盘格计算着色器。
- 添加离屏 `rgba16float` 纹理创建。
- 添加最小的 uniform／参数上传。
- 添加计算管线创建和命令提交。
- 添加纹理到缓冲区回读，并处理行对齐。
- 添加向图像可用 RGBA 缓冲区的转换。
- 在 CLI 边界添加 PNG 编码。
- 添加命令：

```bash
mixture render-builtin checker --size 64 --out checker.png
```

- 添加执行报告，包含适配器、尺寸、计算通道数、可用的耗时和回读字节数。
- 添加固定版本软件适配器上的棋盘格基准。
- 为 `doctor` 增加棋盘格计算／回读探针，完整探测成功后报告 `healthy`。
- 保留显式跳过模式；跳过时报告 `unverified`，绝不能报告 `healthy`。
- 可行时添加显式清理／drop 测试。

### 测试与验证

```bash
cargo test -p mixture-wgpu checker
cargo test -p mixture-wgpu readback
cargo run -p mixture-cli -- render-builtin checker --size 64 --out ./tmp/checker.png
cargo xtask gpu-smoke
```

### 验收标准

- 棋盘格确实通过 `wgpu` 计算执行生成。
- 回读尺寸与字节数经过验证。
- 固定版本软件适配器上的输出具有确定性。
- GPU 错误保留准确的发生阶段。
- 不创建图模型，也不引入超出棋盘格所需的通用渲染器框架。

### 不在范围内

- 节点注册表；
- `.mix` 解析器；
- 资源池；
- 浏览器目标。

---

## PR-005 — `feat(format): implement strict .mix v1 decoding and graph validation`

**仓库状态：** 已实现：有界严格解码、六个版本化契约、确定性图／参数／公开绑定验证、只读已验证文档、CLI `validate` 和定向夹具。本地格式／核心／工作区检查通过，远端 CI 仍待运行。见[源文件结构与验证](./docs/file-format.zh-CN.md)。

### 目标

建立带版本的源文档，并在任何 GPU 工作之前拒绝无效输入。

### 建议分支

```text
feat/mix-v1-validation
```

### 必需改动

- 为以下内容添加 serde 数据结构：
  - 文档版本；
  - 节点；
  - 节点版本；
  - 参数；
  - 端口和边；
  - 暴露参数。
- 实现严格且有界的解码。
- 添加初始节点契约类型，不执行像素计算。
- 注册 M2 节点契约：
  - `constant-scalar`；
  - `constant-color`；
  - `checker`；
  - `levels`；
  - `blend`；
  - `material-output`。
- 实现以下验证：
  - ID 唯一；
  - 节点类型／版本已知；
  - 参数类型和范围有效；
  - 端口已知且兼容；
  - 重复边和单输入端口连接约束；
  - 环检测；
  - 恰好一个材质输出；
  - 必需的 `baseColor` 已连接；
  - 可选且未连接的材质通道使用文档规定的默认值；
  - 暴露参数目标有效；
  - 安全限制。
- 添加轻量 CLI 适配命令 `validate <file.mix> [--json]`。
- 添加有效和无效夹具。
- 添加专项文件格式文档。

### 测试与验证

```bash
cargo xtask test-format
cargo xtask test-core
cargo run -p mixture-cli -- validate examples/checker.mix --json
```

### 验收标准

- 格式错误或超预算输入无法进入 `mixture-wgpu`。
- 诊断稳定，并在适用时标明节点／端口／参数。
- 明确测试环和类型不匹配。
- 验证不修改或修复输入语义。
- 核心测试无需访问 GPU。

### 不在范围内

- RenderPlan；
- 图执行；
- 预设、布局、资源或子图；
- 旧版导入。

---

## PR-006 — `feat(core): compile validated graphs into deterministic RenderPlan values`

**仓库状态：** 已实现不可变覆盖／默认值规范化、反向裁剪、字典序拓扑排序、类型化计划／资源／输出映射、经检查的累计／峰值估算、SHA-256、CLI `inspect --plan` 和计划／哈希快照。本地 `test-plan`、`test-core` 和工作区检查通过。图像素执行仍属于 PR-007，远端 CI 仍待运行。见[计划契约与验证](./docs/render-plan.zh-CN.md)。

### 目标

定义唯一 `wgpu` 执行器所需的最小后端无关执行计划。

### 建议分支

```text
feat/render-plan
```

### 必需改动

- 添加编译请求，包含：
  - 输出尺寸；
  - 请求通道；
  - 暴露参数覆盖值；
  - 安全限制。
- 添加应用已验证覆盖值后的不可变规范化文档视图。
- 实现从输出反向裁剪依赖。
- 实现稳定拓扑排序。
- 添加强类型的：
  - `RenderPlan`；
  - `ComputePass`；
  - `KernelId`／`KernelInvocation`；
  - 逻辑资源 ID 与纹理描述；
  - 输出映射；
  - 累计／峰值估算。
- 添加稳定计划哈希。
- 添加 CLI `inspect <file.mix> --plan --json`。
- 为执行计划和哈希添加快照测试。
- 证明未请求的分支不会出现在执行计划中。

### 测试与验证

```bash
cargo xtask test-plan
cargo run -p mixture-cli -- inspect examples/checker.mix --plan --json
cargo xtask check
```

### 验收标准

- 相同语义输入生成相同计划和哈希。
- 哈希不包含路径、时间戳或适配器细节。
- 执行计划保持强类型和简单结构，不是通用着色器 AST。
- `mixture-wgpu` 仍不解析 `.mix`。

### 不在范围内

- 完整图执行；
- 优化处理阶段；
- 资源池；
- 图像资源。

---

## PR-007 — `feat(nodes): execute the six-node graph MVP through wgpu`

**仓库状态：** 已实现穷尽 kernel／WGSL 映射、固定／图共享执行器、渲染器持有的四管线缓存、图 CLI render、带编码及来源的请求通道 PNG、六组定向夹具和三个可读示例。着色器／节点／计划／工作区检查以及完整 Metal／SwiftShader 冒烟在本地通过，原像素基准未修改。远端 CI 仍待运行。见[图渲染与证据](./docs/graph-rendering.zh-CN.md)。

### 目标

将每个初始 `KernelInvocation` 映射到唯一 WGSL 实现，渲染真实 `.mix` 图，从而完成 M2。

### 建议分支

```text
feat/graph-mvp-execution
```

### 必需改动

- 实现 `KernelId` 到 WGSL 的穷尽映射。
- 为以下节点添加 WGSL 和参数编码：
  - `constant-scalar`；
  - `constant-color`；
  - `checker`；
  - `levels`；
  - `blend`；
  - 按实际需要处理 `material-output` 或输出绑定。
- 仅按执行 `RenderPlan` 的需要推广 PR-004 执行器。
- 添加简单的、由渲染器持有的管线缓存。
- 添加支持图的 CLI `render` 命令。
- 支持按请求通道回读和确定性文件命名。
- 为默认值、边界和无效值添加专项节点夹具。
- 添加着色器验证命令。
- 添加三个简短、可读的示例。

### 测试与验证

```bash
cargo xtask shader-check
cargo xtask test-node checker
cargo xtask test-node levels
cargo xtask test-node blend
cargo run -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo xtask gpu-smoke
```

### 验收标准

- `.mix -> RenderPlan -> wgpu -> PNG` 端到端可用。
- 每个 M2 节点都有唯一像素实现和专项夹具。
- 按请求输出裁剪后，实际执行的计算通道数按预期改变。
- 报告包含实际适配器和计划哈希。
- 不引入 CPU 或 TypeScript 像素实现。

### 不在范围内

- 真实感材质质量声明；
- 基准更新工具；
- 噪声、扭曲、法线；
- 2K 优化。

---

## PR-008 — `test(materials): add protected golden tooling and glazed ceramic acceptance`

**仓库状态：** 已实现受保护 `golden check`／独立受保护 `golden update`、`test-material`、验收 schema，以及 1K 釉面陶瓷默认／细格／哑光用例，没有新增节点。已保留机器证据与对照图。针对用户关于平面棋盘展示的反馈，现已补充受控 PBR 证据，将真实导出通道展示在球体和平面样板上。用户明确接受了观感对照，PR-008 本地实现及验收已完成。用户明确暂缓远端 CI，本项工作不关闭里程碑。见[材质基准](./docs/material-goldens.zh-CN.md)。

### 目标

添加更复杂节点前，使视觉正确性可以被审查。

### 建议分支

```text
test/golden-ceramic
```

### 必需改动

- 实现：
  - `cargo xtask golden check`；
  - `cargo xtask golden update <id> --accept`。
- 强制执行更新保护：
  - 拒绝 CI；
  - 要求显式接受标志；
  - 不执行 Git 暂存／提交；
  - 输出指标报告；
  - 输出修改前／修改后／差异对照图。
- 定义材质夹具布局和 `acceptance.json` schema。
- 仅使用 M2 节点添加第一种基准材质：
  - 釉面陶瓷／棋盘格；
  - baseColor、中性或简单法线、roughness，以及已支持时的 height；
  - 至少两个暴露参数变体。
- 添加平铺接缝、非有限值、范围和非退化性检查。
- 添加人工审查记录。
- 添加一份基于容差的真实硬件比较记录。

### 测试与验证

```bash
cargo xtask golden check
cargo xtask test-material glazed-ceramic
cargo xtask check
```

### 验收标准

- 普通测试命令不能替换基准。
- 材质报告准确解释发生了什么变化。
- 第一种材质同时具备机器和人工验收证据。
- 本 PR 不引入新节点。

### 不在范围内

- 噪声与由高度生成的法线；
- 资源生命周期优化；
- Web 查看器。

---

## PR-009 — `feat(materials): add the noise-to-normal pipeline and leather golden`

**仓库状态：** 本地已实现三个契约／kernel、必填 u32 种子、专项夹具、五个 pass 的 1K 皮革、三个公开参数，以及 detail 最小／默认／最大和粗颗粒用例。已记录固定 SwiftShader 与 Metal 的节点／材质／冒烟证据、首份受保护基准、空间／法线因果检查及受控 PBR 视图。皮革人工验收已获用户接受；远端 CI 暂缓，M3 保持开放。见[皮革证据](./fixtures/materials/leather/README.zh-CN.md)。

### 目标

用最少必需节点验证微表面材质行为，不扩大到无关节点。

### 建议分支

```text
feat/leather-material
```

### 必需改动

- 为以下节点添加契约和 WGSL：
  - `fractal-noise`；
  - `gradient-map`；
  - `height-to-normal`。
- `fractal-noise` 必须显式指定确定性种子。
- 记录坐标、平铺、倍频层（octave）和法线约定。
- 添加专项节点夹具，包括边界和种子案例。
- 添加类皮革基准材质，包含：
  - baseColor；
  - height；
  - normal；
  - roughness；
  - 至少三个有实际意义的暴露参数；
  - 一个代表性参数的最小值／默认值／最大值变体。
- 添加参数因果关系指标和视觉审查。
- 通过实际适配器路径验证 1K 渲染。

### 测试与验证

```bash
cargo xtask test-node fractal-noise
cargo xtask test-node gradient-map
cargo xtask test-node height-to-normal
cargo xtask test-material leather
cargo xtask golden check
cargo xtask gpu-smoke
```

### 验收标准

- 在固定版本适配器上证明种子稳定性。
- `height-to-normal` 具有文档规定的切线空间约定。
- 参数变体引发预期的通道变化。
- 皮革验收不能仅依赖直方图或非空输出检查。
- 内置节点数保持在 M3 预算之内。

### 不在范围内

- 变换／扭曲；
- 清漆、绒光（sheen）、AO、曲率或散布；
- 2K 优化，除非相关失败阻塞该材质。

---

## PR-010 — `feat(materials): add directional warp, wood golden, and 2K memory evidence`

### 目标

用方向性材质和实测高分辨率资源行为完成 M3。

### 建议分支

```text
feat/wood-and-2k-evidence
```

### 必需改动

- 为以下节点添加契约和 WGSL：
  - `transform-2d`；
  - `warp`。
- 记录环绕／平铺行为和坐标约定。
- 添加专项变换和扭曲夹具，包括边界／接缝案例。
- 添加各向异性木纹基准材质，包含：
  - 方向性结构；
  - baseColor、height、normal 和 roughness；
  - 至少三个暴露参数；
  - 接缝和方向性度量；
  - 人工视觉审查。
- 为三种基准材质添加 1024 分辨率验收。
- 为最复杂材质添加 2048 分辨率跟踪，报告：
  - 计算通道数；
  - 累计分配估算；
  - 有界峰值估算；
  - 可测量时的实际分配／复用字节数；
  - 渲染和回读时间；
  - 适配器证据。
- 首先测量朴素生命周期模型。
- 仅当实测计划超出文档预算或失败时，才实现最后使用者释放和兼容纹理复用。
- 为任何生命周期改动添加回归覆盖。
- 在仓库检查中强制执行节点数量停止规则。

### 测试与验证

```bash
cargo xtask test-node transform-2d
cargo xtask test-node warp
cargo xtask test-material wood
cargo xtask golden check
cargo xtask gpu-smoke
cargo xtask check
```

### 验收标准

- 全部 M3 基准材质验收项通过。
- 木纹材质在其声明的平铺契约下无缝。
- 2K 结果处于预算之内；否则，提供由证据支持的最小生命周期修复，使其回到预算内。
- 不引入通用优化器、SSA、计算通道融合或纹理格式矩阵。
- 内置节点数不超过十二个。

### 不在范围内

- M4 API 稳定化；
- N-API 或守护进程；
- WebAssembly；
- 节点编辑器；
- 内嵌资源。

---

## PR-010 之后

不要立即开始浏览器或编辑器工作。

先进行 M3 审查，回答：

1. 三种基准材质是否都能被人工审查者接受？
2. 不理解仓库内部实现的人，能否理解公共 `.mix` 术语？
3. 诊断是否足以让另一位开发者修复无效图和 GPU 环境问题？
4. 1K 性能是否支持使用方的交互工作流？
5. 简单生命周期模型是否能使 2K 内存占用有界？
6. 独立原生使用方实际需要哪些 API？

根据答案编写 M4 实施计划。不要照搬旧 Mixture 仓库中未经证实的 M4 任务。

## PR 描述模板

```markdown
## 目标

本 PR 建立哪一个明确结果？

## 为什么现在做

它对应当前里程碑的哪项退出标准或阻塞缺陷？

## 设计

说明职责、数据流和重要权衡。

## 证据

- 针对性测试：
- 仓库检查：
- GPU 适配器／后端：
- 视觉产物（如适用）：
- 修改前／修改后指标（如适用）：

## 风险

哪些内容可能回归，如何覆盖？

## 不在范围内

列出本 PR 刻意不做的相邻工作。
```

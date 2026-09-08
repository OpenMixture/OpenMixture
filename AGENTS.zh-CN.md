# 贡献者与智能体指南

[English](./AGENTS.md) | 简体中文

本文是参与 Mixture 开发的编码智能体和贡献者的操作契约。

调整边界前阅读[架构文档](./ARCHITECTURE.zh-CN.md)，增加范围前阅读[路线图](./ROADMAP.zh-CN.md)，实施初始仓库时阅读[初始 PR 实施计划](./INITIAL_PRS.zh-CN.md)。

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
12. 三种基准材质全部通过 M3 验收前，不得加入第十三个内置节点。

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

当前已实现的仓库命令见[开发指南](./docs/development.zh-CN.md)。`cargo xtask check`、`fmt`、`clippy`、`test`、`test-core`、`test-format`、`test-plan`、`test-node <id>`、`test-material <id>`、`golden check`、受保护的 `golden update <id> --accept`、`doc`、`deps`、`links`、`trace-2k`、`shader-check` 和显式 `gpu-smoke` 目前可用。CLI 已实现纯 CPU `validate` 和 `inspect --plan`、支持 `--skip-probe` 的验证型 `doctor` 及图 `render`／`render-builtin checker`，均提供人类可读与 JSON 模式。下面其余命令是目标接口，不得声称已经实现。

仓库计划提供以下命令：

```bash
# Fast repository checks used during normal development
cargo xtask check

# Focused checks
cargo xtask test-core
cargo xtask test-node <node-id>
cargo xtask test-material <material-id>
cargo xtask test-format
cargo xtask test-plan
cargo xtask shader-check
cargo xtask golden check
cargo xtask gpu-smoke

# CLI workflows
cargo run -p mixture-cli -- doctor --json
cargo run -p mixture-cli -- validate <file.mix> --json
cargo run -p mixture-cli -- inspect <file.mix> --plan --json
cargo run -p mixture-cli -- render <file.mix> --size 512 --out ./out
```

在第一轮实施中，命令可能直到对应 PR 落地才存在。命令一旦引入，就应保持含义稳定。不得用无关命令静默替换文档中的命令。

## 仓库职责地图

以下是计划中的运行时模块职责地图。PR-002 添加了核心诊断与限制；PR-003 添加了 `mixture-wgpu/src/context.rs`、`mixture-wgpu/src/diagnostics.rs` 和 `mixture-cli/src/commands/doctor.rs`。PR-004 添加了固定棋盘格、回读、操作错误、一个 WGSL kernel 和 CLI PNG 编排。PR-005 添加有界文档解码、静态节点契约、图验证和 CLI validate。PR-006 添加编译器规范化／裁剪、类型化计划、哈希及 CLI inspect。PR-007 添加共享执行器、每次调用的资源、kernel 映射／缓存、图 render 和全部 M2 节点夹具。PR-008 在 `xtask/src/golden/` 添加受保护材质基准工具和首个陶瓷夹具，没有新增运行时模块。PR-009 添加噪声、渐变映射和高度转法线的三个显式核心契约／WGSL kernel，以及皮革夹具。PR-010 添加 transform／warp 契约与 kernel、`mixture-wgpu/src/allocations.rs` 中的逐次渲染描述符计数，以及 `xtask/src/golden/trace.rs`。其余模块由对应实施 PR 引入，不创建空的运行时桩。现有模块和命令的链接见[开发指南](./docs/development.zh-CN.md)。

```text
crates/mixture-core/
  src/document.rs       .mix data model and decoding
  src/validation.rs     format, graph, parameter, and budget validation
  src/registry.rs       built-in node contracts
  src/compiler.rs       document -> RenderPlan
  src/plan.rs           backend-neutral execution plan
  src/error.rs          stable diagnostic codes and structured errors
  src/limits.rs         explicit safety limits and measured limit failures
  src/nodes/            one small module per built-in node contract

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

### `mixture-cli`

只负责：

- 文件和目录 I/O；
- 命令行解析；
- 调用 `mixture-core` 与 `mixture-wgpu` 的公共 API；
- PNG 编码以及 JSON／面向人类的报告；
- 进程退出码。

命令模块应保持轻量。如果某段逻辑在 CLI 之外也有用，或需要直接进行单元测试，应移到对应的库 crate。

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
| CLI 命令 | 命令快照／集成测试 |
| 基准材质 | `cargo xtask test-material <id>` 与视觉证据审查 |
| 仓库自动化 | 直接受影响的 `xtask` 测试与 `cargo xtask check` |
| 公共 API | 下游示例或外部使用方夹具 |

完整的平台与 GPU 矩阵由 CI 负责。本地开发应优先选择最小且能够得出明确结论的命令。

## 添加内置节点

新增节点需要打通完整纵向实现，不能只添加注册表项。

必需步骤：

1. 确认当前路线图和节点数量停止规则允许新增该节点。
2. 在 `mixture-core/src/nodes/` 下添加小型节点契约模块。
3. 定义稳定类型 ID、节点版本、端口、参数类型、默认值、范围和验证规则。
4. 添加或扩展穷尽的 `KernelId` 映射。
5. 在 `mixture-wgpu/shaders/nodes/` 下添加且仅添加一份 WGSL 实现。
6. 为默认值、边界、无效参数及至少一个非平凡案例添加专项夹具。
7. 添加智能体可读的节点文档，说明输入、输出、参数、平铺行为和已知精度限制。
8. 在固定版本的软件适配器上运行着色器验证和节点测试。
9. 验证该节点确实改变真实材质，或解决有记录的使用方问题。

节点能够编译不代表完成。契约、着色器、夹具、诊断和视觉证据相互一致，才算完成。

在最初十二个节点的范围内，不要引入用于节点声明的过程宏。在重复模式得到证实之前，优先使用直观的静态 Rust 数据和显式匹配。

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

计划中的命令如下：

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

- 每个 PR 聚焦一个架构衔接点。
- 适配层和生成的绑定保持轻量。
- 避免无关重命名和大范围格式修改。
- 在行为变更之前或同时添加测试。
- 每个 PR 必须说明刻意不纳入范围的事项。
- 大改动按依赖顺序堆叠，并保持可独立审查。
- 不绕过必需检查。
- 不合并会使文档中主要命令路径失效的 PR。

## 完成定义

只有满足以下条件，任务才算完成：

- 职责边界保持完整；
- 针对性测试通过；
- 能得出明确结论的仓库检查通过；
- 结构化诊断仍然有效；
- 公共行为与文档一致；
- 视觉变更具有可审查证据；
- 没有引入隐藏回退或全局状态；
- 没有夹带范围之外的工作；
- 下一个智能体仅凭仓库文件，就能发现如何复现并验证结果。

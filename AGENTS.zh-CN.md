# 贡献者与智能体指南

[English](./AGENTS.md) | 简体中文

本文是编码智能体和贡献者的操作契约，只保留适用于所有任务的规则，细节链接到负责的文档：

| 需要了解 | 负责文档 |
|---|---|
| 边界、约束、数据流 | [ARCHITECTURE.zh-CN.md](./ARCHITECTURE.zh-CN.md) |
| 优先级、当前增量、停止规则 | [ROADMAP.zh-CN.md](./ROADMAP.zh-CN.md) |
| 版本、验收范围、发布 | [发布状态](./docs/release.zh-CN.md)及源码清单 |
| 节点、`.mix`、着色器、基准、GPU、错误、性能与 PR 流程 | [智能体操作手册](./docs/agent-playbooks.zh-CN.md) |
| 里程碑与工作项 ID（MAT-02b、NUM-01、M6-A 等）及证据术语 | [术语表](./docs/glossary.zh-CN.md) |
| 分支、必需检查、CI 范围 | [治理](./docs/governance.zh-CN.md)及[证据保留](./docs/evidence-policy.zh-CN.md) |

不要把数量、版本或带日期的状态写入本指南，应链接到负责文档。[初始 PR 实施计划](./INITIAL_PRS.zh-CN.md)、[M4 实施计划](./M4_PRS.zh-CN.md)和 [M5 实施计划](./M5_PRS.zh-CN.md)是已完成的历史，不是待办。集成、验收、发布是不同状态。

## 使命

Mixture 是基于 Rust 的材质图编译器与无界面纹理渲染器，只有一条像素执行路径：`wgpu` 计算着色器。

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
12. 每个新增内置节点都需要批准的有界引擎用例及明确的目录／版本审查。节点数量既不是目标，也不是上限；已有材质门槛继续必需。

## 依据的优先级

来源相互冲突时，依次为：（1）可执行测试与稳定的公共行为；（2）`ARCHITECTURE.md` 中的约束；（3）已接受的 ADR；（4）`ROADMAP.md` 的范围和停止规则；（5）issue 或 PR 描述；（6）注释和历史说明。旧 Mixture 代码是研究材料，不是兼容性契约。

## 边界要点

- **版本：** `.mix` 与 `.mixpack` 为 v1，`RenderPlan` 与浏览器 API 为 v3。按显式节点类型/版本解析（`fractal-noise@1` 与 `@2` 共存），不得自动迁移文档。节点目录见 `registry.rs` 与[节点契约](./docs/node-contracts.zh-CN.md)，兼容性见[兼容性](./docs/compatibility.zh-CN.md)。
- **外部资源：** 调用方提供紧密排列的 `rgba8-linear` 字节；Core 验证、同步捕获并返回不可变 `PreparedRender`；wgpu 负责上传。禁止隐式路径/URL 查找、PNG 解码或重采样。见[资源契约](./docs/m6a-resource-contract.zh-CN.md)。
- **可移植资产：** `mixture-asset` 是唯一的 `.mixpack` 编解码器（规范 USTAR，纯 CPU）。裁剪前验证哈希与完整资源闭包；不提取路径；拒绝所有 `resourceRef` 覆盖。上限只能降低。见[格式](./docs/m6b-package-format.zh-CN.md)与[适配层](./docs/m6b-04-adapters.zh-CN.md)。
- **浏览器：** `mixture-wasm` 与 `packages/runtime` 保持轻量。导入无副作用，加载与 GPU 创建显式执行；让出执行权前快照输入；每个实例一个 busy 槽。见[浏览器 SDK 契约](./docs/browser-sdk.zh-CN.md)。
- **数值：** 不得重置 golden，或把实测修复泛化到其记录节点和硬件之外。见[稳定噪声](./docs/stable-noise.zh-CN.md)与有界 [gamma=1 修正](./docs/levels-linear-correction.zh-CN.md)；保留节点契约，改变的像素须绑定实现构建。
- **交付：** `main` 需要六项检查。仅文档的 PR 可跳过 GPU/浏览器任务；被跳过的检查不是验收证据。CPU 或软件检查通过不认证硬件像素。
- **证据：** 普通运行输出留在 CI 产物或被忽略的 `tmp/` 中。除非目录已列入 `docs/evidence/retention-exceptions.txt` 并在 PR 中说明理由，`cargo xtask check` 会拒绝证据区域中的原始日志、归档和单次运行目录，以及任何超过 4 MiB 的文件。见[证据保留](./docs/evidence-policy.zh-CN.md)。
- **历史附件：** 显式[归档恢复与验证](./docs/evidence/archives/README.zh-CN.md)由仓库工具负责。仅列明的快照使用维护者选定的证据 Release；保留原回执及当前 golden／人工评审图像，删除前验证已发布字节，不将归档存储视为产品发布或新资格认定。CPU 工作流执行离线恢复测试，普通构建不获取归档。

## 当前工作

遵循[路线图](./ROADMAP.zh-CN.md)的当前增量：[MAT-02 契约](./docs/mat-02-layered-weathering.zh-CN.md)下的 MAT-02b，以及按 [ADR 0009](./docs/decisions/0009-transient-texture-reuse.zh-CN.md) 进行的 PERF-MAT（[工作实现](./docs/perf-mat-texture-reuse.zh-CN.md)同时修改 Core 估算与 wgpu 执行器；完整材质验收单独处理）。每阶段实现前冻结契约与验收用例。材质验收构建输出放在 `target/native-consumer`，不得削弱干净源码检查。 调用方控制映射与公开 Native／浏览器矩阵由[涂漆金属夹具指南](./fixtures/materials/painted-metal/README.zh-CN.md)负责；保留精确图／请求／包身份、冻结像素及耗时门槛，并区分结构／PBR／人工验收门槛。 夹具指南同时负责常量参考、分辨率质量及公开参数隔离门槛，以及经同一执行器进行的仅限测试的原始 half 遮罩／合成观察及生产法线重放／周期边界检查，不能单凭跨运行时像素一致推断这些性质。

## 速查

全部已实现的仓库命令见[开发说明](./docs/development.zh-CN.md)。

```bash
cargo xtask check                      # 宣布改动就绪前运行
cargo xtask test-core | test-format | test-plan | test-consumer | package-check
cargo xtask test-node <node-id>
cargo xtask test-material <material-id>
cargo xtask shader-check
cargo xtask golden check               # golden update <id> --accept 受保护
cargo xtask gpu-smoke
cargo test --locked -p mixture-asset
npm test --prefix packages/runtime
cargo run -p mixture-cli -- doctor --json
cargo run -p mixture-cli -- validate <file.mix> --json
cargo run -p mixture-cli -- inspect <file.mix> --plan --json
cargo run -p mixture-cli -- render <file.mix> --size 512 --out ./out
cargo run -p mixture-cli -- asset inspect <file.mixpack> --json
```

保持命令含义稳定；未实现的命令提案须标注为未实现。

## 仓库地图与职责

| 路径 | 负责 | 不得负责 |
|---|---|---|
| `crates/mixture-core` | 解码、验证、节点契约（`src/nodes/`）、编译、计划哈希、资源身份、诊断 | 适配器、GPU 资源、WGSL、文件 I/O、CLI 格式化、绑定 |
| `crates/mixture-wgpu` | 上下文、每个 `KernelId` 唯一的 WGSL 内核（`shaders/nodes/`）、管线、纹理、回读、GPU 诊断 | `.mix` 解析、默认值、覆盖语义、第二份目录 |
| `crates/mixture-asset` | `.mixpack` 字节、确定性写入、闭包、包预算 | 文件/网络 I/O、GPU 状态、节点语义 |
| `crates/mixture-cli` | 参数解析、文件 I/O、PNG 编码、报告、退出码 | 可复用逻辑（应移入库） |
| `crates/mixture-wasm`、`packages/runtime` | 绑定、显式加载、输入捕获、错误传递、生命周期 | 图语义、包解析、渲染 |
| `xtask/`、`scripts/` | 仓库自动化与验收 | 运行时行为 |
| `fixtures/`、`examples/` | 节点/材质/包验收语料；小型用户示例 | — |

按实际职责引入模块，不要创建空壳。

## 跨项目边界

OpenMixture 任务负责引擎代码、运行时契约、包、生产者侧验收及发行文档。除非用户明确分配，不得修改 Studio 或管理其 issue／PR。Studio 向上游报告引擎缺陷（版本、最小 `.mix`、环境、首个结构化错误）；CI 中固定的 Studio 消费者是测试宿主，不代表所有权。

## 标准工作流程

1. 阅读相关文档、[操作手册](./docs/agent-playbooks.zh-CN.md)中对应章节及附近代码。
2. 确认负责的 crate，避免绕过职责边界。
3. 先添加最小专项测试，再做最小且一致的改动。
4. 运行下表的针对性检查，视觉变更需检查渲染输出，然后运行 `cargo xtask check`。
5. 在同一 PR 中更新公共文档及对应的 `*.zh-CN.md`。

本地修改一行解析代码，不必运行完整 GPU 矩阵；涉及 GPU 或格式的改动，也不能只跑一个狭窄的单元测试就结束。

## 任务与测试映射

| 改动区域 | 最低限度的针对性验证 |
|---|---|
| `.mix` 解码或版本管理 | `cargo xtask test-format` |
| 图验证 | `cargo xtask test-core` 加专项验证测试 |
| 编译器或计划哈希 | `cargo xtask test-plan` |
| 节点契约 | `cargo xtask test-node <id>` |
| WGSL 内核 | `cargo xtask shader-check` 与 `cargo xtask test-node <id>` |
| 资源生命周期或回读 | 专项 `mixture-wgpu` 测试与 `cargo xtask gpu-smoke` |
| CLI 命令 | 命令集成测试；公开报告／退出码契约还运行 `test-consumer` |
| 基准材质 | `cargo xtask test-material <id>` 与视觉证据审查 |
| 资源身份／准备 | `test-core`、`test-plan`、Native／浏览器资源消费；像素变化补跨端比较 |
| 资产 codec／包格式 | `cargo test --locked -p mixture-asset`、`test-consumer`、`package-check`；像素路径补 `gpu-smoke` |
| WASM／JS 生命周期或包适配 | `npm test --prefix packages/runtime`、干净 WASM/npm 构建、candidate 消费及受影响像素比较 |
| 仓库自动化或 CI | 受影响的 `xtask` 测试或工作流 lint，以及 `cargo xtask check` |
| 文档 | 核对来源、中英文同步、`cargo xtask links`；Windows 上串行运行 xtask |
| 证据记录 | `cargo xtask evidence` 与 `cargo xtask links`；归档变更另运行 `python scripts/evidence/test_restore.py`，并按[证据保留](./docs/evidence-policy.zh-CN.md)执行实际恢复与成员校验 |
| 公共 API | 下游示例或外部使用方夹具 |

完整的平台与 GPU 矩阵由 CI 负责。跨端像素复现步骤：[浏览器资源](./docs/m6a-04-browser-resources.zh-CN.md)、[资产验收](./docs/m6b-05-qualification.zh-CN.md)、[稳定噪声](./docs/stable-noise.zh-CN.md)。

## 维护本指南

重大调整须在同一 PR 中更新 `AGENTS.md` 与 `AGENTS.zh-CN.md`：职责边界、公共 API/命令/格式/节点语义/版本、生命周期规则、必需检查或验收范围、交付策略。直接修改受影响的规则并链接负责文档，不要在此添加带日期的状态横幅。每个 PR 描述都应记录对本指南的影响，或说明未改变操作规则。与代码或已接受决策不一致时，在同一次变更中修正。

## 完成定义

- 职责边界保持完整；没有隐藏回退或全局状态。
- 针对性测试与能得出明确结论的仓库检查通过；结构化诊断仍然有效。
- 公共行为、文档及中英文一致；视觉变更有可审查证据。
- 重大调整已反映在两份指南中；没有夹带范围之外的工作。
- 下一个智能体仅凭仓库文件就能复现并验证结果。

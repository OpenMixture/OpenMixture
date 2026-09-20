# Mixture

[English](./README.md) | 简体中文

> 基于 Rust 和 `wgpu` 的小型材质图编译器与无界面纹理渲染器。

**当前状态（2026-09-20）：** 原生 M4／M4.1、有界 M5、记录范围内的 Studio MVP 及普通 Windows Chrome／Edge／Firefox 验收已完成。软件包未发布。准确 Alpha 候选与 Studio 升级已通过记录范围内的验收；下一步为浏览器必需检查决策及独立发布决策。[Alpha 收口](./docs/browser-alpha.zh-CN.md)负责当前工作；不启动 M6。

**已实现：** PR-001 至 PR-015 提供十一种节点，陶瓷、皮革和[木材](./fixtures/materials/wood/README.zh-CN.md)观感均已获接受。[M3 评审](./docs/m3-review.zh-CN.md)记录 1K release 耗时及有界 2K 分配证据。[M4 计划](./M4_PRS.zh-CN.md)验证[公开 Rust 消费路径](./docs/native-sdk.zh-CN.md)、[CLI 报告及退出码](./docs/cli-contract.zh-CN.md)、[GPU 失败与清理契约](./docs/gpu-failures.zh-CN.md)、[最新结果发布](./docs/stale-results.zh-CN.md)及[隔离 Cargo 软件包消费](./docs/package-consumption.zh-CN.md)。[M4 验收](./docs/release.zh-CN.md)包含已完成的[三平台 CPU 及 Linux SwiftShader CI 门槛](./docs/evidence/remote-ci/README.zh-CN.md)。[M5 浏览器运行时](./docs/browser-runtime.zh-CN.md)已完成[记录的 macOS／Linux Chromium 材质验收](./docs/evidence/m5-05/README.zh-CN.md)，独立 [Studio](https://github.com/OpenMixture/Studio) 的 MVP 也已通过[记录的 macOS 保存文件验收](./docs/evidence/studio-qualification/README.zh-CN.md)。当前候选 CI 已独立构建并安装新包完成浏览器验收；每个交付候选仍须绑定自身的源码和归档结果。

`PR-001` 至 `PR-015` 是历史实施批次标识，不是 GitHub PR 编号。新变更遵循[仓库治理](./docs/governance.zh-CN.md)和[证据保留](./docs/evidence-policy.zh-CN.md)规则。

Mixture 的设计目标是读取带版本号的 `.mix` 材质文档，验证并编译其中的有向无环图，通过唯一的 `wgpu` 渲染器执行计算通道，返回所请求的 PBR 纹理通道。

项目从引擎和命令行工具起步，初期不构建完整的材质创作产品。

## 快速开始

安装 [rustup](https://rustup.rs/) 和当前平台的原生链接器，然后在仓库根目录运行：

```bash
cargo xtask check
cargo run --locked -p mixture-cli -- --help
```

仓库固定使用 Rust 1.98.1 / edition 2024，并包含 `Cargo.lock`。检查覆盖格式、依赖边界、Clippy、工作区及独立消费者测试、rustdoc 和本地文档链接，无需 GPU。所有已实现命令和平台前置条件见[开发指南](./docs/development.zh-CN.md)。

核心现已提供[诊断与安全限制 API](./docs/diagnostics.zh-CN.md)，包含强类型错误、确定性 JSON 报告和七类显式资源上限。可通过 `cargo run --locked -p mixture-core --example diagnostics` 运行公共示例。

[GPU 上下文与 doctor 指南](./docs/gpu-context.zh-CN.md)说明适配器选择、结构化失败、退出码和显式 `cargo xtask gpu-smoke` 检查。运行 `cargo run --locked -p mixture-cli -- doctor --json` 检查本机环境。

可运行[内置棋盘格](./docs/builtin-checker.zh-CN.md)：`cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png`。Doctor 默认运行计算／回读探针；仅获取上下文时使用 `--skip-probe`。

无需 GPU 即可验证[棋盘格文档](./examples/checker.mix)：`cargo run --locked -p mixture-cli -- validate examples/checker.mix --json`。参阅[严格文件格式](./docs/file-format.zh-CN.md)和[十一个节点契约](./docs/node-contracts.zh-CN.md)。

## 使命

Mixture 致力于让以下路径可靠且易于嵌入其他应用：

```text
.mix JSON
  -> parse
  -> validate
  -> compile
  -> RenderPlan
  -> wgpu compute passes
  -> Base Color / Normal / Roughness / Height / ...
  -> PNG or raw pixels
```

首先支持的使用方是原生 CLI。只有原生接口契约稳定、且有外部使用方准备接入后，才通过 WebAssembly 支持浏览器 WebGPU。

## 核心保证

Mixture 的设计遵循以下不可破坏的保证：

1. **Rust 负责文档模型、图规则、节点契约、编译和诊断。**
2. **`wgpu` 是唯一的像素执行后端。** 不提供第二套 CPU 渲染器，也不维护独立的 TypeScript WebGPU 渲染器。
3. **执行过程显式可见。** GPU 初始化、适配器选择、编译、请求输出、尺寸和参数覆盖均由调用方明确传入。
4. **不进行隐式语义回退。** GPU 请求失败时返回结构化错误，不静默切换到其他渲染器。
5. **输入从设计上保证确定性。** 随机节点必须显式指定种子，图遍历顺序稳定，编译后的执行计划具有稳定哈希。
6. **公共接口保持精简。** 新增 crate、节点、文件格式特性或优化层，都需要真实材质或使用方的证据支持。
7. **视觉质量属于正确性的一部分。** 扩展节点集合之前，三种基准材质必须通过结构、平铺、GPU 和人工审查验收。

## 明确不做的事项

以下内容不在初始路线图范围内：

- React 或桌面节点编辑器；
- Gallery、Player、市场、账号或协作功能；
- CPU 像素渲染器；
- 单独维护的 WebGL2 渲染器；
- 自动 `WebGPU -> WebGL2 -> CPU` 回退；
- 子图、函数图、自定义 WGSL 或插件系统；
- `.mixar`、二进制容器或内嵌资源；
- 核心中的特定引擎导出配置；
- 3D 烘焙、网格处理或 PBR 预览场景；
- 云渲染或 AI 材质生成。

只有真实使用方证明简单系统不足以满足需求后，才重新考虑这些方向。

## 命令接口

`check`、`validate`、`inspect --plan`、`render`、`doctor` 和 `render-builtin checker` 已实现。选项、通道编码和报告见[计划检查](./docs/render-plan.zh-CN.md)及[图渲染](./docs/graph-rendering.zh-CN.md)。

```bash
# Repository verification
cargo xtask check

# Environment and adapter diagnostics
cargo run -p mixture-cli -- doctor
cargo run -p mixture-cli -- doctor --json

# Document and plan inspection
cargo run -p mixture-cli -- validate examples/checker.mix --json
cargo run -p mixture-cli -- inspect examples/checker.mix --plan --json

# Headless rendering
cargo run -p mixture-cli -- render examples/blend.mix \
  --size 512 \
  --output baseColor,normal,roughness,height \
  --out ./out
```

针对具体任务的开发命令见[贡献者与智能体指南](./AGENTS.zh-CN.md)。

## `.mix` v1 源文件

首个可执行结构见[文件格式指南](./docs/file-format.zh-CN.md)。以下棋盘格当前可以通过验证：

```json
{
  "version": 1,
  "nodes": [
    {
      "id": "checker",
      "type": "checker",
      "version": 1
    },
    {
      "id": "out",
      "type": "material-output",
      "version": 1
    }
  ],
  "edges": [
    {
      "from": {
        "nodeId": "checker",
        "portId": "color"
      },
      "to": {
        "nodeId": "out",
        "portId": "baseColor"
      }
    }
  ],
  "exposedParameters": [
    {
      "id": "frequency",
      "nodeId": "checker",
      "parameterId": "cellsX"
    }
  ]
}
```

布局、元数据、缩略图、预设、资源和引擎目标均被拒绝。`baseColor` 必须有有效 Color 连接；可选通道使用[已记录的材质默认值](./docs/node-contracts.zh-CN.md)。

## 架构概览

```text
                         no wgpu dependency
+-------------------+   -------------------->   +---------------------+
|   mixture-core    |                           |    mixture-wgpu     |
|                   |   RenderPlan              |                     |
| document          | ------------------------> | explicit GpuContext |
| validation        |                           | compute pipelines    |
| node contracts    |                           | resource lifetime    |
| compiler          |                           | readback             |
| diagnostics       |                           | execution metrics    |
+---------+---------+                           +----------+----------+
          ^                                                ^
          |                                                |
          +--------------------+  +------------------------+
                               |  |
                         +-----+--+------+
                         | mixture-cli  |
                         | thin I/O and |
                         | orchestration|
                         +-------------+
```

未来的 `mixture-wasm` crate 必须保持为同一核心编译器和 `wgpu` 渲染器之上的轻量绑定层。

完整的职责划分和执行模型见[架构文档](./ARCHITECTURE.zh-CN.md)。

## 仓库布局

当前布局由三个产品 crate 和一个私有工具 crate 组成：

```text
mixture/
├── .cargo/config.toml      # exposes `cargo xtask`
├── crates/
│   ├── mixture-core/       # document, validation, node contracts, compiler
│   ├── mixture-wgpu/       # the only pixel executor
│   └── mixture-cli/        # validate, inspect, doctor, render
├── xtask/                  # repository-only automation
├── fixtures/
│   ├── nodes/              # focused node cases
│   └── materials/          # golden material acceptance assets
├── examples/               # small user-facing .mix documents
├── docs/                   # focused design and usage guides
├── AGENTS.md
├── ARCHITECTURE.md
├── ROADMAP.md
└── INITIAL_PRS.md
```

不要仅为划分概念而新增 crate。只有运行时、发布、依赖或构建边界无法在现有三个产品 crate 内清晰表达时，才需要新的 crate。

## 基准材质

内置节点类型超过十二个之前，Mixture 必须生成并保留三种已验收的基准材质：

- 釉面陶瓷／棋盘格材质，用于验证图和平铺基础能力；
- 类皮革材质，用于验证微表面高度、粗糙度和法线行为；
- 各向异性木纹材质，用于验证方向性结构和扭曲行为。

每种基准材质必须包含：

- 可读的 `.mix` 源文件；
- 固定的参数变体；
- 固定版本软件适配器上的预期输出；
- 允许明确误差容限的跨适配器比较；
- 平铺与非退化性度量；
- 简短的人工审查记录，说明结果为何可接受。

单元测试通过，不足以证明材质在视觉上正确。

## 设计参考

Mixture 借鉴少量明确的设计思路，不照搬这些项目的完整产品范围：

- [`wgpu`](https://github.com/gfx-rs/wgpu)：为原生与 WebAssembly 目标提供统一的跨平台 Rust GPU API；
- [`vgpu`](https://github.com/vercel-labs/vgpu)：显式 GPU 上下文、精简公共 API、可执行诊断和便于智能体发现的文档；
- [`vinext`](https://github.com/cloudflare/vinext)：清晰的智能体指南、任务与测试映射、轻量适配层及便于审查的小改动。

## 项目文档

- [贡献者与智能体指南](./AGENTS.zh-CN.md)：编码智能体和贡献者的操作规则。
- [架构文档](./ARCHITECTURE.zh-CN.md)：系统边界、不可变约束、数据流和测试模型。
- [路线图](./ROADMAP.zh-CN.md)：里程碑目标、退出标准和停止规则。
- [初始 PR 实施计划](./INITIAL_PRS.zh-CN.md)：历史 M0–M3 实施批次及其验收要求。
- [M4 实施计划](./M4_PRS.zh-CN.md)：已完成的原生消费者实施批次及其历史证据。
- [M5 实施计划](./M5_PRS.zh-CN.md)：浏览器运行时、npm 包消费与独立 Player 的计划工作项。
- [浏览器 SDK 契约](./docs/browser-sdk.zh-CN.md)：软件包、初始化、输入／输出及生命周期要求，区分已实现棋盘格切片与剩余验收。
- [仓库治理](./docs/governance.zh-CN.md)：集成分支、真实 GitHub PR 及必需检查规则。
- [证据保留](./docs/evidence-policy.zh-CN.md)：已接受记录、临时运行输出及产物可用性。
- [中文文档索引](./docs/README.zh-CN.md)：开发指南、架构决策和其他说明的入口。

## 许可证

可任选 [Apache-2.0](./LICENSE-APACHE) 或 [MIT](./LICENSE-MIT) 许可证使用。当前 pre-alpha 软件包保持 `publish = false`；发布需单独决定。

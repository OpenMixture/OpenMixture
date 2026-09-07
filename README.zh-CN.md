# Mixture

[English](./README.md) | 简体中文

> 基于 Rust 和 `wgpu` 的小型材质图编译器与无界面纹理渲染器。

**状态：** 从零构建，处于 pre-alpha 阶段，尚不承诺兼容性。

**已实现：** PR-001 至 PR-005：严格 `.mix v1` 解码／图验证、六个节点契约、共享诊断、显式 GPU 上下文、验证型 `doctor` 和固定棋盘格计算／回读及 PNG 输出。本地检查通过，保留已有 Metal／SwiftShader 棋盘格证据。图编译／渲染是后续工作，远端 CI 证据仍待获取。

Mixture 的设计目标是读取带版本号的 `.mix` 材质文档，验证并编译其中的有向无环图，通过唯一的 `wgpu` 渲染器执行计算通道，返回所请求的 PBR 纹理通道。

项目从引擎和命令行工具起步，初期不构建完整的材质创作产品。

## 快速开始

安装 [rustup](https://rustup.rs/) 和当前平台的原生链接器，然后在仓库根目录运行：

```bash
cargo xtask check
cargo run --locked -p mixture-cli -- --help
```

仓库固定使用 Rust 1.98.1 / edition 2024，并包含 `Cargo.lock`。检查覆盖格式、依赖边界、Clippy、测试、rustdoc 和本地文档链接，无需 GPU。所有已实现命令和平台前置条件见[开发指南](./docs/development.zh-CN.md)。

核心现已提供[诊断与安全限制 API](./docs/diagnostics.zh-CN.md)，包含强类型错误、确定性 JSON 报告和七类显式资源上限。可通过 `cargo run --locked -p mixture-core --example diagnostics` 运行公共示例。

[GPU 上下文与 doctor 指南](./docs/gpu-context.zh-CN.md)说明适配器选择、结构化失败、退出码和显式 `cargo xtask gpu-smoke` 检查。运行 `cargo run --locked -p mixture-cli -- doctor --json` 检查本机环境。

可运行[内置棋盘格](./docs/builtin-checker.zh-CN.md)：`cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png`。Doctor 默认运行计算／回读探针；仅获取上下文时使用 `--skip-probe`。

无需 GPU 即可验证[棋盘格文档](./examples/checker.mix)：`cargo run --locked -p mixture-cli -- validate examples/checker.mix --json`。参阅[严格文件格式](./docs/file-format.zh-CN.md)和[六个节点契约](./docs/node-contracts.zh-CN.md)。

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

## 计划中的命令接口

`check`、`validate`、`doctor` 和 `render-builtin checker` 已实现。文档检查和图渲染仍是后续里程碑的稳定目标接口。

```bash
# Repository verification
cargo xtask check

# Environment and adapter diagnostics
cargo run -p mixture-cli -- doctor
cargo run -p mixture-cli -- doctor --json

# Document and plan inspection
cargo run -p mixture-cli -- validate examples/checker.mix --json
cargo run -p mixture-cli -- inspect examples/wood.mix --plan --json

# Headless rendering
cargo run -p mixture-cli -- render examples/wood.mix \
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

目标布局由三个产品 crate 和一个私有工具 crate 组成：

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
- [初始 PR 实施计划](./INITIAL_PRS.zh-CN.md)：第一轮实施顺序，可据此创建 issue 和堆叠 PR。
- [中文文档索引](./docs/README.zh-CN.md)：开发指南、架构决策和其他说明的入口。

## 许可证

可任选 [Apache-2.0](./LICENSE-APACHE) 或 [MIT](./LICENSE-MIT) 许可证使用。基础工程里程碑期间禁用软件包发布。

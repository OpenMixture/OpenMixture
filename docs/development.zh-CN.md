# 开发指南

[English](./development.md) | 简体中文

## 基础工程、诊断与 GPU 上下文状态

仓库已本地实现[初始计划](../INITIAL_PRS.zh-CN.md)中的 PR-001 至 PR-010。陶瓷、皮革和木材观感均已获用户接受。[M3 评审](./m3-review.zh-CN.md)及[复现脚本](./reviews/m3/README.zh-CN.md)记录本地验收、release 性能和原生消费者缺口。[M4 PR-011](../M4_PRS.zh-CN.md)现已实现[独立公开 Rust 消费者](./native-sdk.zh-CN.md)及纯 CPU `test-consumer`，包含显式 GPU 所有权检查和 1K release 证据。PR-012 添加 [CLI 报告／退出码契约](./cli-contract.zh-CN.md)、完整人类可读诊断上下文及独立 CLI 进程测试。PR-013 添加 [GPU 失败原因、丢失生命周期及清理](./gpu-failures.zh-CN.md)。PR-014 添加[最新请求与有界保留](./stale-results.zh-CN.md)。PR-015 通过 `package-check` 添加[隔离本地包验证](./package-consumption.zh-CN.md)，并提供[兼容性](./compatibility.zh-CN.md)及 [M4 退出／发布评估](./release.zh-CN.md)。M4 已本地验收；远端 CI 继续暂缓。
它包含三个产品 crate 边界和私有仓库工具。核心提供[诊断与安全限制 API](./diagnostics.zh-CN.md)；[显式 GPU 获取与 doctor](./gpu-context.zh-CN.md)已可用。[棋盘格计算／回读和 CLI PNG 输出](./builtin-checker.zh-CN.md)已实现，[严格 .mix 解码／验证](./file-format.zh-CN.md)和[十一个节点契约](./node-contracts.zh-CN.md)已实现。[确定性编译与计划检查](./render-plan.zh-CN.md)已实现，[图执行](./graph-rendering.zh-CN.md)及三个 PNG 示例已实现。所有软件包均禁用发布。

[rust-toolchain.toml](../rust-toolchain.toml)固定使用 Rust 1.98.1、edition 2024、rustfmt 和 Clippy。通过 [rustup](https://rustup.rs/) 安装 Rust，并准备原生 Rust 链接器／工具链：macOS 使用 Xcode Command Line Tools，Linux 使用 C 链接器，Windows 使用 Visual Studio C++ Build Tools。在本仓库运行 Cargo 时，会按需安装固定工具链。

## 可用命令

```bash
cargo xtask check
cargo xtask fmt
cargo xtask clippy
cargo xtask test
cargo xtask test-core
cargo xtask test-format
cargo xtask test-plan
cargo xtask test-consumer
cargo xtask package-check
cargo run --locked -p mixture-cli -- inspect examples/checker.mix --plan --json
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
cargo xtask doc
cargo xtask deps
cargo xtask links
cargo test --locked -p mixture-core diagnostics
cargo test --locked -p mixture-core limits
cargo run --locked -p mixture-core --example diagnostics
cargo run --locked -p mixture-cli -- --help
cargo run --locked -p mixture-cli -- --version
cargo test --locked -p mixture-wgpu context
cargo test --locked -p mixture-cli doctor
cargo run --locked -p mixture-cli -- doctor --json
cargo test --locked -p mixture-wgpu checker
cargo test --locked -p mixture-wgpu readback
cargo xtask shader-check
cargo xtask test-node checker
cargo xtask test-material glazed-ceramic
cargo xtask golden check
cargo xtask trace-2k
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png
cargo xtask gpu-smoke
```

[Cargo 别名](../.cargo/config.toml)使用 `--locked` 启动 xtask。构建／测试／文档命令及生产方依赖解析均使用 `--locked`。唯一包暂存例外是一次离线元数据解析，用于规范化一次性锁；其外部版本／来源／校验和对照已提交固定项检查，然后所有验证均使用 `--offline --locked`。检查器从不重写已提交锁。格式化不解析依赖。首次运行下载已锁定的工具依赖，后续验证可以使用 Cargo 缓存。

`check` 按顺序运行：

1. `cargo fmt --all -- --check`。
2. 基于 `cargo metadata` 的当前依赖策略检查。
3. 覆盖工作区全部目标和特性的 Clippy，将警告视为错误。
4. 工作区测试，包括 CLI 集成测试、工具测试和文档测试。
5. `test-consumer`：独立 Cargo 元数据、格式化、Clippy、单元／进程测试、构建及公开 Rust CPU 调用，随后构建 CLI，在新工作目录显式执行独立 CPU 契约测试。
6. `package-check`：真实本地归档、隔离源码／锁解析、单元测试／Rustdoc、缺失 shader 拒绝及独立 CPU 消费。
7. 工作区 rustdoc 构建，将警告视为错误。
8. 基础离线 Markdown 链接检查，确认所引用的本地文件和目录存在。

任何非预期子进程失败都会使整体检查失败；显式缺失 shader 探针必须失败，并作为负向测试校验。检查不会重写源文件、夹具或基准。Markdown 解析处理行内链接、引用式链接和图片，忽略代码示例；外部 URL、标题锚点、原始 HTML 链接和百分号编码本地路径不在基础检查支持范围内。使用普通相对路径；包含空格的路径使用尖括号。扫描时排除构建／输出目录。

使用 `cargo fmt --all` 应用格式化。变更依赖时有意识地更新 `Cargo.lock`，然后重新运行 `cargo xtask check`。Cargo 工作区和 lint 继承遵循 [Cargo 工作区参考文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)。

## 依赖策略

PR-015 产品／工具工作区唯一允许的直接依赖关系如下，包括构建、开发、可选和特定目标依赖：

| 软件包 | 允许的依赖 |
| --- | --- |
| `mixture-core` | 运行时使用 `serde`、启用 `float_roundtrip` 的 `serde_json` 及 `sha2` |
| `mixture-wgpu` | 工作区内的 `mixture-core`、`wgpu`、`serde`、`half`；仅开发时使用 `pollster`、`serde_json`、`naga` |
| `mixture-cli` | 工作区内的 `mixture-core`、`mixture-wgpu`、`pollster`、`serde`、`serde_json`、`png` |
| `xtask` | `pulldown-cmark`、`serde_json`、`png`、`serde`、`sha2` |

核心使用 `serde` 处理类型化源数据、诊断和限制；PR-005 将已锁定的 `serde_json` 提升为运行时依赖，用于严格有界解码与确定性序列化。`float_roundtrip` 特性修复了已复现的源数据往返一位浮点偏差；不需要新增包或依赖版本。PR-006 添加 `sha2` 用于稳定 SHA-256 计划哈希，将其依赖闭包加入锁文件，不升级已有包。工具使用 `pulldown-cmark` 解析 Markdown，使用 `serde_json` 读取 Cargo 元数据。只有 `mixture-wgpu` 直接依赖 `wgpu`，其 `serde` 用于编码能力报告。`pollster` 在 CLI／测试边界驱动异步获取，`serde_json` 用于 CLI 报告和测试断言。`half` 解码 GPU 半精度回读，开发依赖 `naga` 在无 GPU 环境验证 WGSL。CLI 的 `png` 编码图像，工具的 `png` 解码图像用于基准比较。核心仍不依赖 GPU。原生后端特性策略见 [GPU 指南](./gpu-context.zh-CN.md)。PR-008 仅将现有工作区 `serde` 和 `sha2` 加为工具直接依赖，用于严格验收记录及候选完整性，没有新增或升级软件包／版本。全部已解析依赖版本记录在 [Cargo.lock](../Cargo.lock) 中。

[依赖检查](../xtask/src/dependencies.rs)约束四个 crate 的边界、双许可证元数据、禁用发布，以及上述直接依赖允许列表。它检查架构和范围，不是漏洞数据库检查或传递依赖许可证审计。引入依赖时，在负责该依赖的实施 PR 中扩展策略并说明必要性，保持[架构文档](../ARCHITECTURE.zh-CN.md)要求的方向。

[原生消费者](../examples/native-consumer/Cargo.toml)是独立单包工作区，拥有已提交的锁文件；`test-consumer` 检查该边界，不给产品工作区增加第五个成员。它通过源码 path 使用 `mixture-core`／`mixture-wgpu`，并精确固定生产方已使用的 `pollster` 和 `serde_json` 版本。PR-012 添加仅开发时使用的 `png = "=0.18.1"`，解码 CLI 已完成输出；锁文件新增九项，版本均与产品已解析版本相同。产品锁文件及依赖策略不变。path 依赖及已构建 CLI 验证不证明软件包内容。

## CLI 行为与后续工作

可执行程序名为 `mixture`。帮助／版本返回 `0`。`doctor` 验证真实棋盘格计算／回读，成功返回 `0` 和 `healthy`；显式 `--skip-probe` 返回 `unverified`。获取／探针失败返回 `1` 和 `unhealthy`。`render-builtin checker` 写入 PNG 后返回 `0`，运行失败返回 `1`，尺寸／预算无效返回 `2`。JSON 模式向 stdout 写入一份报告，人类可读模式包含相同的策略和能力证据。`validate` 对有效源文件返回 `0`，输入无效返回 `2`，文件／报告 I/O 失败返回 `1`，且不初始化 GPU。`inspect --plan` 编译成功返回 `0`，源文件／请求无效返回 `2`，文件／报告 I/O 失败返回 `1`，同样无需 GPU。`render` 在获取 GPU 前编译，全部请求 PNG 写入后返回 `0`，源文件／请求无效返回 `2`，GPU／编码／I/O 失败返回 `1`。无效选项和缺少命令返回 `2`，说明写入 stderr。输出 I/O 失败返回 `1`。见 [doctor 用法](./gpu-context.zh-CN.md)和[共享退出码策略](./diagnostics.zh-CN.md)。

`test-format` 运行核心格式／验证／注册表测试及 CLI 验证测试。`test-plan` 运行核心计划／哈希测试与 CLI 检查测试。`test-node <id>` 验证夹具并显式运行所选 GPU 节点用例。`test-material <id>` 和 `golden check` 现已显式渲染 GPU 用例并比较材质验收，不修改基准。`golden update <id> --accept` 独立消费已审查的软件候选并拒绝 CI，见[完整工作流](./material-goldens.zh-CN.md)。

`test-consumer` 将独立构建／测试／CPU 报告及每次先失效再标记完成的状态保存到 `tmp/consumer-check/`。它编译完整 GPU 调用路径及原生 `Send` 约束，但不初始化 wgpu。它还构建真实 CLI，并使用消费者自有文件显式运行 32 次 CPU 契约调用。完成状态的 `cliEvidence` 指向本次新 CLI 捕获目录；消费者普通 Cargo 测试保持 CLI 契约测试为忽略状态，直到这次显式调用。见[消费者指南](../examples/native-consumer/README.zh-CN.md)。

`gpu-smoke` 显式访问 GPU 硬件或配置的软件适配器，将报告保存到 `tmp/gpu-smoke/`，其中 `native-consumer/` 保存独立消费者证据。Rust 消费者渲染两次，销毁 renderer／context 后验证自有字节及元数据。PR-012 还执行 10 次独立 CLI 调用，覆盖 doctor、成功 PNG 及部分写入；原始输出流和实际文件保留在所链接的 CLI 证据目录。它不属于 `check` 和普通工作区测试。适配器策略变量、固定 SwiftShader 准备方式和本地证据见 [GPU 指南](./gpu-context.zh-CN.md)。

## 2K 资源证据

`cargo xtask trace-2k` 独立执行显式 GPU 工作负载，`check` 和普通工作区测试不运行它。使用与 `gpu-smoke` 相同的 `MIXTURE_GPU_BACKEND`、`MIXTURE_GPU_SOFTWARE` 及可选 `MIXTURE_GPU_EXPECT_ADAPTER` 变量配置所需适配器。软件执行要求 Vulkan，以及 [GPU 指南](./gpu-context.zh-CN.md)中经过验证的固定 SwiftShader 源码／加载器配置。

```bash
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 cargo xtask trace-2k
# 配置固定 SwiftShader 源码和加载器后：
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 cargo xtask trace-2k
```

[跟踪任务](../xtask/src/golden/trace.rs)首先以 2048×2048 编译 `glazed-ceramic`、`leather` 和 `wood` 的每个配置用例，请求 `baseColor`、`normal`、`roughness` 和 `height`。三份夹具都必须存在且有效。任务先按估算 `peakBytes` 从大到小选取，再按 pass 数从大到小、材质名字典序、默认用例优先和用例 ID 字典序决定顺序，随后验证 `doctor` 并通过所请求的适配器渲染该精确计划。检查被拒绝时在 GPU 工作前停止；任务不提高安全限制。

瞬态预算为**峰值存活描述符的 512 MiB（536,870,912 字节）**，与默认 `SafetyLimits::transient_bytes` 一致。顺序回读的累计分配可以大于峰值。[核心估算](./render-plan.zh-CN.md)假定所有 pass 纹理和 uniform 保留至执行／回读结束，同时最多只有一个 staging 缓冲区存活。别名通道仍各自回读一次。PR-010 测量现有调度，不添加最后使用者释放、兼容纹理复用或资源池；此类生命周期修改必须先有超预算或失败的实测工作负载作为依据。

`RenderReport.allocations` 是公开的 [AllocationReport](../crates/mixture-wgpu/src/allocations.rs)，CLI 将其序列化为 `execution.allocations`。它统计成功创建的纹理／uniform／staging 描述符，以及累计、峰值、存活、已释放和已复用字节。跟踪任务要求这些计数与所选计划一致，所有请求通道顺序回读完成，最终 `liveBytes` 为零，`releasedBytes` 等于 `cumulativeBytes`，且 `reusedBytes` 保持为零。释放计数记录 `destroy` 调用，不代表驱动或操作系统立即归还了物理内存。驱动分配粒度、着色器／管线／绑定组内存，以及 CPU 像素／PNG 缓冲区均不计入。管线缓存仍归 renderer 所有，不属于这些逐次调用计数。分配观测不改变核心计划、其估算或哈希。

成功运行后，`tmp/trace-2k/<run>/` 包含 `selection.json`、全部检查报告、`doctor.json`、`render.json`、四张 PNG 及其哈希、源码／夹具哈希、精确渲染参数与适配器设置，以及 `trace.json`。`pipelineMs`、`executionMs`、`readbackMs` 和 `totalMs` 是有限的 CPU 墙钟测量值，不声称使用 GPU 时间戳查询。任务拒绝运行期间发生变化的源码／夹具输入或 1K 基准。`latest-software.json` 或 `latest-hardware.json` 记录最近一次尝试；失败时保留已有 CLI JSON／stderr 和 `failure.json`。跟踪通过只证明该工作负载的简单调度有界。它不创建或更新 2K 像素基准，也不决定材质人工验收或关闭暂缓的远端 CI 门槛。

## CI 与里程碑证据

[非 GPU CI](../.github/workflows/ci.yml)在 Linux、macOS 和 Windows 上运行相同的锁定依赖命令，不需要 GPU 或显示器。本地通过仅证明当前主机环境；只有配置的 CI 矩阵在远端仓库实际通过后，才能满足 M0 跨平台验收条件。

[独立 GPU CI](../.github/workflows/gpu-smoke.yml)构建固定 SwiftShader Vulkan 适配器并运行计算／回读、节点回归、全部三种 1K 材质比较及最大案例的 2K 追踪。失败时也保留三个证据目录；远端结果仍待获取。本地 Metal 与固定 SwiftShader Vulkan 证据记录于[棋盘格指南](./builtin-checker.zh-CN.md)。

智能体指南中尚未实现的运行时模块仍是未来职责地图。当前编译根模块为 [core](../crates/mixture-core/src/lib.rs)、[wgpu](../crates/mixture-wgpu/src/lib.rs)、[CLI](../crates/mixture-cli/src/main.rs) 和 [xtask](../xtask/src/main.rs)。棋盘格已有真实 GPU 生成的夹具；[格式夹具](../fixtures/format/README.zh-CN.md)与 [checker.mix](../examples/checker.mix)现已覆盖源文件验证。三个 [M2 示例](../examples/README.zh-CN.md)现均可渲染请求通道。

已实现的核心模块为[文档解码](../crates/mixture-core/src/document.rs)、[验证](../crates/mixture-core/src/validation.rs)、[注册表](../crates/mixture-core/src/registry.rs)、[节点契约](../crates/mixture-core/src/nodes/)、[诊断](../crates/mixture-core/src/error.rs)和[限制](../crates/mixture-core/src/limits.rs)。[Rust 诊断示例](../crates/mixture-core/examples/diagnostics.rs)和 crate 文档测试覆盖公开 API。

PR-006 添加[编译器](../crates/mixture-core/src/compiler.rs)、[规范化与降级](../crates/mixture-core/src/compiler/)、[类型化计划 API](../crates/mixture-core/src/plan.rs) 和 [CLI inspect](../crates/mixture-cli/src/commands/inspect.rs)。计划快照位于核心测试旁，PR-007 [图渲染](./graph-rendering.zh-CN.md)添加执行器、资源、kernel 映射／缓存及 CLI render，没有新增依赖。

PR-009 不增加依赖或锁文件变更。通过 `cargo xtask` 运行 `test-node fractal-noise`、`test-node gradient-map`、`test-node height-to-normal` 和 `test-material leather`；[皮革评审](../fixtures/materials/leather/review/README.zh-CN.md)是可选的外部 Blender 消费者，不是 Rust 运行时依赖。

PR-010 同样不增加依赖或锁文件变更。通过 `cargo xtask` 运行 `test-node transform-2d`、`test-node warp`、`test-material wood`、`golden check` 和 `trace-2k`。公开 Renderer 文档测试编译验证 `AllocationReport` 的访问；定向 GPU 测试验证非对齐宽度的别名回读、逐次调用计数重置，以及回读错误后的释放。

PR-013 添加 CPU 分类与消费者回执守卫测试。`test-consumer` 编译独立 `device_loss` 测试但保持忽略；显式 `gpu-smoke` 执行其冷／热缓存销毁用例，并校验 `native-consumer/status.json` → `deviceLossEvidence` 引用的新建回执。[GPU 失败复现](./gpu-failures.zh-CN.md)和[本地证据](./evidence/pr-013/README.zh-CN.md)区分类型化合成 OOM 与真实设备销毁。

PR-014 将调度放在 `examples/native-consumer/src/latest.rs`，作为既有示例包内的库 target。`test-consumer` 运行六个确定性状态／生命周期测试。显式 `gpu-smoke` 添加 Rust `latest`、五次按代次隔离的 CLI 调用和九内核缓存边界 GPU 回归。消费者根状态通过 `latestEvidence` 引用证据；[复现与限制](./stale-results.zh-CN.md)说明回执与输出结构边界。

PR-015 不新增直接依赖，也不修改已提交锁文件。产品包添加精确同组版本要求、显式包含清单、双语 README 及许可副本。三个小型 wgpu 单元输入改用包内 include 路径，`package-check` 对照规范夹具检查字节一致性。它要求兼容 `tar` 和仓库外 OS 临时目录，在 `tmp/package-check/` 保留原始包证据，仅在 `target/package-consumer` 使用忽略的编译产物缓存。见[完整验证方法](./package-consumption.zh-CN.md)。

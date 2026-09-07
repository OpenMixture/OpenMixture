# 开发指南

[English](./development.md) | 简体中文

## 基础工程、诊断与 GPU 上下文状态

仓库实现了[实施计划](../INITIAL_PRS.zh-CN.md)中的 PR-001 至 PR-007。
它包含三个产品 crate 边界和私有仓库工具。核心提供[诊断与安全限制 API](./diagnostics.zh-CN.md)；[显式 GPU 获取与 doctor](./gpu-context.zh-CN.md)已可用。[棋盘格计算／回读和 CLI PNG 输出](./builtin-checker.zh-CN.md)已实现，[严格 .mix 解码／验证](./file-format.zh-CN.md)和[六个节点契约](./node-contracts.zh-CN.md)已实现。[确定性编译与计划检查](./render-plan.zh-CN.md)已实现，[六节点图执行](./graph-rendering.zh-CN.md)及三个 PNG 示例已实现。所有软件包均禁用发布。

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
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png
cargo xtask gpu-smoke
```

[Cargo 别名](../.cargo/config.toml)使用 `--locked` 启动 xtask。所有会解析依赖的嵌套 Cargo 命令也使用 `--locked`。格式化不解析依赖。首次运行下载已锁定的工具依赖，后续验证可以使用 Cargo 缓存。

`check` 按顺序运行：

1. `cargo fmt --all -- --check`。
2. 基于 `cargo metadata` 的当前依赖策略检查。
3. 覆盖工作区全部目标和特性的 Clippy，将警告视为错误。
4. 工作区测试，包括 CLI 集成测试、工具测试和文档测试。
5. 工作区 rustdoc 构建，将警告视为错误。
6. 基础离线 Markdown 链接检查，确认所引用的本地文件和目录存在。

任何子进程失败都会使整体检查失败。检查不会重写源文件、夹具或基准。Markdown 解析处理行内链接、引用式链接和图片，忽略代码示例；外部 URL、标题锚点、原始 HTML 链接和百分号编码本地路径不在基础检查支持范围内。使用普通相对路径；包含空格的路径使用尖括号。扫描时排除构建／输出目录。

使用 `cargo fmt --all` 应用格式化。变更依赖时有意识地更新 `Cargo.lock`，然后重新运行 `cargo xtask check`。Cargo 工作区和 lint 继承遵循 [Cargo 工作区参考文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)。

## 依赖策略

PR-007 唯一允许的直接依赖关系如下，包括构建、开发、可选和特定目标依赖：

| 软件包 | 允许的依赖 |
| --- | --- |
| `mixture-core` | 运行时使用 `serde`、启用 `float_roundtrip` 的 `serde_json` 及 `sha2` |
| `mixture-wgpu` | 工作区内的 `mixture-core`、`wgpu`、`serde`、`half`；仅开发时使用 `pollster`、`serde_json`、`naga` |
| `mixture-cli` | 工作区内的 `mixture-core`、`mixture-wgpu`、`pollster`、`serde`、`serde_json`、`png` |
| `xtask` | `pulldown-cmark`、`serde_json`、`png` |

核心使用 `serde` 处理类型化源数据、诊断和限制；PR-005 将已锁定的 `serde_json` 提升为运行时依赖，用于严格有界解码与确定性序列化。`float_roundtrip` 特性修复了已复现的源数据往返一位浮点偏差；不需要新增包或依赖版本。PR-006 添加 `sha2` 用于稳定 SHA-256 计划哈希，将其依赖闭包加入锁文件，不升级已有包。工具使用 `pulldown-cmark` 解析 Markdown，使用 `serde_json` 读取 Cargo 元数据。只有 `mixture-wgpu` 直接依赖 `wgpu`，其 `serde` 用于编码能力报告。`pollster` 在 CLI／测试边界驱动异步获取，`serde_json` 用于 CLI 报告和测试断言。`half` 解码 GPU 半精度回读，开发依赖 `naga` 在无 GPU 环境验证 WGSL。CLI 的 `png` 编码图像，工具的 `png` 解码图像用于基准比较。核心仍不依赖 GPU。原生后端特性策略见 [GPU 指南](./gpu-context.zh-CN.md)。全部已解析依赖版本记录在 [Cargo.lock](../Cargo.lock) 中。

[依赖检查](../xtask/src/dependencies.rs)约束四个 crate 的边界、双许可证元数据、禁用发布，以及上述直接依赖允许列表。它检查架构和范围，不是漏洞数据库检查或传递依赖许可证审计。引入依赖时，在负责该依赖的实施 PR 中扩展策略并说明必要性，保持[架构文档](../ARCHITECTURE.zh-CN.md)要求的方向。

## CLI 行为与后续工作

可执行程序名为 `mixture`。帮助／版本返回 `0`。`doctor` 验证真实棋盘格计算／回读，成功返回 `0` 和 `healthy`；显式 `--skip-probe` 返回 `unverified`。获取／探针失败返回 `1` 和 `unhealthy`。`render-builtin checker` 写入 PNG 后返回 `0`，运行失败返回 `1`，尺寸／预算无效返回 `2`。JSON 模式向 stdout 写入一份报告，人类可读模式包含相同的策略和能力证据。`validate` 对有效源文件返回 `0`，输入无效返回 `2`，文件／报告 I/O 失败返回 `1`，且不初始化 GPU。`inspect --plan` 编译成功返回 `0`，源文件／请求无效返回 `2`，文件／报告 I/O 失败返回 `1`，同样无需 GPU。`render` 在获取 GPU 前编译，全部请求 PNG 写入后返回 `0`，源文件／请求无效返回 `2`，GPU／编码／I/O 失败返回 `1`。无效选项和缺少命令返回 `2`，说明写入 stderr。输出 I/O 失败返回 `1`。见 [doctor 用法](./gpu-context.zh-CN.md)和[共享退出码策略](./diagnostics.zh-CN.md)。

`test-format` 运行核心格式／验证／注册表测试及 CLI 验证测试。`test-plan` 运行核心计划／哈希测试与 CLI 检查测试。`test-node <id>` 验证夹具并显式运行所选 GPU 节点用例。材质和基准相关 xtask 命令尚未实现，调用会返回错误。引入顺序见[初始 PR 实施计划](../INITIAL_PRS.zh-CN.md)。

`gpu-smoke` 显式访问 GPU 硬件或配置的软件适配器，将报告保存到 `tmp/gpu-smoke/`。它不属于 `check` 和普通工作区测试。适配器策略变量、固定 SwiftShader 准备方式和本地证据见 [GPU 指南](./gpu-context.zh-CN.md)。

## CI 与里程碑证据

[非 GPU CI](../.github/workflows/ci.yml)在 Linux、macOS 和 Windows 上运行相同的锁定依赖命令，不需要 GPU 或显示器。本地通过仅证明当前主机环境；只有配置的 CI 矩阵在远端仓库实际通过后，才能满足 M0 跨平台验收条件。

[独立 GPU CI](../.github/workflows/gpu-smoke.yml)构建固定 SwiftShader Vulkan 适配器并运行计算／回读、PNG／基准比较和 GPU 回归测试，远端结果仍待获取。本地 Metal 与固定 SwiftShader Vulkan 证据记录于[棋盘格指南](./builtin-checker.zh-CN.md)。

智能体指南中尚未实现的运行时模块仍是未来职责地图。当前编译根模块为 [core](../crates/mixture-core/src/lib.rs)、[wgpu](../crates/mixture-wgpu/src/lib.rs)、[CLI](../crates/mixture-cli/src/main.rs) 和 [xtask](../xtask/src/main.rs)。棋盘格已有真实 GPU 生成的夹具；[格式夹具](../fixtures/format/README.zh-CN.md)与 [checker.mix](../examples/checker.mix)现已覆盖源文件验证。三个 [M2 示例](../examples/README.zh-CN.md)现均可渲染请求通道。

已实现的核心模块为[文档解码](../crates/mixture-core/src/document.rs)、[验证](../crates/mixture-core/src/validation.rs)、[注册表](../crates/mixture-core/src/registry.rs)、[节点契约](../crates/mixture-core/src/nodes/)、[诊断](../crates/mixture-core/src/error.rs)和[限制](../crates/mixture-core/src/limits.rs)。[Rust 诊断示例](../crates/mixture-core/examples/diagnostics.rs)和 crate 文档测试覆盖公开 API。

PR-006 添加[编译器](../crates/mixture-core/src/compiler.rs)、[规范化与降级](../crates/mixture-core/src/compiler/)、[类型化计划 API](../crates/mixture-core/src/plan.rs) 和 [CLI inspect](../crates/mixture-cli/src/commands/inspect.rs)。计划快照位于核心测试旁，PR-007 [图渲染](./graph-rendering.zh-CN.md)添加执行器、资源、kernel 映射／缓存及 CLI render，没有新增依赖。

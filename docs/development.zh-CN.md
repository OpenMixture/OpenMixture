# 开发指南

[English](./development.md) | 简体中文

## 基础工程、诊断与 GPU 上下文状态

仓库实现了[实施计划](../INITIAL_PRS.zh-CN.md)中的 PR-001 至 PR-004。
它包含三个产品 crate 边界和私有仓库工具。核心提供[诊断与安全限制 API](./diagnostics.zh-CN.md)；[显式 GPU 获取与 doctor](./gpu-context.zh-CN.md)已可用。[棋盘格计算／回读和 CLI PNG 输出](./builtin-checker.zh-CN.md)已实现，材质解析和图渲染仍待实现。所有软件包均禁用发布。

[rust-toolchain.toml](../rust-toolchain.toml)固定使用 Rust 1.98.1、edition 2024、rustfmt 和 Clippy。通过 [rustup](https://rustup.rs/) 安装 Rust，并准备原生 Rust 链接器／工具链：macOS 使用 Xcode Command Line Tools，Linux 使用 C 链接器，Windows 使用 Visual Studio C++ Build Tools。在本仓库运行 Cargo 时，会按需安装固定工具链。

## 可用命令

```bash
cargo xtask check
cargo xtask fmt
cargo xtask clippy
cargo xtask test
cargo xtask test-core
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

PR-004 唯一允许的直接依赖关系如下，包括构建、开发、可选和特定目标依赖：

| 软件包 | 允许的依赖 |
| --- | --- |
| `mixture-core` | 运行时使用 `serde`；`serde_json` 仅用作开发依赖 |
| `mixture-wgpu` | 工作区内的 `mixture-core`、`wgpu`、`serde`、`half`；仅开发时使用 `pollster`、`serde_json`、`naga` |
| `mixture-cli` | 工作区内的 `mixture-core`、`mixture-wgpu`、`pollster`、`serde`、`serde_json`、`png` |
| `xtask` | `pulldown-cmark`、`serde_json`、`png` |

核心使用 `serde` 序列化强类型诊断和限制；仅在开发时使用 `serde_json` 编码测试快照和公共示例。工具使用 `pulldown-cmark` 解析 Markdown，使用 `serde_json` 读取 Cargo 元数据。只有 `mixture-wgpu` 直接依赖 `wgpu`，其 `serde` 用于编码能力报告。`pollster` 在 CLI／测试边界驱动异步获取，`serde_json` 用于 CLI 报告和测试断言。`half` 解码 GPU 半精度回读，开发依赖 `naga` 在无 GPU 环境验证 WGSL。CLI 的 `png` 编码图像，工具的 `png` 解码图像用于基准比较。核心仍不依赖 GPU。原生后端特性策略见 [GPU 指南](./gpu-context.zh-CN.md)。全部已解析依赖版本记录在 [Cargo.lock](../Cargo.lock) 中。

[依赖检查](../xtask/src/dependencies.rs)约束四个 crate 的边界、双许可证元数据、禁用发布，以及上述直接依赖允许列表。它检查架构和范围，不是漏洞数据库检查或传递依赖许可证审计。引入依赖时，在负责该依赖的实施 PR 中扩展策略并说明必要性，保持[架构文档](../ARCHITECTURE.zh-CN.md)要求的方向。

## CLI 行为与后续工作

可执行程序名为 `mixture`。帮助／版本返回 `0`。`doctor` 验证真实棋盘格计算／回读，成功返回 `0` 和 `healthy`；显式 `--skip-probe` 返回 `unverified`。获取／探针失败返回 `1` 和 `unhealthy`。`render-builtin checker` 写入 PNG 后返回 `0`，运行失败返回 `1`，尺寸／预算无效返回 `2`。JSON 模式向 stdout 写入一份报告，人类可读模式包含相同的策略和能力证据。无效选项、缺少命令，以及尚未实现的 `validate`／`inspect`／`render` 返回 `2`，说明写入 stderr。输出 I/O 失败返回 `1`。见 [doctor 用法](./gpu-context.zh-CN.md)和[共享退出码策略](./diagnostics.zh-CN.md)。

节点、执行计划、材质和基准相关 xtask 命令尚未实现，调用会返回错误。引入顺序见[初始 PR 实施计划](../INITIAL_PRS.zh-CN.md)。

`gpu-smoke` 显式访问 GPU 硬件或配置的软件适配器，将报告保存到 `tmp/gpu-smoke/`。它不属于 `check` 和普通工作区测试。适配器策略变量、固定 SwiftShader 准备方式和本地证据见 [GPU 指南](./gpu-context.zh-CN.md)。

## CI 与里程碑证据

[非 GPU CI](../.github/workflows/ci.yml)在 Linux、macOS 和 Windows 上运行相同的锁定依赖命令，不需要 GPU 或显示器。本地通过仅证明当前主机环境；只有配置的 CI 矩阵在远端仓库实际通过后，才能满足 M0 跨平台验收条件。

[独立 GPU CI](../.github/workflows/gpu-smoke.yml)构建固定 SwiftShader Vulkan 适配器并运行计算／回读、PNG／基准比较和 GPU 回归测试，远端结果仍待获取。本地 Metal 与固定 SwiftShader Vulkan 证据记录于[棋盘格指南](./builtin-checker.zh-CN.md)。

智能体指南中尚未实现的运行时模块仍是未来职责地图。当前编译根模块为 [core](../crates/mixture-core/src/lib.rs)、[wgpu](../crates/mixture-wgpu/src/lib.rs)、[CLI](../crates/mixture-cli/src/main.rs) 和 [xtask](../xtask/src/main.rs)。棋盘格已有真实 GPU 生成的夹具；图／材质示例目录仍包含范围说明。

已实现的核心模块为[诊断](../crates/mixture-core/src/error.rs)和[限制](../crates/mixture-core/src/limits.rs)。[Rust 示例](../crates/mixture-core/examples/diagnostics.rs)可以运行；根目录的材质示例目录仍为 `.mix` 工作预留。

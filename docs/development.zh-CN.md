# 开发指南

[English](./development.md) | 简体中文

## 基础工程状态

仓库实现了[实施计划](../INITIAL_PRS.zh-CN.md)中的 M0 / PR-001。
它包含三个产品 crate 边界和私有仓库工具。产品 crate 尚未暴露材质或 GPU 运行时 API。所有软件包均禁用发布。

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
cargo run --locked -p mixture-cli -- --help
cargo run --locked -p mixture-cli -- --version
```

[Cargo 别名](../.cargo/config.toml)使用 `--locked` 启动 xtask。所有会解析依赖的嵌套 Cargo 命令也使用 `--locked`。格式化不解析依赖。首次运行下载已锁定的工具依赖，后续验证可以使用 Cargo 缓存。

`check` 按顺序运行：

1. `cargo fmt --all -- --check`。
2. 基于 `cargo metadata` 的 M0 依赖策略检查。
3. 覆盖工作区全部目标和特性的 Clippy，将警告视为错误。
4. 工作区测试，包括 CLI 集成测试、工具测试和文档测试。
5. 工作区 rustdoc 构建，将警告视为错误。
6. 基础离线 Markdown 链接检查，确认所引用的本地文件和目录存在。

任何子进程失败都会使整体检查失败。检查不会重写源文件、夹具或基准。Markdown 解析处理行内链接、引用式链接和图片，忽略代码示例；外部 URL、标题锚点、原始 HTML 链接和百分号编码本地路径不在基础检查支持范围内。使用普通相对路径；包含空格的路径使用尖括号。扫描时排除构建／输出目录。

使用 `cargo fmt --all` 应用格式化。变更依赖时有意识地更新 `Cargo.lock`，然后重新运行 `cargo xtask check`。Cargo 工作区和 lint 继承遵循 [Cargo 工作区参考文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)。

## 依赖策略

M0 唯一允许的直接依赖关系如下，包括构建、开发、可选和特定目标依赖：

| 软件包 | 允许的依赖 |
| --- | --- |
| `mixture-core` | 无 |
| `mixture-wgpu` | 工作区内的 `mixture-core` |
| `mixture-cli` | 工作区内的 `mixture-core`、`mixture-wgpu` |
| `xtask` | `pulldown-cmark`、`serde_json` |

产品没有第三方依赖。工具使用 `pulldown-cmark` 解析 Markdown，使用 `serde_json` 读取 Cargo 元数据；它们均不提供运行时文档处理或渲染行为。工具的传递依赖版本记录在 [Cargo.lock](../Cargo.lock) 中。

[依赖检查](../xtask/src/dependencies.rs)约束四个 crate 的边界、双许可证元数据、禁用发布，以及上述直接依赖允许列表。它检查架构和范围，不是漏洞数据库检查或传递依赖许可证审计。引入依赖时，在负责该依赖的实施 PR 中扩展策略并说明必要性，保持[架构文档](../ARCHITECTURE.zh-CN.md)要求的方向。

## CLI 行为与后续工作

可执行程序名为 `mixture`。只有帮助和版本命令返回成功。缺少参数、未知命令和计划中的运行时命令都返回退出码 2，并在 stderr 中解释当前开发阶段。不会返回虚假的 `doctor`、`validate`、`inspect` 或 `render` 结果。结构化运行时诊断由 PR-002 和后续 CLI 工作引入。

节点、着色器、执行计划、材质、基准和 GPU 相关 xtask 命令尚未在 M0 实现，调用会返回错误。引入顺序见[初始 PR 实施计划](../INITIAL_PRS.zh-CN.md)。

## CI 与里程碑证据

[非 GPU CI](../.github/workflows/ci.yml)在 Linux、macOS 和 Windows 上运行相同的锁定依赖命令，不需要 GPU 或显示器。本地通过仅证明当前主机环境；只有配置的 CI 矩阵在远端仓库实际通过后，才能满足 M0 跨平台验收条件。

智能体指南中的运行时模块列表是未来职责地图。当前编译根模块为 [core](../crates/mixture-core/src/lib.rs)、[wgpu](../crates/mixture-wgpu/src/lib.rs)、[CLI](../crates/mixture-cli/src/main.rs) 和 [xtask](../xtask/src/main.rs)。空的夹具和示例目录包含范围说明，没有伪造材质资产。

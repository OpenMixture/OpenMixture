# mixture-cli

[English](./README.md) | 简体中文

`mixture` 可执行程序是公开 `mixture-core`、`mixture-asset` 和 `mixture-wgpu` 库上的薄 I/O 与报告层。支持 `validate`、`inspect --plan`、`doctor`、`render`、`render-builtin checker` 和 `asset pack|inspect|render`，提供人类／JSON 报告及显式适配器选择。

本包处于 pre-alpha，禁用发布，版本以工作区清单为准，需要匹配版本的 core／asset／wgpu 包。归档包含[命令源码](./src/commands)、两种 README 语言及 [MIT](./LICENSE-MIT)／[Apache-2.0](./LICENSE-APACHE) 许可；输入 `.mix` 和输出目录属于调用者。PNG 顺序导出，I/O 失败时可能留下部分输出。

见仓库 [CLI 契约](https://github.com/OpenMixture/OpenMixture/blob/main/docs/cli-contract.zh-CN.md)、[兼容性策略](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.zh-CN.md)及[本地包验证](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.zh-CN.md)。`cargo xtask package-check` 从真实本地归档构建 CLI，并在生产仓库外运行；不安装、分发或发布二进制。

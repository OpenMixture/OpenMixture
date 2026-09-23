# mixture-cli

English | [简体中文](./README.zh-CN.md)

The `mixture` executable is thin I/O and reporting over the public `mixture-core`, `mixture-asset` and `mixture-wgpu` libraries. It supports `validate`, `inspect --plan`, `doctor`, `render`, `render-builtin checker` and `asset pack|inspect|render`, with human/JSON reports and explicit adapter selection.

The package is pre-alpha and publication is disabled; the workspace manifest owns its version. It requires matching core/asset/wgpu packages. This archive contains the [command source](./src/commands), both README languages and [MIT](./LICENSE-MIT)/[Apache-2.0](./LICENSE-APACHE) licenses; input `.mix` files and output directories belong to callers. PNG export is sequential and may leave partial output on I/O failure.

See the repository's [CLI contract](https://github.com/OpenMixture/OpenMixture/blob/main/docs/cli-contract.md), [compatibility policy](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.md) and [local package verification](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.md). `cargo xtask package-check` builds the CLI from the actual local archive and runs it outside the producing repository. It does not install, distribute or publish a binary.

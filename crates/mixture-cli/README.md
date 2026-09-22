# mixture-cli

English | [简体中文](./README.zh-CN.md)

**M6B-04 (2026-09-22):** [CLI/browser asset adapters](../../docs/m6b-04-adapters.md) implement explicit file workflows and bounded public `inspectPackage` / `renderPackage` APIs. Source versions remain unpublished; combined qualification belongs to M6B-05. Earlier status statements below are historical.

The `mixture` executable is thin I/O and reporting over the public `mixture-core` and `mixture-wgpu` libraries. It supports `validate`, `inspect --plan`, `doctor`, `render` and `render-builtin checker`, with human/JSON reports and explicit adapter selection.

The local 0.1.0 package is pre-alpha and publication is disabled. It requires matching core/wgpu packages. This archive contains the [command source](./src/commands), both README languages and [MIT](./LICENSE-MIT)/[Apache-2.0](./LICENSE-APACHE) licenses; input `.mix` files and output directories belong to callers. PNG export is sequential and may leave partial output on I/O failure.

See the repository's [CLI contract](https://github.com/OpenMixture/OpenMixture/blob/main/docs/cli-contract.md), [compatibility policy](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.md) and [local package verification](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.md). `cargo xtask package-check` builds the CLI from the actual local archive and runs it outside the producing repository. It does not install, distribute or publish a binary.

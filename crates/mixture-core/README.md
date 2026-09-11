# mixture-core

English | [简体中文](./README.zh-CN.md)

Pure Rust `.mix` v1 decoding, validation and deterministic compilation to `RenderPlan`. This crate has no GPU, CLI or image dependency. Start with the complete public API examples in [src/lib.rs](./src/lib.rs) or generated Rustdoc.

The local 0.1.0 package is pre-alpha and publication remains disabled. Eleven versioned node contracts are implemented; document version and plan version remain 1. Reject unsupported inputs explicitly. See the repository's [compatibility policy](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.md) and [local package verification](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.md).

Package contents include source, both README languages and [MIT](./LICENSE-MIT)/[Apache-2.0](./LICENSE-APACHE) licenses. Repository integration fixtures and material evidence are not consumer runtime assets. `cargo xtask package-check` in the repository verifies actual archives in an isolated local workspace; it does not publish packages or claim registry availability.

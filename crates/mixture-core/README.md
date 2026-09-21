# mixture-core

English | [简体中文](./README.zh-CN.md)

Pure Rust `.mix` v1 decoding, validation and deterministic compilation to `RenderPlan`. This crate has no GPU, CLI or image dependency. Start with the complete public API examples in [src/lib.rs](./src/lib.rs) or generated Rustdoc.

The local 0.3.0 package is pre-alpha and publication remains disabled. Thirteen Core node contracts are implemented; document version remains 1 and plan version is 2. Image upload/execution is still pending. Reject unsupported inputs explicitly. See the repository's [compatibility policy](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.md) and [local package verification](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.md).

Package contents include source, both README languages and [MIT](./LICENSE-MIT)/[Apache-2.0](./LICENSE-APACHE) licenses. Repository integration fixtures and material evidence are not consumer runtime assets. `cargo xtask package-check` in the repository verifies actual archives in an isolated local workspace; it does not publish packages or claim registry availability.

M6A-02 adds Core image preparation and plan v2. GPU image execution is not available until the M6A-03 upload path. See the [implementation boundary](https://github.com/OpenMixture/OpenMixture/blob/main/docs/m6a-02-core-resources.md).

M6A-04 implements browser resource requests and synchronous Core snapshots in the unpublished candidate; see [browser resources](https://github.com/OpenMixture/OpenMixture/blob/main/docs/m6a-04-browser-resources.md). Rust `prepare_from`/`AdapterImageBinding`/`ImageData` add only synchronous byte adaptation; Core still owns validation, budgets and hashes.

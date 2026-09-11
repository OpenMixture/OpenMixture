# mixture-wgpu

English | [简体中文](./README.zh-CN.md)

The sole Mixture pixel executor: explicit `GpuContext` ownership, typed `RenderPlan` execution, nine embedded WGSL kernels, owned RGBA8 outputs and structured GPU diagnostics. Public examples are in [src/lib.rs](./src/lib.rs); acquisition never implies a healthy compute/readback probe.

The local 0.1.0 package remains pre-alpha with publication disabled. It requires the matching `mixture-core` package and exposes wgpu 30 types through documented advanced handles and limits. Device loss does not trigger recovery; OOM is classified from typed errors. The caller owns scheduling and aggregate CPU output retention. See the repository's [compatibility policy](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.md).

All runtime shaders live in [shaders/nodes](./shaders/nodes) and are embedded during compilation; no runtime path into the repository is required. Three small unit-test inputs under [src/testdata](./src/testdata) are checked against the canonical repository fixtures by `package-check`. Source, shaders, tests embedded in source, both README languages and [MIT](./LICENSE-MIT)/[Apache-2.0](./LICENSE-APACHE) licenses ship in the local archive. Repository integration tests and material goldens stay outside it.

Use repository `cargo xtask package-check` for isolated CPU/package checks and explicit `cargo xtask gpu-smoke` for packaged GPU/CLI consumption. Default tests do not acquire a GPU. See [local package verification](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.md); packages are not published by these commands.

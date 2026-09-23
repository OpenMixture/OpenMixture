# mixture-wgpu

English | [简体中文](./README.zh-CN.md)

The sole Mixture pixel executor: explicit `GpuContext` ownership, typed `RenderPlan` execution, one embedded WGSL kernel per `KernelId`, owned RGBA8 outputs and structured GPU diagnostics. Public examples are in [src/lib.rs](./src/lib.rs); acquisition never implies a healthy compute/readback probe.

The package remains pre-alpha with publication disabled; the workspace manifest owns its version. It requires the matching `mixture-core` package and exposes wgpu 30 types through documented advanced handles and limits. Device loss does not trigger recovery; OOM is classified from typed errors. The caller owns scheduling and aggregate CPU output retention. See the repository's [compatibility policy](https://github.com/OpenMixture/OpenMixture/blob/main/docs/compatibility.md).

Runtime node shaders live in [shaders/nodes](./shaders/nodes), with shared [half-precision storage conversion](./shaders/precision.wgsl) and are embedded during compilation; no runtime path into the repository is required. Three small unit-test inputs under [src/testdata](./src/testdata) are checked against the canonical repository fixtures by `package-check`. Source, shaders, tests embedded in source, both README languages and [MIT](./LICENSE-MIT)/[Apache-2.0](./LICENSE-APACHE) licenses ship in the local archive. Repository integration tests and material goldens stay outside it.

Use repository `cargo xtask package-check` for isolated CPU/package checks and explicit `cargo xtask gpu-smoke` for packaged GPU/CLI consumption. Default tests do not acquire a GPU. See [local package verification](https://github.com/OpenMixture/OpenMixture/blob/main/docs/package-consumption.md); packages are not published by these commands.

The `wasm32-unknown-unknown` target enables only browser WebGPU execution through `wgpu/webgpu`; `BackendPreference::Auto` selects `BROWSER_WEBGPU` on that target. Native Auto keeps Vulkan/Metal/Dx12 selection. No WebGL, noop or alternate pixel executor is enabled. The browser-facing package performs secure-context/API preflight; unavailable or redacted adapter identity remains backend-supplied evidence.

Browser submission and readback await queue/map callbacks and balanced asynchronous error scopes, yielding to the event loop. Native polling and individual 30-second waits remain unchanged. Browser operations have no added timeout or cancellation promise: tab termination or undelivered platform events can prevent completion. Wall timings use `performance.now()` through `web-time`, including browser scheduling and timer rounding, not GPU timestamp queries. Browser wgpu errors retain typed OOM classification and driver/cause text without claiming a native `Send + Sync` source chain.

Compile this executor with `cargo check --locked -p mixture-wgpu --target wasm32-unknown-unknown` after installing that Rust target. This only verifies compilation; actual browser rendering, odd-width readback, result lifetime and device-loss tests belong to the independent consumer and M5 evidence. See the repository [browser SDK contract](https://github.com/OpenMixture/OpenMixture/blob/main/docs/browser-sdk.md) for the current implemented boundary and remaining gates.

M6A-02 adds Core image preparation and plan v2. M6A-03 adds `Renderer::render_prepared(&PreparedRender)` for Native image uploads and execution. See the [implementation boundary](https://github.com/OpenMixture/OpenMixture/blob/main/docs/m6a-02-core-resources.md).

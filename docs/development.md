# Development

English | [简体中文](./development.zh-CN.md)

## Foundation, diagnostics, and GPU context status

This repository implements PR-001 through PR-005 from [the implementation train](../INITIAL_PRS.md).
It contains three product crate boundaries and private repository tooling. Core provides [diagnostics and safety-limit APIs](./diagnostics.md); [explicit GPU acquisition and doctor](./gpu-context.md) are available. [Checker compute/readback and CLI PNG output](./builtin-checker.md) are implemented. [Strict .mix decoding/validation](./file-format.md) and [six node contracts](./node-contracts.md) are implemented. Graph compilation/rendering remain planned. All packages have publication disabled.

Rust 1.98.1, edition 2024, rustfmt, and Clippy are pinned in [rust-toolchain.toml](../rust-toolchain.toml). Install Rust through [rustup](https://rustup.rs/) and use a native Rust linker/toolchain (Xcode Command Line Tools on macOS, a C linker on Linux, or Visual Studio C++ Build Tools on Windows). Running Cargo in this repository installs the pinned toolchain when needed.

## Available commands

```bash
cargo xtask check
cargo xtask fmt
cargo xtask clippy
cargo xtask test
cargo xtask test-core
cargo xtask test-format
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
cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png
cargo xtask gpu-smoke
```

The [Cargo alias](../.cargo/config.toml) launches xtask with `--locked`. Every nested Cargo command that resolves dependencies also uses `--locked`. Formatting does not resolve dependencies. The first run downloads the locked tooling dependencies; subsequent verification can use the Cargo cache.

`check` runs, in order:

1. `cargo fmt --all -- --check`.
2. The current dependency policy against `cargo metadata`.
3. Clippy on all workspace targets and features with warnings denied.
4. Workspace tests, including CLI integration tests, tooling tests, and doctests.
5. Workspace rustdoc with warnings denied.
6. Basic offline Markdown links to existing local files and directories.

Every failed subprocess fails the enclosing check. Checks do not rewrite sources, fixtures, or baselines. Markdown parsing handles inline links, reference links, and images while ignoring code examples; external URLs, heading anchors, raw HTML links, and percent-encoded local paths are outside this basic check. Use ordinary relative paths, or angle brackets for paths with spaces. Build/output directories are excluded from discovery.

Use `cargo fmt --all` to apply formatting. Update `Cargo.lock` deliberately when changing dependencies, then rerun `cargo xtask check`. Cargo workspace and lint inheritance follow the [Cargo workspace reference](https://doc.rust-lang.org/cargo/reference/workspaces.html).

## Dependency policy

At PR-005 the only allowed direct dependency edges, including build, dev, optional, and target-specific dependencies, are:

| Package | Allowed dependencies |
| --- | --- |
| `mixture-core` | Runtime `serde` and `serde_json` with `float_roundtrip` |
| `mixture-wgpu` | Workspace `mixture-core`, `wgpu`, `serde`, `half`; dev-only `pollster`, `serde_json`, `naga` |
| `mixture-cli` | Workspace `mixture-core`, `mixture-wgpu`, `pollster`, `serde`, `serde_json`, `png` |
| `xtask` | `pulldown-cmark`, `serde_json`, `png` |

Core uses `serde` for typed source data, diagnostics, and limits; PR-005 promotes the already-locked `serde_json` to a runtime dependency for strict bounded decoding and deterministic serialization. Its `float_roundtrip` feature fixes a reproduced one-bit numeric drift across source round trips; no new package or dependency version is needed. The tooling uses `pulldown-cmark` to parse Markdown and `serde_json` to read Cargo metadata. Only `mixture-wgpu` directly depends on `wgpu`; `serde` encodes capability reports. `pollster` drives async acquisition at the CLI/test boundary, while `serde_json` formats CLI reports and test assertions. `half` decodes GPU half-float readback; dev-only `naga` validates WGSL without a GPU. CLI `png` encodes images, and tooling `png` decodes them for golden comparison. Core remains GPU-free. The native backend feature policy is documented in [the GPU guide](./gpu-context.md). All resolved dependency versions are recorded in [Cargo.lock](../Cargo.lock).

[The dependency check](../xtask/src/dependencies.rs) enforces the four-crate boundary, dual license metadata, disabled publication, and this direct-dependency allowlist. It is an architecture/scope check, not a vulnerability database or transitive license audit. Expand the policy with a dependency's owning implementation PR and explain why the new dependency is necessary; retain the direction required by [ARCHITECTURE.md](../ARCHITECTURE.md).

## CLI behavior and later work

The executable is named `mixture`. Help/version return `0`. `doctor` verifies actual checker compute/readback and returns `0` with `healthy`; explicit `--skip-probe` returns `unverified`. Acquisition/probe failures return `1` with `unhealthy`. `render-builtin checker` returns `0` after writing PNG, `1` for operational failure, or `2` for invalid dimensions/budgets. JSON mode writes one report to stdout; human mode includes the same policy and capability evidence. `validate` returns `0` for valid source, `2` for invalid input, and `1` for file/report I/O failure; it never initializes a GPU. Invalid options, missing commands, and unimplemented `inspect`/`render` return `2` on stderr. Output I/O failures return `1`. See [doctor usage](./gpu-context.md) and the [shared exit policy](./diagnostics.md).

`test-format` runs core format/validation/registry tests and CLI validation tests. Node-pixel, plan, material, and golden xtask commands are not implemented yet and return an error. Follow [INITIAL_PRS.md](../INITIAL_PRS.md) for their introduction order.

`gpu-smoke` explicitly accesses GPU hardware or a configured software adapter and saves reports under `tmp/gpu-smoke/`. It is excluded from `check` and ordinary workspace tests. Its adapter policy variables, pinned SwiftShader setup, and local evidence are documented in [the GPU guide](./gpu-context.md).

## CI and milestone evidence

[Non-GPU CI](../.github/workflows/ci.yml) runs the same locked command on Linux, macOS, and Windows. No GPU or display is required. A local pass proves the host environment only; the M0 cross-platform gate remains pending until the configured CI matrix has actually passed on the remote repository.

[Dedicated GPU CI](../.github/workflows/gpu-smoke.yml) builds a pinned SwiftShader Vulkan adapter and runs compute/readback, PNG/golden checks, and GPU regressions; its remote result remains pending. Local Metal and pinned SwiftShader Vulkan evidence is recorded in [the checker guide](./builtin-checker.md).

Unimplemented runtime modules in the agent guide remain a future ownership map. Current compilation roots are [core](../crates/mixture-core/src/lib.rs), [wgpu](../crates/mixture-wgpu/src/lib.rs), [CLI](../crates/mixture-cli/src/main.rs), and [xtask](../xtask/src/main.rs). The checker has a real GPU-generated fixture; [format fixtures](../fixtures/format/README.md) and [checker.mix](../examples/checker.mix) now exercise source validation. Material rendering examples remain future work.

Implemented core modules are [document decoding](../crates/mixture-core/src/document.rs), [validation](../crates/mixture-core/src/validation.rs), [registry](../crates/mixture-core/src/registry.rs), [node contracts](../crates/mixture-core/src/nodes/), [diagnostics](../crates/mixture-core/src/error.rs), and [limits](../crates/mixture-core/src/limits.rs). The [Rust diagnostics example](../crates/mixture-core/examples/diagnostics.rs) and crate doctests exercise public APIs.

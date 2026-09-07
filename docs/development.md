# Development

English | [简体中文](./development.zh-CN.md)

## Foundation, diagnostics, and GPU context status

This repository implements PR-001 through PR-003 from [the implementation train](../INITIAL_PRS.md).
It contains three product crate boundaries and private repository tooling. Core provides [diagnostics and safety-limit APIs](./diagnostics.md); [explicit GPU acquisition and doctor](./gpu-context.md) are available. Material parsing and compute/readback remain unimplemented. All packages have publication disabled.

Rust 1.98.1, edition 2024, rustfmt, and Clippy are pinned in [rust-toolchain.toml](../rust-toolchain.toml). Install Rust through [rustup](https://rustup.rs/) and use a native Rust linker/toolchain (Xcode Command Line Tools on macOS, a C linker on Linux, or Visual Studio C++ Build Tools on Windows). Running Cargo in this repository installs the pinned toolchain when needed.

## Available commands

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

At PR-003 the only allowed direct dependency edges, including build, dev, optional, and target-specific dependencies, are:

| Package | Allowed dependencies |
| --- | --- |
| `mixture-core` | `serde` at runtime; `serde_json` only as a dev dependency |
| `mixture-wgpu` | Workspace `mixture-core`, `wgpu`, `serde`; dev-only `pollster`, `serde_json` |
| `mixture-cli` | Workspace `mixture-core`, `mixture-wgpu`, `pollster`, `serde_json` |
| `xtask` | `pulldown-cmark`, `serde_json` |

Core uses `serde` to serialize typed diagnostics and limits; its dev-only `serde_json` dependency encodes test snapshots and the public example. The tooling uses `pulldown-cmark` to parse Markdown and `serde_json` to read Cargo metadata. Only `mixture-wgpu` directly depends on `wgpu`; `serde` encodes capability reports. `pollster` drives async acquisition at the CLI/test boundary, while `serde_json` formats CLI reports and test assertions. Core remains GPU-free. The native backend feature policy is documented in [the GPU guide](./gpu-context.md). All resolved dependency versions are recorded in [Cargo.lock](../Cargo.lock).

[The dependency check](../xtask/src/dependencies.rs) enforces the four-crate boundary, dual license metadata, disabled publication, and this direct-dependency allowlist. It is an architecture/scope check, not a vulnerability database or transitive license audit. Expand the policy with a dependency's owning implementation PR and explain why the new dependency is necessary; retain the direction required by [ARCHITECTURE.md](../ARCHITECTURE.md).

## CLI behavior and later work

The executable is named `mixture`. Help/version return `0`. `doctor` acquires a real context and returns `0` with `unverified`, or `1` with an `unhealthy` acquisition report. JSON mode writes one report to stdout; human mode includes the same policy and capability evidence. Invalid options, missing commands, and unimplemented `validate`/`inspect`/`render` return `2` on stderr. Output I/O failures return `1`. See [doctor usage](./gpu-context.md) and the [shared exit policy](./diagnostics.md).

Node, shader, plan, material, and golden xtask commands are not implemented yet and return an error. Follow [INITIAL_PRS.md](../INITIAL_PRS.md) for their introduction order.

`gpu-smoke` explicitly accesses GPU hardware or a configured software adapter and saves reports under `tmp/gpu-smoke/`. It is excluded from `check` and ordinary workspace tests. Its adapter policy variables, pinned SwiftShader setup, and local evidence are documented in [the GPU guide](./gpu-context.md).

## CI and milestone evidence

[Non-GPU CI](../.github/workflows/ci.yml) runs the same locked command on Linux, macOS, and Windows. No GPU or display is required. A local pass proves the host environment only; the M0 cross-platform gate remains pending until the configured CI matrix has actually passed on the remote repository.

[Dedicated GPU CI](../.github/workflows/gpu-smoke.yml) builds a pinned SwiftShader Vulkan adapter and runs acquisition smoke; its remote result remains pending. No computation or readback health is claimed.

Unimplemented runtime modules in the agent guide remain a future ownership map. Current compilation roots are [core](../crates/mixture-core/src/lib.rs), [wgpu](../crates/mixture-wgpu/src/lib.rs), [CLI](../crates/mixture-cli/src/main.rs), and [xtask](../xtask/src/main.rs). Empty fixture and example directories contain scope readmes instead of fabricated material assets.

The implemented core modules are [diagnostics](../crates/mixture-core/src/error.rs) and [limits](../crates/mixture-core/src/limits.rs). The [Rust example](../crates/mixture-core/examples/diagnostics.rs) is runnable; the root material-example directory remains reserved for `.mix` work.

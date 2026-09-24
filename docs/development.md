# Development

English | [简体中文](./development.zh-CN.md)

**Current status:** see [release status](./release.md) for integrated features, published versions and hardware qualification scope. Earlier dated records describe their original checkpoints.

## Foundation, diagnostics, and GPU context status

This repository implements PR-001 through PR-010 locally from [the initial train](../INITIAL_PRS.md). Ceramic, leather and wood appearance are accepted by the user. The [M3 review](./m3-review.md) and [reproduction helpers](./reviews/m3/README.md) record local acceptance, release performance and native-consumer gaps. [M4 PR-011](../M4_PRS.md) now implements the [independent public Rust consumer](./native-sdk.md) and CPU-only `test-consumer`, with explicit GPU ownership checks and 1K release evidence. PR-012 adds the [CLI report/exit contract](./cli-contract.md), complete human diagnostic context and independent CLI process tests. PR-013 adds [GPU failure reasons, loss lifetime and cleanup](./gpu-failures.md). PR-014 adds [latest requests and bounded retention](./stale-results.md). PR-015 adds [isolated local package verification](./package-consumption.md) through `package-check`, plus [compatibility](./compatibility.md) and the [M4 exit/release assessment](./release.md). M4 acceptance now includes [three-platform CPU and Linux SwiftShader CI](./evidence/remote-ci/README.md). The remote gates and [bounded M5 browser acceptance](./evidence/m5-05/README.md) are complete. The [first npm Alpha](./evidence/npm-alpha/README.md) is published; Rust crates remain unpublished. Current priorities follow the [Post-Alpha roadmap](../ROADMAP.md).
It contains five product crate boundaries, including the thin WASM binding, and private repository tooling. Core provides [diagnostics and safety-limit APIs](./diagnostics.md); [explicit GPU acquisition and doctor](./gpu-context.md) are available. [Checker compute/readback and CLI PNG output](./builtin-checker.md) are implemented. [Strict .mix decoding/validation](./file-format.md) and [thirteen versioned node types](./node-contracts.md) are implemented. [Deterministic compilation and plan inspection](./render-plan.md) are implemented. [Graph execution](./graph-rendering.md) and three PNG examples are implemented. Rust crates retain publication disabled; the browser npm package has its own recorded release.

Rust 1.98.1, edition 2024, rustfmt, and Clippy are pinned in [rust-toolchain.toml](../rust-toolchain.toml). Install Rust through [rustup](https://rustup.rs/) and use a native Rust linker/toolchain (Xcode Command Line Tools on macOS, a C linker on Linux, or Visual Studio C++ Build Tools on Windows). Running Cargo in this repository installs the pinned toolchain when needed.

## Available commands

The `cargo xtask` alias builds its launcher under `target/xtask-runner`. Nested workspace commands keep their normal target directory, so tests can rebuild the integration-test executable without replacing a running `xtask.exe` on Windows. Use the alias when running repository checks.

```bash
cargo xtask check
cargo xtask fmt
cargo xtask clippy
cargo xtask test
cargo xtask test-core
cargo xtask test-format
cargo xtask test-plan
cargo xtask test-consumer
cargo xtask package-check
cargo run --locked -p mixture-cli -- inspect examples/checker.mix --plan --json
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
cargo xtask doc
cargo xtask deps
cargo xtask evidence
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
cargo xtask test-node checker
cargo xtask test-material glazed-ceramic
cargo xtask golden check
cargo xtask trace-2k
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png
cargo xtask gpu-smoke
```

The [Cargo alias](../.cargo/config.toml) launches xtask with `--locked`. Build/test/doc commands and producer dependency resolution use `--locked`. The sole package-staging exception is an offline metadata pass to normalize a disposable lock; external versions/sources/checksums are checked against the committed pins before all subsequent `--offline --locked` verification. Committed locks are never rewritten by the checker. Formatting does not resolve dependencies. The first run downloads the locked tooling dependencies; subsequent verification can use the Cargo cache.

`check` runs, in order:

1. `cargo fmt --all -- --check`.
2. The current dependency policy against `cargo metadata`.
3. The evidence growth guard: tracked raw run output, evidence archives, per-run CLI folders and files above 4 MiB must be listed in [retention exceptions](./evidence/retention-exceptions.txt); see [evidence retention](./evidence-policy.md).
4. Clippy on all workspace targets and features with warnings denied.
5. Workspace tests, including CLI integration tests, tooling tests, and doctests.
6. `test-consumer`: separate Cargo metadata, formatting, Clippy, unit/process tests, build and public-Rust CPU invocation, followed by a built CLI and the explicit independent CPU contract test in a fresh working directory.
7. `package-check`: actual local archives, isolated source/lock resolution, unit tests/Rustdoc, missing-shader rejection and independent CPU consumption.
8. Workspace rustdoc with warnings denied.
9. Basic offline Markdown links to existing local files and directories.

Every unexpected subprocess failure fails the enclosing check; the explicit missing-shader probe must fail and is validated as a negative test. Checks do not rewrite sources, fixtures, or baselines. Markdown parsing handles inline links, reference links, and images while ignoring code examples; external URLs, heading anchors, raw HTML links, and percent-encoded local paths are outside this basic check. Use ordinary relative paths, or angle brackets for paths with spaces. Build/output directories are excluded from discovery.

Use `cargo fmt --all` to apply formatting. Update `Cargo.lock` deliberately when changing dependencies, then rerun `cargo xtask check`. Cargo workspace and lint inheritance follow the [Cargo workspace reference](https://doc.rust-lang.org/cargo/reference/workspaces.html).

## Dependency policy

For the M5 browser start, the only allowed direct dependency edges in the product/tooling workspace, including build, dev, optional, and target-specific dependencies, are:

| Package | Allowed dependencies |
| --- | --- |
| `mixture-core` | Runtime `serde`, `serde_json` with `float_roundtrip`, and `sha2` |
| `mixture-asset` | Core and existing `serde`, `serde_json`, `sha2`; CPU-only byte codec |
| `mixture-wgpu` | Workspace `mixture-core`, `wgpu`, `serde`, `half`; browser-target-only `futures-channel`, `web-time`; dev-only `pollster`, `serde_json`, `naga` |
| `mixture-cli` | Workspace `mixture-core`, `mixture-asset`, `mixture-wgpu`, `pollster`, `serde`, `serde_json`, `png` |
| `xtask` | `pulldown-cmark`, `serde_json`, `png`, `serde`, `sha2` |
| `mixture-wasm` | Workspace `mixture-core`, `mixture-asset`, `mixture-wgpu`, `serde`, `serde_json`; `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `serde-wasm-bindgen` |

Core uses `serde` for typed source data, diagnostics, and limits; PR-005 promotes the already-locked `serde_json` to a runtime dependency for strict bounded decoding and deterministic serialization. Its `float_roundtrip` feature fixes a reproduced one-bit numeric drift across source round trips; no new package or dependency version is needed. PR-006 adds `sha2` for stable SHA-256 plan hashing; its dependency closure is added to the lockfile without upgrading existing packages. The tooling uses `pulldown-cmark` to parse Markdown and `serde_json` to read Cargo metadata. Only `mixture-wgpu` directly depends on `wgpu`; `serde` encodes capability reports. `pollster` drives async acquisition at the CLI/test boundary, while `serde_json` formats CLI reports and test assertions. `half` decodes GPU half-float readback; dev-only `naga` validates WGSL without a GPU. CLI `png` encodes images, and tooling `png` decodes them for golden comparison. Core remains GPU-free. The native backend feature policy is documented in [the GPU guide](./gpu-context.md). PR-008 adds only existing workspace `serde` and `sha2` as direct tooling dependencies for strict acceptance records and candidate integrity, with no package/version additions or upgrades. All resolved dependency versions are recorded in [Cargo.lock](../Cargo.lock).

[The dependency check](../xtask/src/dependencies.rs) enforces the six-crate boundary, dual license metadata, disabled publication, and this direct-dependency allowlist. It is an architecture/scope check, not a vulnerability database or transitive license audit. Expand the policy with a dependency's owning implementation PR and explain why the new dependency is necessary; retain the direction required by [ARCHITECTURE.md](../ARCHITECTURE.md).

The [native consumer](../examples/native-consumer/Cargo.toml) is a separate one-package workspace with its own committed lockfile; `test-consumer` checks that boundary without joining the engine workspace. It uses source-path `mixture-core`/`mixture-asset`/`mixture-wgpu`, plus exactly pinned `pollster` and `serde_json` versions already used by the producer. PR-012 adds dev-only `png = "=0.18.1"` to decode completed CLI output files; its nine added lockfile entries use the same versions already resolved by the product. The product lockfile and dependency policy are unchanged. Path dependency and built-CLI verification do not prove package contents.

## CLI behavior and later work

The executable is named `mixture`. Help/version return `0`. `doctor` verifies actual checker compute/readback and returns `0` with `healthy`; explicit `--skip-probe` returns `unverified`. Acquisition/probe failures return `1` with `unhealthy`. `render-builtin checker` returns `0` after writing PNG, `1` for operational failure, or `2` for invalid dimensions/budgets. JSON mode writes one report to stdout; human mode includes the same policy and capability evidence. `validate` returns `0` for valid source, `2` for invalid input, and `1` for file/report I/O failure; it never initializes a GPU. `inspect --plan` returns `0` for compilation, `2` for invalid source/request, or `1` for file/report I/O failure; it also needs no GPU. `render` compiles before GPU acquisition and returns `0` after all requested PNGs, `2` for invalid source/request, and `1` for GPU/encoding/I/O failure. Invalid options and missing commands return `2` on stderr. Output I/O failures return `1`. See [doctor usage](./gpu-context.md) and the [shared exit policy](./diagnostics.md).

`test-material brick-paving` runs the release public Native matrix and package roundtrips, writing fresh evidence under `tmp/materials/brick-paving/`. It also records high-frequency stress and matched-adapter timing budgets. This property-based material gate is separate from the original three-material golden baseline workflow; browser and human PBR acceptance remain separate. See the [brick fixture guide](../fixtures/materials/brick-paving/README.md).

`test-format` runs core format/validation/registry tests and CLI validation tests. `test-plan` runs core plan/hash tests and CLI inspection tests. `test-node <id>` validates fixtures and explicitly runs the selected GPU node cases. `test-material <id>` and `golden check` now render explicit GPU cases and compare material gates without changing baselines. `golden update <id> --accept` separately consumes a reviewed software candidate and refuses CI; see [the complete workflow](./material-goldens.md).

`test-consumer` saves separate build/test/CPU reports and an invalidated-then-completed status under `tmp/consumer-check/`. It compiles the full GPU call path and native `Send` bounds, but does not initialize wgpu. It also builds the real CLI and explicitly runs 32 CPU contract invocations using consumer-owned files. The selected fresh CLI capture is linked by `cliEvidence` in the completion status; ordinary consumer Cargo tests keep the CLI contract tests ignored until this explicit invocation. See the [consumer guide](../examples/native-consumer/README.md).

`gpu-smoke` explicitly accesses GPU hardware or a configured software adapter and saves reports under `tmp/gpu-smoke/`, including the independent consumer in `native-consumer/`. The Rust consumer renders twice, drops its renderer/context, then validates owned bytes and metadata. PR-012 also runs 10 independent CLI invocations for doctor, successful PNGs and partial writes; raw streams and actual files remain under the linked CLI evidence directory. It is excluded from `check` and ordinary workspace tests. Its adapter policy variables, pinned SwiftShader setup, and local evidence are documented in [the GPU guide](./gpu-context.md).

GPU smoke runs independent Rust GPU tests with `--test-threads=1`. This keeps node rendering separate from unrelated deliberate device-destruction tests after a concurrent Linux SwiftShader test process terminated with SIGSEGV. Every test still runs, including independent-context checks inside individual tests. This harness policy does not certify arbitrary concurrent device destruction or diagnose the driver's crash cause.

The Linux GPU workflow caches the pinned SwiftShader source/build with an exact key covering OS, architecture, compiler/CMake/Ninja/libc versions and the workflow/setup script (which contains the source revision). Cache hits still verify the revision, configure/build and select the explicit ICD; smoke, package, material and 2K checks always execute. Successful driver builds are saved before testing, so a test failure does not force another 16–29 minute driver rebuild. No partial-key cache fallback is used.

## 2K resource evidence

`cargo xtask trace-2k` runs a separate explicit GPU workload; `check` and ordinary workspace tests do not run it. Configure the desired adapter through the same `MIXTURE_GPU_BACKEND`, `MIXTURE_GPU_SOFTWARE`, and optional `MIXTURE_GPU_EXPECT_ADAPTER` variables as `gpu-smoke`. Software execution requires Vulkan and the verified pinned SwiftShader source/loader setup from [the GPU guide](./gpu-context.md).

```bash
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 cargo xtask trace-2k
# After configuring the pinned SwiftShader source and loader:
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 cargo xtask trace-2k
```

The [trace task](../xtask/src/golden/trace.rs) first compiles every configured case of `glazed-ceramic`, `leather`, and `wood` at 2048×2048 with `baseColor`, `normal`, `roughness`, and `height` requested. All three fixtures must be present and valid. It selects the greatest estimated `peakBytes`, then pass count, lexical material name, default case first, and lexical case ID. It verifies `doctor` and renders that exact plan with the requested adapter. A rejected inspection stops before GPU work; the task does not increase safety limits.

The transient budget is **512 MiB (536,870,912 bytes) of peak live descriptors**, matching the default `SafetyLimits::transient_bytes`. Cumulative allocation across sequential readbacks may exceed the peak. [Core estimates](./render-plan.md) retain every pass texture and uniform through execution/readback, with one staging buffer alive at a time. Aliased channels still get separate readbacks. PR-010 measures this existing schedule; it adds no last-consumer release, compatible texture reuse, or pool. Such a lifetime change requires an over-budget or failed measured workload first.

`RenderReport.allocations` is the public [AllocationReport](../crates/mixture-wgpu/src/allocations.rs), serialized as `execution.allocations` by the CLI. It counts successful texture/uniform/staging descriptor creations and their cumulative, peak, live, released, and reused bytes. The trace requires these counts to agree with the selected plan, all requested readbacks to complete sequentially, `liveBytes` to finish at zero, `releasedBytes` to equal `cumulativeBytes`, and `reusedBytes` to remain zero. Release records the call to `destroy`; it does not prove that the driver or operating system immediately returned physical memory. Driver allocation granularity, shader/pipeline/bind-group memory, and CPU pixel/PNG buffers are excluded. Pipeline caches remain renderer-owned and are outside these per-call counters. Allocation observations do not change the core plan, its estimates, or its hash.

A successful run writes `selection.json`, every inspection report, `doctor.json`, `render.json`, four PNGs and their hashes, source/fixture hashes, exact render arguments and adapter settings, and `trace.json` under `tmp/trace-2k/<run>/`. `pipelineMs`, `executionMs`, `readbackMs`, and `totalMs` are finite CPU wall measurements; no GPU timestamp query is claimed. The task rejects changed source/fixture inputs or 1K baselines during the run. `latest-software.json` or `latest-hardware.json` records the latest attempt; failures retain available CLI JSON/stderr and `failure.json`. A passing trace proves this workload's bounded naive schedule. It neither creates nor updates a 2K golden and does not decide human material acceptance. Remote acceptance is established by the recorded CI runs, not by an isolated local trace.

## CI and milestone evidence

[Non-GPU CI](../.github/workflows/ci.yml) runs the same locked command on Linux, macOS, and Windows. No GPU or display is required. A local pass proves the host environment only. The [accepted remote matrix](./evidence/remote-ci/README.md) closes the M0 cross-platform gate for its recorded revisions and platforms.

[Dedicated GPU CI](../.github/workflows/gpu-smoke.yml) builds a pinned SwiftShader Vulkan adapter and runs compute/readback, node regressions, all three 1K material comparisons, and the largest-case 2K trace. It retains smoke, package, golden, and trace evidence even on failure. The [accepted Linux runs](./evidence/remote-ci/README.md) close the prescribed remote GPU gates; local Metal and pinned SwiftShader Vulkan evidence is separately recorded in [the checker guide](./builtin-checker.md). Both workflows run for PRs, pushes to `main`, and manual dispatch, with 30-day retention requested for new artifacts. Follow [repository governance](./governance.md) for required checks and [evidence retention](./evidence-policy.md) for accepted content.

The agent guide records module ownership and the implementation batches that introduced it. Current compilation roots are [core](../crates/mixture-core/src/lib.rs), [wgpu](../crates/mixture-wgpu/src/lib.rs), [CLI](../crates/mixture-cli/src/main.rs), and [xtask](../xtask/src/main.rs). The checker has a real GPU-generated fixture; [format fixtures](../fixtures/format/README.md) and [checker.mix](../examples/checker.mix) now exercise source validation. All three [M2 examples](../examples/README.md) now render requested channels.

Implemented core modules are [document decoding](../crates/mixture-core/src/document.rs), [validation](../crates/mixture-core/src/validation.rs), [registry](../crates/mixture-core/src/registry.rs), [node contracts](../crates/mixture-core/src/nodes/), [diagnostics](../crates/mixture-core/src/error.rs), and [limits](../crates/mixture-core/src/limits.rs). The [Rust diagnostics example](../crates/mixture-core/examples/diagnostics.rs) and crate doctests exercise public APIs.

PR-006 adds [the compiler](../crates/mixture-core/src/compiler.rs), [normalization and lowering](../crates/mixture-core/src/compiler/), [typed plan API](../crates/mixture-core/src/plan.rs), and [CLI inspect](../crates/mixture-cli/src/commands/inspect.rs). Plan snapshots live beside core tests; PR-007 [graph rendering](./graph-rendering.md) adds the executor, resources, kernel mapping/cache, and CLI render without new dependencies.

PR-009 adds no dependency or lockfile change. Run `test-node fractal-noise`, `test-node gradient-map`, `test-node height-to-normal` and `test-material leather` through `cargo xtask`; [leather review](../fixtures/materials/leather/review/README.md) is an optional external Blender consumer, not a Rust runtime dependency.

PR-010 also adds no dependency or lockfile change. Run `test-node transform-2d`, `test-node warp`, `test-material wood`, `golden check`, and `trace-2k` through `cargo xtask`. The public Renderer doctest compiles access to `AllocationReport`; focused GPU tests verify odd-width aliased readbacks, per-call counter reset, and release after a readback error.

PR-013 adds CPU classification and consumer-receipt guard tests. `test-consumer` compiles the independent `device_loss` test but leaves it ignored; explicit `gpu-smoke` executes its cold/warm destruction cases and verifies the fresh receipt referenced by `native-consumer/status.json` → `deviceLossEvidence`. [GPU failure reproduction](./gpu-failures.md) and [local evidence](./evidence/pr-013/README.md) distinguish typed synthetic OOM from real device destruction.

PR-014 keeps scheduling in `examples/native-consumer/src/latest.rs`, with a library target inside that existing example package. `test-consumer` runs six deterministic state/lifetime tests. Explicit `gpu-smoke` adds Rust `latest`, five generation-isolated CLI calls and the nine-kernel cache-bound GPU regression. The root consumer status points to `latestEvidence`; [reproduction and limits](./stale-results.md) explain the receipts and structural output bound.

PR-015 introduces no direct dependency or committed lockfile change. Product packages add exact peer version requirements, explicit include lists, paired READMEs and license copies. Three small wgpu unit inputs move to package-local include paths; `package-check` checks their byte identity against canonical fixtures. It requires a compatible `tar` and an external OS temporary directory, preserves raw package evidence under `tmp/package-check/`, and uses only an ignored compiler artifact cache under `target/package-consumer`. See [the full verification method](./package-consumption.md).

M5 adds `futures-channel` for nonblocking browser callbacks, `web-time` for browser timing, and the wasm-bindgen/serde bridge for typed transport. These are platform/binding dependencies; they do not enter core or add a pixel executor. Native Cargo package checks still cover the three native packages; browser npm verification is separate. See the [browser build and verification guide](./browser-runtime.md).

[Browser material measurement and comparison](./browser-materials.md) documents `browser-material-measure` and `browser-material-check`.

`cargo xtask browser-quality-calibrate <fresh-output>` verifies 40 independent perturbation controls and writes comparison sheets plus a policy-bound report; see [browser quality](./browser-quality.md). It does not execute a material graph or qualify a browser.

## Independent browser SDK consumer — ENG-03

The [browser consumer](../examples/browser-consumer/README.md) is a separate npm project with its own exact lock. These producer commands run from the engine root; each output directory must be new:

```sh
node --test scripts/browser-runtime/consumer.test.mjs
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

Candidate qualification requires a clean-source archive for HEAD. Registry qualification uses the example's committed exact version, not the candidate's build identity. Each stages outside the checkout, verifies installed bytes and types, builds static assets, and runs nine real browser tests. Automated Chromium must have working WebGPU; unavailable GPU execution is a failure. The example README defines explicit local Chrome/adapter overrides and evidence limits. These Node/browser checks are additional to `cargo xtask check`; the existing browser package/material CI jobs run them without removing pinned Studio coverage. No package is published by these commands.

## Noise-v2 qualification

[NUM-01](./stable-noise.md) defines the explicit migration. `prepare-materials.mjs <fresh-output> <full-revision> --noise-v2` selects committed migrated material inputs; the existing `cargo xtask browser-material-check` then applies unchanged gates. `candidate.mjs verify-noise-v2 <product> <candidate-evidence> <native-v2> <browser-v2>` retains a separate bound receipt. These are producer qualification tools, not automatic document migration or publication.

Retrieve historical attachments on demand using the [archive restore guide](./evidence/archives/README.md). Python 3.10+ standard-library tools verify original bytes, not pixel qualification. The CPU workflow runs `python scripts/evidence/test_restore.py` offline.

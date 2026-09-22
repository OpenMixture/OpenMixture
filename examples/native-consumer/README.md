# Independent Rust consumer

English | [简体中文](./README.zh-CN.md)

M6B-03: the [shared CPU asset codec](../../docs/m6b-03-cpu-assets.md) provides an independent `mixture-asset` public API, covered by source and isolated archive consumers; Rust 0.5.0 is unpublished.

This application has its own Cargo workspace, lockfile and [input.mix](./input.mix). It imports only the public `mixture-core` and `mixture-wgpu` crates, plus `serde_json` for reports and `pollster` to drive native futures. It has no direct wgpu dependency, private imports or producer-owned runtime assets. The two path dependencies locate public source crates; PR-015 separately verifies [actual local archive consumption](../../docs/package-consumption.md).

## CPU check

From the repository root:

```bash
cargo xtask test-consumer
```

This checks the independent workspace, formatting, Clippy, five unit tests, three process tests, and the actual CPU probe from an unrelated working directory. The entire GPU call path and native `Send` bounds compile, but ordinary checks do not acquire a GPU. The explicit `none` acquisition test exercises the public early failure before wgpu initialization. Logs and a completion status are written to `tmp/consumer-check/`; status is invalidated before each run.

PR-012 also builds the real CLI and explicitly runs the independent CPU contract test (32 child invocations). Its GPU contract test compiles here and runs only through explicit GPU smoke. Both contract tests are ignored by ordinary standalone Cargo tests because they require the built CLI and a fresh capture directory; `test-consumer` rejects missing or incomplete execution receipts.

To run only the application's public-Rust probe:

```bash
cargo run --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- check
```

[cpu.rs](./src/cpu.rs) demonstrates decode, validation, exposed overrides, channel selection, deterministic hashes, dependency slicing and structured diagnostics. `repeat=0` identifies `pattern.cellsX` and public ID `repeat`; duplicate requested channels are rejected, while channel order is normalized. Source bytes survive a deterministic round trip. All fixtures belong to this application.

## Explicit GPU check

```bash
cargo run --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- gpu metal hardware 'Apple M5'
```

Replace the backend/name for the intended host. Backends are `auto`, `metal`, `vulkan`, `dx12`, and explicit disabled `none`; policy is `hardware` or `software`. `hardware` means the ordinary wgpu adapter request (`software_adapter=false`), not a guarantee of physical hardware. The optional last argument requires that substring in the actual adapter name. No `MIXTURE_GPU_*` or `WGPU_*` adapter overrides are read by the application.

[gpu.rs](./src/gpu.rs) renders at 65×3 with `repeat=16`, then `repeat=4`. It drops the renderer, which owns the only context, before examining both returned results. Checks consume four channels and their metadata, literal sRGB/straight-alpha colors, a connected linear roughness value, default normal/height pixels, changed parameter pixels, plan/adapter/pass reports, and zero live per-call descriptor bytes. It does not implement any pixel algorithm.

The existing smoke command invokes this same executable from an unrelated working directory and validates its reports:

```bash
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

For pinned SwiftShader, prepare the loader using the [GPU guide](../../docs/gpu-context.md), then:

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
cargo run --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- gpu vulkan software SwiftShader
```

## CLI process contract

[cli_contract.rs](./tests/cli_contract.rs) is a separate test executable that spawns a supplied, already-built `mixture` binary. It imports no Mixture Rust API or CLI-private modules. It writes its own embedded [input](./input.mix) and [missing-warp fixture](./tests/inputs/missing-warp.mix) into a new working directory, then checks stdout/stderr, exits, report fields, overrides, requested channels, actual PNGs and partial-write failures. A dev-only `png` dependency decodes completed files; the application still has no PNG encoder or alternate pixel executor. The added test dependency and its locked closure match the producer's existing versions.

The ordinary entry points are `cargo xtask test-consumer` for the 32 CPU calls and `cargo xtask gpu-smoke` for the 10 GPU-suite calls. Each run keeps raw child stdout/stderr, exact arguments/exits, owned input/output files and `status.json` under a fresh `cli-<pid>-<time>/` directory. `cliEvidence` in the enclosing consumer status points to the current run. A successful test command that matched zero tests cannot pass without that completion receipt. Failed runs retain their files with incomplete status.

For a standalone CPU run from the repository root, choose a capture path that does not exist:

```bash
mkdir -p tmp
cargo build --locked --all-features -p mixture-cli --target-dir target
MIXTURE_CONSUMER_CLI="$PWD/target/debug/mixture" \
MIXTURE_CONSUMER_EVIDENCE_DIR="$PWD/tmp/cli-contract-cpu" \
cargo test --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer --test cli_contract \
  cli_contract_cpu -- --ignored --exact --nocapture
```

The explicit GPU invocation uses the same test binary and environment adapter policy as the repository harness. Use the [GPU guide](../../docs/gpu-context.md) to configure pinned SwiftShader instead of Metal when needed:

```bash
MIXTURE_CONSUMER_CLI="$PWD/target/debug/mixture" \
MIXTURE_CONSUMER_EVIDENCE_DIR="$PWD/tmp/cli-contract-metal" \
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' \
cargo test --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer --test cli_contract \
  cli_contract_gpu -- --ignored --exact --nocapture
```

The child CLI gets explicit backend/software arguments. Its context is acquisition evidence; doctor probe success and file completion are checked separately. A pre-existing `normal.png` directory forces the second canonical PNG write to fail after baseColor completes; the test verifies the completed-file list and both JSON/human error context. This fixture does not claim atomic export, cancellation, freshness scheduling or package-content verification. See the [CLI contract](../../docs/cli-contract.md) and [PR-012 evidence](../../docs/evidence/pr-012/README.md).

## Release measurement

`measure` accepts a caller-owned `.mix`, requests `baseColor`, `normal`, `roughness`, `height` at 1024×1024, and performs a first and identical reused-renderer call. It separately reports compile, context-acquisition and both render-call wall times. File reads/writes, output inspection and JSON encoding are outside those timers; no GPU timestamp or cancellation claim is made. The two owned RGBA8 results are checked after renderer drop and written to a new directory for independent comparison. No PNG encoder is involved.

```bash
cargo build --release --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer
target/native-consumer/release/mixture-native-consumer measure \
  examples/native-consumer/input.mix tmp/native-measure metal hardware 'Apple M5'
```

The parent of the output directory must exist, and the output directory itself must not exist. The measurement input is read with the default byte limit plus one sentinel byte; core owns the actual limit diagnostic. No files or baselines are overwritten. [The PR-011 measurement helper](../../docs/evidence/pr-011/measure_native.py) compares the accepted materials to raw Rust and decoded CLI output, with all hashing/comparison after timing.

This is a focused consumer example, not a new general-purpose CLI contract. Success writes a JSON report and exits `0`; operational or validation failures return `1` with original Mixture diagnostic context/source chains where applicable; usage failures return `2` on stderr. `RenderFailure` retains the selected context alongside the original GPU operation error. There is no fallback or device recovery. Native `Renderer::render` can block its calling thread during polling; an application needing responsiveness should manage its own worker. PR-014 provides the consumer-owned `latest` scheduling example described below.

See the [public native API guide](../../docs/native-sdk.md) for ownership, dependency exposure and acceptance limits.

## Device-loss contract (PR-013)

The independent `tests/device_loss.rs` uses public Rust APIs and owned input. It destroys cold/warm devices, verifies two repeated failures per case without allocations, preserves owned diagnostics after renderer drop, and consumes correct pixels from a separate live context. CPU checks only compile this ignored test. Explicit smoke runs it and validates its fresh `deviceLossEvidence` receipt. A standalone run requires an absolute, not-yet-existing evidence filename whose parent exists:

```bash
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' \
MIXTURE_CONSUMER_DEVICE_LOSS_EVIDENCE="$PWD/tmp/device-loss-$(date +%s).json" \
cargo test --locked --manifest-path examples/native-consumer/Cargo.toml \
  --test device_loss device_loss_contract -- --ignored --exact --nocapture
```

Use the pinned Vulkan policy and loader from the GPU commands above to run the same test on SwiftShader. Receipts start incomplete and become complete only after all assertions pass. This tests device destruction, not physical OOM. See [failure semantics](../../docs/gpu-failures.md).

## Latest requests (PR-014)

The [state module](./src/latest.rs) is a library target within this example package, shared by its executable and independent CLI test. Six CPU tests cover replacement, stale/duplicate/out-of-order completion, newest failure, overflow, drops and directory isolation. The `latest` mode runs actual Rust renders with deterministic event delivery and bounded output ownership. `latest_cli` uses a prebuilt CLI, fresh absolute evidence directory (existing parent), five child calls and explicit cleanup. Build the CLI using the earlier commands before running:

```bash
cargo run --locked --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- latest metal hardware 'Apple M5'
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' \
MIXTURE_CONSUMER_CLI="$PWD/target/debug/mixture" \
MIXTURE_CONSUMER_LATEST_DIR="$PWD/tmp/latest-cli-$(date +%s)" \
cargo test --locked --all-features --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer --test latest_cli latest_cli_contract \
  -- --ignored --exact --nocapture
```

Explicit `gpu-smoke` orchestrates both checks and rejects incomplete evidence. For SwiftShader use the loader and Vulkan/software policy above; add `--all-features` to the standalone Rust command to enable the software feature. See [the contract and memory limits](../../docs/stale-results.md). Submitted GPU work may finish; this is stale-result handling, not GPU cancellation. No new product crate or dependency is added.

## Packaged consumption (PR-015)

The repository verifier stages this application and its owned input/tests outside the producer repository. Only the temporary manifest changes: exact version dependencies resolve to the normalized local core/wgpu archives, and the archived CLI is built alongside them. The source-path fixture above stays independent and unchanged. The external verifier records `packagedCratesValidated: true` after checking actual provenance, pinned dependency identities, missing-asset rejection and consumption; the application itself retains its original provenance-neutral `false` field.

```bash
cargo xtask package-check
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

See [package resolution](../../docs/package-consumption.md), [compatibility](../../docs/compatibility.md) and [M4 exit/release status](../../docs/release.md). Publication is still disabled. The [remote CI record](../../docs/evidence/remote-ci/README.md) documents passing Linux/macOS/Windows CPU checks and the Linux pinned SwiftShader GPU workload; it does not certify untested hardware backends.

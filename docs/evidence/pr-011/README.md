# PR-011 native consumer evidence

English | [简体中文](./README.zh-CN.md)

**Local acceptance, 2026-09-08:** the separate [Rust application](../../../examples/native-consumer/README.md) compiles through public APIs and executes real GPU work on Apple M5 Metal and pinned SwiftShader Vulkan. Both returned results remain usable after renderer/context drop. Product runtime behavior, shaders, accepted baselines and the product lockfile are unchanged. This verifies source-path consumption; package contents, PR-012–015 and deferred remote CI remain open.

## Source and checks

The implementing revision is the PR-011 commit containing this record, on `codex/pr-011-native-consumer`, parent `f56bffe732ae9364c0402ec04ab5e84ac3e1990e`. Measurements were captured from its uncommitted implementation before the final documentation update. The two performance summaries retain the parent revision, dirty source paths, exact source/input/manifest hashes, helper and binary hashes, host, library versions, adapter options, commands, working directories, exit codes and raw-stream hashes. [capture-manifest.json](./capture-manifest.json) verifies those measured source hashes against the final source and inventories the archived evidence.

| Check | Evidence and result |
|---|---|
| Independent CPU consumer | [Command log](./checks/test-consumer.log), [status](./checks/consumer/status.json), [CPU report](./checks/consumer/cpu.stdout.log): five unit tests, three process tests, separate locked build/Clippy/format and actual execution outside the producer directory pass. |
| Compiler/request regressions | [test-plan.log](./checks/test-plan.log) passes. |
| Repository command regressions | [xtask-consumer-tests.log](./checks/xtask-consumer-tests.log) passes. |
| Final repository check | [check.log](./checks/check.log) passes, including the CPU consumer; no GPU is acquired by ordinary checks. |
| Metal explicit GPU smoke | [Command](./gpu-smoke/metal/command.log), [consumer report](./gpu-smoke/metal/native-consumer/gpu.stdout.log), [status](./gpu-smoke/metal/native-consumer/status.json) pass. |
| Pinned software explicit GPU smoke | [Command](./gpu-smoke/software/command.log), [consumer report](./gpu-smoke/software/native-consumer/gpu.stdout.log), [status](./gpu-smoke/software/native-consumer/status.json) pass. |

The CPU consumer asserts stable plan hashing, source round trips, output ordering, duplicate-channel rejection, roughness-only dependency slicing and typed malformed-source/missing-port/limit failures. Invalid public `repeat=0` reports `MIX_PARAMETER_INVALID_VALUE`, compile stage, node `pattern`, parameter `cellsX` and public ID `repeat`. The disabled-backend process test checks the structured early acquisition failure without initializing wgpu.

Both GPU runs consume the application's 65×3 input at `repeat=16` and `repeat=4`. They check four requested channels, connected/default provenance, encoding, dimensions, byte lengths, literal sRGB/straight-alpha and linear scalar/normal values, different override pixels, plan identity, pass count, selected adapter and released per-call descriptors. All pixel checks run after dropping the only renderer/context. [The native API guide](../../native-sdk.md) states the reviewed lifetime, dependency-exposure and native polling contract.

## Paired 1K release measurements

The [measurement helper](./measure_native.py) runs three trials for each accepted default material on Metal and three trials for wood on SwiftShader. Each trial starts one native process followed by one CLI process. Native acquires one context and renders the identical plan twice; no warmup trial is discarded. Every request is 1024×1024 with `baseColor`, `normal`, `roughness`, `height`. [Native](./checks/consumer-release-build.log) and [CLI](./checks/cli-release-build.log) builds use `--release --locked --all-features` and Rust 1.98.1. The host is macOS 26.5.1 arm64; software source pin is `694585a05946e1ed49b6bd577ca6537cbb57f025`.

Each of the 48 channel comparisons checks that first Rust bytes, reused-renderer bytes and decoded CLI pixels are **identical on the same adapter**, then compares them to accepted 1K baselines. All comparisons pass. Software wood is exact; Metal maximum byte differences from software baselines are 0 for ceramic, 1 for leather and 2 for wood, within unchanged material tolerances. The helper verifies baseline file hashes, expected plan hashes and PNG encoding metadata. It does not update or synthesize pixels. This default-material comparison is not a replacement for the full material-variant acceptance matrix already recorded for M3.

Median milliseconds over three trials:

| Adapter / material | Decode + validate + compile | Context acquisition | First render call | Reused-renderer call | CLI process + PNG + report |
|---|---:|---:|---:|---:|---:|
| Metal / glazed-ceramic | 0.094 | 8.345 | 27.229 | 21.730 | 54.627 |
| Metal / leather | 0.089 | 8.087 | 32.614 | 23.641 | 683.056 |
| Metal / wood | 0.109 | 7.912 | 32.878 | 27.747 | 385.638 |
| SwiftShader / wood | 0.113 | 31.567 | 217.801 | 99.588 | 582.201 |

Full per-trial reports, per-channel comparison metrics and min/median/max are in [Metal summary](./performance/metal/summary.json) and [software summary](./performance/software/summary.json); original stdout/stderr files sit beside each summary. Per-trial render calls include pipeline preparation, dispatch, readback and RGBA8 conversion; native source reads, raw-file writes, inspection and JSON encoding are excluded. The external CLI timer includes process lifetime, source/graph work, GPU acquisition/render, PNG encoding/file writes and JSON. These are two useful consumer workflows with different costs, not an isolated bindings-speed comparison. No GPU timestamps or separately measured PNG-encoder time are claimed.

Metal first-call ranges are 26.1–214.4 ms for ceramic, 32.2–212.4 ms for leather and 32.5–227.5 ms for wood. Their first trials spend roughly 180–195 ms preparing pipelines; those samples remain in the record. OS/driver caches are uncontrolled, so even a fresh renderer does not imply an uncached driver. Reused-call ranges are 21.7–22.1, 22.7–24.2 and 26.8–29.4 ms, respectively; software wood is 98.1–100.4 ms. Every reused call records zero pipeline misses, with 4/4/6 cache entries for the three materials. This small local sample supports explicit renderer reuse; it does not establish a frame-rate promise, a UI deadline, cancellation or a release performance SLO.

## Reproduce

From the repository root:

```bash
cargo xtask test-consumer
cargo xtask test-plan
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

For the recorded software checkout and loader (prepare and verify the pin using the [GPU guide](../../gpu-context.md)):

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

Build binaries before measurement. The helper requires Python with Pillow and NumPy for output inspection only; captured versions appear in each summary. Every output directory must be new.

```bash
cargo build --release --locked --all-features -p mixture-cli
cargo build --release --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer
python3 docs/evidence/pr-011/measure_native.py \
  --repo "$PWD" \
  --native-binary "$PWD/target/native-consumer/release/mixture-native-consumer" \
  --cli-binary "$PWD/target/release/mixture" \
  --out "$PWD/tmp/pr-011-repro-metal" \
  --backend metal --expect-adapter 'Apple M5' --trials 3
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
python3 docs/evidence/pr-011/measure_native.py \
  --repo "$PWD" \
  --native-binary "$PWD/target/native-consumer/release/mixture-native-consumer" \
  --cli-binary "$PWD/target/release/mixture" \
  --out "$PWD/tmp/pr-011-repro-software" \
  --backend vulkan --software --expect-adapter SwiftShader \
  --trials 3 --materials wood
```

The helper copies caller input into its new output directory and invokes both binaries from there. Original raw RGBA files, generated PNGs and copied inputs remain under `tmp/pr-011/performance-{metal,software}/`; the durable archive retains raw report streams, hashes and comparison results rather than duplicating those 1K images. GPU smoke snapshots are kept separately for both policies because the normal smoke command reuses `tmp/gpu-smoke/`.

**Intentionally out of scope:** packaged/publication acceptance, PR-012 CLI changes, PR-013 device-loss/OOM changes, PR-014 scheduling, bindings, daemon/IPC, UI, new nodes/shaders, pooling and remote push/CI. No full M4 or release-ready claim is made.

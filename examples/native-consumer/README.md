# Independent Rust consumer

English | [简体中文](./README.zh-CN.md)

This application has its own Cargo workspace, lockfile and [input.mix](./input.mix). It imports only the public `mixture-core` and `mixture-wgpu` crates, plus `serde_json` for reports and `pollster` to drive native futures. It has no direct wgpu dependency, private imports or producer-owned runtime assets. The two path dependencies locate public source crates; packaged-crate consumption remains PR-015 work.

## CPU check

From the repository root:

```bash
cargo xtask test-consumer
```

This checks the independent workspace, formatting, Clippy, five unit tests, three process tests, and the actual CPU probe from an unrelated working directory. The entire GPU call path and native `Send` bounds compile, but ordinary checks do not acquire a GPU. The explicit `none` acquisition test exercises the public early failure before wgpu initialization. Logs and a completion status are written to `tmp/consumer-check/`; status is invalidated before each run.

The equivalent application invocation is:

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

This is a focused consumer example, not a new general-purpose CLI contract. Success writes a JSON report and exits `0`; operational or validation failures return `1` with original Mixture diagnostic context/source chains where applicable; usage failures return `2` on stderr. `RenderFailure` retains the selected context alongside the original GPU operation error. There is no fallback or device recovery. Native `Renderer::render` can block its calling thread during polling; an application needing responsiveness should manage its own worker. Stale-result scheduling is reserved for PR-014.

See the [public native API guide](../../docs/native-sdk.md) for ownership, dependency exposure and acceptance limits.

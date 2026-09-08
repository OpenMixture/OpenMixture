# Explicit GPU context and doctor

English | [简体中文](./gpu-context.zh-CN.md)

`GpuContext` explicitly owns its instance, adapter, device, and queue. `request(options)` acquires them without a window surface or computation and returns an immutable `unverified` acquisition snapshot. PR-004 adds [checker compute/readback](./builtin-checker.md): the CLI doctor runs this probe by default and reports `healthy` only after verifying its pixels. `--skip-probe` retains acquisition-only behavior.

## Commands and selection policy

```bash
cargo run --locked -p mixture-cli -- doctor
cargo run --locked -p mixture-cli -- doctor --json
cargo run --locked -p mixture-cli -- doctor --skip-probe --json
cargo run --locked -p mixture-cli -- doctor --backend metal --power-preference low-power --json
# Requires an installed software Vulkan adapter:
cargo run --locked -p mixture-cli -- doctor --backend vulkan --software --json
# Deterministic failure without initializing GPU state; exits 1:
cargo run --locked -p mixture-cli -- doctor --backend none --json
```

| Option | Meaning |
| --- | --- |
| `--backend auto` | Default: permit the compiled native Vulkan, Metal, and DX12 backends; wgpu selects the adapter |
| `--backend vulkan\|metal\|dx12` | Permit only that backend; unavailable means failure |
| `--backend none` | Disable acquisition before instance creation, returning `MIX_GPU_ADAPTER_UNAVAILABLE` |
| `--power-preference high-performance\|low-power` | Preference, not a guaranteed device type; default `high-performance` |
| `--software` | Require wgpu's software adapter class; never retry with hardware |
| `--skip-probe` | Acquire context only, keeping `unverified` and both probes `notRun` |
| `--json` | Write one complete structured report to stdout |
| `--help`, `-h` | Show help without initializing GPU state |

Values follow their option as a separate argument. Duplicate, unknown, and missing options are rejected. The library applies no `WGPU_*` environment overrides. System driver configuration still matters, including Vulkan loader/ICD selection. There is one adapter request and one device request, with no Mixture retry or alternate semantic executor. Software Vulkan is a driver behind the same wgpu compute path.

wgpu 30.0.1 enables native `vulkan`, `metal`, `dx12`, `std`, `parking_lot`, `serde`, and WGSL input. Default targets support Metal on macOS, Vulkan on Linux, and Vulkan/DX12 on Windows. The optional `software-vulkan` feature on `mixture-wgpu` and `mixture-cli` also enables native Vulkan on macOS for SwiftShader testing. GL, browser WebGPU, and noop remain disabled. A compiled backend is not a guarantee that a driver/adapter exists.

## Verdicts and report

| Result | Verdict | `ok` | Exit |
| --- | --- | --- | --- |
| Actual checker compute/readback and pixel checks pass | `healthy` | `true` | `0` |
| Explicitly skipped probe; acquisition passes | `unverified` | `true` | `0` |
| Adapter, device, shader, execution, or readback fails | `unhealthy` | `false` | `1` |
| Invalid invocation | No runtime report; stderr, including with `--json` | — | `2` |
| Output I/O fails | Report may be incomplete; stderr explains failure | — | `1` |

A `healthy` doctor result proves this fixed probe on the selected context at that moment; it is not a device-loss monitor or proof of future material correctness. It checks the 64×64 output length, opaque black/white channel values, equal black/white counts, and fixed pixel sentinels. This is a probe validator, not another checker renderer.

JSON schema version `1` retains PR-003 fields and adds optional `execution`:

- `schemaVersion`, `verdict`, `computeProbe`, `readbackProbe`: probes are `notRun`, `passed`, or `failed`; acquisition-only snapshots never claim computation;
- `requested`: `backend`, `powerPreference`, `softwareAdapter`, sorted `effectiveBackends`, sorted `requiredFeatures`, complete `requiredLimits`;
- `adapter`: actual `name`, `deviceType`, `backend`, numeric `vendor`/`device`, `driver`/`driverInfo`, complete `supportedLimits`, sorted `supportedFeatures`; null before acquisition;
- `device`: complete enabled `limits` and sorted `features`; null after acquisition failure;
- `execution`: completed checker dimensions, pass count, row layout, byte counts, timings, and adapter evidence; present on a successful full probe;
- `ok` and deterministic `diagnostics`, following [the shared diagnostic contract](./diagnostics.md).

Actual backend names are `Metal`, `Vulkan`, or `Dx12`; compiled flags use names such as `METAL`. Device types use wgpu names such as `IntegratedGpu`, `DiscreteGpu`, and `Cpu`. Limit fields are camelCase. Driver values and timings vary between systems and are not semantic hashes or portable snapshots. Human reports include the same requested/actual capability evidence and probe outcomes.

Device requests use no optional features and `wgpu::Limits::default()`. Adapter limits are reported without bucketing and checked before device creation; requirements are never reduced implicitly. Unsupported limits return `MIX_GPU_DEVICE_REQUEST_FAILED` at `gpuDevice` with the first failed limit name and requested/supported values. GPU capability limits are distinct from core `SafetyLimits` resource ceilings.

Adapter errors use `MIX_GPU_ADAPTER_UNAVAILABLE` / `gpuAdapter`; native device request errors use `MIX_GPU_DEVICE_REQUEST_FAILED` / `gpuDevice`. Device failures retain the selected adapter. Actual native sources stay available via `Error::source` and `driverMessage`; policy/limit preflight does not invent a driver source. Later errors retain [their exact execution/readback stage](./builtin-checker.md), and compute is not marked passed unless its submission completed.

## API and verification

[context.rs](../crates/mixture-wgpu/src/context.rs) owns acquisition; [checker.rs](../crates/mixture-wgpu/src/checker.rs) owns the built-in and probe; [diagnostics.rs](../crates/mixture-wgpu/src/diagnostics.rs) owns reports. Context accessors borrow its handles and immutable acquisition report. `render_checker(&mut self, request, limits)` returns CPU-owned output. `probe_checker(&mut self)` produces a new verified report. There is no global context or cache, and all ordinary tests remain GPU-free.

```bash
cargo test --locked -p mixture-wgpu context
cargo test --locked -p mixture-cli doctor
cargo xtask shader-check
cargo xtask check
cargo xtask gpu-smoke
```

`gpu-smoke` opts into real GPU work and uses `--all-features` to make the macOS software Vulkan feature available. It runs full and skipped doctor, renders/decodes a PNG, compares decoded RGBA with the reviewed SwiftShader golden, renders three graph examples with plan hashes, and executes all ignored library/CLI GPU tests, including eleven node fixture families. It saves reports, PNG, comparison JSON, stderr, and test logs in ignored `tmp/gpu-smoke/`. Neither `check` nor ordinary workspace tests initialize a GPU. A smoke failure fails the task and never updates the golden.

Repository GPU harnesses (`gpu-smoke`, node/material checks, `golden check`, and `trace-2k`) read `MIXTURE_GPU_BACKEND` (`auto`, `vulkan`, `metal`, `dx12`; default `auto`), `MIXTURE_GPU_SOFTWARE` (`0`/`1`; default `0`), and optional `MIXTURE_GPU_EXPECT_ADAPTER` (case-sensitive name substring). Production APIs/direct CLI calls use explicit options, not these test variables.

## Pinned software adapter

[GPU CI](../.github/workflows/gpu-smoke.yml) uses Ubuntu 24.04/Clang 18, builds [SwiftShader revision `694585a05946e1ed49b6bd577ca6537cbb57f025`](https://swiftshader.googlesource.com/SwiftShader/+/694585a05946e1ed49b6bd577ca6537cbb57f025) with bundled LLVM, restricts the Vulkan loader to its ICD, and requires `Cpu`/`Vulkan` with `SwiftShader` in the name. [The setup script](../.github/scripts/setup-swiftshader.sh) pins driver source and disables upstream tests and display integrations. Runner packages remain distribution-managed; this pins the driver, not an entire OS image. CI uploads available evidence even after failure.

On Linux, install the workflow's CMake, Ninja, Clang 18, and Vulkan loader packages, then:

```bash
CC=clang-18 CXX=clang++-18 bash .github/scripts/setup-swiftshader.sh
export VK_DRIVER_FILES="$PWD/tmp/swiftshader/build/Linux/vk_swiftshader_icd.json"
export VK_ICD_FILENAMES="$VK_DRIVER_FILES"
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

On macOS, install CMake/Ninja and Xcode Command Line Tools, then:

```bash
bash .github/scripts/setup-swiftshader.sh
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
  MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

The macOS setup provides `libvulkan.dylib` as an alias to SwiftShader's direct Vulkan API library; no system driver is installed. The [PR-003 acquisition-only report](./evidence/pr-003-apple-m5.json) is retained as historical evidence. Current [Metal](./evidence/pr-004-apple-m5.json) and [SwiftShader Vulkan](./evidence/pr-004-swiftshader.json) full probes passed locally, with matching checker pixels. Remote Linux SwiftShader and the Linux/macOS/Windows non-GPU matrix remain pending; their configuration is not a claim of remote CI completion.

PR-007 shares checker execution with the graph renderer and adds all-node/graph evidence; see [graph rendering](./graph-rendering.md). The pinned driver and explicit adapter policy are unchanged. Historical PR-004 reports retain their old 16-byte checker uniform estimate; the current shared ABI uses 48 bytes.

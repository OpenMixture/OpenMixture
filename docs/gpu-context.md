# Explicit GPU context and doctor

English | [简体中文](./gpu-context.zh-CN.md)

PR-003 implements headless adapter/device acquisition. `GpuContext` owns its instance, adapter, device, and queue; callers construct and retain it explicitly. Acquisition does not create a surface, shader, render/compute pass, texture, or readback operation. `healthy` remains reserved for PR-004's execution and readback probe.

## Run doctor

```bash
cargo run --locked -p mixture-cli -- doctor
cargo run --locked -p mixture-cli -- doctor --json
cargo run --locked -p mixture-cli -- doctor --backend metal --power-preference low-power --json
# Requires an installed software Vulkan adapter:
cargo run --locked -p mixture-cli -- doctor --backend vulkan --software --json
# Deterministic unavailable-adapter diagnostic without initializing a GPU; exits 1:
cargo run --locked -p mixture-cli -- doctor --backend none --json
```

| Option | Meaning |
| --- | --- |
| `--backend auto` | Default: allow the compiled native Vulkan, Metal, and DX12 backends; wgpu selects an adapter |
| `--backend vulkan\|metal\|dx12` | Permit only that backend; an unavailable backend is an error |
| `--backend none` | Disable acquisition explicitly; return `MIX_GPU_ADAPTER_UNAVAILABLE` before creating an instance |
| `--power-preference high-performance\|low-power` | Adapter preference; default `high-performance`, not a guarantee about device type |
| `--software` | Require wgpu's software/fallback adapter class; never retry with hardware if unavailable |
| `--json` | Emit one complete JSON report to stdout |
| `--help`, `-h` | Show usage without initializing GPU state |

Values follow their option as a separate argument. Duplicate, unknown, or missing options are rejected. The library does not apply `WGPU_*` environment overrides. System driver configuration still matters, including Vulkan ICD selection via `VK_DRIVER_FILES` / `VK_ICD_FILENAMES`. Selection makes one adapter request and one device request, with no Mixture retry or alternate executor. Software Vulkan is a driver behind the same wgpu path.

The workspace enables wgpu 30.0.1 native `vulkan`, `metal`, and `dx12` features, plus `std`, `parking_lot`, and `serde`. Effective availability depends on the target: Metal on macOS, Vulkan on Linux, and Vulkan/DX12 on Windows. GL, browser WebGPU, noop, and WGSL input features are not enabled in this PR. The library reports the intersection of policy and compiled backends. It does not equate a compiled backend with an available adapter.

## Report and exit contract

| Result | Verdict | `ok` | Exit |
| --- | --- | --- | --- |
| Adapter and device acquired | `unverified` | `true` | `0` |
| No permitted/available adapter | `unhealthy` | `false` | `1` |
| Device limits unsupported or device request failed | `unhealthy` | `false` | `1` |
| Invalid invocation | No runtime report; usage on stderr even with `--json` | — | `2` |
| Output I/O failed | Report may be incomplete; explanation on stderr | — | `1` |

`ok` describes acquisition success only. Both `computeProbe` and `readbackProbe` are `notRun`, including on failure. Successful acquisition does not prove that rendering works. Human reports also include requested policy, adapter identity/capabilities, selected device features/limits, and actionable diagnostics.

JSON schema version `1` has these fields:

- `schemaVersion`, `verdict`, `computeProbe`, `readbackProbe`;
- `requested`: `backend`, `powerPreference`, `softwareAdapter`, sorted `effectiveBackends`, sorted `requiredFeatures`, and complete `requiredLimits`;
- `adapter`: actual `name`, `deviceType`, `backend`, numeric `vendor`/`device`, `driver`/`driverInfo`, complete `supportedLimits`, and sorted `supportedFeatures`; null before adapter acquisition;
- `device`: complete enabled `limits` and sorted `features`; null when device acquisition fails;
- `ok` and deterministic `diagnostics`, using the [shared diagnostic contract](./diagnostics.md).

Backend identity is `Metal`, `Vulkan`, or `Dx12`; effective backend flags use wgpu names such as `METAL`. Device type uses wgpu names such as `IntegratedGpu`, `DiscreteGpu`, or `Cpu`. Limits serialize with wgpu's camelCase field names. Driver-provided values can vary between systems; these are evidence, not portable snapshots or semantic hashes.

PR-003 requests no optional features and `wgpu::Limits::default()`. Supported limits are reported without limit bucketing, and requested limits are checked before device creation. Unsupported limits produce `MIX_GPU_DEVICE_REQUEST_FAILED` at `gpuDevice`, with the first failing limit's name and requested/supported values; requirements are never reduced implicitly. These GPU capability limits are distinct from core `SafetyLimits` resource ceilings. No material allocations exist yet.

Native adapter/device request errors retain their original `std::error::Error::source` chain and include `driverMessage` beneath a stable Mixture diagnostic. Device failures retain the selected adapter in the report. Preflight policy/limit failures have no fabricated driver source. Adapter failures use `MIX_GPU_ADAPTER_UNAVAILABLE` at `gpuAdapter`; both codes provide actionable suggestions.

## Public Rust API

Use [context.rs](../crates/mixture-wgpu/src/context.rs) and [diagnostics.rs](../crates/mixture-wgpu/src/diagnostics.rs) directly; JSON and blocking orchestration belong to the CLI.

```rust
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions};

async fn acquire() -> Result<GpuContext, mixture_wgpu::GpuContextError> {
    GpuContext::request(GpuContextOptions {
        backend: BackendPreference::Auto,
        ..Default::default()
    }).await
}
```

Context accessors borrow its instance, adapter, device, queue, and immutable acquisition report. There is no global context or cache. A report is a snapshot of acquisition, not a live device-loss monitor. Ordinary library unit tests and CLI tests require no GPU; explicit disabled-backend requests fail before wgpu initialization.

## Verification and pinned software CI

```bash
cargo test --locked -p mixture-wgpu context
cargo test --locked -p mixture-cli doctor
cargo xtask check
# Explicit GPU access, outside ordinary check/test:
cargo xtask gpu-smoke
```

`gpu-smoke` runs the real CLI, verifies its success envelope and adapter policy, then runs the ignored context lifecycle test. That test acquires two contexts, destroys one device, verifies its destruction callback, and polls the other without receiving a device-loss callback. It creates no workload or readback probe. Reports are saved to ignored `tmp/gpu-smoke/doctor.json` and `doctor.stderr.log`; a failed CLI or adapter assertion fails the task rather than skipping GPU coverage.

Only the smoke harness reads `MIXTURE_GPU_BACKEND` (`auto`, `vulkan`, `metal`, `dx12`; default `auto`), `MIXTURE_GPU_SOFTWARE` (`0` or `1`; default `0`), and optional `MIXTURE_GPU_EXPECT_ADAPTER` (case-sensitive adapter-name substring). It passes selection options explicitly to the library/CLI. These variables do not configure production `GpuContext` or direct `doctor` calls.

[GPU CI](../.github/workflows/gpu-smoke.yml) uses Ubuntu 24.04 and Clang 18, builds [SwiftShader revision `694585a05946e1ed49b6bd577ca6537cbb57f025`](https://swiftshader.googlesource.com/SwiftShader/+/694585a05946e1ed49b6bd577ca6537cbb57f025) with its bundled LLVM, restricts the Vulkan loader to that ICD, and requires a `Cpu` / `Vulkan` adapter whose name contains `SwiftShader`. The [setup script](../.github/scripts/setup-swiftshader.sh) pins driver source and disables display integrations and upstream tests. Runner packages are distribution-managed; the driver revision is pinned, not the entire OS image. CI uploads the full doctor report, stderr, and build-environment record even after failure when available. The source/output layout follows [SwiftShader's pinned CMake configuration](https://swiftshader.googlesource.com/SwiftShader/+/694585a05946e1ed49b6bd577ca6537cbb57f025/src/Vulkan/CMakeLists.txt).

To reproduce on Linux after installing the build tools and Vulkan loader listed in the workflow:

```bash
CC=clang-18 CXX=clang++-18 bash .github/scripts/setup-swiftshader.sh
export VK_DRIVER_FILES="$PWD/tmp/swiftshader/build/Linux/vk_swiftshader_icd.json"
export VK_ICD_FILENAMES="$VK_DRIVER_FILES"
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

The [Apple M5 / Metal acquisition report](./evidence/pr-003-apple-m5.json) records local hardware evidence. Local checks and acquisition smoke passed; the new remote SwiftShader job and cross-platform matrix still need a real CI run. M1 remains open because checker execution, readback, PNG output, and `healthy` verification belong to PR-004.

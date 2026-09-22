# Windows numerical investigation — 2026-09-22

English | [简体中文](./README.zh-CN.md)

The [M6A-05 hardware failure](../m6a-05/README.md) is reproduced and localized to noise output before height-to-normal. **This is investigation, not a fix or expanded hardware acceptance.** Production shaders, versions, goldens and the ≤1 resource gate are unchanged.

## Findings

The [investigation receipt](./investigation.json) binds main `c2a08e14be9991d75df24e0f045d842d8db4b570`, added diagnostic source hashes, NVIDIA GT 1030 Native Vulkan/DX12 and Chrome 153.0.8010.48 (NVIDIA/Pascal, non-fallback). Browser information does not expose its underlying native API; do not infer it from the OS. Exact registry `0.3.0-alpha.0` passed all 13 public interface tests. The unchanged public resource comparison failed on **both** Vulkan and DX12: normal maxima at weights 0/0.25/0.5/1 are **0/1/4/8**.

Raw production-shader capture removes resource loading, scalar blending and PNG encoding. At 1024², value noise seed 29, scale 32, four octaves and persistence 0.5:

- Only **9 of 1,048,576** scalar f16 pixels differ between Vulkan and Chrome, each by one adjacent positive f16 value.
- Normal output differs in 39 RGBA8 components, maximum delta **8**, with 11 components above 1. This reproduces the public weight-1 result.
- Uploading the exact Native raw f16 noise texture to Chrome and running the unchanged normal shader gives **zero differing raw f16 components** against Native. The measured blocker is upstream of normal evaluation.
- [Original sparse failure evidence](./baseline.json) retains all nine heights and all normal components above the gate. No CPU noise/normal renderer is used.

Additional instrumentation replaces only the final noise store with four exactly representable byte/256 channels containing the pre-half f32 bit pattern. [Measurements](./prehalf.json) show 90,132 differing f32 pixels, maximum 4 ULP. At the nine original f16 differences, values differ by 1–2 f32 ULP at a half-rounding midpoint. At (590,190), Native gives `0.652099609375` (the midpoint), Chrome `0.6520994901657104`; stored heights become `0.65234375` and `0.65185546875`.

This supports **small noise arithmetic differences → opposite half-rounding choices → UV-scaled derivative amplification**. At 1K, the normal shader multiplies neighboring height differences by 512. Integer half rounding cannot recover identical inputs after f32 values diverge. Instrumentation may affect optimization, so it supports this mechanism rather than proving a driver instruction; identical-input replay is the stronger isolation control.

## Experiment and decision

An experimental shader replaces nested `mix` with `ab=fma(b-a,t,a)`, `cd=fma(d-c,t,c)`, then `fma(cd-ab,u,ab)`. [numerical-experiment.mjs](../../../scripts/browser-runtime/numerical-experiment.mjs) generates it outside product sources.

| Shader / scale | Native vs Chrome | Different f16 heights | Maximum normal RGBA8 delta |
|---|---|---:|---:|
| Production / 32 | Vulkan | 9 | 8 |
| Production / 7 | Vulkan | 85 | 19 |
| FMA / 32 | Vulkan | 0 | 0 |
| FMA / 32 | DX12 | 0 | 0 |
| FMA / 8 | Vulkan | 0 | 0 |
| FMA / 7 | Vulkan | 66 | 19 |

All rows use seed 29, four octaves and persistence 0.5. Identical-input normal replay has zero raw differences in every row. The production scale-7 control shows the residual failure is not introduced solely by the experiment. This is a small diagnostic sample, not catalog/seed/parameter qualification.

**Do not ship the FMA-only rewrite as a fix.** It solves the original fixture but fails another legal scale. WGSL permits floating-point reassociation/fusion and does not provide a portable correctly-rounded guarantee for `fma`; see the [WGSL floating-point rules](https://www.w3.org/TR/WGSL/#floating-point-evaluation). No exact Naga/Tint/driver bug is established here.

The next bounded decision should define a stable value-noise numerical/compatibility contract covering non-power-of-two scales, seeds, octave/persistence boundaries and resolutions. Evaluate explicitly reproducible arithmetic with a node/version decision that preserves v1 behavior. Higher-precision intermediates would be a separate plan/resource-budget change. Any candidate needs frozen resource, Scalar, three-material, pinned-software node and Native/browser consumer gates, plus visual before/after evidence. Cellular noise and warp require separate evidence.

## Reproduction

Start at the repository root, save `cargo run --locked -p mixture-cli -- doctor --json`, then set `MIXTURE_BROWSER_CHANNEL=chrome` and run `node scripts/browser-runtime/consumer.mjs registry - tmp/numerics-registry` with a fresh directory. Set `MIXTURE_RESOURCE_BROWSER_DIR` to the resulting directory containing `resource-1-normal.png`, and `MIXTURE_RESOURCE_EVIDENCE` to an absolute output file. Run this for each explicit backend:

```powershell
$env:MIXTURE_GPU_BACKEND = 'vulkan' # Repeat with dx12.
cargo test --manifest-path examples/native-consumer/Cargo.toml --locked --all-features --target-dir target/native-consumer --test image_resources -- --ignored --nocapture
```

The [Native probe](../../../crates/mixture-wgpu/tests/numerical_probe.rs) uses production shaders through wgpu, is ignored in ordinary tests, and is not a public renderer. Install existing browser-consumer development dependencies in a disposable engine-owned copy (`npm ci --ignore-scripts`), then pass its `node_modules/playwright/index.mjs`. This run used existing ignored `tmp/m6a04-browser-host`. All capture directories must be fresh; adapt absolute paths to your checkout.

```powershell
$env:MIXTURE_GPU_BACKEND = 'vulkan'
$env:MIXTURE_NUMERICAL_OUTPUT = 'D:/Coding/OpenMixture/tmp/probe-native'
cargo test --locked -p mixture-wgpu --test numerical_probe -- --ignored --nocapture
node scripts/browser-runtime/probe-numerics.mjs tmp/m6a04-browser-host/node_modules/playwright/index.mjs tmp/probe-browser tmp/probe-native/noise.rgba16
node scripts/browser-runtime/compare-numerics.mjs tmp/probe-native tmp/probe-browser tmp/probe-comparison.json
```

The comparator verifies noise shader, parameter and exact replay-input identity and reports measurements, **not acceptance**. Files are 1024² tightly packed little-endian RGBA f16. Optional `MIXTURE_NUMERICAL_PARAMETERS` contains eight u32 uniform words; scale 7 is `[29,7,4,0,1056964608,0,0,0]`. Both probes read it. Clear experiment variables before production runs.

```powershell
node scripts/browser-runtime/numerical-experiment.mjs fma tmp/noise-fma.wgsl
$env:MIXTURE_NUMERICAL_SHADER = 'D:/Coding/OpenMixture/tmp/noise-fma.wgsl'
# Repeat both captures in fresh directories and compare.
```

For pre-half instrumentation, generate mode `prehalf` instead of `fma`. Decode each f16 channel, multiply by 256, and assemble four bytes little-endian as f32 bits. Compare those bits and the nine coordinates in `baseline.json`. Normal output from this mode has no material meaning and must not be interpreted or accepted.

## Verification and retention

Registry tests: 13 passed. Both public hardware parity tests: failed as above. Existing `cargo xtask test-node fractal-noise` passed before experimentation. Six capture/replay experiments completed. `cargo xtask check` passed with `CARGO_INCREMENTAL=0` after the first attempt hit a Rust 1.98.1 incremental-cache panic (`Invalid DepKind 488`). This environment failure is separate from GPU parity failure. The PR records final checks after documentation changes.

Git retains source/input identities, selected measurements, all sparse original failing values and reproduction tools. Full raw textures, routine PNGs, doctor output and logs remain ignored under `tmp/windows-numerical`, without a permanent full-run archive. Probe source hashes identify the actual tested additions; later docs are not pixel source. No visual change is accepted. Out of scope: runtime fixes, version changes, golden updates, broader hardware acceptance, publication, Studio changes and deployment.

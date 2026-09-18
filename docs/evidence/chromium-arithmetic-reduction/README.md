# Arithmetic reduction and acceptance-contract review — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**A buffer-only multiply/add witness reproduces the native/Chrome/Edge difference. Both observed answers are permitted by WGSL. No dependency fork or shader correction is justified by this witness alone.** This follow-up supersedes the maintenance recommendation in the [earlier investigation](../chromium-compile-policy/README.md), not its historical measurements. Formal Chrome/Edge full-material certification remains pending; the accepted candidate, native reference, goldens and thresholds are unchanged.

## Smallest retained witness

[minimal.wgsl](./minimal.wgsl) evaluates only `inputs[0] * inputs[1] + inputs[2]`, with runtime storage-buffer inputs and f32 buffer readback. There is no material graph, texture, half conversion, PNG encoder, trigonometric operation, division or `mix`. Four identical lanes guard against a single accidental readback value. [Input bytes](./minimal-input.bin) encode `0.6000000238418579`, `0.241208016872406`, `0.20000000298023224`, and one unused zero. The middle operand is an integer-hash sweep witness, not a claim that this exact seed is a failing full-material pixel.

| Execution | f32 bits | Value |
|---|---|---|
| Original registry wgpu, Release/DXC | `0x3eb07fc5` | 0.3447248041629791 |
| Isolated IEEE-strict diagnostic build | `0x3eb07fc6` | 0.3447248339653015 |
| Ordinary Chrome 153.0.8010.48 | `0x3eb07fc6` | 0.3447248339653015 |
| Ordinary Edge 153.0.4234.32 | `0x3eb07fc6` | 0.3447248339653015 |

[verify.mjs](./verify.mjs) computes the product/sum using exact integer significands and exponents, then rounds to binary32 ties-even. The first answer equals a single final rounding; the second equals rounding after multiplication and again after addition. Their distance is one ULP, `2^-25`. The fused answer is closer to the exact real result: “strict” does not mean more accurate in this example. These are numerical signatures; GPU instruction disassembly was not captured.

The [WGSL draft at revision cd910cf650d05481b60bad2b44476caff974962f](https://github.com/gpuweb/gpuweb/blob/cd910cf650d05481b60bad2b44476caff974962f/wgsl/index.bs), §§15.7.2–15.7.5, permits reassociation and sufficiently accurate fusion, and does not require a rounding mode. Both executions of this finite, positive, normal-number witness are permitted. This establishes conformance of the reduced witness, not a conformance audit of every material operation. WGSL `fma` itself may expand into separate multiply/add, so replacing every expression with `fma` would not establish universal bit identity.

## Connection to production arithmetic

The [eight-expression sweep](./operations.wgsl) uses 65,536 samples per expression. Original versus strict native builds differ in 62,584 cellular coordinate expressions, 8,843 scalar multiply/add expressions, 9,318 `mix` expressions, and 4,796 levels-style output remaps. Both browsers match the strict build across all 524,288 sweep results. The tested fade, explicit `fma`, division and dot-product lanes match across these executions; that observation is limited to these inputs, not proof those operations are universally invariant. [Results](./results.json) retain counts, first witnesses and complete-output hashes.

[cell-trace.wgsl](./cell-trace.wgsl) extracts the production cellular lattice function and evaluates the three octaves and nine neighboring sites for leather's pixel (270, 0), at 1024², seed 271828, scale 64, persistence 0.35. Lattice jitter agrees; coordinate deltas already differ before distance evaluation. In particular the production expression `neighbor + 0.2 + 0.6 * jitter - p` combines small offsets with larger absolute coordinates and then subtracts those coordinates, making reassociation/rounding consequential. Distance differences at that point include propagated delta differences; they do not independently establish a dot-product defect.

The [noise-grid probe](./noise-grid.wgsl) copies the production cellular algorithm and half-rounding helper, but writes raw f32 and rounded-half values to buffers. Of 1,048,576 pixels, 1,032,165 raw values and 6,298 rounded values differ between native compiler policies. These are diagnostic intermediate counts, not the frozen RGBA8 material comparison. A [single-point extraction](./noise-point.wgsl) at (270, 0) gives:

| Execution | Before half rounding | After half rounding |
|---|---:|---:|
| Original native | 0.10690304636955261 | 0.10687255859375 |
| Strict native | 0.10690376162528992 | 0.10693359375 |
| Chrome / Edge | 0.1069038063287735 | 0.10693359375 |

The midpoint is 0.106903076171875. Arithmetic variation crosses it; the integer half-rounding helper consistently rounds its differing inputs. **Strict native and browser raw f32 values still differ** while their rounded values agree. Thus the prior 44/44 PNG match does not establish identical intermediate arithmetic. Extraction and instrumentation can change optimization; these probes demonstrate mechanisms and propagation, not attribution of all 17 failing channel gates to a single instruction. No product shader was changed.

## What the pixel gate guarantees

The [frozen calibration](../m5-05/calibration.md) was based on recorded macOS and Linux implementations. [Browser tolerances](../../browser-tolerances.json) are empirical release regression limits against the declared native reference, not WGSL error bounds or universal cross-device guarantees:

| Channel | Max RGBA8 difference | Max mean RGBA8 difference | Max fraction of changed pixels |
|---|---:|---:|---:|
| baseColor / height | 1 | 0.00001 | 0.00001 |
| normal | 1 | 0.00001 | 0.00002 |
| roughness | 1 | 0.001 | 0.001 |

Any component difference counts as a changed pixel. At 1024², the ratio limits permit at most 10, 20 and 1,048 changed pixels respectively, with mean and maximum limits independently mandatory. A failed gate means this configuration does not meet the frozen comparison contract; it does not alone mean unsupported WebGPU, visually broken material, or a WGSL violation. Plan/semantic equality, structure, seams, causal relationships and exact encoding/checker sentinels remain separate mandatory checks. Passing pixels alone does not establish those properties either.

## Decision supported by this reduction

Keep certification pending and preserve the failed baseline. Do not select a new reference just because it matches browsers, widen tolerances, reset goldens, add developer browser flags, or fork wgpu now. The next bounded experiment should assess numerical conditioning of cellular local-coordinate arithmetic, with before/after evidence against the existing goldens and actual full-material cases. A rearrangement may reduce cancellation but must not be assumed to eliminate all backend variation, preserve existing pixels, or resolve gradient/levels differences. It is not yet an approved production correction.

If the product instead requires bit identity beyond the recorded matrix, specify the operations, quantization boundaries, configurations and supported environments first. Only then evaluate an explicit compiler-policy API, its performance and packaging costs; strict flags alone have already failed to make the noise probe's intermediate f32 values identical. An upstream report can use the minimal witness to discuss the policy/API gap without mislabeling allowed fusion as a compiler bug. No upstream issue or message was sent.

## Reproduction and provenance

Source base is `2cc36863eb5cb4a0f419722b172fb5aceaec239f`; the documentation branch was `ed07716f61bce9771f8492f1999cc737ecc891c3`. [results.json](./results.json) binds probe sources, production shader/fixture/tolerance/lock hashes, executables, native adapter reports and loaded DXC DLL path/hash. Native is GT 1030/DX12, driver 32.0.15.8266. Browser adapter information is redacted; host GPU/ANGLE fields are not inferred WebGPU backend identities. Fresh-profile [Chrome](./chrome-launch.json) and [Edge](./edge-launch.json) receipts contain only profile isolation and local automation switches. Direct WebGPU probes diagnose arithmetic; they do not exercise or certify the archived runtime package.

Run `node docs/evidence/chromium-arithmetic-reduction/verify.mjs` to check the retained input/results against the exact scalar oracle. To rerun GPU execution:

1. Create isolated native checkouts at the source base. Copy [native-probe.rs](./native-probe.rs) to `crates/mixture-wgpu/examples/arithmetic_probe.rs` in each. Keep one original locked registry graph. Apply the previously retained [diagnostic patch](../chromium-compile-policy/diagnostic-only.patch) to a separate copy of wgpu-hal 30.0.1 and add a local Cargo patch only in the second checkout, as described in the earlier investigation. Retain that diagnostic lock delta. Build each with `cargo build --locked --release -p mixture-wgpu --example arithmetic_probe` and distinct target directories.
2. Scope the recorded Chrome DXC directory to the child process PATH. Invoke each executable as `arithmetic_probe.exe <absolute-minimal.wgsl> <output.bin> 4 <absolute-minimal-input.bin>`. Capture stderr (adapter/context) and the emitted `<output.bin>.modules.json` (loaded DLL). Name the outputs `minimal-regular.bin` and `minimal-strict.bin` in a new result directory; copy the input there.
3. Launch fresh browsers using the repository's `scripts/browser-runtime/launch-default-browser.ps1`. Set `MIXTURE_PROBE_PRODUCT` to an existing qualified Studio product directory containing its locked Playwright installation. Run `node docs/evidence/chromium-arithmetic-reduction/browser-probe.mjs <CDP-endpoint> <absolute-minimal.wgsl> <output.bin> 4 <absolute-minimal-input.bin>`. Save outputs as `minimal-chrome.bin` and `minimal-edge.bin`, then run `verify.mjs <new-result-directory>`. The browser probe serves a loopback diagnostic page; no browser feature override is used.
4. The same runners accept `operations.wgsl` with count 524288, `noise-grid.wgsl` with 2097152, `noise-point.wgsl` with 2, or `cell-trace.wgsl` with 216. Omit the input-file argument for zero-initialized input. Compare little-endian f32 bit patterns; operation lanes repeat every eight entries, noise-grid raw/half lanes every two, cell-trace fields every eight. Do not treat diagnostic comparisons as formal material pass receipts.

Git retains the small input/output buffers, executable probe sources, contexts, scalar oracle and measured summaries. Large sweep readbacks and executables remain temporary in ignored `tmp`; their hashes do not promise permanent availability. No rendered output or golden was changed, so no new visual acceptance is claimed.

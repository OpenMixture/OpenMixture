# Warp dual-reference rounding audit — 2026-09-18

English | [简体中文](./README.zh-CN.md)

This is a bounded ALPHA-03 diagnosis, not acceptance of PR #15/#16 or a new numerical contract. Product shaders, old goldens, fixture parameters, tolerances, and node versions are unchanged. All GPU pixel execution uses wgpu/WebGPU and the recorded production WGSL. The Python rational calculations adjudicate scalar witnesses; they are not a CPU material renderer.

**Verdict: all 41 changed software warp texels improve against exact reference A and match staged reference B after PR16. All 14 previously recorded final PNG changes are explained by propagation from 11 of those warp texels.** The candidate has an independently verified numerical benefit on this bounded input set, but still changes old pixels and cannot implement A for all inputs: the independent double-rounding witness produces B instead.

## Actual software pixel adjudication

The [software verdict](./software/verdict-summary.json), [all scalar rows](./software/adjudication.json) and [final pixel dependencies](./software/final-witness-links.json) retain the complete changed set.

| Variant | Changed warp half texels | Before matching A / B | After matching A / B | Original final PNG changes |
|---|---:|---:|---:|---:|
| default | 26 | 0 / 0 | 26 / 26 | 10 across four channels |
| coarse-grain | 15 | 0 / 0 | 15 / 15 | 4 across three channels |
| horizontal-grain | 0 | not applicable | not applicable | 0 |
| straight-grain | 0 | not applicable | not applicable | 0 |

For all 41 points, the exact and staged neighbor selections agree; measured delta/weight match B. The old interpolation probe lands exactly on the midpoint between the two observed half values and matches B_unfused; the FMA probe moves to the side selected independently by A and B. Both half probes reproduce the corresponding full production texture. This localizes these differences to interpolation rounding rather than changed inputs, neighbors, half conversion or readback. It does not prove which physical machine instruction ran.

Example, the pre-existing coarse-grain baseColor witness `(317,863)`: exact A is `71772926317/137438953472` (`0.5222167697284021`), below half midpoint `0.522216796875`. The old interpolation reaches that midpoint and rounds to `0.5224609375`; the FMA candidate rounds to the correct A/B half `0.52197265625`. Height bits change from `13940` to `13937`; the recorded blue output changes from `48` to `47`. No Native output was used to select the reference.

The [software PNG binding](./software/product-png-binding.json) and [native binding](./native/product-png-binding.json) each reproduce **all 32 channels** from the prior PR15/16 product outputs with zero changed pixels, not only the 14 witnesses. The normal-map links include adjacent height texels. This establishes propagation, not a mathematical oracle for the complete nonlinear material/PNG pipeline. The historical original-frozen-golden failures remain unchanged.

[Replaying the same 41 software inputs](./software/cross-backend-replay.json) in native DX12 and ordinary Chrome yields identical delta/weights and identical FMA f32/half results on all three tested backends. Native's old mix already matches A at all 41; Chrome and SwiftShader old mix match A at none. These are buffer replays of the actual frozen inputs, not additional full-material browser qualification.

## References fixed before the software experiment

[oracle.py](./oracle.py) defines independent nearest, ties-to-even models. The reference definitions were committed as `8e7b3b0592368d7aad0fb3fa316b9d09fd6aa880` before the software capture ran. None is asserted to be WGSL's mandatory execution sequence.

- **A:** exact arithmetic on actual binary16 texture values and binary32 strength parameters, through displacement, wrapped neighbor selection and bilinear interpolation, followed by one binary16 rounding.
- **B:** separately rounded binary32 operations for field normalization, strength and size multiplication, and fractional weights; rounded endpoint differences and fused x/y interpolation; then binary16 rounding.
- **B_unfused / B_weighted:** the same staged coordinates with, respectively, separate difference/multiply/add or separately rounded weighted-sum interpolation.
- **A_frozen_weight:** exact interpolation after freezing B's neighbors and weights. This isolates coordinate error from interpolation error; it is not A's complete sampling reference.

All four existing wood variants are captured for both PR15 and PR16. Selection retains **every changed warp texel**, then independently traces the pre-existing 14 PNG witnesses. No failing point is removed and no reference is selected after inspecting GPU output. Single-result delta, weight, interpolation and half probes are compared with unchanged production texture output; their results do not silently replace that output.

## Contract-discriminating literals

[check_literals.py](./check_literals.py) independently checks all 23 fixed PR16 literal inputs: A, B and B_weighted each agree with all 23 expected cases; B_unfused agrees with 22. Thus the old tests cannot distinguish A from B. This is a new arithmetic recomputation, not a new GPU run of those 23 cases.

The added double-rounding witness uses input `[0.5,1]`, size `[2,1]`, field `1` and strength bits `0x39800001`. Its local weight is exactly f32 `0x3a000001`. A yields `0.50048828125`, whereas B yields `0.5`. Native DX12, ordinary Chrome and pinned SwiftShader all produce `0.5` with the production FMA candidate. Native/Chrome single-result interpolation is already the half midpoint `0.500244140625`; the explicit half helper and actual texture agree. This discrepancy requires neither a texture-copy error nor an unfused FMA. Software literal coverage is production texture output; the initial software run did not include the later supplemental literal buffer probes.

The previous half-boundary witness remains useful: A/B require `0.343505859375`; native produces it, Chrome's local-only candidate produces `0.34326171875`, and Chrome's FMA candidate produces the A/B value. Its interpretation is case-specific improvement, not universal exact rounding.

For the 1024×1 tiny-displacement case, both native and Chrome lose the original UV displacement at pixel 256. Local/FMA preserve it. The additional normal-range control changes alternating input from `[0,1]` to `[2^-14,1]`, keeping all other parameters fixed: old output `2^-14`, local/FMA output `5*2^-16`, agreeing with A and B. The original subnormal test remains. This control separates coordinate cancellation from possible subnormal texel-copy behavior; [WebGPU's copy rules](https://www.w3.org/TR/webgpu/#texel-copies) allow subnormal values to be replaced by zero, but that permission is not evidence that flushing occurred on these tested devices.

## Reproduction

The [source identities](./source-identities.json) bind the material, variants, lockfile, helper, unchanged upstream/downstream shaders and three warp sources. Text identities normalize line endings to LF. `generate.py` refuses source drift. Shader copies are diagnostic evidence, not runtime kernel alternatives.

From this checkout, copy `texture_probe.rs` and `buffer_probe.rs` into the mixture-wgpu examples directory as `rounding_texture_probe.rs` and `rounding_buffer_probe.rs`. Build using:

```sh
cargo build --locked --release -p mixture-wgpu --features software-vulkan --example rounding_texture_probe --example rounding_buffer_probe
python docs/evidence/warp-rounding-audit/run.py
python docs/evidence/warp-rounding-audit/check_literals.py
```

For pinned SwiftShader, first use `.github/scripts/setup-swiftshader.sh`, select its ICD explicitly and set `MIXTURE_GPU_BACKEND=vulkan`. On Windows, the default probe backend is DX12; put the recorded Chrome DXC directory on PATH for the compiler-confirmed scalar replay. Keep a fresh output directory under `tmp/warp-rounding-audit`. Remove the two temporary example files after capture; they are not product APIs. The initial software capture used the temporary workflow wiring in the recorded experiment commit.

For ordinary Chrome, launch a fresh profile with the repository's `scripts/browser-runtime/launch-default-browser.ps1`; the only launch switches are profile isolation and loopback debugging. Set `MIXTURE_PROBE_PRODUCT` to the existing independent consumer containing Playwright. `browser-literals.mjs <endpoint> <cases-json> <combined-precision-and-warp-wgsl> <report-json>` executes the literal textures without asserting one of the competing references. `browser-probe.mjs <endpoint> <shader> <output> <float-count> <input-bin>` executes the retained single-result buffers. This is browser diagnosis, not installed-package qualification.

`compare_png.py <capture-dir> <PR15-PNG-root> <PR16-PNG-root>` requires NumPy/Pillow and compares **all** pixels in 32 recorded output channels; only the existing readback encoding is mirrored. `link_witnesses.py <capture-dir>` links the 14 pre-existing PNG differences to captured warp/height changes, including normal-map neighbor dependencies. The original material PNGs remain unchanged.

## Run identity and retention

The [receipt](./receipt.json) binds experiment head `8e7b3b0592368d7aad0fb3fa316b9d09fd6aa880` to actual CI merge checkout `a200b127defe66deec2dd76e70eeae6c8652023d`, [successful software run 35321683099, attempt 1](https://github.com/OpenMixture/OpenMixture/actions/runs/35321683099), and SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`. The job's existing smoke, packaged consumption, frozen material checks and 2K trace passed for the unchanged main runtime. This does not turn the separately dispatched PR15/16 shaders into accepted runtime changes. Later documentation commits are not that tested checkout.

Native execution used GT 1030 / DX12, driver `32.0.15.8266`; ordinary Chrome was `153.0.8010.48` / NVIDIA Pascal. Compiler-confirmed scalar replay records Chrome's DXC module path. Supplemental local analysis/replay sources and buffers are retained with their hashes. The local `cargo xtask check`, final diagnostic Rust compilation, Python reference checks and JS syntax checks passed. No Edge or Firefox GPU run is added here.

Git retains 24 logical software input/warp/height buffers in a content-deduplicated archive, all 41 scalar inputs/results and references, all 14 final dependency links, literal textures, source plans and adapter receipts. `texture-aliases.json` resolves byte-identical buffers without storing duplicates. The [artifact receipt](./software/gpu-artifacts.json) records artifact `10538370576`, size `113355138`, service digest and expiry `2026-10-18T08:20:33Z`; retrieval and offline verification completed on 2026-09-18. Complete routine captures and the original full 32-channel PNG sets remain local/in expiring CI artifacts. Their hash bindings do not promise perpetual re-audit of every original PNG byte. No new visual baseline is accepted.

Run `python docs/evidence/warp-rounding-audit/verify.py` without a GPU to verify retained content hashes, reconstruct all changed texels from the archive, recompute the exact references, verify scalar inputs/results and trace the final witnesses. Text is retained with LF line endings; binary buffers remain byte-exact. This review is agent-generated numerical evidence, not independent human visual approval.

The software texture archive is losslessly repacked as a 4,917,964-byte tar.xz and stored in 64 KiB parts because larger Git/API requests failed. `archive-parts.json` records their order and the complete archive size/SHA-256. The verifier concatenates them in memory, verifies the archive, and checks every decompressed buffer against its original SHA-256. JSON whitespace is minimized for four larger reports. Neither repacking nor text formatting changes the raw buffers, numeric values or experiment results.

## Scope limits

This audit does not select A or B as a product promise, prove conformance for every legal input, certify ordinary Chrome/Edge full materials, or establish a compiler violation. It does not classify every remaining noise/material failure. Existing software regression, browser comparison and structural/visual gates remain separate obligations. Any future intentional numerical rule or compatibility migration requires its own reviewed decision; changing a node version alone does not define rounding semantics.

The next decision is a narrowly scoped numerical/compatibility proposal using this quantified benefit and known old-pixel cost. The 14 PNG differences now have a demonstrated improved warp input, rather than evidence of interpolation degradation. Keep PR15/16 draft and old gates active until that decision. Noise-path diagnosis and complete ordinary-browser qualification remain separate work.

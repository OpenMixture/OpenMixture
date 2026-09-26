# Unresolved painted-metal software parity failure

English | [简体中文](./README.zh-CN.md)

The [Chromium run](https://github.com/OpenMixture/OpenMixture/actions/runs/35836054345) for PR #59 head `611e9c4afad7e4b6558a1c5a4bf538d13196650e`, tested merge `9df804336d295c74b72f32b81371dc81a1b425b9`, fails the frozen painted-metal Native/browser normal gate: maximum difference is 4/255 at 1K and 8/255 at 2K, against the unchanged ≤1/255 limit. Cost and timing checks pass; that does not qualify pixels. Later original/noise-v2 browser material steps did not execute. The candidate remains unaccepted and unpublished.

[receipt.json](./receipt.json) binds the retained reports and images by size and SHA-256, records the source identities and artifact expiry, and distinguishes the Windows control from Linux qualification. [ci-native.json](./ci-native.json) records the failed comparisons. Native/browser normal and height images at both resolutions remain here for inspection; complete ordinary output remains in artifact `10739483613`, expiring 2026-10-23. The material input is the unchanged [before fixture](../perf-mat-before/material.mix).

The Windows control used Chrome 153.0.8010.53's bundled software Vulkan driver: source `ddb2e831e0397ccc0e07eebb27c8684166072a11` (0.7 release) and `611e9c4afad7e4b6558a1c5a4bf538d13196650e` (0.8 debug) produced identical five-channel 1K PNG bytes. Different profiles prohibit a timing comparison. Decoded Windows pixels also match the retained Linux browser pixels exactly. The [decoded comparison](./decoded-comparison.json) finds 422 differing normal pixels against Linux Native, maximum 4. This control does **not** establish Linux before/after equivalence or identify the first divergent node. No shader repair or precision-policy decision follows from it alone.

For the next bounded diagnostic, the existing Chromium job invokes [probe-painted-metal.py](../../../scripts/probe-painted-metal.py) only after the reuse gate fails. It uses the production CLI and the same pinned Native adapter to render eleven fixed Scalar stages through height and normal at 1K. Each derived `.mix`, command, plan hash, CLI report and image is retained under `tmp/painted-metal-stages/` in that run's artifact. Slicing changes allocation, so these outputs locate candidate divergence rather than prove unchanged full-graph behavior. The diagnostic is not a passing gate, an alternative renderer or a platform qualification. No tolerance, golden, required check or shader changes.

Local reproduction with an explicitly configured software Vulkan driver and freshly built CLI:

```bash
python scripts/probe-painted-metal.py --cli target/release/mixture --output tmp/painted-metal-stages
```

Use the actual CLI path on Windows. The output directory must not exist. The script needs only Python's standard library. WSL experiments were stopped; no WSL build or result is used as acceptance evidence. Guide impact: no operational rule changed; this record preserves failure evidence and the diagnostic keeps the existing six gates intact.

## First differing stage — 2026-09-23

[Run 35869537767](https://github.com/OpenMixture/OpenMixture/actions/runs/35869537767) reproduces the failing gate and successfully captures all eleven stages. Its [source/artifact receipt](./stages/receipt.json) and [decoded comparison](./stages/comparison.json) bind Linux Native release output to the earlier Windows Native software control. All derived graph objects and plan hashes match. Macro, exposure, erodeX/Y, band, rustSpread, detailNoise and coatingHeight have identical height and normal pixels. The first observed difference is `detail`: height differs at 5,269 pixels (max 1), normal at 62,663 (max 16). RustMask then differs, and final height/normal reproduce the full material's 3/422 changed pixels and max 1/4 respectively at 1K. This is a Native software diagnostic, not another accepted browser matrix.

The `detail` node is `levels@1` with gamma 1 and [0,1] → [0.5,1] remapping. The retained [Linux](./stages/linux-detail.mix) and [Windows](./stages/windows-detail.mix) requests and their height/normal images preserve this first-divergence evidence. The remaining full stage PNGs are ordinary diagnostic output in artifact `10754816938` (expires 2026-10-23); the retained [comparison script](./stages/compare.py) requires Pillow and the two complete stage directories to reproduce their decoded comparison. Their hashes alone do not preserve those omitted images.

Gamma-one `pow` evaluation near half-float rounding midpoints is a hypothesis, not yet an isolated Linux result. The diagnostic now also renders twelve 1x1 literal controls without noise: exact endpoints and half-representable inputs whose affine result is a rounding midpoint. Independent constant references use round-to-nearest-even binary16; two saturating subtractions and an exact endpoint amplifier expose positive and negative half-step errors in height/roughness. Expected error pixels are `[0,0,0,255]`. The Windows software control returns zero for all twelve; Linux results are pending. Reports remain diagnostic, and no production shader, version, tolerance or migration policy changes here.

## Literal follow-up

[Linux run 35875437605](https://github.com/OpenMixture/OpenMixture/actions/runs/35875437605) completed the twelve original samples with zero error in both signed channels ([retained measurements](./linux-linear-followup.json)). Thus the original literal sample does not reproduce the material failure. A separate [gamma-one correction candidate](../../levels-linear-correction.md) adds exact fixture inputs within the observed differing bins and requires the complete frozen material comparison before acceptance. The earlier failed pixels remain unchanged.

Retention exception: `ci-native-stderr.log` preserves the original failed Native comparison diagnostic used to justify the subsequent correction. It remains a failed historical result and its bound bytes are unchanged.

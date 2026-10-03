# MAT-03 recipe revision 2 — bounded material defaults

English | [简体中文](./mat-03-default-revision.zh-CN.md)

Candidate material-default revision; materialAccepted=false. Only underRatio changes 0.5→0.25 and crown 0.5→0. Relief=0.025, normalStrength=0.5, counts/widths, grain, colors and all gates stay unchanged. Per-pixel visibility at the yarn edges is correct weave-pattern@1 behavior. No node, Rust/WGSL/Core, version, golden or other material changes.

## Metric defined before execution and provenance

The [main contract](./mat-03-woven-surfaces.md) first defined the half-occupancy-core visible-width metric in a82db80: public material.mix/CLI with diagnostic red warp, green weft, blue backing and detailAmount=0; compare actual GPU PNG R>G or G>R. Include every scanline in each full axial cell. The denominator core is abs(fract(coord*count)-0.5)≤(width-bevel)/2. Plain has 32 over crossings and 4096 scanlines per axis; varied has 48 crossings per axis, 6144 warp rows and 4096 weft columns. Monotonic sRGB preserves R/G ordering. Only delivered bytes are measured; there is no CPU pixel executor.

Sweep source is a82db80 (from main 91fdfcf), revision-1 material.mix SHA-256 5cda745457b639c6a320aa445de0e34d2f990954a983900013010737ae8899a2, release CLI SHA-256 a07c565ea50079025ab24b535b57f99d47e4c66a802a24554e866364dcd1766e; GT 1030 Vulkan, NVIDIA 582.66. Thirty public CLI renders cover 15 combinations × two presets. Commands, requests, adapter reports, PNGs and metrics are in ignored tmp/mat03-revision/sweep/ and sweep.json. The uncommitted sweep tool is in the same ignored directory; the main-contract metric fully specifies reproduction.

## Sweep results (min / median)

| underRatio | crown | Plain warp | Plain weft | Varied warp | Varied weft |
|---|---|---|---|---|---|
| 0.25 | 0 | 0.8750 / 0.9750 | 0.8750 / 0.9750 | 0.9091 / 0.9556 | 0.8367 / 1.0000 |
| 0.25 | 0.25 | 0.8500 / 0.9500 | 0.8500 / 0.9500 | 0.8667 / 0.9545 | 0.7959 / 1.0000 |
| 0.25 | 0.5 | 0.8000 / 0.9500 | 0.8000 / 0.9500 | 0.8222 / 0.9222 | 0.7551 / 0.9898 |
| 0.25 | 0.75 | 0.7500 / 0.9250 | 0.7500 / 0.9250 | 0.7778 / 0.9091 | 0.7143 / 0.9898 |
| 0.25 | 1 | 0.7250 / 0.9125 | 0.7250 / 0.9125 | 0.7333 / 0.8667 | 0.6939 / 0.9796 |
| 0.5 | 0 | 0.7250 / 0.9250 | 0.7250 / 0.9250 | 0.7556 / 0.8889 | 0.7143 / 0.9796 |
| 0.5 | 0.25 | 0.7000 / 0.9125 | 0.7000 / 0.9125 | 0.7111 / 0.8667 | 0.6735 / 0.9796 |
| 0.5 | 0.5 | 0.6500 / 0.9000 | 0.6500 / 0.9000 | 0.6667 / 0.8540 | 0.6327 / 0.9796 |
| 0.5 | 0.75 | 0.6000 / 0.8875 | 0.6000 / 0.8875 | 0.6222 / 0.8222 | 0.5918 / 0.9694 |
| 0.5 | 1 | 0.5750 / 0.8750 | 0.5750 / 0.8750 | 0.5909 / 0.8091 | 0.5510 / 0.9694 |
| 0.75 | 0 | 0.5500 / 0.8625 | 0.5500 / 0.8625 | 0.5455 / 0.7778 | 0.5306 / 0.9592 |
| 0.75 | 0.25 | 0.5000 / 0.8625 | 0.5000 / 0.8625 | 0.5000 / 0.7727 | 0.4898 / 0.9592 |
| 0.75 | 0.5 | 0.4500 / 0.8500 | 0.4500 / 0.8500 | 0.4667 / 0.7556 | 0.4490 / 0.9490 |
| 0.75 | 0.75 | 0.4250 / 0.8375 | 0.4250 / 0.8375 | 0.4222 / 0.7556 | 0.4082 / 0.9490 |
| 0.75 | 1 | 0.4000 / 0.8375 | 0.4000 / 0.8375 | 0.4091 / 0.7333 | 0.3878 / 0.9490 |

## Selected defaults and limits

Select underRatio=0.25, crown=0, the grid point maximizing the minimum across all four groups. Plain both axes: min 0.6500→0.8750, median 0.9000→0.9750. Varied warp: min 0.6667→0.9091, median 0.8540→0.9556; weft: min 0.6327→0.8367, median 0.9796→1. Worst per-tile width deficit falls by 64.3% and 55.6%, meeting the predefined halving target. All groups have max=1.

**Residual pinch remains:** worst deficit is 12.5% plain and 16.33% varied. This materially reduces the hourglass contour but does not establish perfectly straight sides or elimination of every bone-like appearance; maintainer visual review remains necessary. Crown=0 gives T=q=1-(2d/width)², a curved parabola, not a plateau. Analytical crossing-center upper/lower heights remain 1>0.25 (0.025>0.00625 after relief); positive relief retains strict separation and a larger centerline dip without weakening normals. True-color/normal crops show rounded profiles and a readable dip; this is not Stage C PBR/human acceptance.

These two defaults cannot guarantee a ratio of 1: at a plain warp-over crossing center in v, take transverse local u=0.30 (inside the half-occupancy core ending at 0.31). The maximum warp T is q=1-(0.60/0.70)²≈0.2653 at crown=0. The minimum weft height is 0.25+0.75*smooth(0.30)=0.412, with transverse weft T=1. Their difference is below -0.06 while e≤0.06, so frozen visibility selects weft. Increasing crown only lowers this warp profile; increasing underRatio only raises that weft height. This concrete counterexample bounds any claim of fully straight sides; it is neither a node defect nor permission to widen ranges.

Finalists baseline=(0.5,0.5), selected=(0.25,0), runner-up=(0.25,0.25) each retain plain/varied five-channel 1K images at tmp/mat03-revision/finalists/{label}-{preset}/. baseColor-crossing.png and normal-crossing.png are byte-verified 256² crops without scaling: origin (192,192) for plain and (85,192) for varied, identical across channels and candidates for each preset.

## Explicit revision and unchanged gates

Retain the [revision 1 plan](../fixtures/materials/woven-fabric/qualification-plan-v1.json), [material](../fixtures/materials/woven-fabric/material-v1.mix) and [recipe](../fixtures/materials/woven-fabric/graph-proposal-v1.json) byte-for-byte. The new plan carries recipeRevision=2 and previousPlan; its inherited reviewMeasurements/freezeBlockers remain revision-1 records, not qualification of new pixels. Revision-2 results are appended to this document rather than rewriting historical receipts.

Ten cases inherit both changed defaults: plain, varied, no-detail, flat, warp-seed, weft-seed, neutral-normal, constant-low, constant-high, max-normal; dense-thin stress inherits them too. Combined-low and combined-high explicitly override both and remain identical. Case membership and every controls object stay unchanged; no relative-default expression changes. The six literal defaults on three weave instances change together; topology and every other parameter stay identical. A regression checks the retained plan byte hash and compares the entire plan/graph/material after restoring only the two named defaults and revision metadata, rejecting threshold, size, case, range, stress, budget or timing drift.

Re-run every Stage A gate; stop on any frozen failure without relaxing it. B structural probes, C dielectric PBR/human decision and D retained evidence/acceptance remain separate PRs; no publication.

## Revision-2 Stage A rerun

Measured on clean source 472ee12598d7e5d84487912f6e1493a2349a70aa (subsequent commit only records these results). Material SHA-256 95db023744a09224de3344613fe503c1e2187590b48667ef5ac199ae49959757; plan 7f2f3181cfaf4a4b92e7e49f2818b63d1879b010aa692e97984aebdf48a40c91; builder 5e99d82f0b4f5bdde8be1de58cbd69ec98ac7916437886a3dcafd4831d9c1dda. Ordinary receipts remain in ignored tmp/mat03-revision/; this is source-bound Stage A evidence, not retained Stage D acceptance.

- Builder: 4 tests pass, including whole-plan/recipe/material unchanged-gate assertion and literal revision-1 plan hash. Retained plan, graph and material bytes independently match main 91fdfcf.
- Release CLI: all 51 rows validate and inspect successfully; 21 passes, 8 physical textures. Descriptor peaks: 256² 4,719,184 B; 1024² 75,498,064 B; 2048² 301,990,480 B; 257×129 2,419,600 B. Budgets remain 64 passes / 536,870,912 B.
- Native release matrix: 51/51 Vulkan and 51/51 DX12 on NVIDIA GeForce GT 1030 (Vulkan NVIDIA 582.66; DX12 32.0.15.8266). Exact repeat/package/sliced-height/owned-output, allocation and endpoint gates pass. Default/varied downsample worst component mean error 0.3352384567/255 ≤4/255 on both backends; stress remains separately recorded, not a quality qualification.
- Clean browser build and installed Chrome 154.0.8037.98 candidate: 23 passed, none skipped/flaky; all 51 woven rows pass. check-woven.mjs re-runs Vulkan and passes all 255 channel comparisons, max component delta 1/255. Browser gated downsample maximum is also 0.3352384567/255. Browser buildId is sha256:c8f1b9f25649b7c01fea0904680b8e63e244ae53d933e5b9fe14f4acee1f0b03. This records Chrome WebGPU, without claiming an undisclosed browser adapter identity.
- cargo xtask test-consumer passes. The ten selected finalist channel images exactly match the revised Native 1K matrix pixels. Raw outputs, source identity, CLI commands and comparison receipts are under the same ignored root.

Matched-GT-1030 timings below are cold / warm median milliseconds (Vulkan from the final browser comparison, DX12 from its standalone matrix). Every frozen timing gate passed; no target changed.

| Preset / size | Vulkan | DX12 |
|---|---|---|
| plain 1024² | 251.56 / 188.14 | 847.64 / 184.88 |
| plain 2048² | 821.17 / 753.92 | 1528.58 / 723.16 |
| varied 1024² | 243.86 / 186.57 | 888.82 / 185.26 |
| varied 2048² | 851.47 / 760.84 | 1519.57 / 729.74 |

No frozen gate failed. SwiftShader timing was not requalified locally; its frozen target remains unchanged. Residual outline narrowing, maintainer visual decision, and separate B/C/D stages remain open; materialAccepted=false.

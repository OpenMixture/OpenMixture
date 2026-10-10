# MAT-03 recipe revision 3 (weave-pattern@2) — accepted within recorded scope

English | [简体中文](./README.zh-CN.md)

On 2026-10-10 (Asia/Shanghai) the maintainer accepted woven material recipe revision 3 with the reply "Accept revision 3"; see the [human decision](./human-decision.json). Revision 3 replaces the three weave-pattern@1 instances with the approved [weave-pattern@2](../../weave-pattern-v2-design.md) (candidate C: occupancy-owned visibility with continuous stacking). Only the node version changes: topology, parameters, defaults (underRatio 0.25, crown 0), cases, sizes, thresholds and budgets are those of revision 2. The [revision-2 acceptance](../mat-03/README.md) and its receipts remain unchanged history; its combined-high bow-tie limitation does not apply to revision 3. Rust 0.10.0 / browser 0.10.0-alpha.0 remain unpublished.

## Source and frozen plan

All results bind clean source revision recorded in [binding.json](./binding.json) (working tree clean, `engineDirty: false`). The [qualification plan](../../../fixtures/materials/woven-fabric/qualification-plan.json) was frozen as revision 3 before any run; revision 2 is kept byte-for-byte as `qualification-plan-v2.json`, `material-v2.mix` and `graph-proposal-v2.json`, and a builder test pins their hashes and allows only the three weave version fields to differ.

Two dated Stage B amendments make probe expectations follow the declared node version. Both were found by the first revision-3 run and are derived from the approved @2 formulas, not fitted to pixels:

| Amendment | @1 expectation (kept as history) | @2 expectation |
|---|---|---|
| `2026-10-10-weave-v2-crossing-oracle` | stored height matches the four-tap upper-yarn oracle within 1/1024 | stored height matches the four-tap @2 stack `Zw+Zf−Zw·Zf` within the same 1/1024 |
| `2026-10-10-weave-v2-isolation` | crown and underRatio change baseColor, roughness, height and normal | crown and underRatio change only height and normal; baseColor, roughness, metallic, coverage and warp-share stay byte-identical (stricter) |

Forcing the @1 crossing oracle against @2 pixels fails at 257×129 (H 0.99707 vs 0.99586), confirming the first amendment was required.

## Machine gates — NVIDIA GeForce GT 1030, Vulkan (NVIDIA 582.66) and DX12

| Gate | Result |
|---|---|
| Stage A Native matrix | 51/51 rows on each backend; exact repeats, loose/package equivalence, endpoints, allocation accounting ([Vulkan](./machine/native-vulkan.json), [DX12](./machine/native-dx12.json)) |
| Plan cost | 21 passes; 2048² peak 301,990,480 descriptor bytes (≤536,870,912), unchanged from revision 2 |
| Native vs Chrome | 255 channel comparisons, maximum component delta 1/255 ([comparison](./machine/comparison.json), [browser](./machine/woven-browser.json)) |
| Downsample quality | worst plain/varied mean error 0.3043/255 (limit 4/255); stress 1.8189/255 recorded, ungated |
| Stage B | six structural probes and noise periodicity pass on both backends; 640 crossings; 16 control-isolation variants; weave translation 672 gated comparisons ([summary](./machine/stage-b-summary.json)) |
| Odd-size warp-share (observation only) | max 1 half step for every case except combined-high (7); max visible-weight difference 0.00049. Revision 2 on @1 reached 27 steps at combined-high |

Release timing (cold 1K / warm-median 1K / warm-median 2K, ms): Vulkan plain 253 / 192 / 803, varied 251 / 198 / 795; DX12 plain 892 / 193 / 766, varied 1149 / 197 / 819 — all inside the frozen hardware budgets (10 000 / 1 000 / 4 000).

## Visual review and decision

The unchanged [dielectric review tooling](../../../fixtures/materials/woven-fabric/README.md) rendered six sheets from the clean Native/Chrome comparison output: [plain](./review/plain-pbr.png), [varied](./review/varied-pbr.png), [warp-seed](./review/warp-seed-pbr.png), [weft-seed](./review/weft-seed-pbr.png), [combined-low](./review/combined-low-pbr.png), [combined-high](./review/combined-high-pbr.png). [preview.json](./review/preview.json) binds the producer build, Chrome version, every input map hash (retained under `review/inputs/`) and the roughness sanity check. Its `humanAccepted: false` is the producer receipt at generation time and is not rewritten.

Agent observations presented before the decision: exposed yarns are straight-sided at every preset (the revision-2 hourglass is gone); combined-high no longer shows bow-ties, while its 4× close-up has fairly dark shading in the under-pass dips; 4× close-ups are soft because they magnify 1024² maps. The maintainer chose "Accept revision 3" over the option of recording the dark dips as a limitation.

## Limits and retention

Only the recorded adapters are qualified; software passes never qualify hardware pixels. Odd-size translated warp-share stays observation-only; software adapters omit 2048² under amendment 4. Nothing is published.

Git keeps the six reviewed sheets, the preview receipt, every bound input map and the machine receipts (81 files, about 17.5 MB, none above 4 MiB). Full comparison image sets, raw logs and per-row Stage B output stay in ignored `tmp/` and are not durable.

# Leather candidate

English | [简体中文](./README.zh-CN.md)

PR-009 adds a brown pebbled leather-like surface using nine total built-in node types. [material.mix](./material.mix) uses five compute passes: seeded cellular fractal noise → levels height → gradient-map color, height-to-normal, and inverted levels roughness. All four 1K output channels are connected. The renderer reports 50,331,792 peak estimated bytes; this is an allocation estimate, not process or driver memory.

Local software/hardware machine gates and the [controlled PBR comparison](./reports/pbr/comparison.png) are complete. [Human acceptance](./reports/human-review.json) is recorded. M3 and remote CI were still open at PR-009; the [full M3 review](../../../docs/m3-review.md) is now complete and the documented [remote CI gates](../../../docs/evidence/remote-ci/README.md) are closed. Historical reports and the accepted ceramic fixture's pixels remain unchanged.

## Parameters and causality

| Public control | Contract range / default in this material | Purpose |
| --- | --- | --- |
| `seed` | u32 `[0,4294967295]` / `271828` | Deterministic grain arrangement; always explicit in source. |
| `grainScale` | Integer `[1,128]` / `64` | Cells per UV repeat; `32` makes broader grains. |
| `detail` | Float `[0,1]` / `0.35` | Relative higher-octave amplitude; `0` uses one octave and `1` weights all three equally. |

The node contract default for persistence is 0.5; this material deliberately selects 0.35. [Variants](./variants/) exercise detail minimum/default/maximum and a second causal grain-scale change. Palette, height remapping (`inputMax=0.6`, `gamma=1.6`), normal strength (`0.002`), and roughness bounds (`0.76 → 0.52`) remain fixed across cases. Relief peaks are slightly smoother/brighter than valleys. The detail control changes both frequency and contrast because octave averaging is normalized; it is not a contrast-preserving filter.

The declared frequency measurement is mean squared adjacent red differences (both axes, including wrap), divided by red variance. Normalization avoids treating contrast alone as finer structure. Measured height ratios relative to default on SwiftShader are:

| Case | Normalized gradient energy ratio | Height changed-pixel ratio |
| --- | --- | --- |
| `detail-min` | 0.8892 | 95.01% |
| `detail-max` | 3.0966 | 98.03% |
| `coarse-grain` | 0.2867 | 99.25% |

Checks also require spatial variation, positive adjacent correlation, bounded repeat-edge jumps, unit-length positive-Z normals, and height/normal direction agreement. At least half of all axis samples must have measurable height slopes; four-byte central differences avoid nearly neutral PNG quantization. Every measured sign agrees on both tested adapters. These checks reject scrambling, a synthetic seam, flat/invalid normals and flipped tangent direction. They complement full-image goldens and appearance review; histograms alone do not accept leather.

## Reproduce

From the repository root, explicitly select the pinned software adapter (see [GPU setup](../../../docs/gpu-context.md)):

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask test-material leather

MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask test-material leather
cargo xtask golden check
cargo xtask check
```

`golden check` needs the same explicit GPU policy environment and checks all material fixtures. Output is written under `tmp/golden/`; it never updates [expected/](./expected/). The separate `cargo xtask golden update leather --accept` consumes an existing, intact software candidate without rendering and refuses CI. See [protected update rules](../../../docs/material-goldens.md).

Software comparison is exact. Hardware requires max absolute error ≤1 byte, mean RGBA error ≤0.15 byte, and zero pixels above one byte. An initial mean limit of 0.10 failed at 0.1080 for detail-min roughness, despite max error 1 and all structure checks passing. [The original report](./reports/metal-initial-tolerance-failure.json) is retained. The final policy allows the measured quantization spread while tightening the previous maximum from 2 to 1 and eliminating all permitted >1-byte pixels. No shader or baseline PNG was changed to resolve that threshold failure.

## Visual evidence and scope

[Review instructions](./review/README.md) reproduce four frames using Blender 4.5.13. Verified baseColor, roughness and the actual generated normal drive the same BRDF and lights in every case. Height remains a diagnostic input; applying it as a second bump would double-count the already generated normal. No additional surface noise or relief is supplied by the scene. Sphere curvature and backing edges are display geometry.

The [reports](./reports/README.md) bind inputs, shader/tooling source, adapter, plan, metrics, scripts and images. The current target is pebbled leather-like appearance, not a scanned or physically calibrated hide. Out of scope for PR-009: transform/warp, wood, coat, sheen, AO, curvature, scatter, 2K optimization, a runtime 3D viewer, remote CI closure and the thirteenth node.

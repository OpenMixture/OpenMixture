# MAT-01 — structured brick and paving materials

English | [简体中文](./mat-01-structured-materials.zh-CN.md)

Status: MAT-01a implementation contract, 2026-09-23. This is the bounded design selected under the [material roadmap](../ROADMAP.md), not a claim that the new nodes or material pixels are implemented or accepted. MAT-01b/c implement it; MAT-01d records qualified sources and review. Publication remains separate.

## Material brief and reviewed catalog delta

Produce a seamless, parameterized family of rectangular fired-clay bricks and paving blocks. The visible target has individually varied blocks, readable recessed mortar, softened block edges and coherent baseColor, roughness, height and normal. Regular paving and alternating staggered brickwork must both be expressible. It is not a chipped-stone generator, photographic reconstruction or mesh displacement renderer.

Two new type/version identities are selected: `brick-pattern@1` (MAT-01b) and `scalar-mask-blend@1` (MAT-01c). No existing identity changes. The completed catalog will have fifteen types and thirteen kernels; `fractal-noise@1` remains supported alongside the latest v2 contract. These counts describe the selected delta, not future admission limits. A bounded brick generator directly owns its repeated rectangular profile; a general shape/scatter system is unnecessary for this use case. The second node supplies reusable spatial Scalar composition without changing uniform-weight `scalar-blend@1`.

Core owns both contracts, full-graph validation, override validation, typed lowering and plan identity. Each new kernel has one WGSL implementation in wgpu. The public registry remains the source of browser metadata; bindings and CLI do not reproduce node semantics. No new crate or external resource format is needed.

## brick-pattern@1

No inputs; one `value: Scalar` output. All parameters can be publicly exposed. An explicit unsigned `seed` is required even when variation is zero or the node is on an unused branch.

| Parameter | Type / inclusive range | Default |
|---|---|---|
| `seed` | Integer 0..4294967295 | Required |
| `columns` | Integer 1..64 | 8 |
| `rows` | Integer 1..64 | 8 |
| `rowOffset` | Float 0..1, fraction of a cell width | 0.5 |
| `mortarX` | Float 0..0.45, horizontal gap as fraction of cell width | 0.08 |
| `mortarY` | Float 0..0.45, vertical gap as fraction of cell height | 0.08 |
| `bevel` | Float 0..0.25, inward profile width in cell coordinates | 0.08 |
| `variation` | Float 0..1, independent per-cell amplitude variation | 0.15 |

A nonzero `rowOffset` requires an even `rows`, including after overrides. Reject violations with `MIX_PARAMETER_INVALID_VALUE`, node ID and `rows` parameter context; do not round rows or silently disable the offset. This preserves alternating row phase across vertical tile boundaries. A single unshifted row is valid.

Coordinates have top-left origin and positive v down. Each texel averages four evaluations at offsets `(0.25,0.25)`, `(0.75,0.25)`, `(0.25,0.75)`, `(0.75,0.75)` within its pixel footprint, divided by output dimensions. This fixed 2×2 box sampling is part of v1 semantics, not a universal antialiasing guarantee.

For each evaluation, form `y = v * rows`, `row = floor(y)` and `x = u * columns - (row % 2) * rowOffset`. Let `local = (fract(x), fract(y))`, and wrap `floor(x)` modulo columns for the cell identity. Compute inward distance

`d = min((1-mortarX)/2 - abs(local.x-0.5), (1-mortarY)/2 - abs(local.y-0.5))`.

The profile is zero when `d <= 0`; otherwise it is one when bevel is zero, or `t*t*(3-2*t)` with `t = clamp(d/bevel, 0, 1)`. Bevel is relative to cell axes, so rectangular cells can have different physical bevel widths on each axis. This limitation is explicit; there is no hidden physical-unit conversion.

Cell randomness uses wrapping u32 arithmetic. Define `H(z)` by `z ^= z >> 16; z *= 0x7feb352d; z ^= z >> 15; z *= 0x846ca68b; z ^= z >> 16`. Set `h = H(seed ^ H(column) ^ H(row + 0x9e3779b9))` and `r = (h >> 20) / 4096`. The 12-bit random fraction is exactly representable in f32. Each sample returns `profile * (1 - variation*r)`; the four-sample mean is stored as `[value,0,0,1]` in rgba16float. f32 profile evaluation and f16 storage remain subject to the frozen cross-runtime gate; random identity alone does not prove normal equality.

Changing seed affects amplitudes only, never cell layout or gap width. Identical layout/seed parameters align multiple pattern nodes. Set variation to zero for a profile/mask; use independent variation values for height and coloring. The generator returns normalized relief, not physical height. Existing levels and height-to-normal nodes set the final scale and normal strength. `mortarX=mortarY=bevel=variation=0` is not guaranteed to return one on exact cell boundaries: the explicit `d <= 0` rule applies.

## scalar-mask-blend@1

Required inputs `a: Scalar`, `b: Scalar`, `mask: Scalar`; output `value: Scalar`. One Float parameter `opacity` in [0,1], default 1. Clamp finite input samples to [0,1]; set `t=opacity*clamp(mask,0,1)`. Return the exact clamped `a` at t=0 and `b` at t=1, otherwise `clamp(a+(b-a)*t,0,1)`. Store `[value,0,0,1]` in rgba16float. No resampling, alpha/color conversion or new random state. All three inputs remain required and validated at endpoints. Periodicity follows its inputs; this node does not repair seams.

This lets the brick profile place surface roughness or relief onto a mortar background. Color composition continues to use the existing masked `blend`; normal is derived from the final composed height. New general arithmetic, AO, normal-blend or morphology nodes are not prerequisites for MAT-01; MAT-02 decides their actual need.

## Compatibility and resource bounds

`.mix v1`, plan/hash v2, API schema 2 and `.mixpack v1` remain unchanged. New typed kernel variants are additive serialized plan variants but source-breaking for exhaustive Rust matches, so the first runtime implementation advances the unpublished candidate to Rust 0.6.0 / browser 0.6.0-alpha.0. Existing serialized variants and old plan hashes must remain unchanged. Old runtimes reject the new type identities explicitly; existing documents/assets are never migrated. The published browser 0.3.0 archive remains immutable. Independent consumers and pinned disposable host metadata assertions must be updated only for the intentional version/catalog delta, preserving all behavioral checks.

The planned brick uniform is 48 bytes: four u32 words `[columns,rows,seed,0]`, then f32 `[rowOffset,mortarX,mortarY,bevel]`, then `[variation,0,0,0]`. Mask blend uses 16 bytes `[opacity,0,0,0]`. Both reuse 8×8 dispatch and the existing output-sized rgba16float intermediate. No extra textures, readbacks or uploads are required by either node. The four-channel material must use at most 12 compute passes and stay below 448 MiB estimated/recorded descriptor peak at 2048² under the current retain-all schedule. CPU and package buffers retain their separate policies. Existing global limits are not raised to pass this fixture.

## Frozen qualification matrix

[qualification-plan.json](../fixtures/materials/brick-paving/qualification-plan.json) is the machine-readable case/budget input for the forthcoming verifier. It contains planned thresholds, not test results. Use independent changes for columns, rows, each mortar width, row offset, bevel, height/color variation and seed; presets are not a substitute for causality tests. A verifier must check expected direction/locality, not merely a changed hash.

- Render four channels at 256², 1024², 2048² and 257×129. The fixed regular, staggered, varied and second-seed cases must all pass. Also test 1×1, 1×17 and 17×1 node outputs, full u32 seeds, legal parameter endpoints, zero variation, and invalid odd staggered rows before GPU acquisition.
- On the pinned SwiftShader adapter, repeated identical inputs are byte-exact. Native/browser comparisons require maximum absolute RGBA8 component error ≤1 for each channel, including normal. Explicitly test recorded GT 1030 Vulkan/DX12 versus browser where available; do not claim other hardware. Investigate failures without silently widening this contract.
- Structural probes use analytically chosen interior/gap sample locations with generous separation from filtered boundaries: zero gap height, positive block interiors, the requested row phase, constant cell interior value before bevel, stable layout across seeds and reproducible random variation. Wider gaps reduce occupied area; larger bevel reduces edge relief without changing cell identity. Zero variation removes seed influence. Mask endpoints reproduce direct inputs exactly.
- Seam probes compare a shifted periodically wrapped render to the corresponding wrap of the unshifted result using a test-only sampling origin in the production kernel probe, plus tiled contact sheets. This is not a second renderer and must not become an unversioned public parameter. Opposite border bytes are distinct samples and need not match.
- Compare 256² outputs with box-downsampled 1024² height/baseColor for the fixed default material: per-channel mean absolute error ≤4/255. Stress 64×64 cells at 256²; retain the measured aliasing result and label that setting outside the default quality guarantee if it fails this same metric. Normal is derived per resolution and is not compared through color downsampling. No automatic gate relaxation.
- Fixed PBR evidence uses a plane and a sphere, the same neutral lighting/camera/material scale for all presets, 1× and 3×3 tiled views and channel close-ups. Preserve images and actual human-review decisions under the evidence policy; automated checks do not manufacture human acceptance.
- Measure one cold render and five warm renders in the same explicit renderer (median warm wall time, including readback; record GPU timing when available). At 1024², GT 1030 hardware budget is 5 s cold / 500 ms warm; pinned software is 30 s / 10 s. At 2048², warm budgets are 2 s hardware / 40 s software. Record exact host/backend; these are stage targets, not published SLA claims. A failure starts PERF-MAT measurement and a separately reviewed optimization, not a larger budget.

MAT-01d requires focused CPU and shader/node checks, new material gates, original and migrated existing material matrices, Native/browser public candidate consumption, package roundtrip preserving the source and resulting plan/pixels, `cargo xtask check`, all six required CI checks and retained review content. Use fresh evidence directories. Commands for new nodes/materials only become runnable once their implementations and harness dispatch land; none are claimed to work by this contract.

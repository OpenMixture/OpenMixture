# PR-008 local evidence

English | [简体中文](./README.zh-CN.md)

The current 1K fixture has twelve requested PNGs: default, fine-tiles, and matte, each with baseColor, normal, roughness, and height. All machine gates pass on the tested adapters. Following feedback on the flat checker previews, controlled PBR frames now show the actual channels under light. [human-review.json](./human-review.json) preserves that feedback and records the user's explicit **acceptance** of the revised evidence. The input manifest is unchanged. No remote CI or full M3 closure is claimed.

| Evidence | Result |
| --- | --- |
| [Repository checks](./verification.json), [full check log](./repository-check.log) | Focused golden/guard and readback tests, all material checks, and `cargo xtask check` pass locally. |
| [Software comparison](./software.json), [doctor](./software-doctor.json) | Pinned SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`, Vulkan, Darwin/arm64. All twelve outputs match the initial golden exactly. |
| [Hardware comparison](./metal.json), [doctor](./metal-doctor.json) | Apple M5, Metal. Maximum RGBA8 error 0, mean error 0, changed-pixel ratio 0 in every channel/case. Limits remain max 1, mean 0.25, and no pixel above 1; exact equality is an observation on this run, not a cross-platform promise. |
| [Initial render](./bootstrap-render.json) | Machine gates pass; golden comparison fails explicitly because there was no previous material baseline. |
| [Explicit initial acceptance](./bootstrap-acceptance.json), [bound candidate](./bootstrap-candidate.json) | Separate `golden update glazed-ceramic --accept`, no rendering/staging/commit. Old baseline is empty. This establishes a new material reference, not a reset of a failed existing golden. |

Every case has eight passes and 75,497,648 logical peak bytes. Fine-tiles changes 50% of baseColor pixels while preserving the other channels. Matte increases every roughness pixel from 43 to 112 while preserving color/normal/height. Alternating occupancy is exactly balanced; two-cell periodic error and extra wrap-boundary error are zero. Float finite/range checks are enforced before encoding by the existing renderer. Timings in the reports are CPU wall time, not GPU timestamps.

## Review images

![Controlled PBR appearance comparison](./pbr/comparison.png)

Left to right: default glossy candidate, matte control, finer pattern. The sphere and sample use actual exported baseColor, normal, roughness, and height. All cases share lighting, camera, exposure, and BRDF. Default and fine-tiles retain narrow light reflections and a clearer reflected sample pattern; matte spreads and blurs those reflections. The sphere and rounded backing are consumer geometry, not generated surface detail. Fine sampling grain is Cycles noise, not texture microstructure. The user accepted this scoped ceramic appearance. The sheet's pending caption reflects its pre-review capture time; the signed JSON record is authoritative.

Full frames: [default](./pbr/default.png), [matte](./pbr/matte.png), [fine tiles](./pbr/fine-tiles.png). The [PBR report](./pbr/review.json) records Blender 4.5.13 / Cycles / Apple M5 Metal, 512 samples, all input hashes, and the scene script hash. The [comparison record](./pbr/comparison.json) binds the displayed sheet; [render log](./pbr/render.log) and [input guard tests](./pbr/input-tests.log) retain execution evidence. Reproduce with the [optional offline review scripts](../review/README.md). No material PNG or existing golden changed to produce these frames.

The original flat channel evidence remains useful for inspecting encoding and tiling:

![Case and channel overview](./overview.png)

The columns are baseColor, normal, roughness, height. The rows are default, fine-tiles, matte. Scalar and normal bytes are diagnostic swatches in this sRGB contact sheet; the [original PNG metadata](../expected/) is authoritative.

![Repeated baseColor tiles](./tiling.png)

Initial before/after/difference sheets explicitly mark the missing old baseline:

- [Default](./initial-contact-default.png)
- [Fine tiles](./initial-contact-fine-tiles.png)
- [Matte](./initial-contact-matte.png)

Comparison sheets use columns before, after, difference amplified four times:

- Software: [default](./software-contact-default.png), [fine tiles](./software-contact-fine-tiles.png), [matte](./software-contact-matte.png).
- Metal: [default](./metal-contact-default.png), [fine tiles](./metal-contact-fine-tiles.png), [matte](./metal-contact-matte.png).

Black difference panels mean no changed components, not an absent render. The initial sheet's hatched panels mean missing comparison data. Agent inspection is recorded separately from a human decision. To accept human review, the reviewer must identify themselves and record the decision against the current manifest and PBR report digests; update that record after any accepted baseline or scene change.

Reproduce using [the fixture commands](../README.md) and [golden workflow](../../../../docs/material-goldens.md). Absolute paths and timestamps inside these captured reports are historical execution evidence; new runs create their own local paths. Out of scope: human approval by automation, new nodes/shaders, photorealistic surface-detail claims, 2K/optimization, browser work, or remote CI acceptance.

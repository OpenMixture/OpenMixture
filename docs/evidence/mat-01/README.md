# MAT-01 candidate qualification — human decision pending

English | [简体中文](./README.zh-CN.md)

This record retains the 2026-09-23 development-host results for clean source `7a92b82e8b60fd37bacc8304eaf72d1cfa407216`, not a declaration that MAT-01 or the roadmap is complete. [PR #52](https://github.com/OpenMixture/OpenMixture/pull/52) owns the qualification tooling. Human PBR acceptance, final required CI and integration are still pending. Nothing is published.

## Exact candidate and measured scope

- [Archive receipt](./archive-receipt.json): unpublished `0.6.0-alpha.0`, SHA-256 `7db498db0397d7252906a56ebab25a8d924326050ec631e7996c20dec705388d`, build `sha256:b987c0ba4794989e818787897855a545809ec38901c900edf83d0960c88e6f52`. [Independent Chrome qualification](./browser-qualification.json) passed all seventeen tests on a clean checkout of the recorded source. This does not qualify an older or later archive.
- Five cases × four sizes × four channels: [Vulkan](./native-vulkan.json) versus [Chrome](./browser-matrix.json) has maximum RGBA8 component difference **0** across all eighty comparisons; [DX12](./native-dx12.json) versus the same Chrome candidate has maximum **1**. The frozen ≤1 gate is unchanged. Both Native paths use the recorded NVIDIA GeForce GT 1030; this is not a claim about other hardware.
- Repeat rendering is exact; package roundtrips preserve exact source bytes, plan hashes and all channel pixels. All cases use ten passes and peak at **369,099,024 descriptor bytes**, below the 448 MiB gate. Returned outputs retain zero live per-call descriptors. These are descriptor accounting measurements, not process RSS.
- Release 1K/2K warm wall-time medians, including readback, are **129.42/578.17 ms Vulkan** and **138.66/625.83 ms DX12**; cold and warm gates pass. These measurements are not a published SLA. Earlier debug measurements are not used as release performance evidence.
- Default 256² versus box-downsampled 1024² mean errors are **0.2934/255 height** and at most **0.1849/255 baseColor**, below 4/255. Independent layout, mortar, bevel, height/color variation and seed checks pass; normal is derived from the final height.
- [Vulkan periodic probes](./seams-vulkan.json) and [DX12 periodic probes](./seams-dx12.json) each pass 48 shifted-origin comparisons through the production WGSL, including odd rectangles and one-pixel axes. Raw f16 components are finite and bounded. The origin injection exists only in the test shader copy, not the public node contract.
- `cargo xtask check` passed on the recorded source, including isolated Rust/CLI package consumption. Current PR CI and final integration must be checked separately; historical checks do not certify a new head.

## Measured quality limit

At **64×64 cells rendered at 256²**, stress height mean downsample error is **14.1268/255** and the baseColor red error is **7.9928/255**. Both exceed the default 4/255 criterion. This setting remains outside the default quality guarantee as specified by the frozen contract; no gate, golden or input is silently changed. Stress pixels are retained alongside the Native matrix.

## Visual review

Fixed WebGPU consumer views use a plane and sphere, 1×/3× tiling, the same camera, neutral lighting and dielectric GGX settings. No geometry displacement is performed. They display already-generated maps; they do not add a material executor.

[Default](./review/default-pbr.png) · [regular](./review/regular-pbr.png) · [staggered](./review/staggered-pbr.png) · [varied](./review/varied-pbr.png) · [second seed](./review/second-seed-pbr.png) · [overview](./review/overview.png).

Agent inspection found readable brick/mortar structure, aligned channels, visible layout/edge/seed variation and no apparent new tiling discontinuity in these views. This is **agent review only**. The maintainer's decision has been requested and has not been received. The regenerated sheets are byte-identical to the five sheets presented for that decision. [Review binding](./review-binding.json) binds the tested source and retained file hashes; [preview receipt](./review/preview.json) records the browser, renderer-source hash and exact input PNG hashes. A future human decision must be recorded separately, not invented from test success or silence.

## Reproduction and retention

Follow the [fixture guide](../../../fixtures/materials/brick-paving/README.md) and [MAT-01 contract](../../mat-01-structured-materials.md). Build a clean archive, consume it with `consumer.mjs candidate`, and run `check-brick.mjs` with explicit Vulkan or DX12 and the expected adapter. `cargo xtask test-node brick-pattern` runs the periodic probes; `cargo xtask test-material brick-paving` runs the Native matrix. Generate review sheets with `brick-material-preview.mjs`. Use new output directories for every run.

Git retains the exact input/asset, all eighty browser channel PNGs, both Native matrices including stress PNGs, all six review sheets, key machine receipts and their hashes. This supports full pixel reinspection of these selected Windows results after routine artifacts expire. Compiler products, npm archive bytes, repeated logs and unselected attempts remain in ignored output or CI artifacts; no permanent external archive or registry publication is claimed. The original three-material goldens, migrated material matrix and historical v1 failures are unchanged.

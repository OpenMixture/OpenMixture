# Qualified gamma-one correction and allocation control

English | [简体中文](./README.zh-CN.md)

Retention review: 2026-09-26. Tested source is `76e8039500ceaa849f7c3792c1712a1310ce824b`; CI tested merge `430bea6ee5c3ece895f3b89a4542a2c24032d4a1`. The [receipt](./receipt.json) binds exact archives, reports and selected images. This qualifies the bounded [gamma-one correction](../../levels-linear-correction.md) and PERF-MAT's four requests, not MAT-02's full seven-preset matrix or aesthetic acceptance. Publication and integration remain separate.

## Results

- [Linux Native/browser](./ci-reuse-native.json): all 20 painted-metal channel comparisons are exact, including 1K/2K normal. The earlier 4/255 and 8/255 failures remain in the [original failure record](../perf-mat-numerics/README.md); no gate was relaxed.
- [GT 1030 Vulkan](./vulkan-native.json) and [DX12](./dx12-native.json) versus the clean [Chrome candidate](./windows-browser.json): 20 comparisons per backend pass. Normal, height, metallic and roughness are exact; baseColor has maximum difference 1/255. Public repeated/sliced output, package roundtrip and destruction checks pass. Scalar/resource/brick comparisons also pass on both backends; their six summaries are retained here.
- Frozen cold/warm budgets pass. Software 1K cold/warm median: 1924.52/1155.30 ms; 2K: 5958.83/5208.72 ms. Vulkan: 231.54/191.08 ms and 1013.20/840.35 ms. DX12: 857.65/201.95 ms and 1760.06/818.54 ms. These are measured wall times on the recorded adapters, not universal performance promises.
- The [original](./ci-original-materials.json) and [explicit noise-v2](./ci-noise-v2-materials.json) material matrices pass. All six checks passed on the tested PR source: [CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528637), [GPU/packages](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528839), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528708), [Chromium](https://github.com/OpenMixture/OpenMixture/actions/runs/35884528870). A later documentation commit is not presented as the tested source.

The local default Chromium launch failed with `spawn UNKNOWN` before browser tests could run. Explicitly selecting installed Chrome succeeded; that launch failure is not reported as a runtime pass. Windows bundled SwiftShader cannot substitute for the pinned Linux golden gate, which passed in CI.

## Separate numerical changes from allocation changes

Compared with the original GT 1030 pre-reuse record, the corrected candidate changes one baseColor pixel by one code value; the other four channels remain exact. Do not attribute that precision correction to storage reuse or overwrite the original baseline.

The independent control builds original pre-reuse source `e7b25c4e36f6d7a2052fdba11d8cdfffad762a9d` with **only** the [same levels shader patch](./retain-all.patch). Its dirty state, modified-file list and shader hash are explicit in the receipt; it is not mislabelled as a clean historical build. The [release build log](./retain-all-build.log), [1K Vulkan report](./retain-all-render.json) and five `control-*.png` files remain here. All five decoded channels match the pooled candidate exactly on GT 1030 Vulkan. No limit or allocator is changed in the control. This isolates reuse under the corrected shader while preserving the original before/after evidence from the earlier implementation.

Reproduce the control in a fresh detached checkout of that base commit, apply the retained patch, build `cargo build --release --locked -p mixture-cli`, and render the unchanged [material](../perf-mat-before/material.mix) at 1024 with `--output baseColor,normal,roughness,metallic,height --backend vulkan`. Compare its decoded PNGs with the candidate's public Native default-1k outputs. Use [PERF-MAT commands](../../perf-mat-texture-reuse.md) for candidate/archive consumption and comparison; keep evidence directories fresh.

## Review and retention

The [before/after/difference contact sheet](./review.html) uses retained Native normal images; [metrics](./review-metrics.json) distinguish build-to-build changes from the cross-runtime gate. At 1K/2K, 98/358 Native normal pixels changed (max 4/8) while corrected Native/browser differences are zero. Agent inspection of the 1K images found the same broad surface structure; this is numerical review, not a human material/PBR decision.

Both exact candidate archives and their build receipts are retained. Complete ordinary output remains in local ignored directories and CI artifact `10763790398` (213,836,291 bytes, expires 2026-10-23T16:12:39Z). Selected reports/images/control pixels are bound here; omitted repeated outputs are not claimed permanently available. Historical v1 hardware failures, arbitrary-gamma/warp/cellular limits and MAT-02's remaining causality, seams, metallic PBR and human review requirements remain unchanged.

Retention exception: the two exact candidate archives preserve the qualified build bytes; `retain-all-build.log` binds the deliberately patched old allocator control to its build. These three files support the numerical/allocator separation recorded here, rather than an ordinary repeated run. Their existing receipt hashes remain unchanged.

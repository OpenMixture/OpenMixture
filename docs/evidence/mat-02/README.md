# MAT-02 qualification — accepted within recorded scope

English | [简体中文](./README.zh-CN.md)

This record closes MAT-02 layered weathering (painted metal, [recipe revision 2](../../mat-02-relative-height.md)) for clean `main` source `6b82a8a525937d8e6e4ba72972f696dee3b88d94`, the merge of [PR #77](https://github.com/OpenMixture/OpenMixture/pull/77). It combines the post-merge CI matrix, a local GT 1030 Native/Chrome matrix on the same source and the maintainer's [human decision](./human-decision.json) of 2026-10-02. The unpublished Rust 0.8.0 / browser 0.8.0-alpha.0 candidate remains unpublished. MAT-03/04 are not started.

## Exact source and gates

- All six required checks passed on `6b82a8a` after merge: three-platform CPU, pinned SwiftShader GPU/materials, WASM/npm and [Chromium materials run 36878078015](https://github.com/OpenMixture/OpenMixture/actions/runs/36878078015). The frozen [qualification plan](../../../fixtures/materials/painted-metal/qualification-plan.json) is unchanged; its SHA-256 `fddad1ca…42a4` matches every receipt.
- **Software:** [CI comparison](./ci-6b82a8a/comparison.json) on SwiftShader Device (LLVM 10.0.0) versus CI Chromium covers 28 frozen rows and 5 stress rows: all 165 channel comparisons have maximum difference **0**. [Native receipt](./ci-6b82a8a/native-swiftshader.json); [browser candidate](./ci-6b82a8a/browser-qualification.json).
- **Hardware:** a clean local candidate (build `sha256:16a40a92…5678`, archive `f5faeb14…cce8`) passed all [21 Chrome consumer tests](./gt1030/browser-qualification.json) on NVIDIA GeForce GT 1030. [Vulkan](./gt1030/native-vulkan.json) and [DX12](./gt1030/native-dx12.json) versus the same [Chrome matrix](./gt1030/browser-matrix.json) each pass 165 comparisons: 160 exact, five baseColor channels (default 1K/2K, macro-seed 1K/2K, detail-seed 1K) differ by **1/255**. The ≤1 gate is unchanged. This does not qualify other hardware.
- Both hosts check exact repeats, loose/package equivalence, sliced height, owned outputs after destruction, 60 endpoint channels against independent constant graphs, eleven control-isolation cases and physical allocation accounting (14 textures; 2K peak **503,316,944** descriptor bytes, below 512 MiB; zero live bytes).
- Default 256² versus box-downsampled 1024² mean error is **0.0990/255** height and at most **0.1860/255** baseColor on every backend, below 4/255.
- Release timing (five warm samples, including readback), cold 1K / warm median 1K / 2K: GT 1030 Vulkan **307 / 197 / 881 ms**, DX12 **834 / 207 / 867 ms**; SwiftShader **1457 / 892 / 4018 ms**. All are within the frozen hardware (10 s / 1 s / 4 s) and software (60 s / 20 s / 80 s) budgets. They are not a published SLA.
- Raw-half mask relations, scalar composition and normal replay, final-height normal periodic boundaries and selected noise-input periodicity are covered by the ignored `mixture-wgpu` tests that `cargo xtask gpu-smoke` runs and requires, which passed in the post-merge SwiftShader GPU check and locally on GT 1030 Vulkan; see the [fixture guide](../../../fixtures/materials/painted-metal/README.md) for each scope.

## Texture reuse (PERF-MAT)

[Vulkan](./gt1030/reuse-vulkan.json) and [DX12](./gt1030/reuse-dx12.json) public reuse checks pass 20 Native/Chrome comparisons each, with matched timing budgets. The retained pre-reuse pixels predate the [gamma-one correction](../levels-linear-correction/README.md), so the same-adapter control uses its corrected retain-all pixels: the [pixel control](./gt1030/reuse-control.json) decodes all five default 1K Vulkan channels and finds them **exact**. Against the original pre-correction record, one baseColor component differs by 1/255, reproducing the documented correction rather than a reuse effect. The original records stay unchanged.

## Measured stress limit

[Stress](../../../fixtures/materials/painted-metal/README.md#high-frequency-and-subpixel-width-stress) downsample measurements are numerically identical on SwiftShader and GT 1030 Vulkan/DX12. `high-frequency` (exposure scale 64, edge width 1) has baseColor 256²/1024² mean error **3.439 / 5.984 / 7.811** (R/G/B); it is outside the default quality guarantee. Height is 0.551. `subpixel-width` stays within the metric (height 0.107, baseColor ≤1.217). The selected [stress PNGs](./gt1030/stress/) are retained. No gate, golden or input changed.

## Visual review and decision

The controlled metallic GGX consumer rendered seven [review sheets](./review/) from the CI SwiftShader 1K maps retained in [review/inputs](./review/inputs/); [preview.json](./review/preview.json) binds producer build, browser (Chrome 153), adapter vendor, input and screenshot hashes and the metallic-shading check. Each sheet shows plane/sphere, 1×/3× tiling, 4× close-ups and channel thumbnails; height is not displaced.

[Default](./review/default-pbr.png) · [intact](./review/intact-pbr.png) · [exposed](./review/exposed-pbr.png) · [rusted](./review/rusted-pbr.png) · [edge rust](./review/edge-rust-pbr.png) · [macro seed](./review/macro-seed-pbr.png) · [detail seed](./review/detail-seed-pbr.png).

Before the decision, agent inspection noted clear wear/exposure layering and no visible tiling seams, but weak rust visibility at defaults and rounded/blocky close-up wear. The maintainer accepted the presented sheets ("接受验收"). The [human decision](./human-decision.json) binds the exact presented bytes. GT 1030 Native maps differ from the reviewed SwiftShader maps by at most 1/255 in five channels; the decision binds the reviewed maps, not every backend's bytes.

## Reproduction and retention

Generate requests and run the Native matrix per the [fixture guide](../../../fixtures/materials/painted-metal/README.md). For hardware, run `node scripts/browser-runtime/build.mjs`, `node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh>` with installed Chrome, then `check-painted.mjs` and `check-reuse.mjs` with explicit `MIXTURE_GPU_BACKEND=vulkan|dx12`, `MIXTURE_GPU_SOFTWARE=0` and the expected adapter. Generate review sheets with `painted-material-preview.mjs`. Use fresh output directories.

Git retains the reviewed sheets and their 35 input maps, the selected stress PNGs, all machine receipts and the [binding](./binding.json). The full channel PNG sets (165 per runtime), candidate archives, logs and the CI artifact `chromium-material-matrix` (376,181,194 bytes) are not retained; the artifact expires under the repository's CI retention. Hashes in the receipts identify, but do not preserve, those omitted pixels.

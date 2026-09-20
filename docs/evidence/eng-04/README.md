# ENG-04 acceptance — Scalar field composition

English | [简体中文](./README.zh-CN.md)

[PR #29](https://github.com/OpenMixture/OpenMixture/pull/29) implements the [approved Scalar contract](../../eng-04-scalar-blend.md). These local results are bound to their original sources, not this later documentation commit. Native CLI images were produced from clean `a4588f2946861e8f3f2d4b84ff9c7439e43d5276`; candidate browser and direct native/browser comparison used clean `4eb8f215e14821ac700574b649fb6ff3ad919530`. The graph and production shader bytes are bound by [review.json](./review.json); the difference between those revisions adds pixel capture/comparison tests, not runtime semantics. Registry consumption used the earlier clean consumer `a4588f2` and the original published package identity recorded separately.

## Results and reviewed content

- [Candidate](./candidate.json): unpublished `0.2.0-alpha.0`, all nine browser tests passed without skips/flakiness. [Browser Scalar results](./candidate-scalar.json) retain four 1K plan hashes, output hashes, ranges, boundary ratios, changes, adapter reports and browser version.
- [Registry](./registry.json): published `0.1.0-alpha.0`, all nine tests passed; [the new graph is rejected](./registry-scalar.json) with `MIX_NODE_UNKNOWN_TYPE`. This is explicit old-runtime rejection, not pixel acceptance for the new feature.
- [Direct comparison](./comparison.json): four weights × height/normal, 1024×1024 RGBA8 each. All eight maximum component deltas are 1, the existing bound chosen before comparison. Each image differs in only 3–15 of 4,194,304 components. Exact cross-backend equality is not claimed. Endpoints match direct source fields exactly within each executor; all requests leave zero live descriptor bytes and owned output survives destruction.
- Height range is 218, 218, 222 and 254 levels at weights 0, 0.25, 0.5 and 1. Native boundary-step/interior-step ratios are 0, 0.001493, 0.001230 and 0.000839, below the fixture limit 2. Consecutive weights pass the fixed >10% changed-byte gate.
- [CLI reports](./cli.json), all eight native and eight browser PNGs, exact dependency locks, and [file digests/sizes](./files.json) are retained. [The tiled contact sheet](./contact.png) shows every native height/normal variant as 2×2 repeats. [Agent review](./review.json) accepts the bounded crossfade: increasingly fine detail, attenuated low-frequency structure, responding normals and no visible tile seam. The browser midpoint normal was also inspected at full resolution. This is agent visual inspection, not an independent human review or a new general material-quality guarantee.

The host is recorded in the review receipt. Native used the reported NVIDIA GeForce GT 1030 / DX12 adapter; automated Chrome `153.0.8010.48` used explicit `--enable-unsafe-webgpu --ignore-gpu-blocklist`. Browser adapter hardware identity is redacted; do not infer native adapter identity for browser execution. These results do not establish ordinary-profile support. The PR separately runs pinned Linux SwiftShader, all existing three-material goldens/v2 gates, CPU platforms and package checks. No golden was replaced.

## Reproduce and retain

At the relevant clean source revision, use the pinned repository toolchain and fresh output paths:

```powershell
cargo xtask test-node scalar-blend
$env:WASM_BINDGEN = '<wasm-bindgen 0.2.128 executable>'
node scripts/browser-runtime/build.mjs
$env:MIXTURE_BROWSER_CHANNEL = 'chrome'
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
$env:MIXTURE_GPU_BACKEND = 'dx12'
$env:MIXTURE_GPU_SOFTWARE = '0'
node scripts/browser-runtime/check-scalar.mjs tmp/sdk-candidate tmp/sdk-scalar-comparison
```

Render each weight with `mixture render fixtures/nodes/scalar-blend/two-noise.mix --size 1024 --output height,normal --set detailWeight=0.5 --backend dx12 --out tmp/eng-04/visual/0.5 --json`, substituting 0/0.25/0.5/1. [contact.py](./contact.py) assembles those existing PNGs with Pillow. Rendering and evidence generation do not update goldens or publish packages.

Preliminary failures remain failures: an initial test addressed normal output by array position instead of selecting height; the test was corrected without changing gates. The first isolated Cargo run exposed a fixture path outside the consumer; the fixture is now bundled and source equality is checked. One browser attempt passed pixel assertions but failed to serialize BigInt evidence; the accepted rerun fixes reporting. None justified a tolerance or shader-semantic change.

The content needed for this bounded decision remains in Git. Complete logs, Playwright reports and candidate archives remain temporary local/30-day CI outputs; hashes cannot restore expired originals. Reproduction creates a new run. Later protected CI results belong to their own revisions and are linked in the PR; they do not rewrite these receipts.

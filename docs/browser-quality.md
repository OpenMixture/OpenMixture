# Browser texture agreement, profile v2

English | [简体中文](./browser-quality.zh-CN.md)

The [decision](./decisions/0006-browser-quality-gates.md) applies to `browser-material-check` and `browser-material-measure`. It measures already-rendered textures; it adds no pixel executor. The [JSON profile](./browser-quality-v2.json) is versioned with the comparator, and current qualification receipts must identify its exact bytes. Historical [M5-05 calibration](./evidence/m5-05/calibration.md) and `browser-tolerances.json` remain unchanged for legacy diagnostics. Native goldens and Studio's separate saved-file comparator are unchanged.

## Budgets and intended use

| Measurement | Frozen engineering limit | Reason / scope |
|---|---|---|
| Maximum RGB encoded difference | 1 byte level | Preserve the old amplitude cap; density of balanced quantization residuals alone is no longer a failure |
| Maximum absolute signed RGB mean in local windows | 0.25 byte level | A 16×16 neighborhood may accumulate at most 64 signed code units per component; detects coherent single-level shifts without averaging across the whole image |
| Windows | 16×16, stride 8, periodic wrap | Overlap avoids only testing a non-overlapping grid; wrapped windows also inspect borders; not a guarantee for every smaller spatial pattern |
| Alpha and scalar encoding | Alpha 255 in both; height/roughness R=G=B | Encoding is not a floating-point tolerance |
| Normal direction | Maximum 1 degree | Compare normalized decoded tangent-space vectors; near-unit normals differing by one RGB code per component have a sub-degree perturbation; a conservative engineering allowance, not a visual just-noticeable threshold |
| Normal length difference | 0.02 | Detect additional encoded-vector distortion; existing absolute normal/height relationship rules remain active |
| Normal directional responses | Diffuse max 0.02, highlight max 0.08 | Five fixed upper-hemisphere directions; diffuse `max(dot(n,l),0)`, highlight its 64th power; sampled sensitivity only, not a full material renderer |
| Height first-difference error | Max 2/255 per texel, including wrap edges | Two endpoints can each differ by 1/255; reports propagation into slope. No physical displacement scale is implied |
| Roughness response | Max 0.05 of peak-normalized GGX cross-section | Sensitive to changes in highlight shape, especially low roughness; alpha=r², floor r=0.04, eight fixed cosine samples, as defined in the implementation |

These budgets are selected from output quantization and explicit engineering response allowances, not fitted to current browser residuals. They are not perceptual certification. The normal/roughness response samples do not reproduce a consumer's whole BRDF; applications using unusually sharp lighting or physical displacement need additional consumer-specific tests. Color comparison remains in encoded sRGB units, preserving the strict single-code cap; it is not a CIE perceptual metric. Sub-window patterns and unsampled lighting can evade the response samples. Existing structural, seam and causality checks therefore remain mandatory.

## Controls and freeze

Run `cargo xtask browser-quality-calibrate tmp/quality-controls` with a fresh output directory. For each channel, identity, broad balanced ±1 code and one sparse +1 code must pass. Uniform bias, a 16×16 shifted patch, a wrapped boundary patch, an eight-pixel seam stripe, a two-code isolated spike, erased eight-code detail and invalid alpha must fail. This generates 40 source/perturbed/difference sheets and `calibration.json`, whose `ok` asserts agreement with predetermined labels, not that all perturbed images pass.

Unit tests add independent analytic cases and encoding/normal checks. Review controls and freeze source/profile identities before browser re-evaluation. Do not tune a failed threshold after observing a browser report. A defect in the comparator must be fixed, versioned/recorded and the controls rerun before starting a new evaluation.

## Reports and stopping

Report schema 2 includes `profile`, `profileSha256` and `gates`: `semantics`, `materialStructure`, `numericalAgreement`, `legacyPixelRegression`. Each channel's `comparison` holds v2 metrics; `legacyComparison` holds the original amplitude/mean/changed-pixel metrics and original verdict. The full matrix must satisfy the first three gates for acceptance. Legacy failure remains visible but does not override them. `measurement only` retains measurements without accepting numerical agreement and is rejected by package qualification.

Runtime acquisition/execution failures remain structured failures, separate from comparison. A completed matrix and passing response budget do not imply a new browser execution if the inputs are historical. Retrospective comparisons must use copies in fresh directories and bind their original PNG/receipt/source identities. New browser support claims require an installed candidate, actual ordinary-profile execution, all existing lifecycle and failure tests, and appropriately scoped visual review.

Stop chasing last-bit alignment once the recorded scope meets every required gate and differences have no unexplained structural consequence. Investigate failed amplitude, bias, response or semantic checks. Keep the source/reference fixed; do not select another Native implementation to obtain a pass.

The GGX distribution and alpha=r² convention follow [Filament’s material model](https://google.github.io/filament/main/filament.html); peak normalization, sampling points, floor and error budget here are our diagnostic choices, not Filament acceptance criteria.

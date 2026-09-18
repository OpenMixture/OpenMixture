# Independent warp local-texel correction — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**Implemented in draft [PR #15](https://github.com/OpenMixture/OpenMixture/pull/15), but not mergeable under the unchanged golden gate.** Candidate `91015e02255f20105586145db23bbd23b3c93546` branches directly from `2cc36863eb5cb4a0f419722b172fb5aceaec239f`; it does not include PR #14's cellular change. It corrects loss of small warp displacement, not every cross-backend arithmetic difference. No golden, tolerance, dependency or noise change is included.

## Change and regression proof

The executor passes one render size to all resource allocations. Warp therefore replaces the equivalent absolute-UV conversion with local texel displacement: `delta=(2*field-1)*strength*size`, integer neighbor base `pixel+floor(delta)`, and weight `fract(delta)`. The pixel coordinate never enters the floating-point weight calculation. With strength bounded to [-1,1] and field clamped to [0,1], delta lies in [-size,size], so adding size before neighbor modulo handles negative coordinates. The neutral/zero-strength fast paths, both axes, four texture loads, interpolation and explicit half rounding remain.

Existing warp and wood checks passed before editing. A new literal test specifies an alternating 1024×1 input and a positive `2^-26` UV shift: every low texel must retain exactly `2^-16`, while high texels round to 1 in binary16. The [old shader fails at pixel 256](./red-node-fixed.log), returning zero; the [new shader passes](./node.log). Three additional literal cases cover odd width, odd height, negative displacement and negative full-period wrapping. Expectations are analytical literals, not a CPU resampler or baseline update. [Literal results](./literal-probes.json) retain all 17 warp probes.

Local shader validation, warp fixtures, all four hardware wood cases and `cargo xtask check` pass. The independent pinned consumer's type/unit/production build also passes. CI passes Linux/macOS/Windows checks, WASM packaging, configured Chromium candidate qualification, and the pinned software node/GPU/package checks. The required software material gate fails as described below; the subsequent 2K trace is skipped.

## Exact candidate and compatibility

The [candidate record](./candidate.json) binds clean source, fixed Studio `56c510ab57daa1b68ef660525a648a582730a37e`, archive SHA-256 `cbb944c7b886044070f958c2f9b3e25e51c4cf493f0715dae0a884df4def769e` and build ID `sha256:83ce87cf204ef7f78a86f06c90ffd38eeb0c034e6a249f08318abe49f2c2d8cf`. The [native bundle](./native-manifest.json) uses the same candidate, registry dependencies and Release/DXC/DX12. Three ordinary fresh browsers tested this same package and all 11 cases/44 channels, including lifecycle/stress and synthetic failure diagnostics.

| Comparison | Result |
| --- | --- |
| Ordinary Chrome 153.0.8010.48 | 27/44 within frozen gates, 19 exact |
| Ordinary Edge 153.0.4234.32 | Same channel metrics as Chrome |
| Ordinary Firefox 156.0 | 44/44 exact |
| Pinned SwiftShader old goldens | Ceramic/leather pass; wood fails 12/16 channels |
| Software wood structural/relationship checks | Pass |
| Native original/candidate compatibility | Ceramic/leather exact; 12 wood channels change, max byte difference 1 |

The full browser pass count remains the same as the original candidate: ceramic 12/12, leather 4/16, wood 11/16. It is not comparable to PR #14's 33/44 as if this were a regression: that candidate additionally changed cellular noise. Chrome/Edge production Player checks pass download, encoding, disposal and absent test harness; Firefox production download is not covered by the current verifier.

Software default wood changes 242 baseColor, 253 height, 380 normal and 62 roughness pixels against its exact old golden; straight-grain remains exact. [Full metrics](./summary.json), [software report](./software/wood.json), [software contact](./software/contact-default.png) and [native before/after sheet](./contact.png) preserve the failure. Native default counts are separately 244/256/384/62. Agent visual review found no obvious structural disruption at sheet scale; it does not constitute human acceptance or override pixel comparisons.

## Full-grid diagnostic limit

The earlier [37-point proposal](../residual-path-reduction/README.md) was deliberately not a general guarantee. Replaying the new production warp shader over 1024² pixels with identical native input textures leaves **26 differing half texels**, down from 54. The three upstream textures are byte-identical before/after; [diagnostic inputs](./diagnostic/pipeline.json), output hashes and all 26 witnesses are retained. This diagnostic uses WebGPU directly and is separate from the installed-runtime matrix. It proves neither a new precision bound nor a portable bit-identity guarantee. Additional affine/interpolation variation remains; this work does not silently rewrite it.

## Delivery decision and reproduction

Keep PR #15 draft, preserve its required failing GPU check, and do not merge or update goldens. The concrete small-displacement bug is corrected and reviewable, but old-pixel compatibility is unresolved. A numerical change with different accepted pixels needs a separate explicit compatibility decision; this task does not authorize that acceptance. No compiler fork, tolerance relaxation, publication or noise change is proposed.

Reproduce on candidate `91015e02255f20105586145db23bbd23b3c93546`: run `cargo xtask shader-check`, explicit-adapter `cargo xtask test-node warp`, `cargo xtask test-material wood`, and `cargo xtask check`. To demonstrate the red test, use this test file with only the warp shader restored from the parent in a separate checkout. For pinned software use the existing `.github/scripts/setup-swiftshader.sh` and GPU workflow; never update a golden. Browser preparation and ordinary runs follow the [previous candidate procedure](../local-coordinate-candidate/README.md), substituting this candidate, archive and fresh directories. The retained diagnostic browser runner was used in ignored `tmp/warp-local-evaluation/diagnostic`, with the previous standalone texture runner; reproduce full native stage files there before identical-input replay.

[CI state](./ci-state.json) binds PR merge `01074654cf98e1b432bf1108c7ae00440fcf1962`; local work used candidate head. GPU run [35207157131](https://github.com/OpenMixture/OpenMixture/actions/runs/35207157131), attempt 1, failed wood; [artifact metadata](./gpu-artifacts.json) identifies `10490403918`, expiry 2026-10-17T09:55:26Z. Source/tests are in PR #15; this record is in PR #13 so the tested candidate identity remains fixed. Reports, selected full-resolution images and sheets remain in Git. Complete archives, full PNG sets, 8 MiB stage readbacks and routine logs remain ignored local/expiring CI output. Hashes do not preserve omitted bytes. No accepted baseline or general platform support is claimed.

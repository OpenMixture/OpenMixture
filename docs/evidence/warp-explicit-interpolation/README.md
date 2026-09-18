# Explicit warp interpolation candidate — 2026-09-18

English | [简体中文](./README.zh-CN.md)

**Not accepted for merge.** [PR16](https://github.com/OpenMixture/OpenMixture/pull/16) isolates explicit interpolation on top of [PR15](https://github.com/OpenMixture/OpenMixture/pull/15). The half-boundary regression is repaired on the tested Chrome/Edge configuration, and all 23 literal cases pass on native DX12 and pinned SwiftShader. Frozen software wood goldens still fail. No baseline or tolerance was changed.

## Source and tests

Tested source is `1c65e8631c2e2bdba8fe3aaf8b7c5762c1c215f3`, parent `91015e02255f20105586145db23bbd23b3c93546`. A later evidence commit is not the tested package source. The sole production warp shader replaces three nested interpolations with `fma(b-a,t,a)`; coordinates, displacement, wrapping and half conversion are unchanged. WGSL permits unfused fma, so this is not a universal bit-identity guarantee.

Six new literal fixtures cover unequal dual-axis weights on a 3×2 texture, mixed signs across seams, large opposite shifts, full periods, a thin odd-height texture and the reduced half-boundary witness. Expectations are literal exact values, not a CPU renderer or regenerated golden. Native passes before and after; Chrome fails only the half-boundary witness before and passes all 23 after. Edge passes all 23 after. The [browser runner](./browser-literals.mjs) only converts texture storage and executes the production WGSL.

| Verification | Result |
|---|---|
| Shader validation, native warp node and 23 literals | Pass |
| Pinned SwiftShader GPU/node/package consumers, 23 literals | Pass |
| Local `cargo xtask check` | Pass |
| Native hardware golden checks, all three materials | Pass under existing hardware gates |
| Native release PNGs versus PR15 | All 44 channels pixel-identical |
| Installed candidate, ordinary Chrome | 27/44 channels pass frozen browser gates; 19 exact |
| Frozen SwiftShader ceramic/leather | Pass |
| Frozen SwiftShader wood | 12/16 channels fail; maximum component difference 1 |
| CI 2K trace | Skipped after golden failure |

See [summary](./summary.json), [CI snapshot](./ci-checks.json), [software literals](./software-literals.json), [software golden report](./software-report.json), and [Chrome comparison](./chrome-comparison.json). Local GT 1030 uses DX12, driver 32.0.15.8266 and Chrome DXC; the native bundle uses release builds. Browser versions are Chrome 153.0.8010.48 and Edge 153.0.4234.32. Edge's launcher lost its transient parent; the surviving owned-profile process was recorded. Edge coverage here is direct literal diagnostics, not ordinary-profile installed-runtime qualification. No new Firefox run is claimed.

## Pixel consequences and decision

Relative to PR15's pinned software outputs, seven wood channels change by 1–4 pixels each, at most one byte: coarse-grain baseColor/height/normal change 2/1/1 pixels; default baseColor/height/normal/roughness change 2/3/4/1. Straight and horizontal grain remain unchanged. Against the original frozen golden, default differences are 244/254/380/61 pixels respectively for baseColor/height/normal/roughness. Structural, finite/range and material relationship checks pass, but the exact software gate remains binding.

[All changed software pixel witnesses](./software-witnesses.json) and [before/after/difference crops](./software-pr15-pr16-crops.png) retain these additional changes. Crop columns are PR15, PR16, and absolute difference ×255, each 32×32 crop enlarged 4×. The [frozen-golden contact sheet](./software-contact-default.png) separately compares the old accepted baseline to PR16. Full default height/normal PNGs are retained under `before/` and `after/`. Visual inspection found sparse quantization changes; this does not waive numerical gates or assert human acceptance.

The candidate improves the tested interpolation witness but neither establishes pixel-preserving compatibility on software nor closes full material qualification. Keep PR16 draft. A follow-up must determine the intended compatibility treatment for the genuine local-coordinate correction and separately address remaining noise arithmetic; selecting a favorable reference or relaxing tolerances is not a fix. No compiler strategy, dependency fork or upstream issue was added.

## Reproduce and retain

At the tested source, run `cargo xtask shader-check`, `cargo xtask test-node warp`, `cargo xtask golden check`, and `cargo xtask check`. Native policy was `MIXTURE_GPU_BACKEND=dx12`, `MIXTURE_GPU_SOFTWARE=0`, `MIXTURE_GPU_EXPECT_ADAPTER=NVIDIA`, with the recorded Chrome directory on PATH. The [GPU run](https://github.com/OpenMixture/OpenMixture/actions/runs/35316278778) builds SwiftShader revision `694585a05946e1ed49b6bd577ca6537cbb57f025` via the unchanged repository workflow, with Vulkan/software/SwiftShader policy. The download artifact identity and expiry are retained in [gpu-artifacts.json](./gpu-artifacts.json).

For browser literals, concatenate production `precision.wgsl` and `nodes/warp.wgsl` into a temporary file. Set `MIXTURE_PROBE_PRODUCT` to the pinned Studio directory with Playwright installed and run `node browser-literals.mjs <CDP-endpoint> <native-literals.json> <combined.wgsl> <fresh-output.json>`. Use parent shader bytes for the before result and identical literal inputs. The ordinary Chrome material run uses the repository `candidate.mjs`, `prepare-materials.mjs` and `default-browser.mjs` workflows; [package identity](./package-candidate.json), [native manifest](./native-manifest.json) and browser receipt bind archive, source, inputs and frozen criteria.

Git retains focused pass/failure content and selected images. Full ordinary logs, duplicated material PNGs and build products remain in ignored `tmp/warp-fma-*`; the full CI artifact expires on its recorded date. No durable external archive, publication, visual acceptance or successful full-material qualification is claimed.

Run `python docs/evidence/warp-explicit-interpolation/verify.py` from the repository root to audit retained literal and gate outcomes without a GPU. The source-head CPU checks passed on Linux, macOS and Windows; WASM packaging and the configured Linux Chromium matrix also passed.

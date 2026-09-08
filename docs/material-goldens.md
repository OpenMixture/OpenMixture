# Material goldens and review

English | [简体中文](./material-goldens.zh-CN.md)

PR-008 implements protected material comparisons and the [glazed ceramic fixture](../fixtures/materials/glazed-ceramic/README.md). It uses the existing CLI to validate, compile, and render all cases through `wgpu`; tooling measures the returned PNGs. There are no new nodes, shaders, runtime APIs, or pixel executors. Remote CI acceptance remains deferred by the current task; neither PR-008 nor a local pass closes M0/M1/M2 or M3.

## Commands

```bash
# Explicit GPU operations; default adapter policy is the local hardware policy.
cargo xtask test-material glazed-ceramic
cargo xtask golden check

# A separate acceptance operation, consuming the last complete SOFTWARE candidate.
# It never renders and never stages or commits files.
cargo xtask golden update glazed-ceramic --accept
```

`test-material` checks the named material; `golden check` checks every material directory containing `acceptance.json`, in lexical order, and reports all failed materials. A check returns success only when every machine gate, plan identity, and golden comparison passes. Missing goldens are a failure with review artifacts, not an implicit acceptance. Ordinary `check`/`test` run the fixture, measurement, and guard tests without acquiring a GPU or changing baselines.

The adapter variables from [GPU smoke](./gpu-context.md) also apply here: `MIXTURE_GPU_BACKEND=auto|metal|vulkan|dx12`, `MIXTURE_GPU_SOFTWARE=0|1`, and optional `MIXTURE_GPU_EXPECT_ADAPTER`. Software material runs require explicit `vulkan`, `Cpu`, and `SwiftShader`; hardware runs reject CPU adapters. Every run retains requested policy, selected adapter, doctor, and per-case render reports, including the original diagnostic on failure. There is no fallback.

Use [the existing setup script](../.github/scripts/setup-swiftshader.sh) to build the pinned driver. On macOS:

```bash
bash .github/scripts/setup-swiftshader.sh
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
  MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask golden check
```

On Linux, use the script's `VK_DRIVER_FILES`/`VK_ICD_FILENAMES` settings documented in the GPU guide and the same three `MIXTURE_GPU_*` values. `MIXTURE_SWIFTSHADER_SOURCE` can point to an existing checkout instead of `tmp/swiftshader/source`. The harness verifies its actual Git HEAD against the fixture pin and refuses tracked source changes. It records the source path, OS/architecture, and loader environment separately from the selected adapter. This is source/adapter evidence, not a hermetic binary-build attestation. A clean source checkout alone cannot attest that an arbitrary loader configuration uses a binary built from it.

## Two-step baseline workflow

1. Run the software check. It writes a fresh, retained `tmp/golden/<id>/software-<run>/` directory with all case/channel PNGs, doctor/render reports, `report.json`, `candidate.json`, and review sheets. Hardware comparisons use separate `hardware-<run>/` directories. The `latest-software.json`/`latest-hardware.json` files identify the corresponding runs.
2. Inspect the full-size PNGs, all `contact-<case>.png` sheets, `overview.png`, and `tiling.png`. Explain a semantic change before accepting it. A new material displays **BEFORE MISSING** and hatched absent images; missing data is never shown as a zero-error comparison. Difference RGB is absolute component error amplified by four; alpha error is included in all three difference components. Reports use unamplified values.
3. Run `golden update <id> --accept` only to accept the inspected software candidate. It refuses any present `CI` or `GITHUB_ACTIONS` variable (including `CI=false`), incomplete/failed candidates, stale sources or baselines, altered artifacts, unsafe paths, and replay of an already accepted candidate. It rechecks PNG metadata, structure, and causality before replacement. It does not invoke the renderer or Git staging/commit.
4. The update prepares a complete new directory, retains the previous `expected/` under the review run, then installs the new `expected/`. If installation fails after moving the old directory, it attempts to restore the old directory and reports any error. `update.json` records old/new hashes, the reviewed report digest, and the explicit flag. Review directories remain available; nothing automatically commits acceptance evidence.
5. Rerun the software check and a hardware comparison; retain useful reports in the fixture's `reports/` directory. Record actual human review separately. `--accept` accepts pixel files; it cannot assert that a human has reviewed them.

Candidates bind runtime source, shaders, tooling source, manifests/lockfile, toolchain, the source material, acceptance contract, variants, and artifact bytes with SHA-256. Source order in these manifests is deterministic. Source edits during rendering invalidate the candidate. This prevents accidental stale acceptance; local review files are not a signature or a boundary against someone deliberately rewriting the manifest and its hashes. Existing baseline corruption fails before rendering and must be investigated rather than overwritten to make a test pass.

## Fixture and schema

The [JSON Schema](../fixtures/materials/acceptance.schema.json) describes repository acceptance v1. The authoritative typed decoder and cross-field checks are in [model.rs](../xtask/src/golden/model.rs), independent of the `.mix` source schema.

```text
fixtures/materials/<id>/
  material.mix                  versioned source graph
  acceptance.json               machine gate contract
  README.md / README.zh-CN.md    intent, controls, limitations, reproduction
  variants/<case>.json           public parameter overrides
  expected/manifest.json         pinned driver, case plan hashes, PNG hashes
  expected/<case>/<channel>.png  software-generated reference outputs
  reports/                      retained comparisons, sheets, review records
```

V1 requires a 1024×1024 request for `baseColor`, `normal`, `roughness`, and `height`; one `default` case followed by at least two variants (at most sixteen cases). IDs are lowercase ASCII letters, digits, and hyphens, starting with a letter. Each non-default case names a matching `variants/<id>.json` with a nonempty `overrides` object. Overrides are passed to public CLI `--set` options; the compiler remains their semantic owner.

Every channel declares its encoding and connected/default provenance. Each case provides all four structural checks. Default has no causality rules; each variant specifies all four, including unchanged channels, and must declare at least one meaningful change. Unknown fields, invalid thresholds, missing channels, duplicate case IDs, odd/undersampled/non-dividing checker cells, and wrong driver pins fail before rendering. This 1K schema is scoped to PR-008/009; representative 2K evidence belongs to PR-010.

## What is measured

| Gate | Definition |
| --- | --- |
| PNG contract | Exact dimensions, RGBA8, no animation; sRGB color versus linear scalar/normal metadata; connected/default provenance in successful CLI output. |
| Finite/range | Existing `mixture-wgpu` readback checks **every f16 component before encoding**, rejecting NaN, infinity, and values outside `[0,1]`. A completed render therefore proves zero rejected components; PNG integers are not used to infer hidden float values. Tests for that boundary remain in `readback.rs`. |
| Channel statistics | Per-RGBA minimum, maximum, and mean in encoded 0–255 units. These are not linear-light color errors. |
| Golden comparison | Per-component and overall max/mean absolute error; fraction of pixels with any changed component; fraction above `pixelThreshold`. All configured limits must pass, inclusively. Software uses exact decoded RGBA; hardware uses fixture tolerances. |
| Uniform structure | Every component is within the declared sentinel tolerance. Neutral normal and zero height are intentional constants, not failed detail generation. |
| Alternating structure | Two distinct RGBA colors, balanced occupancy, minimum RGB contrast, expected transition counts including wrap on every row/column, and zero error under two-cell periodic shifts. |
| Seam | Per-axis wrap jump versus an interior cell-boundary jump. A checker intentionally changes color at the wrap; zero **excess** jump and correct periodic structure establish its seam contract. This does not claim a general continuous-material seam metric. |
| Causality | Exact unchanged channels; a minimum changed-pixel fraction; or minimum increase in mean red normalized to `[0,1]` for a scalar response. Histogram changes alone cannot prove frequency changes. |

Contact sheets are labeled, nearest-sampled previews. Linear scalar and normal bytes are displayed as diagnostic swatches in the sRGB sheet; the original linear PNGs and their metadata remain authoritative. The 2×2 repeat preview is assembled from rendered pixels, not another semantic renderer or a shaded 3D preview. It cannot demonstrate lighting quality, generated surface detail, or photorealism.

The ceramic fixture also provides an optional [controlled Blender appearance review](../fixtures/materials/glazed-ceramic/review/README.md). It consumes the verified exported PNGs in a fixed external scene and records input, script, and output hashes. The [comparison](../fixtures/materials/glazed-ceramic/reports/pbr/comparison.png) shows glossy/matte response and pattern density under identical light. This fixture-owned review is outside Mixture's runtime; it neither executes `.mix` nor replaces a texture golden or human decision.

## Verification and acceptance status

```bash
cargo test --locked -p xtask golden
cargo test --locked -p mixture-wgpu readback
cargo xtask test-material glazed-ceramic
cargo xtask golden check
cargo xtask check
```

The [material record](../fixtures/materials/glazed-ceramic/README.md) distinguishes machine comparisons, agent visual inspection, and human approval. Human acceptance is never fabricated by the harness. PR-009 noise/normals, resource lifetime optimization, 2K work, Web viewing, new nodes, and remote CI closure are outside PR-008.

## PR-009 spatial and normal gates

The [leather fixture](../fixtures/materials/leather/README.md) uses the three new nodes and these additive machine checks. Existing ceramic schema/pixels remain unchanged. Acceptance schema version stays 1; `relationships` is optional, with cross-field constraints still enforced by strict Rust validation.

| Gate | Definition |
| --- | --- |
| `spatial` | Red span, standard deviation, wrapped adjacent Pearson correlation on both axes, and repeat-edge mean jump / interior adjacent mean jump; denominator is floored at one byte for quantization. Every alpha is opaque. |
| `normal` | Maximum decoded XYZ unit-length error, mean `1-nz` tilt, positive Z, opaque alpha and the same seam ratio. |
| `normalizedGradientEnergy` | Mean squared adjacent red differences across both axes including wrap, divided by red variance; then variant/default ratio. The allowed interval excludes one and a minimum changed-pixel ratio also applies. Constant defaults cannot pass. Normalization prevents contrast alone from masquerading as frequency change. |
| `heightNormalDirection` | Compare signs of wrapped central height differences with normal X/Y, using only height steps of at least four bytes; neutral normal components do not count as agreement. Require both coverage and above-chance agreement. Image v is down and tangent Y up. This does not reconstruct expected normal pixels. |

Leather detail min/default/max and coarser grainScale must satisfy image goldens, spatial, normal and causality constraints together. Controlled PBR views consume actual PNGs, applying the height slope through the exported normal once with no second bump. See [leather appearance review](../fixtures/materials/leather/review/README.md). Ceramic has user acceptance; leather human review is accepted. Remote CI and PR-010 directional wood/2K work remain open.

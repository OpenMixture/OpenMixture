# Shader numerical stability evaluation — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**Local cellular coordinates substantially improve conditioning, but are not a drop-in pixel-compatible fix.** Keep the production shader and frozen acceptance references unchanged. This evaluates the next step from the [arithmetic reduction](../chromium-arithmetic-reduction/README.md); it does not certify a new browser package or adopt a compiler policy.

## Controlled changes

Three diagnostic variants were defined before their measurements: original arithmetic; local coordinates only; and local coordinates plus a rationalized square-root difference. The [one-line isolated patch](./diagnostic-local.patch) replaces

```wgsl
vec2<f32>(neighbor) + vec2<f32>(0.2) + 0.6 * jitter - p
```

with

```wgsl
vec2<f32>(vec2<i32>(x, y)) + vec2<f32>(0.2) + 0.6 * jitter - fract(p)
```

Integer lattice identity, wrapping, seed, nine-site search, octave weights and half-rounding are unchanged. Small jitter offsets are no longer added to and subtracted from absolute lattice coordinates as large as 4095. This is a real-arithmetic equivalence, not a floating-point identity. The third variant also replaces `sqrt(second)-sqrt(nearest)` with `(second-nearest)/(sqrt(second)+sqrt(nearest))`; it was not promoted to full-material testing because it did not consistently outperform the smaller change.

## Measured conditioning and portability

The 1024² default leather noise buffer probe uses seed 271828, scale 64, three octaves and persistence 0.35. Compare the original-registry native Release/DXC execution with each ordinary browser:

| Probe formula | Raw f32 values that differ | Half-rounded values that differ | Maximum raw difference |
|---|---:|---:|---:|
| Original | 1,032,165 | 6,298 | 9.0971589e-6 |
| Local coordinates | 730,400 | 223 | 2.9802322e-7 |

Chrome and Edge give the same counts. The half-rounded mismatch count decreases by 96.46%; the maximum raw difference decreases about 30.5 times. **Maximum half-rounded difference remains 0.00048828125**, so fewer mismatches do not imply a tighter worst-case half-step bound. Neither variant is bit-identical across implementations. See [browser-grid.json](./browser-grid.json). The separately patched strict native diagnostic gives the same grid comparisons on this host. The rationalized variant reduces its half mismatch count slightly further to 202, but does not lower the maximum raw difference and sometimes increases error in the scalar samples.

The [sample plan](./sample-plan.json) covers six configurations, 256 reproducible points each, including image corners, zero/max seeds, scale 1/128, octave 1/6, persistence 0/0.01/0.35/1, 129×65, 1024² and 2048² sizes. Original, local and rationalized probes ran on regular native, strict native, ordinary Chrome and ordinary Edge.

An initial [coordinate-generating sweep](./measurements-samples.json) mixes UV-division variation with cellular arithmetic. At non-power-of-two dimensions this obscures the latter. The follow-up supplies **identical stored f32 UV inputs** to all implementations, then compares sparse scalar outputs to a binary64 estimate using the same f32 constants and exact power-of-two coordinate scaling. This is a higher-precision diagnostic reference, not an exact oracle, CPU material renderer, new golden or altered acceptance target. [Fixed-input measurements](./measurements-fixed.json) include all results and worst witnesses, including regressions:

| Fixed-input configuration | Original native max absolute error | Local native max absolute error |
|---|---:|---:|
| Default leather | 3.39554e-6 | 1.06089e-7 |
| Max period, odd dimensions | 4.36555e-5 | 7.42572e-8 |
| Max period, 2K | 4.43897e-5 | 8.57328e-8 |
| Min period | 1.10067e-7 | 1.27004e-7 |

The min-period case becomes slightly worse, and gains one half mismatch versus Chrome where the original sample had none. The low-persistence local case also retains one mismatch. Thus this is strong improvement at large coordinate magnitudes, not uniform dominance. The unfixed odd-size sweep retains coordinate-generation error around 1.2e-4 even after localizing cellular arithmetic; it must not be attributed solely to the distance formula. All tested full-grid raw/rounded values were finite and in [0,1].

## Full material and golden consequences

Only the local-coordinate change was applied in an isolated checkout at `909de3e2ecce1d1f02da07031b820c35a30b5554`, with the original registry dependency graph. Before editing, the original fractal-noise node and leather material checks passed on GT 1030/DX12. After editing, shader validation, fractal-noise fixtures, and all four leather hardware material cases passed. These native material checks include their existing golden tolerances and structural/relationship checks. Logs are retained here; their ordinary detailed runs remain temporary.

Separate original/local **Release** executables rendered all 11 material cases/44 channels, with identical requests and plan hashes. [Comparison](./material-comparison.json):

- Ceramic and wood: all 28 channels pixel-identical.
- Leather: all 16 channels change, maximum difference one RGBA8 unit. Default height changes 363 pixels; default normal changes 885. Across cases, changed counts range from 54 to 1,299.
- Applying the unchanged browser numeric envelope to this native-before/native-after comparison yields 32/44 within the envelope. This is a compatibility diagnostic, **not a browser qualification result**. Twelve leather comparisons exceed that envelope even though native hardware quality/golden gates pass.

The [before/after/difference sheet](./contact.png) uses ×64 absolute RGB differences, with [original](./original-default/height.png) and [local](./local-default/height.png) full-resolution default height retained alongside color, normal and roughness pairs. Agent inspection found no obvious structural change at the displayed scale; sparse quantization differences are measurable and remain binding. No human acceptance or golden update is claimed. Pinned SwiftShader exact-golden execution and the changed package's full browser matrix were not run; neither is implied by passing hardware tests.

## Assessment and decision

Local cellular coordinates are the strongest candidate for a separately reviewed numerical change. Do not integrate them merely to turn the browser gate green: they alter existing output and still leave cross-implementation differences. Before adoption, require unchanged golden evaluation on the pinned software adapter, a reviewed explanation of any pixel change, full candidate/browser regression including Firefox, and an explicit compatibility decision. This evaluation alone does not authorize replacing baselines.

The rationalized square-root expression is not justified by these results; do not add it. Remaining shader risks have different owners:

| Area | Evidence and next boundary |
|---|---|
| UV generation / sampling | Non-power-of-two division is an independent precision source; localizing distance does not fix it. |
| Gradient-map, blend, bilinear interpolation | Multiply/add and `mix` allow arithmetic variation; the earlier sweep demonstrates it. These kernels were not changed or newly certified here. |
| Levels | Nonlinear remapping can propagate/amplify upstream differences; existing endpoint guards remain. No `pow` accuracy repair is established here. |
| Height-to-normal | Neighbor differences and resolution scaling propagate height perturbations; the observed changed-normal count exceeds changed-height count. This is not independent proof of a normalization bug. |
| Half storage | The existing integer helper consistently rounds its own input; different pre-rounding values may straddle a midpoint. |

WGSL permits reassociation/fusion and does not promise cross-backend bit identity ([specification](https://gpuweb.github.io/gpuweb/wgsl/#reassociation-and-fusion)). This evaluation does not establish an upstream compiler defect or justify a maintained dependency fork. Performance was not benchmarked; no speedup claim is made. Product `.mix` semantics, source/shader files, dependency policy, archive identity, references, and tolerances remain unchanged in the main working branch. Chrome/Edge full-material certification remains pending.

## Reproduction and retention

[provenance.json](./provenance.json) binds tested sources, executables, original input hashes and retained files. Native buffer runners and the ordinary-browser runner are the [previously retained programs](../chromium-arithmetic-reduction/README.md); loaded compiler paths are in the module records, and fresh browser launch receipts are retained here. Browser backend identity is not inferred from the host GPU. The regular production executable was rebuilt after the isolated experiment and its original output checked.

Run the following from a fresh repository checkout; generated data uses ignored `tmp/shader-stability`. Preserve any earlier local run before reusing that directory.

1. Run `node docs/evidence/shader-stability/generate-grid.mjs`, `generate-samples.mjs`, and `generate-fixed-input.mjs` (each under that directory). They create the three variants and reproducible sample/input files. No product shader is edited.
2. With the previous native runners, execute `<variant>.wgsl` at count 2097152 for original/local/local_rational, naming outputs `<variant>-regular.bin` and `<variant>-strict.bin`. Run `<variant>-samples.wgsl` at count 3072 without an input file, and `<variant>-fixed.wgsl` at count 3072 with `uv-input.bin`. Use matching `-samples-<mode>.bin` and `-fixed-<mode>.bin` names. Repeat the small probes on ordinary Chrome/Edge; full browser grids were run for original/local only. Set the native child DXC PATH and `MIXTURE_PROBE_PRODUCT` as in the prior procedure.
3. `node docs/evidence/shader-stability/measure.mjs samples` and `node docs/evidence/shader-stability/measure.mjs fixed` recompute the summaries. The retained small buffers permit independent sample recomputation; large grid buffers must be rerun. Binary64 estimates intentionally do not decide release pass/fail.
4. For material evaluation, create an isolated checkout at the recorded source. Run `cargo xtask test-node fractal-noise` and `cargo xtask test-material leather` before applying `diagnostic-local.patch`; afterward run shader-check and both checks again with explicit DX12 policy. Build original/local Release CLI executables into **separate target directories** and place copies at the documented ignored paths. `render-materials.mjs original` / `local` reproduce all cases; run `compare-materials.py` with Python/Pillow to regenerate metrics and the contact sheet. Never run `golden update` for this diagnostic.

Small sampled buffers, inputs, scripts, measurements, patch, selected full-resolution PNGs and the contact sheet are retained in Git. Large grid buffers, full 44-channel PNG sets, executables and detailed golden runs remain ignored temporary outputs; hashes do not retain those bytes. This is an evaluation record, not a new accepted visual baseline.

# Residual leather and wood arithmetic — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**The remaining paths reduce to three concrete operations: cellular site-offset arithmetic, value-noise `mix`, and warp UV multiply/add.** This follows the rejected [local-coordinate candidate](../local-coordinate-candidate/README.md). Production code, dependencies, goldens and tolerance files remain unchanged. This is diagnosis and a bounded repair proposal, not a new accepted candidate.

## Bind the probes to the actual failures

Source is candidate `e41410859c457c13739198c580bc1e1e2af45ed9`. The fixed default leather and wood fixtures at 1024² provide 16 and 37 final height PNG witnesses, respectively. The [pipeline](./pipeline.json) uses the unchanged candidate WGSL plus its precision helper, original uniform bytes, rgba16float textures, and original texture loads and workgroups. The [native runner](./texture_probe.rs) uses registry wgpu/DX12 in Release; the [browser runner](./browser-textures.mjs) attaches to ordinary fresh Chrome/Edge. These are diagnostic GPU dispatches, not another product renderer or a WASM qualification claim.

Every scalar texture is read back before downstream execution. Both final height textures, converted to RGBA8, reproduce the corresponding candidate's entire native/Chrome PNG with zero differences. Edge matches Chrome across all seven full textures. [Summary](./summary.json) and [witness half bits](./half-witnesses.json) retain the measurements; large full texture readbacks remain temporary with hashes recorded.

Next, each downstream Chrome dispatch receives exactly the native input texture bytes, using the **same unmodified shader**. This separates propagation from an operation's own arithmetic:

| Stage | Native/Chrome differing half texels | With identical native inputs |
| --- | ---: | ---: |
| Leather grain | 223 | 223 (source, no texture inputs) |
| Leather height/levels | 127 | 0 |
| Wood grain | 54 | 54 (source) |
| Wood stretch | 32 | 0 |
| Wood distortion | 134 | 134 (source) |
| Wood warp | 176 | 54 |
| Wood height/levels | 176 | 0 |

Thus the tested levels and stretch differences are propagated, while wood warp also introduces independent differences. These counts are half textures, not final RGBA8 pixel counts. All seven tested textures were finite and bounded in [0,1]. This is not a universal claim about levels, transform or all parameter variants.

## Three retained minimal operations

Each [cellular](./min-cellular.wgsl), [mix](./min-mix.wgsl) and [warp](./min-warp.wgsl) probe uses four stored f32 inputs and a single scalar result. Input/output buffers and compiler-module receipts are retained. Chrome and Edge agree in each case; native differs by 1 ULP. [Exact rational calculations](./summary.json) use the actual f32 inputs/constants, not replacement material references.

1. **Leather `(685,60)`:** in octave 0, the nearest site has identical jitter `0.22351199388504028`; its X delta is `0 + 0.2 + 0.6*jitter - 0.84375`. Native yields `-0.5096428394317627`, Chrome/Edge `-0.5096427798271179`. This site contributes to the nearest-distance result. The traced grain rounds to `0.2132568359375` versus `0.213134765625`, then height becomes `0.52392578125` versus `0.5234375`, producing final 134 versus 133. No different hash, texture coordinate or levels input-independent defect is needed to reproduce this witness.
2. **Wood distortion `(934,121)`:** octave 0 has identical lattice values and fade. `mix(0.09506094455718994, 0.5717524290084839, 0.8831931352615356)` yields native `0.516071617603302`, Chrome/Edge `0.5160715579986572`. Subsequent octave accumulation has additional rounding differences; the texture becomes `0.440185546875` versus `0.43994140625`, changing displacement and final height 119 versus 118. The witness establishes `mix` as an upstream difference, not that this one interpolation explains every wood pixel.
3. **Wood warp `(831,109)`:** even identical input textures differ. The UV expression `0.81201171875 + (2*0.2462158203125-1)*f32(0.018)` yields `0.8028754591941833` versus `0.8028755187988281`. After multiplying by 1024 and subtracting 0.5, X is `821.6444702148438` versus `821.64453125`. The unchanged neighbors `0.625` and `0.7314453125` produce `0.6936008334159851` versus `0.6936073303222656`, straddling half midpoint `0.693603515625`. Half output is `0.693359375` versus `0.69384765625`; final height is 170 versus 171.

The [noise trace](./noise-trace.wgsl) matches the actual grain/distortion half value for all 53 selected witnesses on native and Chrome. It is still instrumentation: the all-intermediate warp trace **hides the mismatch**, because exposing intermediate values changes optimization. The separate `warp-only-uv/p/t/mix/half.wgsl` probes retain only one result per execution and recover the actual witness. Do not infer correctness from an instrumented version that becomes equal.

The reviewed [WGSL draft](https://github.com/gpuweb/gpuweb/blob/cd910cf650d05481b60bad2b44476caff974962f/wgsl/index.bs) permits reassociation/fusion and defines arithmetic accuracy, not universal bit identity. These small differences do not establish a compiler violation. Exact rational errors alternate which implementation is closer: Chrome is closer for the cellular witness, native for mix and warp. A strict compiler policy is therefore not itself a numerical oracle. No upstream issue was sent.

## Repair proposal, with an explicit limit

**First evaluate warp in local texel coordinates as a separate change.** For equal-sized input/output textures, express sampling as `pixel + deltaTexels`, where `deltaTexels = (2*field-1)*strength*size`. Use integer `pixel + floor(deltaTexels)` for the wrapped base and `fract(deltaTexels)` for interpolation. This removes the absolute `uv + offset` rounding followed by size amplification. Preserve the zero-field/zero-strength fast path, negative wrapping, both axes, and the current bilinear/half semantics. Prove the equal-size assumption from the executor before implementation.

The [local-texel scalar probe](./warp-local-texel.wgsl) tests the same 37 selected wood coordinates and verifies unchanged neighbor selection against exact arithmetic. Compared with the original scalar probe, maximum absolute interpolation error falls from `4.244320734869689e-6` to `6.489608495030552e-8` (about 65×). Native/Chrome raw mismatches fall from 4 to 1; half mismatches from 4 to 0. See [all sample results](./proposal-samples.json). This is a failure-selected sparse diagnostic, **not** a full shader implementation, general improvement proof, performance measurement or acceptance pass. The proposal will change old pixels and must independently face frozen goldens and the three-browser matrix.

**Treat noise as a separate numerical-contract decision.** Local cellular coordinates removed large-coordinate cancellation but retained sensitive affine arithmetic; value-noise has interpolation and octave-accumulation variation. Do not blindly replace `mix` with `fma`, insert another half quantization, or force strict compilation. An accuracy-oriented affine/interpolation rewrite must be evaluated against exact fixed-input calculations and old-pixel compatibility. If portable bit identity is required, it needs an explicitly specified rounding/arithmetic implementation in the existing WGSL path, not an unproven compiler switch; that larger semantic scope requires separate review. This investigation does not justify a combined noise/warp rewrite.

## Reproduce and retain

From the repository root, prepare candidate checkout `tmp/local-coordinate-engine` and the pinned consumer with dependencies at `tmp/local-coordinate-product`, as in the previous record. Copy the retained scripts into a fresh `tmp/path-reduction` (the browser import and input paths intentionally refer to those ignored diagnostic locations). `generate.py` builds the exact seven-node dispatch plan. Build `texture_probe.rs` as a temporary example in the isolated mixture-wgpu crate using its own target directory, Release, and the prior recorded DXC directory in PATH. Run `texture_probe <pipeline.json> <native-output-directory>`. Run `browser-textures.mjs <CDP-endpoint> <output-directory>`; pass an extra `native` argument for identical-native-input replay. Launch browsers with the repository's ordinary launcher. Restore the isolated checkout after copying the example source.

`generate-traces.py` prepares 53×256 noise entries and 37×32 warp entries. Run the [existing buffer runners](../chromium-arithmetic-reduction/README.md) with counts 13568 and 1184 and the corresponding stored input file. Minimal probes use count 4. `warp-only.py` generates single-result probes; `local-proposal.py` generates the texel-coordinate experiment. `proposal-measure.py` uses exact rational scalar arithmetic, not CPU texture execution. `verify.py` checks retained small evidence without a GPU.

Small probe buffers, sources, plans, summaries, adapter/launch receipts, and hashes remain in Git. Full 8 MiB-per-stage readbacks remain temporary; their hashes do not preserve those bytes. The bound candidate PNG evidence remains in the previous record. No new golden, artifact publication, browser certification, compiler fork, or production change is included.

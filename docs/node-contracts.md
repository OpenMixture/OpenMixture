# Built-in node contracts, version 1

English | [简体中文](./node-contracts.zh-CN.md)

The eleven version-1 contracts in [mixture-core](../crates/mixture-core/src/registry.rs) lower to typed plans and [execute through the sole `wgpu` path](./graph-rendering.md). PR-005–007 established the six M2 nodes; PR-009 added noise, gradient mapping and height-derived normals; PR-010 adds scalar transform and warp. Constants share one WGSL kernel, material-output maps resources, and the fixed checker shares the graph checker shader.

## Common rules

All eleven node types require `version: 1`. Connections match `Scalar`, `Color`, or `Normal` exactly; every input has at most one incoming edge. An input without a default is required. Omitted parameters use the defaults below; unknown names, invalid types, and out-of-range values are errors. All parameters below are mutable and can be exposed through a unique public binding. The randomized `fractal-noise` requires an explicit integer seed in the source, including on unused branches; an override does not repair a missing source seed.

Float parameters accept finite JSON numbers; integer parameters require unsigned integer tokens (`8` is valid, `8.0` and `8e0` are not). Colors are arrays of exactly four finite numbers in `[0, 1]`, representing linear RGBA, with straight alpha. Float/color bounds are inclusive. The source model retains f64 JSON values; compilation explicitly lowers them to f32 GPU parameters. Parameter validation does not execute pixels or convert color spaces.

Coordinates use a top-left origin. Pointwise constant, levels, and blend operations introduce no coordinate transform. Their tiling depends on their inputs; constant outputs are seamless. Detailed GPU precision and golden evidence for graph execution belong to PR-007.

## constant-scalar

[Contract module](../crates/mixture-core/src/nodes/constant_scalar.rs). No inputs; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `value` | Float `[0, 1]` | `0.0` |

Defines the same scalar at every pixel. Its meaning as roughness, height, mask, or another scalar channel is assigned by its consumer.

## constant-color

[Contract module](../crates/mixture-core/src/nodes/constant_color.rs). No inputs; output `color: Color`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `value` | RGBA `[0, 1]` per component | `[1, 1, 1, 1]` |

Defines the same linear RGBA color at every pixel.

## checker

[Contract module](../crates/mixture-core/src/nodes/checker.rs). No inputs; output `color: Color`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `cellsX` | Integer `[1, 1024]` | `8` |
| `cellsY` | Integer `[1, 1024]` | `8` |
| `colorA` | RGBA `[0, 1]` per component | `[0, 0, 0, 1]` |
| `colorB` | RGBA `[0, 1]` per component | `[1, 1, 1, 1]` |

For output pixel `(x, y)`, cell indices are `floor(x * cellsX / width)` and `floor(y * cellsY / height)`. Even summed parity selects `colorA`; odd parity selects `colorB`. Uneven dimensions produce uneven cell widths, and small images can undersample cells. Even cell counts preserve alternating continuity when repeated across that axis; odd counts are permitted but repeat adjacent same-color boundary cells. No filtering or randomness is implied.

Defaults match the [PR-004 fixed checker](./builtin-checker.md). The graph contract permits colors and frequencies that the fixed probe command does not expose. PR-007 extends the single existing WGSL path to execute both fixed and graph checker invocations.

## levels

[Contract module](../crates/mixture-core/src/nodes/levels.rs). Required input `in: Scalar`; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `inputMin` | Float `[0, 1]` | `0.0` |
| `inputMax` | Float `[0, 1]` | `1.0` |
| `gamma` | Float `[0.01, 100]` | `1.0` |
| `outputMin` | Float `[0, 1]` | `0.0` |
| `outputMax` | Float `[0, 1]` | `1.0` |

`inputMin < inputMax` is required after resolving defaults. The declared operation is `t = clamp((in - inputMin) / (inputMax - inputMin), 0, 1)`, followed by `outputMin + pow(t, 1 / gamma) * (outputMax - outputMin)`. Reversed output bounds are allowed for inversion. Positive gamma and distinct input bounds prevent undefined divisions. PR-007 implements this formula in WGSL; Rust validates/lowers parameters and converts readback encoding only.

## blend

[Contract module](../crates/mixture-core/src/nodes/blend.rs). Required inputs `a: Color` and `b: Color`; optional `mask: Scalar`, default `1.0`; output `color: Color`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `mode` | Enum `normal`, `multiply`, `screen` | `normal` |
| `opacity` | Float `[0, 1]` | `1.0` |

Define `t = opacity * clamp(mask, 0, 1)`. RGB is a linear interpolation from `a.rgb` to the mode result by `t`: `b.rgb` for normal, `a.rgb * b.rgb` for multiply, and `1 - (1 - a.rgb) * (1 - b.rgb)` for screen. Alpha interpolates from `a.a` to `b.a` by `t` for every mode. This is a component blend contract, not source-over compositing. No hidden premultiplication or extra blend modes are implied.

## material-output

[Contract module](../crates/mixture-core/src/nodes/material_output.rs). Exactly one sink is required per document. It has no outputs and no parameters. Inputs appear in this stable contract order:

| Input | Kind | Policy / default |
| --- | --- | --- |
| `baseColor` | Color | Required valid connection |
| `normal` | Normal | Encoded neutral XYZ `[0.5, 0.5, 1.0]` |
| `roughness` | Scalar | `1.0` |
| `metallic` | Scalar | `0.0` |
| `height` | Scalar | `0.0` |
| `ambientOcclusion` | Scalar | `1.0` |
| `opacity` | Scalar | `1.0` |
| `emissive` | Color | Linear opaque black `[0, 0, 0, 1]` |

The encoded normal corresponds to tangent-space +Z; its RGBA storage filler (alpha 1) is separate from this logical XYZ value. M2 contains no `Normal` producer, so valid M2 documents use this default. A color output cannot masquerade as a normal. `ValidatedDocument::material_channels()` exposes the connected/default status without generating textures.

## Fixtures and checks

The [two-node checker](../fixtures/format/valid/checker.mix) exercises default parameters and an exposed defaulted frequency. [all-m2.mix](../fixtures/format/valid/all-m2.mix) connects all six contracts, applies levels to a scalar blend mask, and supplies explicit roughness. [Invalid documents](../fixtures/format/README.md) and public API tests cover parameter boundaries, enums, port direction and kinds, required/defaulted inputs, graph errors, and public bindings.

```bash
cargo test --locked -p mixture-core --test registry
cargo test --locked -p mixture-core --test validation
cargo xtask test-format
```

These tests validate contracts without a GPU. The existing fixed checker golden is unchanged. PR-007 [node fixtures](../fixtures/nodes/README.md) and `test-node` now verify actual graph pixels; no additional pixel golden is claimed.

PR-006 [typed lowering](./render-plan.md) maps these source contracts to Constant, Checker, Levels, and Blend invocations; material-output remains a mapping. Constants also materialize optional defaults. This adds no source node type. PR-007 implements the exhaustive GPU mapping and pixel tests.

## PR-009 additions

PR-009 adds three contracts and three WGSL kernels, bringing the catalog to nine nodes and the renderer cache bound to seven pipelines. The existing six contracts, M2 plan snapshots and checker goldens are unchanged. `ParameterContract::default` is now `Option<ParameterDefault>`: `None` means required in the source. This is an intentional pre-publication Rust API change; downstream callers must handle `None`. Existing defaults remain `Some`, and missing seed reports `MIX_PARAMETER_INVALID_VALUE` with node/parameter evidence. The `.mix v1` shape is unchanged; adding versioned node types is additive.

### fractal-noise

[Contract](../crates/mixture-core/src/nodes/fractal_noise.rs), [WGSL](../crates/mixture-wgpu/shaders/nodes/fractal-noise.wgsl), [fixtures](../fixtures/nodes/fractal-noise/README.md). No inputs; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `seed` | Integer `[0, 4294967295]` | Required; no default |
| `scale` | Integer `[1, 128]` | `8` |
| `octaves` | Integer `[1, 6]` | `4` |
| `persistence` | Float `[0, 1]` | `0.5` |
| `basis` | Enum `value`, `cellular` | `value` |

Sample at pixel-center UV `(x+0.5, y+0.5)/(width,height)`, with image v down. Each octave doubles the integer lattice period, multiplies its amplitude by persistence, and derives its own u32 seed. Divide the weighted sum by total amplitude and clamp to `[0,1]`. Persistence zero equals one octave. The full u32 seed is uploaded as an integer, never through f32.

The single WGSL uses a documented wrapping u32 avalanche hash (multipliers `0x7feb352d`, `0x846ca68b`) and the upper 24 bits scaled by `2^-24`. Cell identity wraps modulo the octave period. Value noise interpolates four lattice values using quintic fade. Cellular uses one site in each cell's central 60%, independent X/Y jitter, and the clamped difference between second and first nearest distances in a bounded 3×3 neighborhood. This is the defined local cellular field, not an unbounded nearest-site search. It produces grain interiors separated by dark valleys.

Both bases repeat over one UV tile. Adjacent border pixels need not match; they are distinct pixel-center samples. Cellular gradients have cell-boundary cusps. There is no antialiasing: high scale/octaves can undersample at small resolutions. Arithmetic is f32 and storage is f16; exact software baselines and explicit hardware tolerance cover the final RGBA8 result, not universal cross-driver float identity. The WGSL is the sole pixel formula.

### gradient-map

[Contract](../crates/mixture-core/src/nodes/gradient_map.rs), [WGSL](../crates/mixture-wgpu/shaders/nodes/gradient-map.wgsl), [fixtures](../fixtures/nodes/gradient-map/README.md). Required input `in: Scalar`; output `color: Color`. Parameters `colorA` and `colorB` are straight linear RGBA in `[0,1]`, default opaque black and white. Interpolate all four components with `clamp(in,0,1)`; convert color RGB to sRGB only on output encoding. This is a two-endpoint ramp, with no stops or hidden color-space conversion. Tiling follows the input; f32 interpolation and f16 storage apply.

### height-to-normal

[Contract](../crates/mixture-core/src/nodes/height_to_normal.rs), [WGSL](../crates/mixture-wgpu/shaders/nodes/height-to-normal.wgsl), [fixtures](../fixtures/nodes/height-to-normal/README.md). Required input `in: Scalar`; output `normal: Normal`. `strength` is Float `[0,8]`, default `1`.

Read wrapped left/right/up/down neighbors. `du = (right-left)*width/2`, `dv = (down-up)*height/2`; encode `normalize(-strength*du, +strength*dv, 1)*0.5+0.5` with alpha 1. Image u points right and v down; tangent X points right and Y up (OpenGL convention). Derivatives are per UV unit, so the same resolved surface has comparable strength at different resolutions. This does not correct undersampling. Strength zero or a constant input gives neutral normal; a one- or two-texel axis has identical central neighbors and zero derivative. Wrapped neighbors preserve periodic input semantics, including legitimate boundary slopes. Strong/high-frequency inputs can approach grazing normals; quantized nearly neutral components require care in downstream measurements.

```bash
cargo xtask test-node fractal-noise
cargo xtask test-node gradient-map
cargo xtask test-node height-to-normal
```

The normal tests feed literal half-float horizontal/vertical ramps directly through the production shader at rectangular sizes to establish wrap, Y sign, UV scaling and zero strength. They do not implement a CPU normal renderer. The [leather fixture](../fixtures/materials/leather/README.md) exercises all three additions as a material consumer.

## PR-010 scalar resampling additions

PR-010 adds `transform-2d` and `warp`, bringing the catalog to eleven nodes and the renderer cache bound to nine pipelines. The source JSON shape, document/node version 1, prior node contracts and existing pixel/plan/hash baselines are unchanged. These are additive catalog entries; existing documents need no migration. ENG-02 graduates that historical pre-M3 count gate. The [registry tests](../crates/mixture-core/tests/registry.rs), included in `cargo xtask check`, retain explicit reviewed type/version identities and contract/default/seed checks. Future additions follow [node admission](../ARCHITECTURE.md#72-reviewed-node-catalog-and-admission); this update adds no node or pipeline.

Both nodes operate on `Scalar` input and output. Derive colors and tangent normals after transforming or warping height; neither node implicitly accepts `Color` or `Normal`. They introduce no random operation and require no additional seed. Input periodicity comes from their source graphs, whose randomized nodes retain the explicit-seed requirement.

### Shared sampling convention

For output texel `(x,y)`, use center UV `(x+0.5,y+0.5)/(width,height)`, with u right and v down. At the requested source UV, compute `p=fract(sampleUV)*inputDimensions-0.5`. Read the four integer neighbors around `floor(p)`, wrapping each coordinate modulo the corresponding input dimension before loading, then bilinearly interpolate using `fract(p)`. Negative neighbors wrap to the opposite edge. This explicitly uses four `textureLoad` calls: no sampler, mip chain or antialiasing is implied.

Arithmetic is f32, intermediate scalar storage is `[value,0,0,1]` in f16, and exported RGBA8 replicates red into RGB with opaque alpha. One-texel axes are valid and wrap to the same texel. Repeat sampling preserves a periodic source's boundary convention; it does not repair an existing input seam. Opposite border pixels are distinct center samples, so seamless tiling does not require their byte values to be identical. High integer scales or rapidly varying warp fields can undersample input detail.

### transform-2d

[Contract](../crates/mixture-core/src/nodes/transform_2d.rs), [WGSL](../crates/mixture-wgpu/shaders/nodes/transform-2d.wgsl), [fixtures](../fixtures/nodes/transform-2d/README.md). Required input `in: Scalar`; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `scaleX` | Integer `[1,64]` | `1` |
| `scaleY` | Integer `[1,64]` | `1` |
| `quarterTurns` | Integer `[0,3]` | `0` |
| `offsetX` | Float `[-1,1]` | `0` |
| `offsetY` | Float `[-1,1]` | `0` |

Let `p=uv-0.5`. Inverse-rotate those coordinates first, then scale on the source axes, then translate: `sampleUV=rotated*vec2(scaleX,scaleY)+0.5+vec2(offsetX,offsetY)`.

| `quarterTurns` | Inverse-rotated coordinates | Visible pattern rotation |
| --- | --- | --- |
| `0` | `(p.x,p.y)` | none |
| `1` | `(p.y,-p.x)` | clockwise 90 degrees |
| `2` | `(-p.x,-p.y)` | 180 degrees |
| `3` | `(-p.y,p.x)` | clockwise 270 degrees |

The pivot is the UV tile center. Rotation precedes per-axis scaling so it rotates the anisotropic grain axis as well: `scaleX=8, scaleY=1` produces eight source periods horizontally at zero turns, and vertically at one turn. These are sample-period counts; increasing a scale repeats/compresses the source instead of enlarging it. Rotation uses normalized UV coordinates; rectangular output dimensions do not swap. Positive offsets move source sampling right/down and move the visible pattern left/up.

Integer scales and quarter-turn rotations preserve a periodic source for every permitted setting, including fractional offsets. Fractional scales and arbitrary rotation angles are deliberately outside this v1 contract because they generally break output-tile periodicity. With both scales one, zero turns and zero offsets, the shader directly loads the corresponding input texel, preserving scalar pixels exactly.

### warp

[Contract](../crates/mixture-core/src/nodes/warp.rs), [WGSL](../crates/mixture-wgpu/shaders/nodes/warp.wgsl), [fixtures](../fixtures/nodes/warp/README.md). Required inputs `in: Scalar` and `displacement: Scalar`; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `strengthX` | Float `[-1,1]` | `0.05` |
| `strengthY` | Float `[-1,1]` | `0` |

Read the displacement scalar at the current output texel, set `d=2*clamp(field,0,1)-1`, then sample `in` at `sampleUV=uv+d*vec2(strengthX,strengthY)` with the shared repeat-bilinear rule. The strengths are independent signed UV displacements. Field `0.5` is neutral; field `1` applies positive strength, and field `0` applies negative strength. Positive X/Y displacement samples right/down, moving the visible pattern left/up. The field is read at the output coordinate, not at the displaced input coordinate; its clamp is shader semantics and does not relax source parameter validation.

A neutral field texel or a zero strength vector uses a direct input load and preserves its scalar pixel exactly. The required field connection is still validated and compiled when strength is zero. Periodic source and displacement inputs preserve the tile contract because an output-tile step adds an integer source-UV step. Large displacements can fold the sampled pattern; no monotonicity or antialiasing is promised.

```bash
cargo test --locked -p mixture-core --test resampling --test registry
cargo xtask shader-check
cargo xtask test-node transform-2d
cargo xtask test-node warp
```

The [resampling API tests](../crates/mixture-core/tests/resampling.rs) cover defaults, typed input order, repeated bindings, exact kinds, required connections, dependency slicing and parameter/hash behavior. The [literal GPU probes](../crates/mixture-wgpu/tests/support/resampling_probe.rs) use 17 transform and 13 warp cases with hand-specified half-float inputs and exact expected outputs. They establish interpolation across X/Y seams, negative offsets, all rotations, rotation-before-scale order, single-texel dimensions, displacement polarity/clamping and output-coordinate field reads through the production WGSL. Node graph cases reference the existing noise identity baseline without changing it, and check meaningful parameter changes and warm-cache repeatability. Node evidence is saved under `tmp/node-tests/<backend>/`, including `<node>-literal-probes.json`.

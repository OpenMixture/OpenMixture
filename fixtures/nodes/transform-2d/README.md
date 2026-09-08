# transform-2d fixtures

English | [简体中文](./README.zh-CN.md)

Version 1 accepts required `in: Scalar` and emits `value: Scalar`. Parameters are integer `scaleX`/`scaleY` in `[1,64]` (default `1`), integer `quarterTurns` in `[0,3]` (default `0`), and float `offsetX`/`offsetY` in `[-1,1]` (default `0`). Fractional scales are rejected deliberately: each scale counts source sample periods per output tile.

At pixel-center UV with a top-left origin and v down, set `p=uv-0.5`; inverse-rotate first: `p`, `(p.y,-p.x)`, `-p`, or `(-p.y,p.x)` for turns 0–3. Then sample at `rotated * scale + 0.5 + offset`. `quarterTurns` visibly rotates the pattern clockwise around the tile center, including its anisotropic scale. Rotation is in normalized UV space; rectangular output dimensions do not swap. Positive offsets move the sampling point right/down, moving the visible pattern left/up.

Sampling performs bilinear interpolation of four wrapped texels at `fract(sampleUV)*dimensions-0.5`. Negative neighbors wrap before loading. There is no sampler, mip chain, or antialiasing. Integer scales and quarter rotations preserve a periodic input's tile contract; this does not repair an existing input seam. Border pixels are different pixel-center samples and need not match. High scales can undersample fine input detail. Arithmetic is f32, intermediate storage is f16, and scalar export replicates red into RGB. The default identity uses a direct texture load to preserve scalar pixels exactly.

[cases.json](./cases.json) and [input.mix](./input.mix) cover identity on the existing noise baseline, anisotropy, clockwise rotation, fractional offsets, maximum scales, negative offsets, one-texel axes, scalar range endpoints and invalid parameters/missing input. [constant.mix](./constant.mix) provides exact constant sentinels. The existing noise golden is referenced read-only rather than duplicated or changed.

The [literal GPU probes](../../../crates/mixture-wgpu/tests/support/resampling_probe.rs) upload hand-specified half-float rows and rectangles directly to the production WGSL. Exact expected values establish positive/negative and X/Y half-texel interpolation, edge wrapping, all quarter turns, rotation-before-scale order, full-period offsets and one-texel behavior. Whole-graph checks also establish warm-cache repeatability and meaningful parameter changes. These are fixture expectations, not a CPU transform implementation.

```bash
cargo xtask test-node transform-2d
```

Use the explicit adapter policy in the [GPU guide](../../../docs/gpu-context.md). Evidence is written under `tmp/node-tests/<backend>/`, including `transform-2d-literal-probes.json`. See the [contract](../../../docs/node-contracts.md) and sole [WGSL implementation](../../../crates/mixture-wgpu/shaders/nodes/transform-2d.wgsl). The wood material is the directional consumer; transform operates on scalar data before color or normal derivation.

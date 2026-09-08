# warp fixtures

English | [简体中文](./README.zh-CN.md)

Version 1 requires `in: Scalar` and `displacement: Scalar`, and emits `value: Scalar`. Float parameters `strengthX` and `strengthY` are in `[-1,1]`, defaulting to `0.05` and `0`. They are signed displacements in UV units, independently on each axis.

At each output pixel center, load the displacement field at that output texel. Define `d=2*clamp(field,0,1)-1` and sample the input at `uv+d*strength`, with a top-left origin and v down. Field `0.5` is neutral, `1` applies positive strength, and `0` applies negative strength. Positive X/Y displacement samples right/down and moves the visible pattern left/up. Zero strength or a neutral field uses a direct input load and preserves scalar pixels exactly. The displacement field is not sampled at the displaced source location.

Input sampling performs bilinear interpolation of four wrapped texels at `fract(sampleUV)*dimensions-0.5`, including negative neighbors. No sampler, mip chain or antialiasing is implied. Periodic source and displacement inputs preserve the tile contract; the node does not repair an input seam. Adjacent border pixels remain distinct pixel-center samples. Large strengths or rapidly changing fields can fold and undersample the pattern. Computation uses f32 and intermediate storage f16; scalar export replicates red into RGB. The node is deliberately scalar-only; derive colors/normals after warping height.

[cases.json](./cases.json) and [input.mix](./input.mix) cover the neutral-field identity on the existing noise baseline, positive/negative half-texel shifts, vertical displacement, zero and maximum strengths, one-texel axes, scalar range endpoints, invalid parameters and missing displacement. [constant.mix](./constant.mix) supplies exact constant sentinels. The existing noise golden is referenced read-only rather than duplicated or changed.

The [literal GPU probes](../../../crates/mixture-wgpu/tests/support/resampling_probe.rs) use hand-specified half-float scalar inputs, fields and expected values through the sole production WGSL. They check neutral and zero-strength identities, displacement clamping/polarity, negative strengths, X/Y bilinear seam wrapping, one-texel behavior and a spatially varying field loaded at output coordinates. Whole-graph tests verify meaningful displacement changes and exact warm-cache repeats. No CPU pixel renderer is used.

```bash
cargo xtask test-node warp
```

Use the explicit adapter policy in the [GPU guide](../../../docs/gpu-context.md). Evidence is written under `tmp/node-tests/<backend>/`, including `warp-literal-probes.json`. See the [contract](../../../docs/node-contracts.md) and sole [WGSL implementation](../../../crates/mixture-wgpu/shaders/nodes/warp.wgsl). The wood material combines a transformed grain source with a separately seeded periodic displacement field.

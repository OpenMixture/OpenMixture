# ENG-04 — Scalar field composition

English | [简体中文](./eng-04-scalar-blend.zh-CN.md)

ENG-04 implements `scalar-blend` v1. The [approved design PR](https://github.com/OpenMixture/OpenMixture/pull/27) retains the original proposal; this page describes the implementation. Acceptance is recorded with the implementation PR and its source-bound evidence.

## Use case and contract

Combine independently seeded low-frequency structure (seed 11, scale 4) and high-frequency detail (seed 29, scale 32), then derive height and normal from the combined field. The [complete fixture](../fixtures/nodes/scalar-blend/two-noise.mix) exposes `detailWeight`. Existing Color `blend` and unary Scalar `levels` cannot express this two-Scalar operation.

Required Scalar inputs `a` and `b`; output `value: Scalar`. Finite Float `weight` in [0,1], default 0.5. Clamp each finite input sample to [0,1]. After existing f64→f32 parameter lowering, weight 0 selects a exactly and weight 1 selects b exactly; otherwise compute `clamp(a + (b-a)*weight, 0, 1)`. Store `(value,0,0,1)` with `mixture_half4` in rgba16float. RGBA8 readback is not lossless. No cross-adapter last-bit guarantee is added.

This is a crossfade: increasing detail attenuates the low-frequency contribution. It is not additive displacement. Both inputs remain required and validated at endpoints. Integer texel reads preserve compatible tiling but do not repair input seams. No random state, masks, new sampling, color conversions, CPU pixel executor or new crate is introduced.

## Compatibility and delivery

The source packages advance to Rust 0.2.0: new variants in exhaustive public `KernelId`/`KernelInvocation` can break downstream exhaustive matches. Consumers must handle `ScalarBlend`. No incidental `non_exhaustive` retrofit is made. The independent Rust consumer compiles against the new API.

The browser candidate advances to 0.2.0-alpha.0. API schema 1, `.mix` version 1, plan version/hash domain and existing serialized variants remain unchanged. Old plan hash snapshots remain regression tests. Published npm 0.1.0-alpha.0 stays immutable and its separate registry test requires `MIX_NODE_UNKNOWN_TYPE` for this new graph. Unsupported node versions, input kinds and weights retain structured errors.

Candidate qualification substitutes only the runtime dependency/version/integrity in the independent host. The pinned disposable Studio CI host has two explicit producer-owned compatibility adjustments: catalog size 11→12 and runtime version 0.1.0-alpha.0→0.2.0-alpha.0. The staging script fails if either exact original assertion changes, retains original/adapted test source and digests, and preserves all lifecycle, pixel, material and deployment checks. No Studio repository, product upgrade or deployment is changed. This is a reviewed version-contract update, not permanent compatibility with old catalog counts.

Rust packages and the new browser candidate remain unpublished. Publication and downstream upgrades are separate work.

## Verification

- `cargo xtask test-core` and `test-plan`: defaults, bounds, missing/wrong inputs, versions, unused-branch overrides, deterministic ordering/defaults/hashes and output slicing.
- `cargo xtask shader-check` and `test-node scalar-blend`: literal endpoints/midpoint, equal/reversed inputs, 65×3 and single-pixel axes; injected finite out-of-range texels verify saturation through the production WGSL.
- `cargo xtask test-consumer` and `gpu-smoke`: independent Rust API consumption, 1K two-noise weights 0/0.25/0.5/1, exact direct-field endpoint height/normal equivalence, non-degenerate output, weight causality, periodic boundary metrics and owned data after renderer destruction.
- Independent browser candidate consumption runs the same fixture and structural gates; registry consumption explicitly rejects it. All nine browser tests must execute without skips/flakiness. Existing three-material/v2 comparisons and pinned software-GPU checks remain required.
- `cargo xtask check` and six protected CI checks gate integration. Existing golden pixels are not updated. Visual review uses 1K height/normal variants and tiled contact sheets; [fixture gates](../fixtures/nodes/scalar-blend/README.md) are fixed before acceptance.

Spatial masks, blend-mode/math families, additive relief, HDR domains, graph rewrites, image resources, portable packaging, UI authoring and publication are out of scope.

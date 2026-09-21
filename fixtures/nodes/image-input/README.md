# image-input v1 — Core fixture

English | [简体中文](./README.zh-CN.md)

**M6A-03:** `cargo xtask test-node image-input` now runs the production shader. Frozen byte/orientation/ramp/padding, blend endpoint and lifecycle cases live in [image_probe.rs](../../../crates/mixture-wgpu/tests/support/image_probe.rs); see [Native implementation](../../../docs/m6a-03-native-resources.md). The M6A-02 notes below are historical.

[height.mix](./height.mix) expresses the approved external-height/noise crossfade and height/normal outputs. The required `resourceId` is `heightSource`; use same-size, packed `rgba8-linear` bytes. `detailWeight` preserves scalar-blend crossfade semantics. The [resource contract](../../../docs/m6a-resource-contract.md) defines input orientation, R interpretation, budgets, seams and version policy.

M6A-02 verifies this source through `cargo xtask test-core`, with 2×2 row-major bytes `[0,11,22,0, 64,33,44,1, 128,55,66,2, 255,77,88,3]`. G/B/A are intentionally distinct; the Core test binds these bytes without evaluating pixels. Invalid/edge/override/budget fixtures are explicit cases in the [Core resource tests](../../../crates/mixture-core/tests/resources.rs). Native GPU upload and pixel qualification are pending M6A-03; `test-node image-input` is not an implemented GPU acceptance target yet. No pixel golden is accepted here.

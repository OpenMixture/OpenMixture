# Scalar composition fixtures

English | [简体中文](./README.zh-CN.md)

`input.mix` exercises literal Scalar inputs; `cases.json` covers defaults, endpoints, equal/reversed inputs, range bounds, 65×3 and one-pixel axes. The production-WGSL probe injects finite out-of-range samples to test saturation. Run `cargo xtask test-node scalar-blend`.

`two-noise.mix` combines seeds 11/29 at scales 4/32, exposing `detailWeight`. Height and normal are derived from the same combined field. The independent native and browser consumers test 1K weights 0, 0.25, 0.5, 1, exact endpoint equivalence to direct source fields (including normals), changed pixels and height tile-boundary continuity. Before accepting results, the fixed gates require height range >20 RGBA8 levels, >10% changed bytes between consecutive weights and mean boundary step / mean interior step <2. These are fixture-specific structural gates, not universal material quality thresholds.

Existing material goldens are unchanged. The source requires an ENG-04 runtime; published npm 0.1.0-alpha.0 rejects its unknown type. [Contract and compatibility](../../../docs/eng-04-scalar-blend.md).

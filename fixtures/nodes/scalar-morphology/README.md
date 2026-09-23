# Scalar morphology fixtures

English | [简体中文](./README.zh-CN.md)

The [selected MAT-02 contract](../../../docs/mat-02-layered-weathering.md) defines wrapped single-axis erosion/dilation. [Cases](./cases.json) cover defaults, radius endpoints, both axes/operations and invalid values through the public graph compiler. Core tests additionally cover required Scalar input, unknown parameters, unsupported versions, non-finite JSON, deterministic hashing and dependency slicing.

`cargo xtask test-node scalar-morphology` executes the production WGSL with fixed binary support sets and exact raw half comparisons: constant fields, origin/center impulses, thin lines, centered/wrapped rectangles, radii 0/1/16, both axes/operations, sizes 1x1/1x17/17x1/19x11, periodic translations, axis commutation and radius monotonicity. Fractional/out-of-range literals test exact identity, extrema and saturation. Set union/intersection is only an independent test oracle, not a CPU pixel renderer. Evidence uses `MIXTURE_NODE_EVIDENCE_DIR`. Public Native/browser qualification and material acceptance are separate.

The independent browser consumer adds 192 fixed support-set cases through the installed public SDK. `node scripts/browser-runtime/check-morphology.mjs <candidate-qualification> <fresh-output>` executes the same named cases through public Native preparation/rendering and requires exact plan hashes and RGBA8 bytes. Candidate mode requires 18 browser tests; registry mode keeps its historical 13 and does not claim the new node. The wrapper is required by Chromium CI; a test definition is not a passing qualification result.

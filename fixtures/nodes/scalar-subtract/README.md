# Scalar subtraction fixtures

English | [简体中文](./README.zh-CN.md)

The [MAT-02 contract](../../../docs/mat-02-layered-weathering.md) admits `scalar-subtract@1` to retain W-I edge bands that disappear through a three-node half-precision composition. [Cases](./cases.json) cover defaults, zero/one/equal/ordered inputs, positive differences, degenerate sizes and unknown-parameter rejection. [Amplification](./amplified.mix) maps the exact 1/4096 difference to white using existing levels; this independently verifies that the actual graph stores the tiny edge, instead of hiding it in RGBA8 rounding.

`cargo xtask test-node scalar-subtract` also executes production WGSL on literal negative/above-one values, fractional equal fields, half-representable differences and periodic shifts. Raw rgba16float comparisons are exact before output encoding. Core tests verify required input kinds, unsupported versions, no parameters, ordered/repeated bindings, source immutability and sliced hashes. The package-owned ABI test requires two inputs and 16 zero bytes. These tests add no CPU pixel executor.

The independent browser/Native consumers each define 64 cases: eight input pairs × four sizes × direct/amplified output, with exact repeats and analytical pixel expectations. `node scripts/browser-runtime/check-subtract.mjs <candidate-qualification> <fresh-output>` compares exact plan hashes and bytes using the two public runtimes; Chromium CI requires it. Candidate mode requires 19 tests, while published registry consumption remains 13 and excludes the unavailable node. Runtime 0.7 is unpublished; node and full painted-metal qualification remain separate. No original golden or earlier failure is rewritten.

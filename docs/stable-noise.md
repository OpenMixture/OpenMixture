# Stable value noise — NUM-01

English | [简体中文](./stable-noise.zh-CN.md)

The maintainer authorized explicit noise-semantic migration and material requalification on 2026-09-22, following the [Windows investigation](./evidence/windows-numerics/README.md). This change implements `fractal-noise@2` in the unpublished **0.4.0-alpha.0** candidate. It does not publish the candidate or retrospectively qualify 0.3.0 hardware pixels.

## Contract and compatibility

`.mix` remains version 1; plan/API schema remain 2. The reviewed catalog still has thirteen types and eleven kernels. The latest `fractal-noise` contract is version 2. `node_contract_version` resolves explicit versions; version 1 remains executable for frozen regressions, existing consumers and before/after comparisons. Neither validation nor rendering rewrites a document. Version 3 is rejected. New source must explicitly select version 2 to obtain the changed value-noise semantics.

For `basis=value`, v2 lowers to `NoiseBasis::StableValue`; v1 still lowers to `Value`. The plan carries both node version and the distinct basis, preventing reuse of a legacy plan identity. Old v1 plan hashes remain unchanged. `basis=cellular` retains its previous floating-point evaluation and precision limits in both versions. No universal hardware parity claim for cellular, warp or full arbitrary graphs follows from stable value-noise arithmetic.

The explicit seed, hash, periodic lattice, center-sampled coordinates, quintic interpolation, octave doubling and normalized weighted sum remain the mathematical design. Parameter names, defaults and ranges are unchanged. Only the value basis adopts the following finite arithmetic contract:

1. Represent values in Q0.24 integers on `[0, 2^24]`, including both endpoints. The lattice value is the existing high 24 hash bits.
2. Compute each center coordinate as the integer rational `(2*pixel+1)*period/(2*size)`. Use its exact integer cell and floor its fractional part to Q0.24.
3. Multiply Q0.24 values with an exact 48-bit product assembled from 12-bit limbs, rounding to nearest with ties to even. Interpolate by adding or subtracting the rounded nonnegative difference product.
4. Evaluate the quintic fade through de Casteljau interpolation of controls `[0,0,0,1,1,1]` in the specified shader order. All intermediates remain in `[0,1]`; this avoids signed polynomial cancellation. Independent f64 polynomial probes bound deviation from the ideal fade to 16 Q units.
5. Convert the already-lowered f32 persistence to Q0.24 by multiplying by `2^24` and truncating to u32. Derive subsequent weights and each weighted value with the same rounded multiplication. Accumulate integer sums, then floor the normalized ratio to Q0.24 using 24 steps of integer division.
6. Convert the final integer to f32 exactly and scale by the exact power `2^-24`. Apply the existing integer ties-to-even f16 conversion and texture storage. No hardware floating-point division, polynomial evaluation or multiply-add determines a value-basis rounding decision.

Six octaves bound sums and total weights to `6*2^24`; doubled division remainders remain below `12*2^24`. At the runtime's 8192 device dimension limit, center-coordinate numerators are below `2*8192*4096`. These bounds fit u32. The first weight is one, so the normalization denominator is nonzero. Power-of-two conversion is exact even at the included upper endpoint. No optional u64/f64 GPU feature, second renderer, dependency or hidden adapter fallback is introduced.

## Explicit migration and evidence

To migrate a document, change only the selected `fractal-noise` node versions from 1 to 2, retain the original document, and review new height/normal pixels before adopting it. The repository provides `material-noise-v2.mix` next to each of the three original material fixtures. The original `material.mix` files and goldens remain v1 regression inputs; they are not overwritten to absorb changed semantics. Public Native resource and Scalar fixtures now select v2. The independent browser consumer uses explicit v2 fixture files for the 0.4 candidate and the unchanged v1 files for the exact published 0.3 registry baseline.

The producer-owned material preparation accepts an explicit migration option:

```sh
node scripts/browser-runtime/prepare-materials.mjs tmp/browser-native-v2 <full-tested-commit> --noise-v2
```

This selects committed migrated sources; it does not transform inputs silently. The manifest records `noiseVersion:2` and exact source hashes. `browser-material-check` verifies those hashes against the migrated files, applies the same frozen structure, parameter causality, channel relationship and numerical gates, and rejects unknown migration identities. Browser CI runs both the original 11-case/44-channel matrix and the migrated matrix in the same pinned disposable consumer, without modifying Studio. `candidate.mjs verify-noise-v2` binds a separate receipt to the candidate/archive and existing public-interface/deployment gates.

## Verification and acceptance limits

`cargo xtask test-node fractal-noise` now exercises both source versions, repeatability, seed causality, octave/persistence identities, small rectangular/degenerate dimensions, a new exact v2 129×65 baseline and production arithmetic probes against independent u64/f64 operations. This arithmetic oracle does not render CPU material pixels. The new v2 baseline is separate from the original two v1 baselines. `cargo xtask test-core`, `test-plan`, `shader-check`, `gpu-smoke` and `check` retain their normal meanings.

The manual numerical probe uses basis word 2 for the new value path; six 1K Windows Vulkan/Chrome diagnostic cases match raw f16 height/normal exactly, including the formerly failing scale 7. These observations do not replace clean candidate package qualification, DX12 coverage, migrated material review or CI. Acceptance records must bind each tested source/archive separately. The historical Windows failure remains correct for v1 and the published 0.3 release.

Out of scope: automatic document rewriting, publishing 0.4, cellular/warp numerical redesign, texture-format changes, threshold relaxation, Studio upgrades and deployment.

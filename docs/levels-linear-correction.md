# Gamma-one levels correction

English | [简体中文](./levels-linear-correction.zh-CN.md)

Status: working correction, pending material/cross-runtime qualification. The frozen MAT-02 painted-metal graph fails software Native/browser normal parity at 1K and 2K. [Stage isolation](./evidence/perf-mat-numerics/README.md) first observes differing pixels at `detail`, a `levels@1` remap from [0,1] to [0.5,1] with gamma 1; its input noise stage matches. This bounded prerequisite is reviewed separately from physical texture reuse. It does not complete MAT-02 or start MAT-03/04.

The sole production WGSL now evaluates gamma 1 as `curved = t`, followed by the existing affine output remap. Other effective f32 gamma values retain `pow(t, 1/gamma)`, endpoint clamps and half conversion. `pow(t,1)=t` is the declared mathematical identity, but approximate power evaluation can move a value across a binary16 rounding midpoint. The full failing material must pass before attributing its failure conclusively to this correction.

## Compatibility decision

Retain `levels@1`: this corrects its existing gamma-one identity rather than introducing a different function, range, default or coordinate convention. Core contracts/lowering, kernel ABI, `.mix`/`.mixpack`, plan/API/report versions and plan hashes remain unchanged; wgpu owns the correction. Deliberately do not preserve an approximation error as a second node version. No document migration, automatic rewrite or golden reset is permitted.

Old/new builds can differ at rounding boundaries. Pixel caches must bind implementation/shader builds under the [compatibility record](./compatibility.md); plan hashes do not identify pixels. The unpublished 0.8 candidate requires new archive qualification. Published 0.3 packages and historical failures remain unchanged. This does not promise exact results for arbitrary gamma, affine bounds, cellular, warp or all hardware.

## Verification

Before the shader edit, the original eight levels cases passed on Windows bundled SwiftShader at source `594279a1960e5bdc56ed05040f32b4f82880a194`. Linux [run 35875437605](https://github.com/OpenMixture/OpenMixture/actions/runs/35875437605) again failed material parity; its twelve initial literal samples all produced zero error. Those samples did not reproduce the failure and do not establish agreement at all midpoints.

Eight additional exact [levels cases](../fixtures/nodes/levels/cases.json) cover binary16 inputs within observed differing RGBA8 bins, including both rounding parities. Expected bytes follow the dyadic affine result rounded to binary16 nearest-even and the existing RGBA8 conversion. They use the production graph executor at odd dimensions and sample both dispatch ends. Existing default, inverse, endpoint, tiny-interval and non-unit-gamma cases are unchanged. This numerical oracle is not a CPU rendering API.

Required: `cargo xtask shader-check`, `cargo xtask test-node levels`, existing material regressions and `cargo xtask check`; independent new Native/browser archive consumption; painted-metal 1K/2K comparison; original and explicit-noise-v2 material matrices; all six checks and recorded Vulkan/DX12 paths. Inspect retained before/after material images if pixels change. All frozen thresholds remain unchanged. Until these pass, this is a candidate correction, not an accepted numerical/platform claim.

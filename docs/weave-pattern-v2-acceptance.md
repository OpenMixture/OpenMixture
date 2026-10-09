# weave-pattern@2 — frozen node contract and acceptance

English | [简体中文](./weave-pattern-v2-acceptance.zh-CN.md)

The maintainer's three decisions on PR #90, 2026-10-09, are recorded and frozen before implementation:

1. Approve candidate C: B's X/Y/D occupancy ownership and Vw/Vf, continuous stacking H=Zw+Zf−Zw·Zf; ordered 2×2 box, H=ΣH/4, C=ΣC/4, S=ΣVw/ΣC (.5 when empty). All @1 parameters, ranges, node defaults and 48-byte ABI unchanged; no new control or seed.
2. Approve @2 alongside @1 with unchanged @1 shader bytes, lowering and pixels; old runtimes explicitly reject @2. Separate KernelId::WeavePatternV2/typed invocation, exactly one new WGSL; 18 latest types, 19→20 identities, 16→17 kernels. Next unpublished Rust 0.10.0/browser 0.10.0-alpha.0; formats v1 and plan/API v3 unchanged; nothing published.
3. Approve NODE implementation. Woven material revision and MAT-03 A–D requalification on @2 remain a separate later task, outside this PR.

The [approved formulas, continuity arguments and parameter table](./weave-pattern-v2-design.md) and [machine-readable frozen probes](../fixtures/nodes/weave-pattern-v2/acceptance-plan.json) together define the contract. The plan fixes every geometry, size, offset and threshold. Commit these before implementation. Record and stop on any frozen failure without relaxation.

Visible-core min/median=1 derives from all 792 research geometries at 128/64 px pitch; test every crossing/axial scanline in both axes for all frozen geometries at 1024²/2048². Unequal widths, endpoints, density32, empty/partial coverage, weighted box share, determinism and invalid parameters have fixed cases. Analytical centers have C=1, exact S parity, upper1/lower r and separation≥.25; helper half comparisons allow 1/1024. At support/cell edges and lift joins use ±1e−5 UV, height delta≤.01, sparse analytical tolerance1/1024 and production-formula encoded normal tolerance4/255. That normal budget covers half-neighbor error amplified by the 1K central derivative and final rounding, relief=.025/strength=.5; it is not PBR acceptance.

Periodicity: zero origin exact at every size/mode; both-power-of-two sizes exact raw half in every mode; 257×129 H/C at most one adjacent half step for plain/varied/combined-low/combined-high only. The pre-freeze independent f32 study found maximum H/C one step in each of those four cases; modeled share already reached seven steps. Odd S/P remain recorded observations, never claimed passes. Inject origins only in a test copy with a unique source anchor; no production shader/ABI test feature. Hardware executes four sizes; actual reported Cpu marks 2048² notRunOnSoftware under Stage B amendment4.

Public Native/browser delta≤1/255. Compare @1 CLI bytes against preimplementation main and verify its shader hash. Research references are sparse test oracles, not a CPU renderer. Implement invariant8 and the node playbook; preserve existing material/golden/acceptance records.

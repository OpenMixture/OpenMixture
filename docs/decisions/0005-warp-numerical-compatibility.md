# ADR 0005: Warp numerical accuracy and compatibility

English | [简体中文](./0005-warp-numerical-compatibility.zh-CN.md)

**Status: Proposed, 2026-09-18.** This is an ALPHA-03 decision proposal, not acceptance of PR #15/#16, a baseline update, or browser qualification. Existing contracts and gates remain authoritative until an explicit acceptance record and the corresponding implementation review exist. No architecture or roadmap invariant changes in this PR.

## Context

[PR #17's retained audit](https://github.com/OpenMixture/OpenMixture/blob/e82372c6867d47810f782efa07c624e419f417bc/docs/evidence/warp-rounding-audit/README.md) separates two predeclared references: A evaluates exact rational arithmetic from actual half inputs and f32 parameters, then rounds once to half; B models specified staged f32 round-to-nearest-even operations with fused interpolation, then half conversion. Both are diagnostic models, not competing browser baselines.

For PR #15 local coordinates versus PR #16 explicit interpolation, all 41 changed software warp texels improve relative to exact A and match A/B after the change. All 14 recorded final PNG channel-pixel differences trace through 11 unique warp texels. This supports the bounded interpolation correction. It does not adjudicate every change from main to PR #15, nor prove all inputs improve. The new double-rounding counterexample produces B's `0.5` on Native/Chrome/SwiftShader while A is `0.50048828125`; explicit `fma` therefore does not implement universal exact-real-to-half rounding.

The [existing contract](../node-contracts.md) specifies scalar repeat-bilinear displacement, f32 arithmetic and half storage; it also describes the absolute-UV sampling calculation. Local coordinates are algebraically equivalent in real arithmetic but not in finite precision. That documentation and the frozen pixels must be reviewed explicitly, rather than silently declaring the old calculation irrelevant. [Architecture sections 5, 11 and 16](../../ARCHITECTURE.md) distinguish versioned meaning, measured pixel evidence and compatibility decisions.

## Proposed decision

Continue toward a bounded **v1 numerical implementation correction**, conditional on the acceptance steps below. Preserve the field clamp, signed UV strength, output-coordinate field read, wrapping, bilinear meaning, neutral/zero direct-load identity, f32 parameter lowering and half storage. Specify the ideal repeat-bilinear formula as the accuracy target, with finite-precision evaluation and recorded adapter coverage; do not promise correctly rounded A for every input, exact B on every backend, or cross-device bit identity.

Local texel coordinates and explicit interpolation remain separately reviewable implementation candidates. A and B stay fixed diagnostic references. A measures ideal-formula error; B explains one operation sequence. Neither replaces the original software golden or same-source browser comparison reference. Do not choose between A and B per failing pixel to manufacture a pass.

The [WGSL floating-point rules](https://www.w3.org/TR/2026/CRD-WGSL-20260915/#floating-point-evaluation) permit evaluation variation, and [the `fma` definition](https://www.w3.org/TR/2026/CRD-WGSL-20260915/#fma-builtin) permits an unfused implementation. A spelling change is not a portable rounding guarantee or proof of a compiler defect. A backend mismatch alone is insufficient for an upstream bug claim.

### What each gate protects

| Gate | Purpose | What it does not prove |
|---|---|---|
| Literal node cases and exact diagnostic references | Displacement direction, wrapping, identity, interpolation and localized arithmetic error | All-domain correct rounding or whole-material compatibility |
| Frozen pinned-software RGBA8 goldens | Detect implementation/dependency changes to accepted material appearance | Mathematical correctness of every old byte |
| Frozen browser per-channel max/mean/mismatch gates | Bound final texture drift for fixed materials, variants and a same-source reference | Universal device support, WGSL conformance, or a perceptual quality guarantee |
| Seam/range/non-finite/causality checks and visual review | Preserve material structure, usable outputs and approved appearance | Byte equivalence |
| Source, package, shader and adapter identities | Identify the tested executable and consumer configuration | Pixel identity from a plan hash alone |

Keep all existing thresholds and variants unchanged. A failed comparison remains failed even if the scalar result is more accurate. Mathematical adjudication can justify a separately reviewed compatibility change; it cannot relabel a regression run as passing.

## Alternatives and version boundary

| Alternative | Disposition |
|---|---|
| Keep main and its frozen pixels | Current behavior until the proposal and candidates are accepted; retains the documented precision limitation |
| v1 implementation correction with explicit pixel compatibility review | Recommended conditional route; no promise that old cached images remain identical |
| Define A or B as mandatory new numerical semantics | Separate versioned design and migration decision; current candidates do not establish this guarantee |
| Support old and new warp versions concurrently | Not a one-line version bump; requires explicit version dispatch, lowering, kernel/cache identity and consumer tests |
| Change tolerances, choose a convenient native reference, or fork the compiler now | Not justified by this evidence; excluded from this work |

If review determines the documented evaluation order is itself promised meaning, or chooses a new mandatory rounding/order contract, the v1 route stops. Changing that meaning requires node-version and migration design under the architecture; preserving ports and parameters alone does not establish compatibility. There is no automatic requirement to increment a node version for every changed pixel either.

The current [registry](../../crates/mixture-core/src/registry.rs) looks up one contract per type ID; [validation](../../crates/mixture-core/src/validation.rs) rejects any other version. [Warp lowering](../../crates/mixture-core/src/plan.rs) and [kernel mapping](../../crates/mixture-wgpu/src/kernels.rs) select one `KernelId::Warp` and one WGSL implementation. Merely changing `version: 1` to `2` would reject existing documents without retaining old execution. A concurrent-version design must preserve one WGSL per KernelId and deterministic plan/hash identities, and review the current nine-kernel cache bound. It must not introduce a second executor or silently rewrite `.mix` files.

## Consequences and migration

For the recommended route, source documents, parameter meanings and node versions remain unchanged only after the semantic review above accepts that classification. Rendered pixels and cached textures can change. Consumers needing old accepted bytes must retain the old package/build and its recorded environment; this proposal provides no general reproducibility guarantee for untested drivers. Regenerate derived textures deliberately with a recorded build, and key pixel caches by implementation/package identity as well as plan and output settings. Plan hashes identify normalized graph semantics, not shader bytes.

There is no source migration or cache implementation in this PR. A future acceptance must publish old/new source and archive identities, affected materials/channels, reviewed images, unchanged/failing gates, and the cache/re-render guidance. The existing `golden update <id> --accept` path is a separate explicit action after review; the flag is not evidence of human approval. Preserve original goldens and failed results in their historical records. An accepted new baseline must be bound to the approved candidate, not selected after browser results are observed.

## Verification and ordered next work

1. **Compatibility dossier for main → PR #15.** Account for every changed software wood output channel and all its variants, retaining before/after/difference images. Separate coordinate changes from interpolation changes; use PR #17's same-input method for unexplained coordinate witnesses. Record any worsened exact-reference error and downstream amplification. PR #17's 41-texel result covers only PR #15 → #16 and cannot replace this step. Stop on unexplained changes; do not cherry-pick passing parameters.
2. **Close the semantic decision.** Review the old sampling wording and explicitly decide whether these equivalent-formula changes are permitted v1 implementation corrections. Record the interpretation, numerical limits and pixel-compatibility consequences. If new meaning is selected, prepare a separate version/migration ADR and implementation before changing contracts. This proposed ADR alone does not authorize either route.
3. **Prepare candidate material acceptance.** Retain source-bound images and quality/seam/range/causality results for all three materials, all existing variants, and the 2K/resource/package checks. Keep the original baseline comparison visible. Resolve the compatibility and visual decision before any guarded baseline installation; do not weaken protected CI to merge the candidates.
4. **Requalify ordinary browsers independently.** After the candidate/reference identity is fixed, install the same complete package and run the unchanged 11-case/44-channel matrix in ordinary Chrome, Edge and Firefox, recording actual adapter/backend and versions. PR #16's ordinary Chrome result remains 27/44 passing; direct Edge literals and earlier Firefox results do not certify that package. Investigate residual noise/leather/wood paths with fresh same-input probes tied to that exact source. Do not infer all residual causes from older candidate runs.

A numerical fix can be accepted for its measured scope while a browser remains unqualified, but ALPHA-03 must not be marked complete for that browser. No compiler strategy, material baseline or support promise follows automatically from this proposal.

For this documentation-only PR, run `cargo xtask links` and `cargo xtask check`. Runtime acceptance still requires the node/shader/material/GPU/consumer checks prescribed by [AGENTS.md](../../AGENTS.md), visual evidence and live required checks on the actual implementation revision. The next executable task is step 1, not another interpolation rewrite.

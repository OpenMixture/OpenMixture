# Numerical contract and compatibility review — 2026-09-18

English | [简体中文](./numerical-contract-2026-09-18.zh-CN.md)

**Disposition: retain the existing v1 acceptance contract; do not merge PR15/16 as pixel-compatible fixes.** This review is complete, but does not accept either candidate, change a baseline, introduce a numerical API or promise universal cross-device identity. It records an agent review of repository contracts and measured evidence, not an independent human approval or a second-model consultation. Recommendations below require a separate implementation/compatibility proposal where they change public commitments.

## What the current contract actually guarantees

The authority order in [AGENTS.md](../../AGENTS.md), the [architecture](../../ARCHITECTURE.md), [compatibility record](../compatibility.md), [node contracts](../node-contracts.md) and [golden policy](../material-goldens.md) govern this review.

| Layer | Existing obligation | What it does not establish |
|---|---|---|
| Graph semantics | Seeded deterministic graph/plan traversal, specified ports/parameters, pixel-center coordinates, wrapping, field location, identity fast paths and output encodings | Identical GPU arithmetic across implementations |
| Numerical execution | f32 arithmetic, rgba16float intermediates, explicit half rounding in the sole WGSL executor, finite/range validation | Exact evaluation of the complete real-valued expression followed by one final rounding |
| Software regression | Exact decoded RGBA8 agreement with frozen fixtures on the recorded pinned SwiftShader environment, plus independent semantic/structure checks | A universal mathematical oracle or a guarantee for another compiler/driver/build |
| Browser qualification | Fixed candidate, same-source declared native reference, frozen channel limits, and separate plan/structure/lifecycle checks | Native is always more accurate, every browser is supported, or a failed comparison is a WGSL defect |
| Consumer compatibility | Source/node/plan/API boundaries and explicit implementation/archive identities | A plan hash alone identifies rendered pixels |

The [comparison implementation](../../xtask/src/golden/pixels.rs) checks maximum component error, mean over **all four RGBA components**, and the ratio above the pixel threshold. It records component means but does not separately gate each of them. The current [browser thresholds](../browser-tolerances.json) have pixelThreshold zero, so any component change counts. At 1024², ratio limits allow at most 10 changed baseColor/height pixels, 20 normal pixels, or 1,048 roughness pixels; maximum and overall mean remain independently mandatory. These are empirical regression limits, not a perceptual score, an f32 error budget or WGSL accuracy limits. They protect the declared output comparison, and remain binding even when the images look alike.

## Correctness and compatibility findings

1. **PR15 is an independently motivated stability correction with a pixel-compatibility impact.** With equal texture sizes, integer pixel plus local displacement is mathematically equivalent to the original sample position. Its 1024×1 alternating-input witness retains a 2^-26 UV shift as 2^-16 texels; the old absolute-coordinate calculation loses it. That proves an avoidable precision loss relative to ideal sampling and a useful proposed regression. It does not prove that the old implementation violated a pre-existing whole-expression accuracy bound: v1 specifies f32 but no such bound. The new test must not be described as an old normative test which was already failing. [PR15 evidence](../evidence/warp-local-texel/README.md) also establishes changed software wood pixels and failing frozen goldens.
2. **PR16 is an empirical interpolation improvement, not a portable exactness fix.** Its source `1c65e8631c2e2bdba8fe3aaf8b7c5762c1c215f3` passes 23 literal cases on native/SwiftShader and direct Chrome/Edge probes. The reduced browser half-boundary failure disappears. However, seven software wood output channels change by 1–4 pixels relative to PR15; frozen software wood still fails 12/16 channels. Installed Chrome remains 27/44 passing, 19 exact. Native release outputs remain identical in all 44 channels. [Immutable PR16 report](https://github.com/OpenMixture/OpenMixture/blob/5e360686da38197ee6a12f3f29fd5f20faea2935/docs/evidence/warp-explicit-interpolation/README.md) retains the source/run identities and changes. These findings reject the claim of a pixel-preserving replacement; they do not establish that every changed software pixel is mathematically worse.
3. **The scalar oracle is conditional.** The [26-point exact rational oracle](../evidence/warp-interpolation-reduction/README.md) evaluates stored samples with the already-rounded stored weight. Agreement there does not prove the displacement, neighbor choice, complete bilinear expression, upstream noise, or final PNG is correctly rounded. Original native and browser texture values are bound to those witnesses, but failure-selected samples do not establish a global error bound. Correct rounding of an intermediate f32 into half is also different from rounding the exact full expression into half.
4. **No general compiler defect is established.** The reviewed [WGSL draft](https://github.com/gpuweb/gpuweb/blob/cd910cf650d05481b60bad2b44476caff974962f/wgsl/index.bs), §§15.7.2–15.7.5 and §17.5.32, allows arithmetic variation and unfused fma. Its “correctly rounded” wording does not require ties-to-even for ordinary f32 operations. The [earlier minimal multiply/add witness](../evidence/chromium-arithmetic-reduction/README.md) establishes two allowed results for that witness, not conformance of all material operations. Neither strict compilation nor explicit fma is an independent mathematical reference. This review does not send an upstream issue.

The earlier phrase “concrete bug fixed” should therefore be read narrowly as **demonstrated numerical loss improved on tested inputs**, not “browser/spec violation proved” or “compatibility approval obtained.” Historical reports are preserved; this dated clarification does not rewrite their measurements.

## Why a small error can still fail the gate

Half conversion has decision boundaries. Two nearby f32 results on different sides of a boundary can produce adjacent half values and later different RGBA8 pixels. The existing integer half helper standardizes the conversion of the value it receives; it cannot make differing incoming values identical. A smaller real-valued error does not imply a smaller changed-pixel count against a historical baseline.

For a diagnostic rounding analysis, let `x` be an independently defined scalar result, `m` its distance to the nearest half-rounding boundary, and `epsilon` a justified bound on upstream numerical error. If the entire error interval stays in one rounding cell, quantized output is stable; an interval crossing a boundary does not certify a unique half result. Exact boundary ties need separate treatment. Such intervals are proposed analysis, **not an implemented gate or authorization to ignore boundary pixels**. No epsilon is selected from a desired pass rate.

## Compatibility boundary and decision rules

| Proposed change | Treatment |
|---|---|
| Algebraically equivalent spelling, same documented numerical promise, all existing gates pass | May remain a v1 implementation change after normal review; record new build/archive identities. |
| Same real-valued sampling meaning, but frozen accepted pixels change | Intentional numerical-output compatibility change, not a transparent patch. Requires explicit impact and acceptance review before any golden action. This is the present PR15/16 situation. |
| New required rounding sequence, quantization precision, deterministic mode, field interpretation, ports or defaults | Numerical/semantic contract change. Review node-version and migration policy under the architecture; do not silently reinterpret v1. Format version changes only if the document language requires them. |
| Universal bit-identical pixels across implementations | New product scope, not implied by “deterministic graph.” Requires a defined numerical model, supported environment boundary, performance/packaging evidence and architecture/roadmap review before implementation. |

Do not automatically increment a node version for every changed byte, and do not assume a runtime version alone can authorize changed node meaning. For existing consumers, retain the original source/build/archive and outputs; `.mix` plus plan hash is insufficient to reproduce old pixels. This does not introduce a second renderer or a hidden compatibility fallback.

An intentional golden migration is distinguishable from making a test green only when: the defect/desired numerical behavior is specified independently of candidate output; predeclared tests and error analysis support it; all material variants and consumer impacts are disclosed; the prior baseline/failures/images are retained; and an explicit compatibility/visual decision precedes the existing guarded update workflow. Updating a software golden cannot by itself qualify Chrome: browser comparison against the same-source native reference remains a separate gate. None of these acceptance actions is authorized or performed by this review.

## Recommended next bounded task

**Audit warp's displacement, interpolation and half-boundary error before proposing another rewrite or compiler policy.** Freeze the test input set before comparing implementations: current 23 literals, all existing wood variants, both signs/axes, endpoint/neutral and near-neutral fields, thin and odd dimensions, and a recorded fixed-seed held-out sample set. Do not choose samples or references based on which backend wins.

Use separate scalar analyses for (a) actual f32 parameters/half texture inputs through displacement and neighbor selection and (b) interpolation with a frozen weight, then trace their propagation separately. Retain the coordinate wrapping, precision and rounding assumptions. Label a reference as ideal arithmetic or a specified staged rounding model, never simply “native truth.” Compare original, local-only and explicit-interpolation implementations against the same definitions; record raw errors, neighbor changes, quantization-boundary crossings, finite/range/seam checks and both unchanged material gates. A scalar rational/interval oracle remains diagnostic and does not become a CPU material executor.

The result must either justify a narrowly specified intentional numerical correction with known compatibility cost, or show insufficient benefit to adopt it. If a product requirement for broader bit identity emerges, stop local spelling experiments and prepare a separate numerical-model proposal. Do not begin a compiler fork, blanket fma replacement, tolerance relaxation or baseline reset. PR15/16 stay draft; Chrome/Edge full-material qualification remains pending.

## Review verification and scope

On 2026-09-18, reread the contracts, shader rounding helper and executable comparison logic, and reran the retained multiply/add oracle, 26-point texture-bound oracle and PR16's 23-case evidence verifier; all passed. The WGSL editor's draft still identified revision `cd910cf650d05481b60bad2b44476caff974962f`. No GPU matrix was rerun for this documentation review. Candidate findings refer to their recorded revisions, not the later review commit. This record changes no shader, fixture, golden, tolerance, dependency, required check or support declaration.

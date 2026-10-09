# weave-pattern@2 visibility proposal

English | [简体中文](./weave-pattern-v2-design.zh-CN.md)

**Historical design research. Candidate C is approved and the node contract is now recorded in [frozen acceptance](./weave-pattern-v2-acceptance.md). Implementation follows separately; pending wording below retains its design-time identity.** The maintainer started this follow-up on 2026-10-09, based on acceptance branch `e78ae955307f02b424113ee086e28c97c294bbc0` (PR #88). The [accepted MAT-03 record](./evidence/mat-03/README.md), recipe revision 2, original ranges and `weave-pattern@1` remain unchanged. This proposal does not extend accepted visual scope or authorize implementation.

## Bounded problem and evidence

The [@1 contract](./mat-03-woven-surfaces.md#proposed-weave-pattern1-pending-catalogversion-review) selects visibility from transverse-profile heights. At an over-yarn edge its profile approaches zero while the under yarn remains elevated. Selecting the locally taller yarn therefore produces a bow-tie outline. That is correct @1 behavior. PR #87's range-only proposal was closed unmerged; narrowing one control did not solve the interaction. The accepted limitation includes plain's 0.875 minimum visible-core width and combined-high's approximately 0.42; the original 432-point sweep has 156 points meeting 0.75, with only 8/144 at crown=1. Stage B recorded up to 27 half steps of odd-size translated share sensitivity.

The proposed use case remains opaque plain weave, including unequal axes, without twill, a weave catalog, cloth simulation, fiber geometry or universal antialiasing. Desired properties are straight exposed yarn cores, continuous lift/dip, rounded crowns, no height step at an ownership switch, height-independent coverage, exact center parity/separation, periodicity and common visible weights for color, roughness and height. Geometric continuity alone is not a human visual acceptance decision.

## Common definitions and sampling

Use the following definitions for every candidate, retaining @1 coordinates and controls. `F(t)=s²(3−2s)`, `s=clamp(t,0,1)`. All parity is Euclidean. Wrap UV once with `fract`, then `x=nw*u`, `y=nf*v`, `i=floor(x)`, `j=floor(y)`, `a=fract(x)−1/2`, `b=fract(y)−1/2`.

```text
Aw = F((ww/2 − |a|)/bevel)       Af = F((wf/2 − |b|)/bevel)
qw = max(1 − (2a/ww)^2, 0)       qf = max(1 − (2b/wf)^2, 0)
Tw = qw*(1−crown+crown*qw)        Tf = qf*(1−crown+crown*qf)
sx = F(fract(x−1/2))             sy = F(fract(y−1/2))
Lw = even(i+floor(y−1/2)) ? 1−sy : sy
Lf = even(j+floor(x−1/2)) ? sx : 1−sx
hw = Tw*(r+(1−r)*Lw)            hf = Tf*(r+(1−r)*Lf)
C = Aw + (1−Aw)*Af
Vw = Aw*(1−Af*(1−D))            Vf = C−Vw
```

Here `r=underRatio`. @1 baseline uses `e=bevel*(1−r)`, `D=F((hw−hf+e)/(2e))`, `H=Vw*hw+Vf*hf`. At each pixel use the ordered four samples `(p+(.25,.25)) / size`, `(.75,.25)`, `(.25,.75)`, `(.75,.75)`. Output height `ΣH/4`, coverage `ΣC/4`, or warp-share `ΣVw/ΣC` (0.5 when ΣC=0). Do not average individual shares. Clamp normalized outputs, use f32 arithmetic and rgba16float `[value,0,0,1]` storage. Each of the three graph instances must copy identical geometry. Relief and grain remain external material operations.

## Three candidates

### A — centreline selection, existing weighted height

`D=F(1/2+Lw−Lf)`, `H=Vw*hw+Vf*hf`. The fixed ramp half-width is 0.5 in lift-difference units; it adds no parameter. It removes transverse crown and underRatio from selection, but still blends ownership inside the overlap. At crossing centers the lift difference is ±1, giving exact saturated parity. Center heights remain 1 and r. The ramp and lifts are C1; weighted fields are C1 at cell boundaries because the affected occupancy is identically zero in the inter-yarn gap. The finite ramp can still lose core width near longitudinal ends, and it can select an upper surface below the lower one. It is an experiment, not the recommendation.

### B — occupancy-gated ownership, existing weighted height

Define signed cell signals with transitions confined to the unoccupied gaps:

```text
X = (even(i) ? 1 : −1) * F((1/2−|a|)/((1−ww)/2))
Y = (even(j) ? 1 : −1) * F((1/2−|b|)/((1−wf)/2))
D = (1+X*Y)/2
H = Vw*hw + Vf*hf
```

In overlap (`Aw>0` and `Af>0`), X/Y are exactly ±1, so D is exact crossing parity everywhere across the occupied width. Outside overlap its value cannot affect Vw: `∂Vw/∂D=Aw*Af=0`. For evaluation, `D=even(i+j)?1:0` is therefore algebraically equivalent for H/C/V; the proposal defines the C1 gap ramp above, not an observable hard switch. No tuning parameter is added.

Ownership is selected before fractional occupancy compositing. In a feather where Aw<0.5, the background yarn can still contribute more RGB even when D=1. The straight-core score deliberately excludes this antialiasing edge; approving this distinction is part of reviewing the proposal, not an assertion that every nonzero-coverage pixel must be red-dominant.

X/Y and their first derivatives agree across cell boundaries (both approach zero with zero derivative); their inner plateau joins also have zero derivative. Aw/Af are C1. Thus visible weights and weighted height are C1, with no ownership step. However, choosing ownership alone does not fix physical height order at an edge: hw can be less than hf while warp remains selected. The sweep's undercut diagnostic distinguishes this smooth valley from a discontinuity.

### C — occupancy-gated ownership with continuous stacking (recommended)

Use B's D and visible weights, and raise the over surface to meet the underlying surface continuously:

```text
warp-over: Uw = Af*hf + (1−Af*hf)*hw; Uf = hf
weft-over: Uw = hw; Uf = Aw*hw + (1−Aw*hw)*hf
H = Vw*Uw + Vf*Uf
  = Zw + Zf − Zw*Zf, where Zw=Aw*hw, Zf=Af*hf
```

The simplified expression is identical on both sides of the ownership switch, including gaps. It is the proposed evaluation form; avoid a subtract-from-one formulation near zero. This is a change to height as well as visibility, requiring @2. It is not a color-only repair. Color and roughness use the same Vw/Vf (via coverage and share); height is exactly their composition with the adjusted visible surfaces.

Where the lower occupancy is full, the upper surface exceeds the lower by `(1−lowerHeight)*upperProfileHeight` inside the upper support, and meets it as the upper profile tends to zero. At either crossing center the upper is exactly 1, the lower r, separation `1−r≥0.25`; material separation is relief times that value before storage. Exact zero relief remains zero. Arbitrarily tiny positive relief cannot promise a stored-half separation after underflow. The lift functions and transverse profiles are retained; raising the support alters the rendered dip/crown shape and must receive a new PBR review.

### Continuity, periodicity and precision comparison

Tw/Tf alone have a derivative corner at their support boundary for crown<1. Their products with occupancy do not: occupancy vanishes quadratically there, so Zw/Zf and their first derivatives vanish. In A/B the same factors occur in every visible height term. C's polynomial in Zw/Zf is therefore C1 across support edges and ownership switches. The absolute-value cusp at a yarn center is hidden by the occupancy plateau; Tw/Tf are smooth even profiles there. Lift joins use F with zero endpoint derivative. Even counts make signs, lifts and fields repeat over each tile; opposite border texels are distinct samples, not equality probes. Box filtering preserves these analytical continuity/periodicity properties. Stored-half quantization does not preserve differentiability.

@1 has selector derivative up to `0.75/[bevel*(1−r)]` with respect to height difference (150 at the smallest e). A bounds its derivative with respect to lift difference by 1.5. B/C remove that term from observable weights entirely: their remaining slopes are occupancy slopes, bounded by `1.5*n/bevel` per UV axis. Gap-selector derivatives may be large but multiply zero overlap. C has `∂H/∂Zw=1−Zf`, `∂H/∂Zf=1−Zw`, both in [0,1]. None of this proves bit-exact odd-size translated f32 coordinates, or eliminates ordinary half-storage changes. S still divides by coverage; P=S*C is the observable quantity and empty-space S remains unobservable.

## Research measurements and limits

Research changes are confined to documentation and ignored files. GPU observations bind the unchanged material/shader hashes below; a tree with documentation edits is not described as a clean candidate qualification.

Research scripts, full rows, CLI receipts and images are in ignored `tmp/wp2-design/`, not the product or a second renderer. `METHOD.md` was written before the candidate sweep. The evaluator uses real-valued analytical formulas (JavaScript doubles), the ordered 2×2 box, round-to-nearest-even half storage for fields/color blends, and sRGB8 before the unchanged revision-2 visible-width test. It includes every scanline of the 2×2 crossing motif, equivalent to all tile crossings for the even-count equal-axis sweep. Ties lose visibility. The fixed transverse core is `|a|≤(width−bevel)/2`; the metric does **not** prove an unchanged antialiased outer silhouette.

The grid is r={.25,.30,.35,.40,.45,.50,.55,.60,.65,.70,.75}, equal widths={.55,.65,.75,.80,.85,.90}, bevel={.02,.05,.08,.12}, crown={0,.5,1}: **792 geometries per formula**. Evaluate 8×8 at 1024² and 32×32 at analytical 4096² (128 pixels/pitch; same motif, different UV derivatives/normals), plus 32×32 at 2048² (64 pixels/pitch). Analytical 4096² is not a supported material size or budget measurement. Relief=.05, normalStrength=.5; diagnostic presets additionally use their actual frozen controls at 1024², including unequal 12×8 varied. No thresholds, ranges or accepted cases change.

| Formula | Core ≥.85 /792 | Minimum width (128 /64 px) | Worst median (128 px) | Max J | Undercut samples |
|---|---:|---:|---:|---:|---:|
| @1 | 38 | 0.357143 / 0.333333 | 0.625000 | 1.329807 | 7056 |
| A centreline | 792 | 1.000000 / 0.962963 | 1.000000 | 0.999925 | 114720 |
| B ownership | 792 | 1.000000 / 1.000000 | 1.000000 | 0.999925 | 114720 |
| C stacking | 792 | 1.000000 / 1.000000 | 1.000000 | 0.999947 | 0 |

All candidates pass 792×4 analytical crossing-center parity/coverage/positive-separation checks. C also has warp/weft minimum and median 1 in 48 unequal-width/endpoint/pitch checks (.6/.9 and transpose, r=.25/.75, crown=0/.5/1, bevel=.02/.12, 64/128 px). Undercuts count H below the fully occupied lower yarn when upper occupancy is nonzero on a 32×32 motif sample grid; these are analytical diagnostics, not GPU failures.

| Formula | Max adjacent normal distance: 8×8 /32×32 | Observable switch-zone max at 8×8 | Max ΔVw: counts 8 /32, δUV=1e−7 |
|---|---:|---:|---:|
| @1 | 0.683411 / 1.402983 | 0.651219 | 3.843754e-4 / 1.537716e-3 |
| A centreline | 1.620941 / 1.961020 | 1.137134 | 2.253811e-5 / 9.015469e-5 |
| B ownership | 1.620941 / 1.961020 | N/A | 2.250019e-5 / 9.000307e-5 |
| C stacking | 0.586013 / 1.291706 | N/A | 2.250019e-5 / 9.000307e-5 |

The sensitivity table above measures pre-filter sample Vw on a fixed 32×32 grid in the 2×2 motif, perturbing x/y separately by 1e−7 UV. A supplemental check applies the ordered 2×2 box at the same grid and compares filtered, pre-storage mean Vw. C improves this worst case by about 7.73× (versus 17.1× for point samples).

| Formula | Box-filtered max ΔVw: counts 8 /32 |
|---|---:|
| @1 | 1.727882e-4 / 6.912081e-4 |
| A | 2.239057e-5 / 8.956453e-5 |
| B | 2.235371e-5 / 8.941713e-5 |
| C | 2.235371e-5 / 8.941713e-5 |

J is the maximum adjacent analytic box-height jump divided by interval length times the largest local slope sampled at endpoints/midpoint (floor 1e−9). Intervals are 1/128 cell on a fixed 16×16 motif grid; symmetric derivative step is 1e−6 cell. It is a sampled diagnostic, not a bound on every point. Normal diagnostics apply the production formula to half-stored final height: cyclic central differences scaled by image dimensions, normalize `(-du*strength,+dv*strength,1)`. Global adjacent normal-vector distance includes real yarn edges. Switch-zone measurements require `Aw*Af>.25` and `.1<D<.9`; B/C have no observable selector-switch samples, so that entry is **not applicable**, not a measured zero seam. Analytical C1 proofs and human review remain separate.

J can exceed one when its three derivative samples miss a local extremum; the @1 value is not evidence of a height discontinuity. The contact sheet `tmp/wp2-design/contact-native-crops.png` uses rows plain/varied/combined-high and paired @1/C columns for visibility, height and normal, all native-resolution 256² crops.

On 2026-10-09, the existing release CLI on source branch e78ae95 rendered actual material.mix diagnostic overrides for plain/varied/combined-high at 1024² on NVIDIA GeForce GT 1030 Vulkan. GPU width metrics match the @1 rows below. A further 16 nontrivial height/coverage/share pixels were observed by cloning the material only in tmp and amplifying the expected half bin through the existing levels node: all returned byte 128, identifying the same stored half value as analytical rounding. The window is [h−ULP/4,h+ULP/4]; an adjacent half would saturate to 0 or 255, unlike a direct RGBA8 comparison. No raw-readback API was added. This small pixel check does not prove whole-evaluator GPU equivalence. The original 432-point subset also exactly reproduces 156/432 at .75 and 8/144 for crown=1.

CLI SHA-256: `a07c565ea50079025ab24b535b57f99d47e4c66a802a24554e866364dcd1766e`; material SHA-256: `95db023744a09224de3344613fe503c1e2187590b48667ef5ac199ae49959757`; @1 WGSL SHA-256: `284293d994578e263a3e867aed7d4ebbbb89541a44797f79020fc1814448d925`.

```text
mixture render fixtures/materials/woven-fabric/material.mix --size 1024 \
  --output baseColor,height,normal --backend vulkan --out tmp/wp2-design/gpu-plain \
  --set warpColor=[1,0,0,1] --set warpDark=[1,0,0,1] \
  --set weftColor=[0,1,0,1] --set weftDark=[0,1,0,1] \
  --set backingColor=[0,0,1,1] --json
```

Full per-preset arguments are retained in each command.json; existing wovenFabricRequest maps detailAmount=0 to the color endpoints. Local research reproduction commands (scripts intentionally ignored):

```text
node tmp/wp2-design/verify.mjs
node tmp/wp2-design/sweep.mjs
node tmp/wp2-design/images.mjs
node tmp/wp2-design/conditioning.mjs
```

| 1024² preset | @1 warp min /median | @1 weft min /median | C both axes min /median |
|---|---:|---:|---:|
| plain | 0.875000 / 0.975000 | 0.875000 / 0.975000 | 1 / 1 |
| varied | 0.909091 / 0.955556 | 0.836735 / 1.000000 | 1 / 1 |
| combined-high | 0.416667 / 0.708333 | 0.416667 / 0.708333 | 1 / 1 |

A separate explicit-f32 arithmetic model (not GPU qualification) checks 257×129 shifts (W,0), (0,H), (W+1,H+1) against the corresponding reference pixel, recording stored P=S*C. Raw S steps divide by the smaller endpoint ULP.

| Preset /formula | Nonzero ΔP comparisons | Max ΔP | Max raw S steps |
|---|---:|---:|---:|
| plain / @1 | 682 | 0.000488281250 | 4 |
| plain / C | 741 | 0.000450611115 | 1 |
| varied / @1 | 441 | 0.000488281250 | 5 |
| varied / C | 384 | 0.000243425369 | 1 |
| combined-high / @1 | 2124 | 0.000488281250 | 27 |
| combined-high / C | 1635 | 0.000483751297 | 7 |

C reduces the full-grid real-arithmetic worst visible-weight sensitivity by about 17.1×, but improvement is not pointwise: the f32 substudy gives combined-high +1e−7 UV maxima @1 1.74902e−5 versus C 4.76837e−5; plain 6.42538e−5 /4.29153e−6, varied 1.17004e−4 /3.57032e−5. C still reaches 7 raw S steps at odd size. No exactness or one-step gate is proposed from this result, and no Stage B amendment changes.

The recommendation is C because it combines B's straight core with a continuous upper/lower height meeting, which B alone lacks. This remains a bounded design recommendation. Agent inspection of the diagnostic normals finds rounded relief and strong ordinary yarn-edge gradients at dense/max-relief settings; it is not a guarantee of fine-fabric appearance, no visible lines on every adapter, or human acceptance. A new GPU implementation, structural probes and PBR sheets are required before extending the accepted material scope.

Diagnostic paths: `tmp/wp2-design/{plain,varied,combined-high}-{v1,stack}-{visibility,height,normal}.png`, plus `*-crossing.png` (256² native crop). These are analytical research images; `gpu-*` directories separately hold actual @1 CLI renders. Height display is divided by each preset's relief for readability, not displaced. `filtered-sensitivity.json`, `unequal-width.json`, `sweep.json`, `summary.json`, `diagnostics.json`, `conditioning.json`, `gpu-validation.json` and the source-bound research manifest identify the measurements. No images, evaluator or raw logs are committed.

## Proposed public contract and ABI

Pending maintainer review: **`weave-pattern@2`**, no inputs, one Scalar `value` output. Keep `mode` enum `height` (default), `coverage`, `warp-share`, with three identically configured instances. All parameter names/types/ranges/defaults stay as the NODE @1 contract (not the material defaults):

| Parameter | Type / inclusive range | Node default |
|---|---|---|
| warpCount, weftCount | Integer, even 4..32 | 8, 8 |
| warpWidth, weftWidth | Float .55.. .9 | .7, .7 |
| bevel | Float .02.. .12 | .08 |
| crown | Float 0..1 | .5 |
| underRatio | Float .25.. .75 | .5 |
| mode | Enum listed above | height |

No new control, seed or randomness. External fractal grain still needs explicit seeds. Odd counts must be rejected, never rounded; bad types/ranges/enums use `MIX_PARAMETER_INVALID_VALUE` with node/parameter identity; unknown keys use `MIX_PARAMETER_UNKNOWN`. Bevel continues to set occupancy feather width but no longer sets depth-selection steepness. UnderRatio retains centreline lift meaning; it does not control visibility. This changed meaning at edges is explicit @2 semantics.

Retain the 48-byte uniform sketch: bytes 0..15 u32 `[warpCount,weftCount,mode,0]`, mode height=0/coverage=1/share=2; bytes 16..31 f32 `[warpWidth,weftWidth,bevel,crown]`; bytes 32..47 f32 `[underRatio,0,0,0]`. All padding zero. Same 8×8 dispatch and scalar rgba16float binding convention. No new ABI size, multi-output compiler feature or public raw-output API. Deterministic fixed evaluation order, exact identities, finite normalized storage; driver-level rounding still needs measured Native/browser ≤1/255 qualification.

## Catalog, versions and material impact

Propose a separate `KernelId::WeavePatternV2`/typed invocation and exactly one new WGSL for @2 in a **later implementation PR**, retaining @1 shader bytes and lowering. Current registry `BUILT_INS` is a latest-contract list of 18 types, with `node_contract_version` retaining noise@1 separately. @2 would keep **18 latest types**, change latest weave to version 2, and retain explicit weave@1 lookup: **19→20 supported type/version identities**, **16→17 kernels**. Explicit reviewed identity tests must cover both weave versions; count-only tests are insufficient. This is a proposed catalog change, not an edit in this PR.

Propose the next unpublished candidate **Rust 0.10.0 / browser 0.10.0-alpha.0**. The current unpublished 0.9 candidate is already bound to accepted receipts; the new kernel/invocation variants affect exhaustive Rust matches and introduce new pixels. A distinct pre-1.0 minor candidate makes that compatibility boundary explicit even without a format bump. `.mix`/`.mixpack` v1 and plan/browser API v3 remain unchanged. Old runtimes reject explicit weave@2; new runtimes render old weave@1 documents identically. No auto-migration, manifest edit or publication here.

A future woven recipe revision must explicitly use @2 in all three structural instances, preserve revision 2 bytes, and requalify MAT-03 **A–D** for the changed height/share pixels: public Native/browser matrix and budgets; structural/control/periodic/normal replay (including newly reviewed conditioning rules); producer-bound dielectric sheets and a separate human decision; retained evidence and acceptance. The arithmetic change does not add graph nodes or passes in the proposal; equal topology suggests unchanged descriptor allocation, but @2 pass/peak/time numbers are **not measured**. Existing acceptance remains confined to @1's recorded scope.

## Maintainer decisions and later gates

1. Approve/reject C's occupancy ownership plus continuous stacking, including its changed height shape and preserved parameter ranges/defaults. Decide whether the analytical evidence is enough to freeze node acceptance cases before implementation.
2. Approve/reject the explicit @2 coexistence, separate kernel (18 types / 17 kernels), and 0.10 candidate proposal.
3. Freeze implementation probes for straight visible cores at both densities, unequal widths, boundary controls, C1/normal switch behavior, center parity/separation, weighted box share, determinism and f32/half conditioning. Research sweep scores are not a substitute for GPU gates; do not reuse odd-size exactness claims without measurement.
4. Authorize a separate node implementation slice, then a separately versioned material and A–D requalification. No implementation is authorized by this design PR.

The later slice must satisfy [Agent Guide invariant 8](../AGENTS.md) and the [node playbook](./agent-playbooks.md#adding-a-built-in-node): frozen contract/cases first, Core contract and semantic validation, typed lowering and every exhaustive match, one WGSL, fixtures/docs, targeted tests, explicit catalog expectations, pinned software plus hardware and public Native/browser regression checks. Existing goldens and accepted @1 semantics remain regression obligations.

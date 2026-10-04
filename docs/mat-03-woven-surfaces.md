# MAT-03 — staged woven material qualification

English | [简体中文](./mat-03-woven-surfaces.zh-CN.md)

Material status: **plan frozen; node runtimeImplemented: true; materialAccepted: false**. Stage A and accepted recipe revision 2 are merged. Stage B gated probes are complete under the third amendment; odd-size translated share/visible weight remain observation-only limitations; C/D and full material acceptance remain pending.

Current material is [explicit recipe revision 2](./mat-03-default-revision.md): only defaults underRatio=0.25 and crown=0 change; all frozen gates remain identical. Stage A observations and original-default descriptions below are retained revision-1 history, not qualification of revision 2. The maintainer accepted these defaults on 2026-10-04; residual pinch is recorded, and full PBR/human review remains Stage C. materialAccepted=false.

## Material-default revision: pinch metric defined before the sweep

The maintainer identified revision 1's per-pixel height visibility as allowing the under yarn to win at the upper yarn's transverse edges. This is correct frozen weave-pattern@1 behavior. This experiment varies only material defaults underRatio∈[0.25,0.75] and crown∈[0,1], without changing the node, thresholds or other controls. Crown=0 remains a rounded parabola, not a flat top. This metric was defined before execution in a82db80, when revision 1 was still current.

Run plain and varied at 1024² through public material.mix/CLI. Diagnostic colors alone are warpColor=[1,0,0,1], weftColor=[0,1,0,1], backingColor=[0,0,1,1], detailAmount=0; geometry is unchanged. Monotonically sRGB-encoded baseColor R/G identify actual GPU-visible warp/weft weights; no CPU height or visibility evaluator is used. At each warp-over crossing (i+j even), examine every pixel row of its full axial cell j/nWeft≤v<(j+1)/nWeft. The denominator is the pixel width of that warp's fixed half-occupancy core: abs(fract(u*nWarp)-0.5)≤(warpWidth-bevel)/2. The numerator counts pixels in that core with R>G; ties do not win visibility. At weft-over crossings (odd), transpose the measurement: columns, G>R, weftWidth/weftCount core. Report min/median/max over all scanlines in the tile, plus crossing and scanline counts; exclude neither crossing centers nor segment ends. The half-occupancy core excludes fixed antialiasing edges; it does not establish an unchanged full yarn silhouette.

Selection target: maximize the minimum of the four plain/varied × warp/weft groups within the allowed range, and at least halve each preset's worst width deficit 1-min. Report medians and residual pinch; numerical improvement is not complete elimination or human acceptance. An ideal straight side scores 1. Coarse grid: underRatio={0.25,0.5,0.75} × crown={0,0.25,0.5,0.75,1}, including original 0.5/0.5. Retain true-color baseColor/normal/height for finalists with co-located native-resolution 256² crops. Keep relief=0.025 and normalStrength=0.5; inspect height/normal for a readable dip instead of reducing normal strength to hide the contour. Outputs and uncommitted sweep/crop tools live in ignored tmp/mat03-revision/.

## Stage A source-bound local observations

The public matrix passed at clean commit **69fb6074477f3c156e775779cb0a3770cef5189a**. This result-only update changes records, not the frozen recipe, controls, cases or gates. Plan reviewMeasurements binds the measured plan/source/builder SHA-256 and browser buildId; later metadata is not the tested snapshot. The earlier standalone Vulkan run used 9d0ddad with the same graph/gates; the table uses the final 69fb607 comparison run.

All 51 CLI validate/inspect rows passed (48 material + 3 stress), each with 21 passes and 8 physical textures. Descriptor peaks at 256²/1024²/2048²/257×129 are **4719184/75498064/301990480/2419600 B**. GT 1030 Vulkan (NVIDIA 582.66) and DX12 (32.0.15.8266) each passed 51 rows: exact repeats, loose/package equivalence, sliced height, owned outputs after destruction, constant references, allocation accounting and four matched timing budgets.

Clean Chrome **154.0.8037.98** candidate consumption passed **23** tests, including 51 woven rows. Browser reports BrowserWebGpu with an empty adapter name; this does not identify its hardware. Vulkan versus Chrome compared **255** channel images with maximum component delta **1/255** (limit ≤1/255). Worst plain/varied per-component mean error across both downsample pairs was **0.365744/255** on Native and **0.365744/255** in Chrome (limit ≤4/255). Dense/thin stress remains separately labeled; these measurements do not extend the default quality guarantee.

| Backend | Row | Cold render ms | Median of five warm renders ms |
|---|---|---:|---:|
| Vulkan | plain-1024x1024 | 380.661 | 272.581 |
| Vulkan | plain-2048x2048 | 1138.523 | 1070.967 |
| Vulkan | varied-1024x1024 | 312.148 | 241.999 |
| Vulkan | varied-2048x2048 | 1209.119 | 1020.760 |
| Dx12 | plain-1024x1024 | 931.150 | 244.278 |
| Dx12 | plain-2048x2048 | 1770.509 | 944.424 |
| Dx12 | varied-1024x1024 | 1005.098 | 219.167 |
| Dx12 | varied-2048x2048 | 1748.931 | 969.360 |

Cold 2K is recorded only; frozen cold 1K/warm 1K/warm 2K budgets are unchanged. Plain 1K baseColor/normal and varied baseColor were visually inspected for sanity, not as Stage B structural proof or Stage C PBR/human acceptance. Raw logs, requests, images and row receipts are under ignored tmp/woven-matrix/ (native-vulkan-2, native-dx12, browser, comparison); CLI commands/resolver are in the same ignored directory. No node/shader, version, other material or golden changed. Pinned SwiftShader was not run locally. Stages B–D remain later PRs and materialAccepted=false.

## Staged plan and freeze boundary

This PR's first docs/plan-only commit freezes the [material plan](../fixtures/materials/woven-fabric/qualification-plan.json) and [recipe](../fixtures/materials/woven-fabric/graph-proposal.json), before executable material and matrix work.

- A (this PR): frozen material plan, executable material and public Native/browser matrix.
- B (separate later PR): raw-half, structural, periodic, normal-replay and stress probes.
- C (separate later PR): dielectric PBR review sheets and human decision.
- D (separate later PR): retained evidence and acceptance record.

Twelve cases each run at 256², 1024², 2048² and 257×129 with five channels; three dense/thin stress requests are separate. Accepted control ranges remain unchanged, with crown=0.5, relief=0.025, underRatio=0.5 defaults. All-channel cross-runtime maximum component error is ≤1/255. For plain and varied baseColor/height, unrounded box averages from 1024→256 (4×4) and 2048→1024 (2×2) must have each RGB component's mean absolute error ≤4/255. Stress quality scope is labeled separately; parity/repeats still apply, with structural probes in B. Budgets are ≤64 passes and ≤536870912 B descriptor peak at 2K. GT 1030 cold 1K / median of five warm 1K / warm 2K targets are 10000/1000/4000 ms; pinned SwiftShader targets are 60000/20000/80000 ms. Only matched-adapter budgets qualify. Stop on any frozen gate failure; never relax it.

Original plan/proposal bytes are retained as qualification-plan-draft.json and graph-proposal-draft.json; graph-design.json retains revision 4 and original receipts (52 passes, 10 textures, 2K 369100128 B). Earlier proposal/status wording below is historical and superseded by this section and the frozen plan.

## Approved node freeze and implementation scope

On 2026-10-03 the maintainer approved PR #80 (eb3ea30) decisions (a) the four bounded failures justify one node; (b) weave-pattern@1 with no inputs, one value: Scalar output and height/coverage/warp-share modes; (c) all formulas, sampling and the 48-byte ABI below, crown=0.5, underRatio=0.5 and candidate material relief=0.025; (d) 18 types / 16 kernels and unpublished Rust 0.9.0 / browser 0.9.0-alpha.0, unchanged document/package v1 and plan/API v3; (f) this PR's first, docs-only commit freezes the node contract and [node acceptance cases](./weave-pattern-acceptance.md), before implementation commits.

**The node contract is frozen**: ports, parameters, analytical formulas, sampling and ABI under “Proposed weave-pattern@1” below are approved normative requirements, no longer pending catalog review. Historical proposal wording is retained for provenance and superseded by this decision. **The material contract, graphs and qualification plan remain draft, unfrozen, unimplemented and unaccepted**. The material structural matrix and PBR/human qualification belong to a later PR; node acceptance cannot substitute for them.

## Material brief and coordinates

One opaque, dielectric woven-fabric family: a regular plain weave and a varied plain weave with unequal axis densities/widths, two yarn colors and seeded longitudinal grain. “Varied” retains the same alternating weave, not twill or a second weave catalog. Warp runs along image v (vertical), weft along u (horizontal); origin is top left, u right, v down. One normalized UV tile repeats on both axes. Even warp/weft counts preserve alternating crossing phase. Counts determine center spacing (1/count); width is a separate fraction of that axis's spacing. No physical millimeter or mesh-density promise is made.

At crossing (i,j), warp must be above weft when (i+j) is even, and below when odd. Exchanging yarn width, spacing, color or seed must not exchange that order. Height is a zero-relative surface relief, not two-sided geometry. Output baseColor, roughness, metallic=0, height and a normal derived only from final stored height. Gaps show an opaque dark backing; opacity, AO and displacement outputs are excluded. Directional grain is encoded in maps; the current normal/roughness channels do not provide an anisotropic BRDF.


## Historical direction: existing-node findings and node review

On 2026-10-03, after reviewing revision 4 at ca2e98b, the maintainer requested a **dedicated node proposal**, not implementation. The four bounded rounds failed to deliver continuous over/under yarn paths, coherent color/roughness/height, rounded crowns and smooth crossing transitions **simultaneously**. This is evidence against the tested brick/selector compositions, not a proof that every possible existing-node graph is incapable. No further tuning or new runtime work is authorized by this design slice. Revision 4 stays the retained, executable existing-node baseline in [graph-design.json](../fixtures/materials/woven-fabric/graph-design.json); [graph-proposal.json](../fixtures/materials/woven-fabric/graph-proposal.json) is a separate, non-executable candidate.

| Round / source | Recorded five-channel measurements | Observed failure and subsequent tradeoff |
|---|---|---|
| 1: initial contract 7555c8b; review amendment c5d0fd6, retained on engine main e73e2b9 | 1K: 40 passes, 16 textures, 142,607,312 B. 2K compile rejects 570,426,320 B against 536,870,912 B. | Flat planks and invisible default grain. Broader brick bevels, grain in color/roughness and a reordered recipe fixed budget/detail in round 2, but did not remove the clipped crown or narrow selector transition. |
| 2: 92b0b2108f6442174119711fffcfe04914e7a6a3 | 45 passes, 14 textures; 1K 125,830,240 B; 2K 503,317,600 B. | Crossing seam lines and flat crowns. Round 3 widened Q and layered narrow crowns; the resulting max-height outline no longer matched the still-independent rectangular color/coverage masks, and produced bone/quilted segments. |
| 3: 793f73e96c5c7be6b1ff280d9a23b6d442198043 | 53 passes, 14 textures; 1K 125,830,464 B; 2K 503,317,824 B. | Maintainer confirmed seams/crowns fixed, but rejected channel-outline mismatch and bone/quilted appearance. Round 4 made normalized max height also serve as color/roughness coverage, reduced underRatio and halved default relief; channel probes aligned but depth became backing blend weight. |
| 4: ca2e98b326ce5aad5f5029f9f10ef82ac113419a | **52 passes, 10 textures; 1K 92,276,064 B; 2K 369,100,128 B**. | Maintainer observed under-thread spans sinking into dark backing, so yarns read as separate floating capsules. Default relief was 0.0125 instead of 0.025. Zero-difference neutral mask probes and the improved pinch ratio do not establish continuous yarn appearance. This baseline is retained, not accepted. |

These are the measurements already bound by the plan's reviewMeasurements, recipeIteration, reviewMeasurementsRevision3/4 and their input/binary hashes; this design-only change does not claim fresh GPU runs. Native images used NVIDIA GeForce GT 1030 / Vulkan / NVIDIA 582.66. Later rounds validated all eight cases and inspected both sizes; round 1's initial review covered five validation cases and the plain plan. Prior sections below preserve exact commands, source identities, limits and local evidence paths. The latest capsule finding is the maintainer's 2026-10-03 visual decision, not a new numeric metric.

The structural problem in these recipes is that brick-pattern's minimum of transverse/axial inward distances describes bounded cells, not a continuous yarn centerline. Shift/max removes end caps but leaves clipped shoulders; a common cell selector then changes transverse profile and longitudinal lift together. Separate color masks lose the actual relief outline; substituting height for occupancy aligns edges but dims still-present under-yarn into the backing. Width, bevel, Q ramp, crown and underRatio tuning redistributed these defects. A dedicated analytic centerline, an independent occupancy envelope and one shared visibility calculation directly address this coupling. Whether the proposed formulas actually look like fine fabric remains a required future GPU/PBR/human decision.

## Proposed weave-pattern@1 (pending catalog/version review)

The name parallels brick-pattern and describes a bounded procedural pattern. **Version 1 means plain weave only**: one opaque family, including unequal warp/weft counts and widths, no twill, weave catalog, cloth simulation, fiber geometry, anisotropic BRDF or universal antialiasing. It generates structural fields; existing nodes still own grain, color, roughness, relief scaling and normal derivation.

### Ports and parameters

No inputs. Exactly one output port **value: Scalar**, default None. [NodeContract](../crates/mixture-core/src/registry.rs) has `outputs: &'static [PortContract]` and named output lookup, so its metadata/edge validation can describe several ports. This is **not end-to-end multi-output execution**: [lower.rs](../crates/mixture-core/src/compiler/lower.rs) takes outputs.first(), stores one ResourceId keyed only by node ID and resolves connected inputs by producer node ID; [ComputePass](../crates/mixture-core/src/plan.rs) has one output/output_desc. The proposed mode parameter uses three ordinary instances and one shared KernelId/WGSL instead of changing that compiler/plan/resource model. Three fields are the proposed minimum: height cannot double as occupancy without repeating the capsule failure, and height/coverage do not identify which of two yarn colors is visible. Packing unrelated fields into Color would violate port meaning and lacks a scalar extraction contract.

| Parameter | JSON type / inclusive domain | Proposed default |
|---|---|---|
| mode | Enum: height, coverage, warp-share | height |
| warpCount, weftCount | Integer tokens 4..32, **even only** | 8, 8 |
| warpWidth, weftWidth | Float 0.55..0.9, fraction of transverse pitch | 0.7, 0.7 |
| bevel | Float 0.02..0.12, inward occupancy feather in transverse pitch units | 0.08 |
| crown | Float 0..1, interpolation of parabolic and squared-parabolic cross-sections | 0.5 |
| underRatio | Float 0.25..0.75, lower/upper center-height ratio | 0.5 |

crown is an **approved profile control**, frozen by the node decision above. The family/count/width domains remain the accepted ones. All parameters have defaults; every provided value is validated even when irrelevant to a selected mode or on an unused branch. Reject odd counts (including overrides) with MIX_PARAMETER_INVALID_VALUE, node ID and the offending count parameter; never round. Wrong JSON type, enum case, non-finite/out-of-range values also use MIX_PARAMETER_INVALID_VALUE; unknown keys use MIX_PARAMETER_UNKNOWN. Validation precedes GPU acquisition. The generic Integer range cannot express evenness, so node-specific semantic validation is required. No random variation is proposed and there is **no seed parameter** on this deterministic generator; the two existing fractal-noise@2 grain nodes still require independent explicit u32 seeds. Adding randomized structure later would require a separate reviewed contract/version change.

### Analytical structure and continuity

Definitions below are the normative **frozen node contract**. Evaluate f32 in the stated order; clamp named normalized quantities to [0,1]. Let `F(t)=s*s*(3-2*s)`, `s=clamp(t,0,1)`. UV origin is top left, positive u right/v down. Wrap each UV with fract before `x=u*warpCount`, `y=v*weftCount`. Let `i=floor(x)`, `j=floor(y)`, `a=fract(x)-0.5`, `b=fract(y)-0.5`. Warp runs along v, weft along u.

For each axis with transverse local coordinate a (or b) and width w:

- Occupancy `A=F((w/2-abs(a))/bevel)`. It depends only on transverse distance, never lift or which yarn is above.
- `q=max(1-(2*a/w)*(2*a/w),0)`; `T=q*((1-crown)+crown*q)`. T has a rounded maximum at the yarn center and no finite plateau. Outside support q=0. This polynomial avoids trigonometric/pow variability; it is not an exact ellipse.

Call these Aw/Tw and Af/Tf. Define a continuous warp lift along its whole centerline: `k=floor(y-0.5)`, `t=fract(y-0.5)`, `p=EuclideanMod(i+k,2)`; `Lw=1-F(t)` if p=0, otherwise F(t). For weft: `l=floor(x-0.5)`, `t'=fract(x-0.5)`, `p'=EuclideanMod(j+l,2)`; `Lf=F(t')` if p'=0, otherwise 1-F(t'). Signed integer k/l may be -1; Euclidean parity must not use a negative remainder or unchecked unsigned cast. Adjacent intervals meet with equal values and zero first derivatives.

Compute per-thread heights `Hw=Tw*(underRatio+(1-underRatio)*Lw)`, `Hf=Tf*(underRatio+(1-underRatio)*Lf)`. At crossing centers `((i+0.5)/warpCount,(j+0.5)/weftCount)`, both transverse profiles equal 1: even i+j gives Hw=1,Hf=underRatio, odd gives the reverse. Positive relief therefore gives strict upper>lower center separation. Even counts make lift phase periodic in both tile axes. An index switch across a transverse cell boundary is hidden inside the zero-occupancy gap; there is no longitudinal cap. On an uncovered warp centerline Aw=1, Af=0, coverage stays 1 and Hw>=underRatio even while the yarn dips. This is the key distinction from revision 4's coverage=height.

One visibility calculation serves every mode:

- `e=bevel*(1-underRatio)>0`; `D=F((Hw-Hf+e)/(2*e))`. This smooth depth selector saturates to the correct yarn at crossing centers because e<(1-underRatio). It avoids a hard max switch; the selected transition width still needs visual review.
- `C=Aw+(1-Aw)*Af`. `Vw=clamp(Aw*(1-Af*(1-D)),0,C)`; `Vf=C-Vw`. The visible weights partition coverage, not height. Where only one yarn is present its share is full regardless of depth.
- `H=Vw*Hw+Vf*Hf`. Return normalized height H, independent coverage C and conditional warp share `S=Vw/C` (0.5 when C=0), as selected by mode. A share may jump in empty space, but its coverage-weighted contribution is zero there; require continuity of visible weights/height, not an unobservable selector through a gap. Opaque backing supplies the uncovered fraction.

### Sampling, storage and ABI sketch

Reuse brick's fixed 2×2 box footprint: pixel offsets (0.25,0.25), (0.75,0.25), (0.25,0.75), (0.75,0.75), divided by output width/height, evaluated in that order. Accumulate H, C and Vw independently. height returns sumH/4; coverage returns sumC/4; **warp-share returns sumVw/sumC**, or 0.5 for sumC=0, not the unweighted mean of sample shares. This keeps averaged visible occupancy coherent at partial footprints before half storage; separate half stores and later blends still introduce rounding that must be qualified. All three instances must use identical geometry parameters and dimensions; the fixture caller resolves one bundle and copies it. Individual node validation does not enforce equality between different instances. External grain is sampled by its existing nodes, so no joint four-sample integration or universal antialiasing is promised.

Each mode stores [value,0,0,1] in rgba16float after f32 evaluation and the existing half conversion. No packed secondary channel, CPU executor, upload or implicit lookup. Same-adapter repeats must be deterministic; f32 arithmetic, division, f16 storage and normal amplification still require Native/browser <=1/255 qualification, including 256/1024/2048 and 257×129. Compare periodic translated evaluations, not opposite border texels. Do not claim GPU seam freedom or exact center separation for arbitrarily small relief after half underflow without measured bounds.

Proposed uniform ABI is **48 bytes**, three 16-byte rows (offsets in bytes):

| Offset | Four words |
|---|---|
| 0 | u32 warpCount, u32 weftCount, u32 mode (height=0, coverage=1, warp-share=2), u32 reserved=0 |
| 16 | f32 warpWidth, f32 weftWidth, f32 bevel, f32 crown |
| 32 | f32 underRatio, f32 reserved=0, f32 reserved=0, f32 reserved=0 |

Propose one KernelId::WeavePattern invocation and one WGSL file with a common sample helper for all modes, 8×8 dispatch, group 0 binding 0 uniform and binding 1 output storage texture. Dimensions come from the output texture. Determinism includes fixed mode codes, no random state, fixed evaluation/accumulation order, zeroed padding and ordinary canonical plan hashing of the resolved parameters. No ABI/kernel code is added here.

### Draft graph and unmeasured resource projection

The separate [proposal graph](../fixtures/materials/woven-fabric/graph-proposal.json) has three weave instances (height, coverage, warp-share), six existing grain passes, two gradient maps, visible-grain selection, roughness remap/backing blend, color yarn/backing blends, height levels, normal and constants: **21 projected passes**. Shared C/S drive color and roughness; only the height mode is scaled by relief. Proposal defaults restore relief=0.025 and underRatio=0.5 and add crown=0.5; the retained baseline's defaults/bytes do not change. Acceptance of those candidate defaults is pending. Grain seed/color/roughness controls remain in existing nodes.

A static lifetime trace for the explicit numeric ID order projects **8 textures** (4 Scalar, 3 Color, 1 Normal), 592 uniform bytes, no image resources, one sequential staging buffer. At 2K: 8*33,554,432 + 592 + 33,554,432 = **301,990,480 B projected peak**, versus the baseline's **measured 369,100,128 B**. Logical retain-all bytes would be 21*33,554,432=704,643,072, not the governing peak. The trace is in graph-proposal.json; it assumes existing exact-descriptor reuse, no implicit default passes and the proposed 48-byte uniform. This is **not a compiled plan, measured budget pass or performance result**. Keep <=64 passes and <=536,870,912 B unchanged; measure the real implementation later. Recomputing the shared sample helper in three passes is an intentional small-kernel tradeoff, not assumed pass fusion.

### Historical proposal gates (node approval superseded by the freeze above)

Pending approval, catalog **17 types / 15 kernels → 18 / 16**; retained fractal-noise@1/v2 coexistence does not add a second type. Propose the next **unpublished** candidate **Rust 0.9.0 / browser 0.9.0-alpha.0**, because the new KernelId/KernelInvocation variant affects exhaustive Rust matches and downstream consumers. At the proposal checkpoint, manifests stayed 0.8.0 / 0.8.0-alpha.0. The authorized node implementation now selects 0.9.0 / 0.9.0-alpha.0; publication remains excluded. Keep .mix/.mixpack v1 and plan/API v3; preserve old documents and existing plan/hash encodings. Old catalogs reject weave-pattern@1 with MIX_NODE_UNKNOWN_TYPE, or a known type's unknown version with MIX_NODE_UNSUPPORTED_VERSION; never substitute another recipe or auto-migrate. The current CLI cannot validate/render the proposed graph as an implemented node.

A **later, separately authorized implementation PR**, after catalog/version and contract review, must follow [AGENTS invariant 8](../AGENTS.md) and the [node playbook](./agent-playbooks.md):

1. Bind the approved bounded MAT-03 use case, exact contract, controls/acceptance cases and compatibility/version choice; no implementation before that approval.
2. Add the small Core contract, default/override/full-graph validation (including odd counts), typed lowering, one exhaustive KernelId/KernelInvocation, canonical parameter serialization/hash behavior and correct uniform size. Audit every exhaustive Rust match and downstream metadata assertion.
3. Add exactly one WGSL implementation and focused fixtures for defaults, all modes, boundaries, invalid inputs, unequal-axis case, periodic translations, center parity/separation and empty/partial coverage. Keep old catalog expectations explicit, not count-only/self-derived; update reviewed 18-type/16-kernel expectations only in that later PR.
4. Add node docs and paired public guides, focused Core/plan/node tests, shader validation, pinned software node checks, GT 1030 and Native/browser public-consumer comparisons. Future cargo xtask test-node weave-pattern requires its harness entry; it is not runnable now. Include cross-mode identical-parameter/size consistency and weighted box-filter probes; any test-only layer observation must reuse the production helper/sole executor.
5. Run the complete material structural/aliasing/control/zero-relief/normal-replay/periodic/odd-size matrix; preserve ceramic/leather/wood, MAT-01/MAT-02 regressions, package/consumer gates, repeat/parity <=1/255, unchanged memory/pass/timing limits, PBR and separate human continuous-yarn review. Reproduce revision-4 baseline as the comparison; candidate compilation alone is not success.
6. Synchronize bilingual docs/guide impact, run relevant shader/node/material checks and cargo xtask check, retain review bytes per evidence policy and all six required checks. No golden overwrite to hide failure, publication, new crate or Studio work.

Explicit maintainer decisions before proceeding: (a) approve/reject the bounded findings as grounds for one node rather than more composition; (b) approve/revise weave-pattern@1, one Scalar/mode contract and three instances versus a separately scoped multi-output design; (c) approve/revise occupancy/lift/profile/visibility/filter formulas, crown control, bevel meaning and candidate defaults; (d) approve/reject the 18/16 catalog and unpublished 0.9 candidate versions with unchanged document/plan/API versions; (e) choose exact structural, sampling, continuity and human/PBR acceptance probes and retained-image requirements; (f) explicitly authorize a later implementation only after contract/catalog review. None is implied by this proposal, and no freeze or implementation acceptance is claimed.

## Existing-node feasibility before admission (historical)

Reviewed baseline: main `5785068`, [registry](../crates/mixture-core/src/registry.rs), [actual node contracts](./node-contracts.md), [MAT-01 profile formula](./mat-01-structured-materials.md), and the production [brick shader](../crates/mixture-wgpu/shaders/nodes/brick-pattern.wgsl). The table is the original source/contract derivation; the source-bound Native review below adds limited rendered observations, not material qualification. MAT-02's [input measurements](./evidence/mat-02-input-feasibility/README.md) establish the precedent for later source-bound measurements, not evidence for fabric.

| Attempt / actual contract | Concrete limitation | Existing-node construction / remaining risk |
|---|---|---|
| `checker@1` provides summed cell parity. | Its output is Color; no Color-to-Scalar conversion exists, so it cannot connect to Scalar height/mask ports. Its integer pixel indexing also differs from profile subsamples. | Do not connect it or add an implicit conversion. Use a staggered Scalar brick field for crossing selection. |
| `brick-pattern@1` uses min(x inward distance,y inward distance), one bevel, cell amplitude and four quarter-pixel samples. | A single tall brick is not a continuous rounded thread: with rows=1, mortarY=0, bevel>0, its profile still falls to zero at v=0. Setting bevel=0 removes the smooth transverse profile too. Variation is per brick, not continuous longitudinal grain. | Two copies separated by half a tile along the thread, combined with max, remove the axial end profile in the underlying continuous field. For bevel<=0.25 one copy always has axial inward distance>=0.25. Four-sample averaging, bilinear phase shift and half storage prevent treating this algebra as a measured pixel guarantee. |
| Staggered `brick-pattern@1` with columns=warpCount/2, rows=weftCount, rowOffset=0.5. | Ordinary brick height does not itself express two crossing thread layers. | Shift source X by 0.5/warpCount; at crossing centers the alternating cells give Q=1/0. Use mortarX=0.45, mortarY=0, bevel=0.25, variation=0. This fixes an alternating selector, not an arbitrary weave. Row-edge bevel provides a transition; its smoothness and odd-size resampling remain qualification questions. |
| `scalar-mask-blend@1` is clamped pointwise interpolation; `scalar-subtract@1` is saturating subtraction. | Neither generates spatial order. `mix(A,B,Q)` alone can suppress a visible non-overlapping thread and is not max(A,B). | Compose profiles first. Synthesize max(A,B): D=subtract(A,B); M=scalar-blend(B,D,weight=0.5); levels(M,inputMin=0,inputMax=0.5,gamma=1). In real arithmetic this is max; two extra half roundings must be measured, especially before normal derivation. |
| `scalar-morphology@1` is wrapped axial min/max over integer radius 0..16. | Dilation/erosion changes both support and width in texels, can remove thin threads and cannot recover missing subpixel shape or create crossing parity. | Not needed in this recipe. It is not a continuous-width profile or antialiasing substitute. |
| `fractal-noise@2` value basis has one isotropic integer scale, explicit seed, periodic lattice and no antialiasing. | Alone it does not produce independent axis grain; cellular/v1 noise is not covered by this design. | Existing `transform-2d@1` scaleX=8, scaleY=1 makes grain elongated along v; quarterTurns=1 rotates it for weft. Two explicit seeds allow independent grain. Integer scaling preserves periodicity but bilinear resampling is not prefiltering. |
| `levels`, constants, color `blend`, Scalar blends and `height-to-normal@1`. | Pointwise operations cannot repair upstream seams/aliasing; normal differentiation can amplify f16 steps. | They supply tint, bounded relief, roughness and normal composition without a new channel or executor. Test actual stored height, not an RGBA8 reconstruction. `warp@1` is unnecessary and its folding/precision risks would not solve these gaps. `image-input@1` would import caller-authored structure, not demonstrate procedural feasibility. |

Historical conclusion before the four-round review: the current catalog can express layout, center parity and detail algebraically, but that was not a simultaneous visual/material pass. The maintainer now requests the bounded proposal above; the following measurements remain historical source-bound observations, not proof of impossibility or approval of a new node.

### Initial source-bound review (revision 1; historical)

On 2026-10-02, before these amendments, clean branch revision `7555c8b25558fb7a2aedd2a14adc2474b6562006` was tested against main `5785068d8d3e49a503bfe30cb1d90d28f0bc548e`. A freshly built release CLI used `cargo build --release --locked -p mixture-cli --target-dir target/native-consumer`. The plan's `reviewMeasurements` records the binary, lockfile, original graph/plan and resolved-document SHA-256 identities, exact CLI argument arrays and exit codes. The amended plan is not the measured input. Resolve defaults plus each case using the fixture mapping, replace every `$` value, move node input references to ordinary `edges`, and connect the five outputs to a `material-output@1` node in a `.mix v1` document. The temporary resolver is not a repository tool.

- `mixture validate <case.mix> --json` passed with no diagnostics for plain, varied, flat, combined-low and combined-high.
- `mixture inspect <plain.mix> --plan --size 1024 --output baseColor,normal,roughness,metallic,height --json` compiled plan v3: **40 passes, 16 physical textures, peakBytes 142,607,312, logicalTextureBytes 335,544,320**.
- The identical request at `--size 2048` exited 2 with **MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED**, configured **536,870,912**, observed **570,426,320**. This is a compile-time descriptor-budget failure with plan-v3 reuse already applied; no 2K GPU allocation/render occurred. It **fails the draft <=512 MiB @2048² budget and blocks freezing**. Resolve by recipe change or a separately scoped, measured PERF-MAT slice, **never by raising the limit**; the generic CLI suggestion about caller limits does not authorize changing this material gate.
- `mixture render <case.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json` succeeded for plain and varied on **NVIDIA GeForce GT 1030 / Vulkan, NVIDIA driver 582.66**. Inspection of the original baseColor/normal PNGs shows alternating over/under structure and the varied 12×8 counts/unequal widths. This is visual layout evidence, not a complete independent-control causality test. Profiles read as flat-topped planks with narrow bevels rather than rounded yarn; default grain is not visibly resolved. Detail only attenuates relief: default detailAmount=0.03 acts on relief=0.025, rather than coloring the yarn. These are unresolved quality risks under decision 2, not proof that a new node is needed.

Raw reports, the resolver, five resolved graphs, receipt and ten PNGs are in ignored `tmp/mat03-review-amendment/`; they are local review output, not durable accepted evidence. The retained plan summary and [fixture reproduction steps](../fixtures/materials/woven-fabric/README.md) bind the observations to the tested source. No browser/DX12/software comparison, timing qualification, full structural probes, PBR or human acceptance is asserted.

### Maintainer decisions and recipe revision 2

On 2026-10-02 the maintainer **accepted decision 1 as drafted**: opaque plain weave plus one varied plain weave, even warp/weft counts 4..32 and widths 0.55..0.9 of spacing; no twill or other weave. Decision 2 requires rounder yarn and visible default directional grain before freeze. Decision 4 selects **existing-node recipe changes first**, keeping **536,870,912 bytes** unchanged; PERF-MAT engine work is not authorized. No new node is proposed. The preceding revision-1 observations remain historical; their budget blocker is resolved only for the newly measured revision-2 requests below.

Both before/after runs use clean engine main `e73e2b99c85987551d11db78eb90dfbf0564100a` and the same freshly built release CLI under `target/native-consumer`. The after graph/plan were temporary candidate inputs; the plan's `recipeIteration` binds their hashes, generated inputs, CLI identity, reports and PNGs. This annotated plan was not the measured input. All eight plan cases validate without diagnostics before and after. All eight after cases compile at both sizes with all five outputs; plain and varied both render successfully at both sizes on NVIDIA GeForce GT 1030 / Vulkan (NVIDIA 582.66). No baseline 2K render was possible because compilation rejected it.

| Recipe / full five-channel size | Passes | Physical textures | peakBytes | Result |
|---|---:|---:|---:|---|
| Main revision 1, 1024² | 40 | 16 | 142,607,312 | Compiles |
| Main revision 1, 2048² | — | — | 570,426,320 observed | Compile rejected; plan is null |
| Revision 2, 1024² | 45 | 14 | 125,830,240 | Compiles and plain/varied render |
| Revision 2, 2048² | 45 | 14 | 503,317,600 | Compiles and plain/varied render |

Measurements are identical across the eight cases at each inspected size. Revision-2 logical texture bytes are 377,487,360 at 1K and 1,509,949,440 at 2K; they are not the scheduled peak. The 2K peak is below the unchanged limit and 45 passes remain below 64. No allocator, shader, runtime or version changed. Numeric prefixes on fixture node IDs establish a specific legal order for the existing lexical ready-node scheduler; reordering the JSON array alone has no effect. Earlier temporary candidate schedules still failed the limit and remain under the ignored review directory; the delivered schedule is backed by actual CLI compilation, not a liveness estimate.

The public bevel range remains 0.02..0.12, now mapped to brick `profileBevel=0.19+0.5*bevel` (0.20..0.25). This broadens the curved shoulders. Default detailAmount increases from 0.03 to 0.08, retaining range 0..0.1. Value-v2 noise uses scale 4, two octaves, persistence 0.5 and an 8×1 integer transform (weft quarter-turn 1); levels remaps [0.3,0.7] to [0,1]. That independent grain still attenuates each height, and now drives each yarn's color ramp from `color*(1-4*detailAmount)` to color. Q selects grain for roughness between `yarnRoughness*(1-2*detailAmount)` and yarnRoughness, then coverage mixes over backing roughness. No-detail returns unmodulated yarn colors/roughness; seeds do not move W/F/Q.

Agent inspection of original channel PNGs observes broader curved shoulders and longitudinal color/roughness bands at defaults. Some flat crown remains, especially on wider yarns; these images do not establish an exact circular profile or final fabric quality. Maintainer visual acceptance stays pending. Review `tmp/mat03-recipe-review/after/plain-1024/` and `after/varied-1024/`: each contains baseColor.png, normal.png, roughness.png, height.png and metallic.png; matching 2048 folders also exist. Baseline images/reports are under `baseline/`. No preview tool was modified and no images/tooling were committed. This is local review output, not retained human acceptance, browser parity, timing qualification or the full sampling/structural matrix.

### Recipe revision 3 — crossing and crown review (2026-10-02)

The maintainer did **not accept revision 2 at 92b0b21**. Revision 3 remains draft, unfrozen and unimplemented as a qualified material. No default, accepted count/width range, runtime, node or version changes. Source-bound review uses branch baseline `92b0b2108f6442174119711fffcfe04914e7a6a3`, engine main `e73e2b99c85987551d11db78eb90dfbf0564100a`, and the freshly rebuilt release CLI. The plan's `reviewMeasurementsRevision3` records exact commands, input/binary hashes, results and crop hashes; the annotated plan is not the measured snapshot.

Q's brick bevel increases from 0.025 to 0.25, spreading its edge transition. Each height profile now multiplies the original W/F by a second, axial-cap-free crown: transverse mortar=0.45, bevel=0.25, the same half-tile axial shift/max construction, then levels output 0.4..1. The positive floor preserves the original width support and count/spacing meanings. At analytical crossing centers both crowns equal 1 and Q remains exactly alternating 0/1; upper height is relief and lower is relief*underRatio, strictly separated for relief>0. These are continuous-field facts, not a claim of qualified mixed-footprint/f16 pixels. The crown retains a small 0.05-pitch plateau, so it is rounder, not an exact ellipse.

Four height-grain attenuation passes are removed to keep intermediate lifetimes bounded and smooth the height surface. Directional grain still drives color and roughness at the unchanged default detailAmount=0.08; height/normal are now independent of detailAmount and the grain seeds. This recipe mapping change must be included in later control-isolation tests. The existing max composition can still produce derivative ridges where thread heights meet.

| Full five-channel request (plain, varied and combined-high; also all other plan cases) | Passes | Physical textures | peakBytes |
|---|---:|---:|---:|
| Before revision 2, 1024² | 45 | 14 | 125,830,240 |
| Before revision 2, 2048² | 45 | 14 | 503,317,600 |
| After revision 3, 1024² | 53 | 14 | 125,830,464 |
| After revision 3, 2048² | 53 | 14 | 503,317,824 |

All eight cases validate without diagnostics and compile at both sizes before/after. The unchanged limits remain 64 passes and 536,870,912 bytes; the 2K compile blocker stays resolved. After logical bytes are 444,596,224 / 1,778,384,896, not the governing peak. Plain and varied render at 1K on NVIDIA GeForce GT 1030 / Vulkan / NVIDIA 582.66 with explicit `--backend vulkan`. No new 2K render or browser/timing qualification is asserted in this iteration.

Inspection of the original normal maps and native-resolution crops shows substantially softened straight crossing lines and narrower crowns. Varied counts/widths and color/roughness grain remain visible. Residual max-join ridges, the small flat center and final PBR fabric appearance require maintainer judgment: **visual acceptance remains pending**, with no new node proposed. Review `tmp/mat03-recipe-review-3/after/plain-1024/` and `after/varied-1024/` for baseColor.png, normal.png, roughness.png and height.png. The plain directory also contains normal-crossing-256.png and height-crossing-256.png: source rectangle x=64,y=64,width=256,height=256, no scaling or tone adjustment. Height is dark because its original linear range is small; it is not raw-half evidence. Matching baseline crops/maps are under `before/`. Resolver, reports and images remain ignored local review output, not durable acceptance or committed tooling.

## Retained recipe revision 4 — historical review (2026-10-03)

The maintainer judged revision 3 at `793f73e96c5c7be6b1ff280d9a23b6d442198043` to have fixed crossing seam lines and flat crowns, but **did not accept the material**. Revision 4 remains draft, frozen: false, runtimeImplemented: false. It uses existing nodes only. The accepted plain/varied family, count/width meanings and all control ranges remain unchanged.

Let P/R be the crowned warp/weft profiles. Keep Q's smooth brick selector with mortarX=0.45 and bevel=0.25. Define normalized layer shapes A=P*mix(underRatio,1,Q), B=R*mix(1,underRatio,Q), shared surface coverage C=max(A,B), and shared visible-thread selection D=clamp((A-B+0.1)/0.2,0,1). Implement D without signed subtraction: levels inverts B, scalar-blend forms (A+1-B)/2, then levels maps [0.45,0.55] to [0,1]. Existing subtract/blend/levels still synthesize max. Color=mix(backing,mix(weftColor,warpColor,D),C); D also selects directional grain for yarn roughness, then C blends yarn over backing roughness. Height=relief*C; normal derives from that same stored height. Thus relief, color edges and roughness use the same surface shape, replacing the independent rectangular color composites and coverage union. Metallic stays zero. C is a normalized surface weight, not a binary geometry mask or alpha transparency.

Default underRatio changes **0.5 → 0.25**, relief **0.025 → 0.0125**, within their unchanged ranges. Count, width and other defaults do not change. Each narrow crown's positive floor changes 0.4 → 0.65, retaining ceiling 1, transverse mortar 0.45, bevel 0.25 and the axial half-tile max construction. This reduces the difference between shoulder and crown; W/F still set the outer support and spacing. Lowering the under-thread keeps it from cutting as deeply into the upper shoulders; smaller relief reduces the puffy appearance. At analytical crossing centers P=R=1, Q=0/1, A/B=1/underRatio in alternating order, D=0/1; upper>lower for relief>0 remains strict. Mixed footprints and half quantization still need their separate gates. Grain remains in color/roughness only. Relief zero keeps the woven color/roughness but zero height and neutral normal. Because C/D now share layer order, underRatio affects all nonmetallic channels; relief still affects height/normal only.

### Source-bound review and alignment observations

Before uses clean branch `793f73e`; after uses that engine with the revised draft graph/defaults. Both use the same freshly rebuilt release CLI from engine main `e73e2b99c85987551d11db78eb90dfbf0564100a`. The plan's `reviewMeasurementsRevision4` records graph/plan/binary/resolved-input hashes, commands, adapter and observations. Its annotated plan follows the measured snapshot; graph bytes match. All eight cases validate without diagnostics and compile with all five outputs at both sizes. Plain/varied 1K renders succeed on **NVIDIA GeForce GT 1030 / Vulkan / NVIDIA 582.66**, explicitly using `--backend vulkan`.

| All five outputs; plain, varied, combined-high (and the other five cases) | Passes | Physical textures | peakBytes |
|---|---:|---:|---:|
| Before revision 3, 1024² | 53 | 14 | 125,830,464 |
| Before revision 3, 2048² | 53 | 14 | 503,317,824 |
| After revision 4, 1024² | 52 | 10 | 92,276,064 |
| After revision 4, 2048² | 52 | 10 | 369,100,128 |

The ceilings stay **64 passes / 536,870,912 bytes**; the 2K compile blocker remains resolved. Logical bytes are 436,207,616 / 1,744,830,464, not the scheduled peak. Fewer duplicate composites and the explicit numeric node-ID order reduce live intermediates; no scheduler/allocator change.

Alignment was checked with two plain-1K neutral probes through the same release Vulkan executor. For C, set both yarn colors white/backing black, yarn roughness 1/backing 0, and expose C through the height output. For D*C, set warp white/weft black, warpGrain=1/weftGrain=0, roughness ramp 0..1/backing 0, and expose scalar-mask-blend(0,D,C). Compare each production baseColor with a separate gradient-map of the same scalar reference (preserving sRGB encoding), and roughness with its linear scalar reference. **Both probes have max RGB byte delta 0 and zero differing pixels**, across the full 1024² image and x=64,y=64,256×256 crop. This checks actual shared coverage/selection edges; normal is a derivative, not an identical mask. It does not qualify arbitrary colors, raw-half precision, browser parity or the full control matrix.

A separate bounded pinch observation renders isolated thread shapes as normalized linear RGBA8. For x=[144,160,176,192,208,224,240], count y=0..127 where weft>warp and weft>13/255: before widths [76,72,50,46,50,72,76], after [80,78,66,64,66,78,80] pixels. The center/near-end ratio rises from 46/76 (~0.605) to 64/80 (0.8), consistent with less hourglass pinch; this is not a frozen geometric-width tolerance. Matched actual crops show color/roughness following the rounded crossing shape and less bulbous relief. Rounded ends and stylized fabric appearance remain review risks, especially outside defaults. **Maintainer visual acceptance remains pending.** No new node is proposed. Narrower/multi-sample selectors and translated-height attempts were rejected as insufficient; the multi-sample version introduced bands.

Review `tmp/mat03-recipe-review-4/after/{plain,varied}-1024/{baseColor,normal,roughness,height}.png`. Plain also has `{baseColor,normal,roughness,height}-crossing-256.png`: native x=64,y=64,width=256,height=256, no rescale or tone adjustment. Corresponding before maps/crops, input snapshots, receipts, probes and resolver remain in the ignored root. No pixels or tooling are committed. Height is intentionally dark in its small original linear range. These are local review observations, not retained human acceptance, PBR/timing qualification, new 2K renders or full structural/sampling/parity evidence.

## Retained revision-4 controls, composition and precision

The fixture guide owns exact caller mappings and the JSON owns defaults/cases. Controls are ordinary caller preparation, not expressions evaluated by Core or a MAT-04 reusable graph API.

| Control | Proposed inclusive range / default |
|---|---|
| warpCount, weftCount | Even integers 4..32 / 8,8; spacing independently 1/count |
| warpWidth, weftWidth | Fraction 0.55..0.9 / 0.7,0.7; physical UV widths width/count |
| bevel | Shoulder control 0.02..0.12 / 0.08; mapped brick bevel 0.19+0.5*bevel |
| relief, underRatio | 0..0.05 / 0.0125; 0.25..0.75 / 0.25 |
| detailAmount | 0..0.1 / 0.08; grain modulates color and roughness, never relocates centers |
| warpSeed, weftSeed | Required u32 0..4294967295 / fixture values 1729,65537, even at detailAmount=0 |
| warpColor, weftColor, backingColor | Linear RGBA [0,1], alpha=1; defaults in plan |
| yarnRoughness, backingRoughness | 0..1 / 0.8,0.95 |
| normalStrength | 0..1 / 0.5 |

W/F retain independent width/spacing support. Revision 4 uses the crowned P/R, normalized A/B and shared C/D construction above. At analytical centers upper height is relief, lower is relief*underRatio, preserving strict order for nonzero relief; sampled/f16 behavior still needs the proposed probes. Relief zero yields zero height and neutral normal without removing color/roughness structure.

Color and roughness share C and D with the height structure, as specified above. Metallic is zero. Profile/selector seeds are explicit zero with variation zero; only grain seeds randomize color/roughness, never W/F/Q or height/normal. Colors affect baseColor only; roughness controls affect roughness only; normal strength affects normal only; relief affects height/normal only. underRatio affects all nonmetallic channels through C/D. Width/count controls keep their independent geometric meanings; changing warp spacing must leave the isolated weft profile unchanged, and vice versa.

All production arithmetic remains f32 with rgba16float storage at every pass. Max synthesis is not exact real arithmetic after storage; no implicit higher precision, CPU renderer or golden reset is allowed. Outputs retain existing sRGB color, linear Scalar bytes and OpenGL tangent normal conventions. Detail and shape share normalized UV coordinates at every size, without hidden resolution-specific topology changes.

## Proposed qualification gates (not frozen)

- Render all plan cases and all five channels at 256², 1024², 2048² and 257×129. Validate defaults, individual endpoints, combined extremes and both independent alternate seeds. Invalid odd counts, missing seeds, non-finite/out-of-range values and unknown controls must fail before request construction. Existing Core node validation remains authoritative.
- For the baseline, independently observe W/F/Q, both pre-max thread heights and final height through the sole executor. For the proposal, freeze test-only observations of Aw/Af, Hw/Hf, Vw/Vf, C and H using the same production helper; no extra public output or alternate executor is proposed. Check crossing center parity, strict upper/lower separation when relief>0, no backing holes along a thread center, independent widths/counts, finite normalized outputs and exact seed repeats. Use analytical center probes at selected UVs in addition to pixel observations; odd rectangles generally do not contain exact crossing centers. Freeze probe coordinates/tolerances before implementation.
- Compare translated evaluations at UV+(1,0), +(0,1), +(1,1), wrapped neighborhoods and production normal replay from raw stored height. Opposite border pixels need not match. Provisional raw-half max-composition tolerance is 1/1024; exact repeat and normal replay are required. This is a proposed test-only observation path, not an implemented command or public raw-half API.
- Same-adapter repeats and loose `.mix`/`.mixpack` equivalence must be byte-exact. Every Native/browser RGBA8 component, including normal and stress cases, must differ by **<=1/255**. Cover pinned SwiftShader and separately identified GT 1030 Vulkan/DX12 versus Chrome; no inherited fabric qualification or general hardware guarantee.
- Default and varied baseColor/height at 256² versus unrounded 4×4 box means of 1024² bytes: per-component mean absolute error <=4/255. Also measure 1024² versus 2048² with 2×2 box means. Normal angular/delivered-byte errors, crossing-loss rate and directional-grain contrast need recorded metrics; their additional acceptance thresholds remain a freeze question. Stress at 32×32 threads and 0.55 width documents aliasing separately; it cannot excuse a default/varied failure. At 257×129 this gives roughly 4 texels per weft pitch: no universal antialiasing guarantee. Fixed brick 2×2 samples and noise/bilinear resampling are not a mip chain.
- Draft ceiling <=64 passes and fixed <=512 MiB descriptor peak at 2048². Revision 4 measures 52 passes / 10 physical textures / 369,100,128 bytes at 2K for all eight cases, resolving the compile blocker without engine changes. Logical retain-all bytes (1,744,830,464) are not the governing peak. Do not raise the limit or start PERF-MAT engine work. Timing remains unqualified: GT 1030 cold 1K <=10 s, median of five warm 1K <=1 s and 2K <=4 s; pinned SwiftShader <=60 s /20 s /80 s. Record parse/compile, cold/warm render, readback/conversion, retained outputs, descriptor/physical peaks and actual adapters before freeze.
- Retain producer-bound channel sheets and fixed camera/light dielectric PBR plane/sphere, 1×/3× tiling and 4× close-ups for plain and varied cases. Human review must separately judge crossing readability, yarn direction, seams, stepped normals and fabric appearance. Numeric parity is not PBR/human acceptance. Preserve reviewed bytes and an actual maintainer decision under the [evidence policy](./evidence-policy.md).
- Preserve ceramic/leather/wood, MAT-01 and MAT-02 regressions, independent Native/browser package consumption and all six required checks. Record exact source, graph/request/plan, package, shader/build and adapter identities. Ordinary outputs go in ignored `tmp/` or CI artifacts; the limited local review receipt does not constitute accepted pixels or complete qualification.

## Decisions before a later freeze PR

1. **Accepted by the maintainer, 2026-10-02:** opaque plain plus one varied plain weave; even counts 4..32, relative widths 0.55..0.9, no twill/other weave. This accepts the family/control domain, not the complete contract.
2. **Direction changed, 2026-10-03:** revision 4 is retained but rejected for floating-capsule appearance. Decide the proposed identity/modes, analytical structure and new crown control/defaults above; visual acceptance is still pending, not inferred from mask probes.
3. Approve exact crossing/seam/normal and directional-detail probes, tolerances, stress labels and quality range after feasibility, but before implementation/freeze.
4. **Budget unchanged:** all eight retained revision-4 cases compile at 2K within 536,870,912 bytes. The node proposal has only a static estimate, requiring later actual measurement; no limit increase or PERF-MAT work is authorized. Timing and complete qualification remain pending.
5. Approve/reject the proposed 18-type/16-kernel catalog and unpublished Rust 0.9.0 / browser 0.9.0-alpha.0 choice. Keep document/package v1 and plan/API v3. No manifest changes or implementation until explicit approval and a later authorized PR.

MAT-03a ends this slice with reviewable drafts only. A later MAT-03a PR resolves feasibility and freezes the contract; only then may minimal implementation seams, material construction and qualification be assigned. No cloth simulation, fiber geometry, arbitrary weave catalog, universal antialiasing, anisotropic shading API, subgraph system, new crate, runtime/shader change, version-manifest change, golden change, Studio work or publication is included.

## Stage B frozen structural probes

The maintainer accepted material recipe revision 2 defaults on 2026-10-04; this does not accept the full material. The additive `structuralProbes` section in [the plan](../fixtures/materials/woven-fabric/qualification-plan.json) owns the exact probes. No Stage A field, threshold, case, size, timing budget, default or retained revision-1 receipt changes. C dielectric PBR/human review and D retained evidence/acceptance remain separate; materialAccepted=false.

### Crossing structure and zero relief

Observe raw binary16 H/C/S by aliasing the real graph's height output, keeping all 21 passes. For plain and varied at 256², 1024², 2048² and 257×129, visit every crossing UV ((i+0.5)/warpCount,(j+0.5)/weftCount) and gap UV (i/warpCount,j/weftCount). Observe the containing pixel floor(UV*size); the height oracle uses its four actual quarter-pixel sample UVs. C=1 and S=1 for even crossing parity, S=0 for odd; gaps H=C=0,S=0.5 are exact. Every crossing tap must have upper>lower, and captured H must exceed every lower tap. The sparse four-tap height oracle has absolute tolerance 1/1024 (two binary16 ULPs below one plus f32 operation error); this does not permit error on exact endpoints or periodic/repeat checks. Check all three instances' identical geometry and same-mode raw fields. Flat-case final height is exactly (0,0,0,1), normal (0.5,0.5,1,1), everywhere at all four sizes. These probes do not remove the accepted residual pinch or prove PBR appearance.

### Public control isolation

At 257×129, plain plus the plan's sixteen one-control variants must repeat exactly. Each listed affected delivered channel must change; all unlisted channels remain byte-identical. Three colors affect baseColor only; two roughness endpoints affect roughness only; normalStrength affects normal only; relief affects height/normal only. Each u32::MAX grain seed affects baseColor/roughness, leaving geometric height/normal unchanged. Counts, widths, bevel, crown and underRatio affect baseColor/roughness/height/normal, leaving metallic unchanged. Raw H/C/S remain finite normalized; nongeometry controls leave them unchanged, and crown/underRatio leave C unchanged. This proves bounded causality on the real graph, not arbitrary control combinations or browser raw-half equivalence.

### Final-height normal replay and periodic boundaries

All twelve plan cases × four sizes upload captured raw final height through the existing production height-to-normal kernel. Offsets (0,0), (1,0), (0,1), (floor(width/2),floor(height/2)) must match the corresponding cyclic shift of captured raw graph normals exactly. CPU code permutes bytes only. Opposite border texels need not be equal. This probes derivative periodicity, not visual seam acceptance or upstream noise periodicity.

### Selected noise inputs and weave translation

Noise inputs are production value-noise v2, scale 4, octaves 2, persistence 0.5, seeds 1729/65537/u32::MAX/0, at all four sizes. Unwrapped input origins (width,0),(0,height),(width+3,height+5) must reproduce the corresponding cyclically indexed raw-half baseline exactly; nonconstant and finite normalized baselines are required. Weave translations cover H/C/S for all twelve cases × four sizes with full-period origins (width,0),(0,height),(width,height), compared exactly against real material aliases. These are selected-input checks, not arbitrary graph or browser periodicity. The reviewer permits test-only origin instrumentation: only a uniquely matched sampling-origin expression changes in a test copy; reserved uniform words are zero at baseline. Production formulas, shader files, ABI, Core lowering and public API remain unchanged. The requested supplemental period-plus-interior weave shift is (width+3,height+5), compared against cyclically indexed graph fields; every frozen case, size, full-period shift and exact tolerance remains required.

### Stress and failure boundary

Dense-thin at its existing 256²/1024²/257×129 sizes checks finite normalized H/C/S, C=0 ⇒ H=0 and S=0.5, identical geometry, and exact normal replay for all four offsets. It remains outside default/varied quality guarantees; no undersampled crossing-center quality promise is added. Any frozen failure stops work and records case, size, field, pixel, actual/expected values and adapter; no tolerance relaxation.


### Stage B implementation and local results

First commit 106546e freezes the probe contract; implementation ed6905fcbc18047e8ef639527923d67c07108f09 follows it. On 2026-10-04, GT 1030 Vulkan (NVIDIA 582.66) and DX12 (32.0.15.8266) passed crossing structure (640 crossings and 640 gaps per backend, including all-instance mode equality), flat (four sizes), control isolation (sixteen variants), normal replay (48 rows × four offsets = 192 exact comparisons), and stress (three sizes, twelve exact replays). Focused Vulkan/DX12 tests used implementation bytes committed as ed6905f (initial focused runs began before that commit); woven_tests.rs SHA-256 160dd5b0c600ee0926b9705ce99f4e9ff54a7333d3a3a0a7c7eeae66eebc2c1b. Node and full smoke runs used that implementation commit; no runtime or recipe changes followed.

The four builder tests pass, including a hash of the frozen additive Stage B section and exact comparison of all remaining Stage A fields. Clippy, test-node weave-pattern and full Vulkan gpu-smoke pass (including the five new ignored probes, existing painted tests, Native/CLI and packaged GPU consumption). The existing clean-source release Native woven matrix passes all 51 rows and four timing gates on Vulkan. Warm medians: plain 1K/2K 183.82/791.17 ms; varied 228.61/901.08 ms. Material SHA-256 remains 95db023744a09224de3344613fe503c1e2187590b48667ef5ac199ae49959757; plan SHA-256 is 392419a6fd210429d513fa0bf1a2e1300313c4299476d421b38b9cf834916523. Ordinary logs and matrix receipts are in ignored tmp/woven-stage-b/; this is not retained Stage D evidence.

**Historical status at dd6acf5:** Stage B was incomplete; selected noise-input periodicity and weave full-period translation were frozen but unimplemented/unrun, pending instrumentation clarification. Existing node periodic tests are not a substitute for those material-specific probes. No frozen probe failed and no tolerance was relaxed. Stage C/D and material acceptance remain pending; this branch is not a Stage B acceptance claim.

### Authorized periodic-input probes

The reviewer resolved the instrumentation question. Both probes now have test-only implementations; results must pass before Stage B can be marked complete. The noise probe reuses the MAT-02 renderer/readback helper for the frozen four parameter sets (including combined-low seed 0). The weave probe compares every mode against raw fields from the real graph at zero origin before testing unwrapped full periods and period-plus-interior shifts. Nonconstant baselines and exact raw-half comparisons prevent vacuous success. Production shader bytes, uniform layout, lowering, recipe defaults and all plan bytes remain unchanged.

```bash
cargo test --locked -p mixture-wgpu --test nodes node_fractal_noise_gpu_woven_periodic_inputs -- --exact --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib node_weave_pattern_gpu_woven_periodic -- --ignored --nocapture
```

Full gpu-smoke discovers both ignored tests; test-node fractal-noise and test-node weave-pattern explicitly include the respective material probe.

### Frozen periodicity failure — Stage B remains incomplete

The authorized implementation was tested from clean commit 47e2dd851303dccdee95b58f5da06cb391645978. On GT 1030, the woven noise probe passed on Vulkan (NVIDIA 582.66) and DX12: 48 exact translated-image comparisons and 16 zero-origin checks per backend, covering the frozen four parameter sets and four sizes.

The weave probe **failed** on Vulkan: case plain, size 257×129, height mode H, unwrapped origin (257,0), pixel (59,0). Actual half words [2149,0,0,15360] differ from graph baseline [2150,0,0,15360]; the scalar values are 0.0001341104507446289 versus 0.0001342296600341797 (difference 0.00000011920928955078125). The frozen tolerance is exact; this is a failure, not a tolerated visual difference. Before the failure, 45 comparisons passed for plain at 256²/1024²/2048² across all modes and origins, and the odd-size zero-origin H check passed. Thus the failure is in a full-period translation after zero-origin graph identity was verified; it is not evidence of full material periodicity.

Per the stop rule, remaining weave cases and DX12 weave were not run, and full gpu-smoke/check were not rerun on this implementation. Test compilation, clippy and xtask compilation passed before the probes. Earlier five-probe Vulkan/DX12 passes and full smoke/check results above belong to the earlier implementation; they do not close this failure. Stage B remains incomplete; C/D and materialAccepted=false are unchanged, and PR #84 stays draft.

Ordinary logs and a source/file-hash receipt are in ignored tmp/woven-stage-b/periodic/ (vulkan-noise.log, dx12-noise.log, vulkan-weave.log, result.json). Production shader bytes, Core lowering, ABI, material defaults, plan bytes and tolerances remain unchanged from dd6acf5. Work stopped without a production repair or relaxed comparison; further disposition requires maintainer review of this failing probe.

### Maintainer amendment — 2026-10-04: exact where exact

The maintainer amended only the weave translation rule in structuralProbes.amendments (2026-10-04-exact-where-exact). The original exact wording and failure receipt remain intact above and in the plan: clean implementation 47e2dd8, recorded at 8771d83, plain 257×129 H at origin (257,0), pixel (59,0), half 2149 versus 2150. The accepted analysis is f32 rounding of fract(1+x) for a non-power-of-two denominator, not a production seam; normal periodic-boundary evidence remains a separate gate. This explanation does not grant any broader numerical relaxation.

- Zero-origin identity remains exact at every size.
- Both axes power-of-two (256²/1024²/2048²): every full-period and period-plus-interior shift remains raw-half bit-exact.
- 257×129 only: each component may differ by at most one adjacent finite binary16 step. Signed values use monotonic ranks (negative bits complemented, nonnegative bits XOR 0x8000); -0/+0 are adjacent, and larger sign/zero crossings fail. Exact gates still compare bits, including signed zero.
- Record differing component/pixel counts, maximum half-step distance and maximum absolute difference per case/size/mode and origin. Stop if any amended gate fails; incomplete coverage is explicitly reported.

Nothing else in the frozen plan changes. A regression removes only this named amendment entry and checks the complete previous plan hash, alongside the existing Stage A and original Stage B guards. Stage B is not complete until the amended probes pass; C/D and materialAccepted=false remain unchanged.

### Amended probe result — 2026-10-04: stopped above one ULP

Clean implementation 8f84e7d7cd435a1760b5ed01531b66f07348af4e (amendment frozen first in 3aba3e6) failed on NVIDIA GeForce GT 1030, Vulkan, NVIDIA 582.66. Command: `cargo test --locked -p mixture-wgpu --lib node_weave_pattern_gpu_woven_periodic -- --ignored --nocapture`. The first violation is plain 257×129 warp-share, origin (257,0), pixel (231,23): actual half [2492,0,0,15360], expected [2496,0,0,15360], **4 steps against the amended maximum 1**. Scalar values are 0.00017499923706054688 versus 0.00017547607421875 (absolute difference 0.000000476837158203125). No further tolerance change or production repair was made.

All 45 plain power-of-two comparisons (three sizes × three modes × five origins) were exact, with zero differing components and maximum 0 ULP. All three plain odd-size zero-origin identities were exact. Odd-size observed statistics follow; differing components and differing pixels have the same counts because only the scalar component changes. Counts sum pixel occurrences across origins, not distinct tile locations.

| Plain 257×129 mode | Origin | Differing components/pixels | Maximum half steps |
|---|---|---:|---:|
| height | (257,0) | 93 | 1 |
| height | (0,129) | 87 | 1 |
| height | (257,129) | 162 | 1 |
| height | (260,134) | 177 | 1 |
| coverage | (257,0) | 153 | 1 |
| coverage | (0,129) | 299 | 1 |
| coverage | (257,129) | 445 | 1 |
| coverage | (260,134) | 443 | 1 |
| warp-share | (257,0), failing image | 123 | **4** |

Mode totals: height 519 differences / max 1 ULP; coverage 1340 / max 1 ULP; warp-share 123 / max 4 ULP through its first translated image only. Each listed translated image has maximum absolute difference 0.00048828125; that maximum need not occur at the maximum-ULP pixel. The already-rendered failing image was scanned completely to retain its statistics, then execution stopped: 56 image comparisons passed and the 57th failed. Remaining warp-share origins, varied/combined and all other cases, and DX12 weave were not run.

The four builder tests (including amendment-only whole-plan protection) and the comparator unit test passed. Full Vulkan gpu-smoke and cargo xtask check were not rerun under the stop rule. Earlier crossing, flat, control-isolation, normal-replay and stress passes on both backends belong to ed6905f; earlier noise-input passes on both backends belong to 47e2dd8. They are unchanged historical evidence, not a pass of this incomplete run. Stage B remains incomplete, C/D pending, materialAccepted=false, PR #84 draft.

Ordinary outputs: ignored tmp/woven-stage-b/amendment/{builder.log,unit.log,weave-vulkan.log,result.json} and vulkan/woven-weave-periodic.json (per-image and cumulative case/size/mode counts, adapter, first failure). These are local review receipts, not retained Stage D evidence. Production WGSL, ABI, Core lowering, material recipe/defaults and all other frozen gates remain unchanged.

### Second maintainer amendment — 2026-10-04: visible warp weight

The second structuralProbes.amendments entry preserves the original rule, first amendment and failure at clean 8f84e7d / record 6991f46 (plain 257×129 S, origin (257,0), pixel (231,23), half 2492 vs 2496). Before any rerun, freeze only the odd-size nonzero-origin warp-share gate as follows. Zero-origin stays exact at all sizes; power-of-two sizes stay exact in **all three raw modes**, including S. Odd-size H/C retain the one-half-step gate.

Let S_i,C_i be captured half values from the share and coverage instances at the same pixel and origin, and P_i=f32(S_i*C_i). Define u(x) as the larger adjacent binary16 spacing at nonnegative stored x, with u(0)=2^-24. For i=1,2, set eSi=u(S_i)/2, eCi=u(C_i)/2 and E_i=C_i*eSi+S_i*eCi+eSi*eCi. Freeze:

**|f64(P_1)-f64(P_2)| ≤ B = E_1+E_2+min(1,max(S_1+eS1,S_2+eS2))*max(u(C_1),u(C_2)).**

Each E_i follows by expanding the product under independent half-storage rounding intervals. The final term is one already-approved coverage step weighted by the largest possible share in those intervals. The multiplier is exactly 1; no term is fitted to the four-step result. This is an acceptance envelope, not proof that all coordinate perturbations satisfy it: independent changes of the geometric share are not bounded by coverage error alone, and any observed excess still fails. Nonnegative half products are exactly representable in f32 (at most 22 significant bits, minimum exponent -48); compare converted products and compute B in f64 without an extra arithmetic allowance. All operands must remain finite in [0,1].

Raw S differences remain reported but are not gated for odd translated images. Record maximum product difference and difference/bound with their pixels, plus raw S count/max steps. Zero difference/zero bound gives ratio 0; positive difference/zero bound fails. A regression removes only the second entry and verifies the complete preceding plan hash, then retains the original guards. No case, size, recipe default or other gate changes; Stage B completion remains conditional on results.

### Second-amendment result — 2026-10-04: visible-weight bound fails

Clean implementation 4af7e46b37c49baf73fc27948c571cd80cf7d104 (docs/plan freeze first in 73bba64), NVIDIA GeForce GT 1030 Vulkan, NVIDIA 582.66: the same focused weave command exits 101. Plain 257×129 warp-share, origin (257,0), pixel (231,23) has S half 2492 versus 2496, but **C half is 15360 (1.0) on both sides**. Thus this pixel does not support the contextual claim that tiny coverage causes the discrepancy: coverage weighting leaves it unchanged. Small S is not small C.

P difference = 4.76837158203125e-7; frozen B = 4.618195816874504e-7; **difference/B = 1.032518275775145**, the maximum ratio in the fully scanned failing image. This exceeds the frozen envelope. No term, multiplier or tolerance was changed after measurement.

All 45 plain power-of-two comparisons remain raw-half exact in all modes. All three odd-size zero-origin identities remain exact. Odd-size H has 519 differing pixel occurrences across four shifts, maximum 1 half step; C has 1340, maximum 1. Their per-origin counts match the first-amendment table above. The first translated odd-size S image has 123 differing pixels, maximum 4 raw half steps. Its maximum absolute product difference is 0.00048828125 at (231,20), with B=0.0016491413116455078 and ratio 0.29608211652450483; maximum absolute difference and maximum ratio occur at different pixels. Only this one translated S image was measured; remaining origins/cases and DX12 weave are unrun. 56 comparisons passed, the 57th failed.

The four builder tests pass, including the second-amendment-only full-plan guard and unchanged first-amendment/original guards. The analytical product comparator unit test passes after correcting its hand-calculated expected constant (the frozen formula was unchanged); the initial unit expectation failed before GPU execution. Full Vulkan gpu-smoke and cargo xtask check were not rerun under the stop rule. Earlier five structural probes remain prior passes on Vulkan/DX12 (ed6905f), and noise periodicity remains a prior pass on both (47e2dd8); these are not new reruns. Stage B is incomplete; PR #84 stays draft, C/D pending, materialAccepted=false.

Ordinary receipts: ignored tmp/woven-stage-b/amendment2/{builder.log,unit.log,weave-vulkan.log,result.json} and vulkan/woven-weave-periodic.json. The latter records each measured case/size/mode/origin, raw counts, product statistics and both coverage samples. Production files, ABI, Core, recipe defaults and other plan fields are unchanged. This is local review evidence, not Stage D acceptance.

### Third maintainer amendment — 2026-10-04: odd translated share is observation-only

Before rerunning the weave probe, the maintainer freezes structuralProbes.amendments entry 3. Original rules and amendments 1/2 remain unchanged history, including clean 4af7e46 / record f9e0a51: plain 257×129 S at origin (257,0), pixel (231,23), half 2492 versus 2496, C=1 on both sides, visible-weight ratio 1.032518275775145. The earlier tiny-coverage explanation was wrong. The accepted corrected analysis identifies the steep depth/visibility selector D (transition width 2*bevel*(1-underRatio)) amplifying f32 rounding of fract(1+x).

Gates stay exact for zero-origin identity at all sizes/modes and for every power-of-two translation in **all three raw modes including S**. Odd 257×129 H/C remain within one adjacent half step. Only nonzero-origin odd-size S and P become **observation-only: outside the periodicity quality guarantee, never a pass**. Report raw S difference count/max steps, maximum absolute P difference and maximum ratio to the unchanged amendment-2 envelope, with their pixels and values, for every case/size/mode/origin. Finite normalized fields and same-origin coverage still have gates. This limitation is separate from material acceptance and resembles the explicit stress quality scope.

A regression removes only amendment 3 and restores the full preceding plan hash, then checks the earlier guards. All cases, sizes, recipe revision 2 defaults, thresholds and budgets are unchanged. Stage B can complete only when its remaining gated rules pass; the odd-share limitation must remain visible in every result summary.

### Third-amendment results and packaged-test repair

Stage B gated structural probes are complete within the recorded GT 1030 Vulkan/DX12 scope; **odd-size translated S/P remain an ungated measured limitation, not a pass**. C PBR/human review and D retained evidence/acceptance remain separate PRs; materialAccepted=false. Clean implementation 30931b5 (third amendment frozen first at 69489d4) completed the weave matrix on both backends via `cargo test --release --locked -p mixture-wgpu --lib node_weave_pattern_gpu_woven_periodic -- --ignored --nocapture`: 672 gated comparisons pass and 48 observation-only rows per backend, covering all twelve cases, four sizes and all three modes. Power-of-two translations and every zero-origin identity remain exact; odd H/C remain within one half step. Vulkan reports NVIDIA 582.66; DX12 reports GT 1030 with empty driverInfo.

The other five structural probe passes on both backends belong to ed6905f (crossings, flat, controls, normal replay, stress); noise periodicity passes on both belong to 47e2dd8. Those implementations are unchanged apart from relocating shared test readback. Full gpu-smoke and check remain mandatory before PR readiness; their final run results are recorded in the PR.

Odd 257×129 S/P observations below aggregate four translated images per case. Counts are pixel occurrences across shifts, not distinct tile positions. Maxima are independently aggregated; raw step and product-ratio maxima need not share a pixel. These rows are **outside the periodicity quality guarantee**.

| Case | Vulkan differing pixels | DX12 differing pixels | Max raw steps (both) | Max absolute ΔP (both) | Max ratio (both) |
|---|---:|---:|---:|---:|---:|
| plain | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| varied | 528 | 528 | 5 | 0.00048828125 | 1.5597867479055598 |
| no-detail | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| flat | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| warp-seed | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| weft-seed | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| combined-low | 181 | 180 | 5 | 0.00048828125 | 3.7911884487226954 |
| combined-high | 2964 | 2963 | 27 | 0.00048828125 | 6.802103515084418 |
| neutral-normal | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| constant-low | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| constant-high | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| max-normal | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |

The package-check failure was reproduced: isolated lib tests could not read tests/support/periodic_scalar_readback.rs, excluded from the crate package. Commit a7279b5 moves the unchanged helper to src/periodic_scalar_readback.rs; executor includes it only under cfg(test), and integration tests reuse that file via a private path module. No public API or duplicate implementation is added. The first repaired package run compiled successfully but its source-identity guard rejected concurrent source edits; a stable rerun passed packaged Rust and CLI CPU consumption. Four builder tests pass, including the third-amendment-only whole-plan guard.

Receipts are ignored tmp/woven-stage-b/amendment3/{package-before.log,package-after.log,package-stable.log,builder.log,weave-vulkan.log,weave-dx12.log,summary.json} and {vulkan,dx12}/woven-weave-periodic.json. Each observation has passed:null, an explicit quality scope, raw counts and P difference/ratio with pixels. Production WGSL, ABI, Core, defaults, versions and goldens are unchanged. These local results are not Stage D retained acceptance.

### Fourth maintainer amendment — 2026-10-04: software size scope

Frozen before implementation: structuralProbes.amendments entry 4 cites CI run 37185577047 at 671cf76, artifact swiftshader-material-evidence / woven-weave-periodic.json. SwiftShader Device (LLVM 10.0.0) completed 582/720 rows with zero failures before cancellation at constant-low 2048² (07:23:46–08:07:53). This partial receipt is **not a completed software qualification**. Earlier rules and all three amendments remain history.

Select from the **actual reported adapter deviceType**: Cpu uses 256², 1024², 257×129; all other types retain 256², 1024², 2048², 257×129. Environment variables alone cannot select the reduced scope. Preserve the original per-probe scopes: crossing, flat, normal replay, selected noise and weave translation use the full backend set; isolation remains odd-size only, stress retains its original three sizes. These narrower scopes are not new software omissions. Hardware full-matrix probes must fail if the material's 2048² size is missing or not executed.

Each probe receipt states its frozen and executed size sets and records each omitted 2048² row as **notRunOnSoftware**, passed:null, with amendment id 2026-10-04-software-size-scope. Never count omissions as passes. Hardware evidence still requires 2048²; software passes never qualify hardware pixels. No comparison, tolerance, other amendment, Stage A gate or workflow timeout changes.

Runtime estimate (not a measurement): 2048² contributes 78.522% of a four-size pixel sum. The first 582 translation rows represent 788,155,143 compared pixels (nine complete cases plus 15/15/12 rows at 256/1024/2048). The reduced complete matrix represents 206,507,700 pixels, ratio 0.262014. Charging the entire interrupted 44m07s interval to that work gives about 11.56 minutes; adding main's stated 9–15 minutes gives a rough **20.56–26.56 minute job target**, below the unchanged 45-minute limit. Compilation, dispatch, readback and other probes are not perfectly pixel-linear; only CI can verify software runtime and completion. C/D and materialAccepted=false remain unchanged.

# MAT-02 — layered painted metal (draft contract)

English | [简体中文](./mat-02-layered-weathering.zh-CN.md)

Status: preparatory design, 2026-09-23. MAT-01 remains the current implementation/acceptance increment. This document prepares MAT-02 under the [material roadmap](../ROADMAP.md); it does not accept a new catalog identity, start its runtime implementation, close MAT-01 or publish a package. Resolve the decisions and freeze the executable qualification plan before implementing the node.

## Material and missing operation

Produce a seamless painted-metal family with intact paint, exposed metal, and rust concentrated around chipped paint boundaries. One shared set of masks must drive baseColor, metallic, roughness and final height/normal. The visual target includes visibly raised paint, recessed exposed substrate, independently adjustable chipped-edge width and rust amount, and reproducible seeded surface variation. It is not physical corrosion, mesh-aware edge wear or a baking system.

The current catalog already supplies `fractal-noise@2` value noise, `levels@1`, masked color `blend@1`, `scalar-mask-blend@1`, constants and height-to-normal. These can remap, invert, multiply and compose normalized fields without admitting a general arithmetic library: multiplication of a and m is `scalar-mask-blend(a=0,b=a,mask=m)`. Derive normals from the final height instead of adding a normal-composition node.

The missing operation is a neighborhood minimum/maximum: existing pointwise remapping cannot erode a mask by a chosen spatial distance. Warping relocates samples and does not supply an erosion/dilation contract. A chipped-edge band must retain the original mask while subtracting its eroded interior; adjusting noise thresholds alone is not independent edge-width control.

## Proposed scalar-morphology@1

One required `in: Scalar` input and one `value: Scalar` output. No random state. Proposed parameters:

| Parameter | Type / range | Default |
|---|---|---|
| `operation` | Enum `erode`, `dilate` | `erode` |
| `axis` | Enum `x`, `y` | `x` |
| `radius` | Integer 0..16, in output texels | 2 |

At integer pixel p, load the red components at every integer offset k in [-radius,+radius] along the selected axis. Wrap coordinates modulo that axis dimension, including dimensions smaller than the kernel. Clamp finite samples to [0,1], then take their minimum for erosion or maximum for dilation. Include the center. Store `[value,0,0,1]` in rgba16float. Radius zero is exact identity for normalized inputs. There is no interpolation, averaging, hidden texture sampler or CPU pixel executor.

An x instance followed by a y instance with equal radius produces a square Chebyshev neighborhood. It is not a Euclidean disk, distance transform or blur. Opposite axis order is equivalent for equal rectangular kernels. Thin features may disappear under erosion; that is specified behavior. Single-pixel axes and repeated wrapped samples remain valid. Finite/range checks and exact analytical probes must cover these cases.

Radius is deliberately a discrete texel count. The fixture consumer maps a public reference width (0..8 texels at 1024) to each axis using integer nearest rounding, `floor((referenceWidth * axisLength + 512) / 1024)`, with a minimum of one for nonzero reference width. The planned matrix is bounded to axes <=2048, so mapped radii remain <=16. This mapping belongs to the explicit fixture request builder, not hidden node semantics. Values outside the node range are rejected, never clamped. A square image has equal radii; rectangular outputs use independent x/y radii to preserve normalized axis width. Resolution rounding remains an explicit limitation, tested at small/odd sizes.

Core owns the contract, validation, typed kernel parameters and hashing. wgpu owns the sole implementation and 16-byte u32 uniform `[operation,axis,radius,0]` with documented discriminants. Each instance is one 8x8 compute dispatch and one existing rgba16float intermediate; two-dimensional morphology uses two ordinary graph nodes. No new crate, implicit resource lookup, additional executor or multi-pass hidden node is proposed.

## Proposed scalar-subtract@1

Required inputs `a: Scalar`, `b: Scalar`; output `value: Scalar`, no parameters or random state. For finite samples, clamp each input to [0,1], then return `max(a-b,0)` as `[value,0,0,1]` in rgba16float. Equal inputs produce exact zero; b=0 preserves normalized a; a<=b produces zero. This is saturating subtraction, not signed arithmetic or absolute difference. Core owns the contract/lowering and wgpu owns one pointwise WGSL kernel, using the existing empty-parameter uniform convention.

This second candidate is justified by the inner edge band W-I. Masked interpolation with a zero endpoint computes W*(1-I), which incorrectly gives 0.25 for W=I=0.5 even at zero erosion radius. In exact arithmetic, three existing nodes can synthesize subtraction: invert b, mix a with that inverse at weight 0.5, then remap [0.5,1] to [0,1]. Their intermediate half-float storage can erase the band: for exactly representable a=0.5 and b=0.499755859375, rounding the inverse gives 0.5 and the composed result is zero, while a single subtraction retains 0.000244140625. This arithmetic counterexample motivates the single-pass contract; it is not GPU qualification. Reproduce it through the public GPU path before catalog admission and retain the compared graph. Tests must include equal fractional fields, endpoints, ordering, exact half-representable differences and periodic inputs. Do not replace subtraction with that product or add unrelated arithmetic modes. Neither proposed identity is admitted until MAT-02a review.

## Composition and controls

Use a low-frequency seeded value-noise field remapped to the exposure mask W. Erode W along x/y to obtain I. The chipped-edge band is `B=max(W-I,0)`, using the proposed saturating subtraction. At zero radius B is exactly zero, including fractional W. Rust coverage R is B modulated by explicit rust amount and, if visually necessary, a second independently seeded value-noise field. Keep `0<=R<=W<=1` by construction. Full-rust coverage of the exposed area is a separate preset/explicit blend toward W, not an accidental change to the edge mask.

Base color first mixes paint/substrate with W, then rust with R. Metallic is `W*(1-R)` for the selected dielectric paint/rust and metallic substrate. Roughness uses the same masks and fixed, distinct paint/substrate/rust endpoints. Height places paint above substrate and optionally raises rust within R; normal always derives from this final height. Color-only controls must not affect height, normal, metallic or roughness. Rust color must not change coverage. Surface-detail amplitude must not move the macro exposure boundary.

Public conceptual controls cover exposure amount, exposure scale, explicit macro/detail seeds, chipped-edge width, rust amount, paint/substrate/rust colors, their roughness values, paint thickness, rust relief and normal strength. They are caller-side parameter mappings until MAT-04. Prefer the existing catalog except for the demonstrated neighborhood and saturating-subtraction gaps. AO, curvature, blur, additional Scalar operators, material-layer types and new normal blending require a separate demonstrated failure before admission.

## Qualification to freeze before implementation

The machine-readable plan must name exact presets, seeds, endpoints, sample locations, tolerances and request mappings; the following requirements must not be weakened after viewing failures.

- Presets: default chipped paint, intact coating (W=0), exposed clean substrate (W=1,R=0), rusted exposed substrate (W=1,R=1), edge rust, changed macro seed and changed detail seed. Render all five material channels at 256², 1024², 2048² and 257x129. Endpoint presets must analytically reproduce their constant material values and neutral normals when detail is disabled.
- Node probes: radius 0/1/16, both operations/axes, 1x1/1x17/17x1 and odd rectangles, constant fields, impulses, thin lines, centered rectangles and features crossing the periodic boundary. Assert exact expected interior values and affected pixel sets, min<=input<=max, monotonicity with radius, x/y composition equivalence and translated periodic equivalence. Independent scalar expected-value assertions in tests are allowed; no CPU rendering API or fallback is introduced.
- Causality: increasing edge width expands B without changing W; zero width produces exactly zero B even for fractional W; rust amount changes R without changing W/I; seed changes are deterministic; amount endpoints are exact; color changes remain in baseColor. Check `R<=W`, metallic and roughness relationships, height ordering and normal consistency on the actual material, not only separate toy graphs.
- Repeated same-adapter pixels and loose/package roundtrips are exact. Every Native/browser RGBA8 channel comparison keeps maximum component error <=1, including normal. Cover pinned software and the explicitly recorded GT 1030 Vulkan/DX12 versus Chrome paths; investigate failures without generalizing existing noise qualification.
- Freeze default 256² versus downsampled 1024² height/baseColor mean-error limits before shader work. Start with the MAT-01 4/255 target, then establish feasibility from existing-node inputs and the specified morphology mapping. Record high-frequency and subpixel-width stress separately; never relabel a default failure as stress. The integer radius mapping is not a promise of exact resolution independence.
- Initial resource ceiling: <=24 passes and <=512 MiB descriptor peak at 2048², with existing global safety limits unchanged. Target GT 1030 cold 1K <=10 s, five-warm median 1K <=1 s and 2K <=4 s; pinned software <=60 s / 20 s / 80 s respectively. These provisional design budgets must be frozen in the plan before implementation. An actual graph exceeding them triggers PERF-MAT measurements and a separate optimization slice, not a larger budget. Record the compiler's retain-all estimate before choosing an optimization.
- Preserve fixed-light/camera PBR views with metallic shading, plane/sphere, close-ups and repeated tiling. Extend the consumer preview to read the metallic map; dielectric-only MAT-01 views cannot qualify this material. Retain real human decisions and the reviewed bytes. Keep original and migrated material regressions, public package consumption, all six required CI checks and paired documentation.

## Compatibility decisions and implementation order

The two proposed nodes fit `.mix v1`, API/plan schema 2 and package v1 without changing existing node semantics. New Rust kernel variants affect exhaustive matches; propose the next unpublished minor candidate (0.7), but select exact source versions only after MAT-01 integration state is rechecked. Old runtimes must reject the new identities explicitly. Do not auto-migrate documents or update old goldens.

| Item | Deliverable / prerequisite |
|---|---|
| MAT-02a | After MAT-01 exit, accept the bounded contract, explicit catalog/version decision, concrete graph/control mapping and frozen qualification plan. Resolve provisional budgets from the declared workload, without claiming measured results. |
| MAT-02b | Two reviewed Core contracts, exactly one WGSL kernel per identity, ABIs, focused fixtures, diagnostics, periodic/endpoint tests and public node guides; split the two seams into independent PRs. No material acceptance claim from compilation alone. |
| MAT-02c | Painted-metal graph, independently mapped controls, five-channel causality tests and metallic-capable consumer PBR evidence. Measure the real graph; start PERF-MAT separately if the frozen budget fails. |
| MAT-02d | Independent Native/browser exact candidate, package roundtrips, old/new material matrices, retained machine/human evidence, six checks and integration record. Publication remains separate. |

The preparation here does not reorder stages. MAT-03 woven surfaces and MAT-04 reusable recipes remain part of the full roadmap; neither is replaced by this material or by fixture-only parameter mappings.

# Woven fabric — MAT-03a draft design

English | [简体中文](./README.zh-CN.md)

The [contract](../../../docs/mat-03-woven-surfaces.md), [graph design](./graph-design.json) and [qualification plan](./qualification-plan.json) are **draft, not frozen**, with `runtimeImplemented: false`. No executable material, request builder, woven test command, render receipt or accepted pixels exist in this slice. Existing node implementations are available; that does not mean this material is implemented. `$` values are caller-side placeholders, not `.mix` syntax or a runtime expression language.

## Construction and caller mapping

The recipe lists all 40 node instances in dependency order, exact type/version identities, inputs and five output aliases. It uses only current catalog nodes. The contract explains why direct checker/brick/morphology attempts are insufficient and how existing composition can address them; it does not prove rendered quality or justify a new node yet.

Merge each case over plan defaults, reject unknown/non-finite/out-of-range controls using the contract table, and require even axis counts. Resolve the following before constructing a future ordinary `.mix v1` document:

| Placeholder | Caller mapping |
|---|---|
| warpGap, weftGap | 1-warpWidth, 1-weftWidth (each 0.1..0.45) |
| halfWarpCount | warpCount/2 (integer) |
| crossingOffsetX | 0.5/warpCount |
| detailMin | 1-detailAmount |
| underHeight | relief*underRatio |
| Other placeholders | Identically named validated controls; explicit zero seeds for unvaried profile/selector nodes |

Warp columns and weft rows determine independent spacing. Their gap controls determine independent relative widths; physical UV width equals width/count. Each profile combines a brick with its half-tile axial shift using saturating subtraction, uniform half mixing and levels to synthesize max. Q is a shifted staggered Scalar brick field; even parity selects warp-over. Each noise field uses value v2, scale 2, one octave, integer directional transform 4×1; weft rotates by one quarter turn. All production pixels must eventually use wgpu only.

The graph observes W/F/Q, separate thread heights and coverage via inspection aliases. Both color layer orders use the same profiles and Q. Roughness uses coverage, metallic uses zero, normal consumes final height. Half storage happens at every intermediate; aliases and mathematical equations are not an alternative executor.

## Draft matrix and budget

Eight base cases × four sizes × five channels give 160 channel comparisons per Native/browser pairing, before independent control sweeps and stress. Sweeps change one named control over the plain defaults; each color replacement changes only that color. Stress is a separate dense/thin case at three sizes (15 further channel comparisons). Combined-high stays a required finite/structural/parity case; only plain and varied have the draft 4/255 downsample guarantee. Stress must never replace either quality case.

All cases require exact repeats, package equivalence and <=1/255 cross-runtime components. Flat relief must produce zero height and neutral normal; metallic is zero in every case. The contract also requires raw-height normal replay, periodic translation probes, independent axis/seed causality and PBR/human review. The plan's freeze blockers are unresolved; proposed assertions are not implemented tooling.

The static retain-all 2K texture estimate is 40×2048×2048×8 = 1,342,177,280 bytes. It is not plan-v3 physical peak and excludes buffers/readback/driver overhead. Proposed limits are 64 passes, 512 MiB descriptor peak and the plan's separate hardware/software timing targets. Obtain real compiler/resource/timing measurements before freezing these targets; do not start speculative optimization.

## Evidence and next review

Read the contract's five maintainer decisions before a later freeze PR. First build/render only an existing-node feasibility graph through public Native/browser consumers, preserving source/request/build/adapter identity and failures. No new-node proposal is justified merely by the recipe's size. Any demonstrated gap requires a separate minimal identity/ABI/version review. Qualification builds use `target/native-consumer`; ordinary output uses ignored `tmp/` or CI artifacts. Retain accepted images and human decisions under the [evidence policy](../../../docs/evidence-policy.md). No existing golden, manifest, runtime or Studio content changes here.

# Woven fabric — MAT-03a recipe revision 4 (draft)

English | [简体中文](./README.zh-CN.md)

The [contract](../../../docs/mat-03-woven-surfaces.md), [graph design](./graph-design.json) and [plan](./qualification-plan.json) define this draft. Decision 1 accepts the family/control domain, not the complete contract. Historical revision observations remain in the contract and plan.

## Recipe revision 4 — shared structure and reduced pinch (2026-10-03)

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

## Caller mapping and reproduction

Merge case controls over defaults and validate against contract ranges; reject odd counts, unknown, non-finite or out-of-range controls. `$` values are caller placeholders, not Core expressions. Resolve warpGap=1-warpWidth, weftGap=1-weftWidth, halfWarpCount=warpCount/2, crossingOffsetX=0.5/warpCount, profileBevel=0.19+0.5*bevel, warpDark/weftDark=respective color RGB*(1-4*detailAmount) with alpha=1, roughnessMin=yarnRoughness*(1-2*detailAmount). Other placeholders directly use same-named controls (including underRatio); fixed seeds are zero. Old detailMin/underHeight mappings are no longer used.

Preserve numeric node IDs; array order does not change Core's lexical scheduling. Emit ordinary `.mix v1`: copy node id/type/version/resolved parameters, convert input source.port references into from/to edges, append material-output@1 (id=material), and wire outputs to corresponding channels. Exclude design fields/placeholders. Inspection aliases warpShape/weftShape mean normalized A/B, not relief-scaled heights; coverage=C, surfaceOrder=D. The historical baseline uses 793f73e's graph/defaults/mappings.

```text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <case.mix> --json
mixture inspect <case.mix> --plan --size <1024|2048> --output baseColor,normal,roughness,metallic,height --json
mixture render <plain|varied.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
```

Here mixture is `target/native-consumer/release/mixture.exe`. Use a fresh ignored directory; retain JSON, stderr, exit codes and input snapshots separately. The two neutral probes override resolved node constants/outputs as described above; exact commands/input hashes are in the plan. No new repository tooling is required.

## Remaining freeze gates

Complete all structural/independent-control/periodic/odd-size probes, raw-half normal replay, downsampling/stress, Native/browser <=1/255, exact repeats/package equivalence, timing, material regressions, public package consumption, PBR/human acceptance, six checks and catalog/version review. Retain human-reviewed bytes under the [evidence policy](../../../docs/evidence-policy.md) before claiming acceptance; local probes do not replace these gates.

# Woven fabric — MAT-03a recipe revision 3 (draft)

English | [简体中文](./README.zh-CN.md)

The [contract](../../../docs/mat-03-woven-surfaces.md), [graph design](./graph-design.json) and [qualification plan](./qualification-plan.json) remain **draft, frozen: false, runtimeImplemented: false**. Decision 1 was accepted on 2026-10-02: opaque plain weave plus one varied plain weave, even counts 4..32 and widths 0.55..0.9 of spacing; no twill or other weave. The full contract is not frozen. The maintainer selected existing-node recipe changes for memory and visual improvements; no new node or PERF-MAT engine work is authorized.

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

## Recipe and caller mapping

Revision 3 has 53 existing-node instances and five outputs. Numeric prefixes on node IDs establish a legal order for Core's existing lexical ready-node scheduler. Reordering the JSON array alone would not change scheduling. The specific recipe order reduces physical slots from 16 to 14 while adding grain composition; this is not an engine optimizer. Preserve IDs and references when reproducing these measurements.

Merge each case over plan defaults, validate against the contract's unchanged control ranges and reject odd counts, unknown/non-finite/out-of-range controls. `$` values are caller placeholders, not runtime expressions. Resolve:

| Placeholder | Mapping |
|---|---|
| warpGap, weftGap | 1-warpWidth, 1-weftWidth |
| halfWarpCount, crossingOffsetX | warpCount/2, 0.5/warpCount |
| profileBevel | 0.19+0.5*bevel; public bevel 0.02..0.12 maps to brick 0.20..0.25 |
| underHeight | relief*underRatio |
| warpDark, weftDark | Respective linear color RGB × (1-4*detailAmount), alpha=1 |
| roughnessMin | yarnRoughness*(1-2*detailAmount) |
| Other placeholders | Same-named controls; fixed zero seeds remain explicit on unvaried profiles/selector |

Both axis profiles retain shifted-brick/max construction. Revision 3 multiplies height by the narrower crown described above; it is not an exact circle and retains a small flat center. Counts determine independent spacing; widths are fractions of spacing. Q retains alternating over/under selection.

Default detailAmount is now 0.08 (previously 0.03); its range stays 0..0.1. Each explicitly seeded value-v2 noise uses scale 4, octaves 2, persistence 0.5, then an 8×1 transform; weft uses quarterTurns=1. Levels remaps 0.3..0.7 to 0..1. Each grain drives a gradient-map between the yarn's dark and full color; revision-3 height is independent of grain. Q-selected grain drives yarn roughness between roughnessMin and yarnRoughness; coverage mixes over backing roughness. At detailAmount=0, colors and yarn roughness become unmodulated, while the required seeds remain present. Color/roughness controls stay channel-isolated, and seeds do not move W/F/Q. Full public causality tests remain pending.

The graph's inspection aliases name W/F/Q, both pre-max heights, coverage and both grains. Normal uses final stored height; metallic remains zero. Every production pixel still comes from wgpu, with f32 arithmetic and half storage at every pass.

## Historical source-bound revision 1 → 2 review

Both runs use clean engine main `e73e2b99c85987551d11db78eb90dfbf0564100a` and the same release CLI built into `target/native-consumer`. The after recipe/plan were temporary candidate inputs, bound by hashes in `recipeIteration`; this subsequently annotated plan is not the measured input. This describes the historical revision-2 graph; current revision-3 binding is above. Historical `reviewMeasurements` remain bound to revision 1 and its older source, not revision 2.

All eight plan cases validate with zero diagnostics before and after. For every case the full five-channel inspection gives:

| Recipe / size | Passes | Physical textures | peakBytes | Result |
|---|---:|---:|---:|---|
| Before / 1024² | 40 | 16 | 142,607,312 | Pass |
| Before / 2048² | — | — | 570,426,320 observed | MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED, exit 2; plan null |
| After / 1024² | 45 | 14 | 125,830,240 | Pass |
| After / 2048² | 45 | 14 | 503,317,600 | Pass |

The unchanged limit is **536,870,912 bytes**, and the pass ceiling remains 64. The compile-time 2K blocker is resolved for these requests without raising limits or modifying runtime. After logical texture bytes are 377,487,360 (1K) and 1,509,949,440 (2K); retain-all arithmetic does not govern the peak. No baseline 2K render could run after compile rejection.

Plain and varied were rendered at 1024² and 2048² on **NVIDIA GeForce GT 1030 / Vulkan / NVIDIA 582.66**, all five channels, all four renders successful. Agent inspection observes broader curved shoulders and default longitudinal grain in color and roughness. Some flat crown remains, particularly in wide weft; exact circular cross-sections and final fabric quality are not established. **Maintainer visual acceptance remains pending.** No PBR preview, browser parity, DX12/software matrix, timing qualification or full structural/stress gate is claimed.

## Reproduction and images

Resolve the chosen revision's recipe/defaults using the mapping above. Emit `{version:1,nodes,edges}`: copy node id/type/version/resolved parameters, move each `inputs` reference `source.port` into `from:{nodeId:source,portId:port}` → `to:{nodeId:target,portId:inputName}` edges, append `material-output@1` with ID `material`, and wire all five output aliases to matching channels. Do not retain design-only fields or `$` placeholders in `.mix`. For the historical revision-1 recipe, use main `e73e2b9` and its original mappings. Use a fresh ignored directory and preserve stdout JSON separately from stderr and process exits.

```text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <case.mix> --json
mixture inspect <case.mix> --plan --size <1024|2048> --output baseColor,normal,roughness,metallic,height --json
mixture render <plain|varied.mix> --size <1024|2048> --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
```

Here `mixture` is `target/native-consumer/release/mixture.exe`. The temporary resolver/search helpers are in ignored `tmp/mat03-recipe-review/`; none is committed or a new repository command. Review the original channel PNGs:

- `tmp/mat03-recipe-review/after/plain-1024/{baseColor,normal,roughness,height}.png`
- `tmp/mat03-recipe-review/after/varied-1024/{baseColor,normal,roughness,height}.png`
- Matching `plain-2048` / `varied-2048` folders contain the four channels plus metallic; baseline 1K images/reports are under `baseline/`.

Each run directory retains input snapshots, resolved graphs, raw reports, receipt and PNG hashes. Earlier rejected candidate schedules also remain in the ignored root. No preview tool was changed; raw channel PNGs are the review artifact. These are local review files, not durable human-accepted evidence; hashes do not guarantee future availability. Retain reviewed bytes and an actual decision under the [evidence policy](../../../docs/evidence-policy.md) before claiming acceptance.

## Remaining freeze gates

The plan still covers eight cases × four sizes × five channels (160 comparisons per Native/browser pairing), separate independent sweeps, and dense/thin stress at three sizes. Only plain/varied have the proposed 4/255 downsample guarantee; stress cannot excuse their failure. Exact repeats/package equivalence, <=1/255 cross-runtime components including normals, zero-relief/metallic checks, crossing/axis/seed causality, raw-height normal replay, periodic/odd-size probes, downsampling and stress remain to be qualified. Complete timing, material regressions, public package consumption, PBR/human review, six checks and catalog/version review before a later freeze. The family/control-domain decision is accepted; the material and contract are not.

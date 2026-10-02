# Woven fabric — MAT-03a recipe revision 2 (draft)

English | [简体中文](./README.zh-CN.md)

The [contract](../../../docs/mat-03-woven-surfaces.md), [graph design](./graph-design.json) and [qualification plan](./qualification-plan.json) remain **draft, frozen: false, runtimeImplemented: false**. Decision 1 was accepted on 2026-10-02: opaque plain weave plus one varied plain weave, even counts 4..32 and widths 0.55..0.9 of spacing; no twill or other weave. The full contract is not frozen. The maintainer selected existing-node recipe changes for memory and visual improvements; no new node or PERF-MAT engine work is authorized.

## Recipe and caller mapping

Revision 2 has 45 existing-node instances and five outputs. Numeric prefixes on node IDs establish a legal order for Core's existing lexical ready-node scheduler. Reordering the JSON array alone would not change scheduling. The specific recipe order reduces physical slots from 16 to 14 while adding grain composition; this is not an engine optimizer. Preserve IDs and references when reproducing these measurements.

Merge each case over plan defaults, validate against the contract's unchanged control ranges and reject odd counts, unknown/non-finite/out-of-range controls. `$` values are caller placeholders, not runtime expressions. Resolve:

| Placeholder | Mapping |
|---|---|
| warpGap, weftGap | 1-warpWidth, 1-weftWidth |
| halfWarpCount, crossingOffsetX | warpCount/2, 0.5/warpCount |
| profileBevel | 0.19+0.5*bevel; public bevel 0.02..0.12 maps to brick 0.20..0.25 |
| detailMin, underHeight | 1-detailAmount, relief*underRatio |
| warpDark, weftDark | Respective linear color RGB × (1-4*detailAmount), alpha=1 |
| roughnessMin | yarnRoughness*(1-2*detailAmount) |
| Other placeholders | Same-named controls; fixed zero seeds remain explicit on unvaried profiles/selector |

Both axis profiles retain shifted-brick/max construction. The broader bevel produces rounder shoulders, though the profile is not an exact circle and wider yarns retain some flat crown. Counts determine independent spacing; widths are fractions of spacing. Q retains alternating over/under selection.

Default detailAmount is now 0.08 (previously 0.03); its range stays 0..0.1. Each explicitly seeded value-v2 noise uses scale 4, octaves 2, persistence 0.5, then an 8×1 transform; weft uses quarterTurns=1. Levels remaps 0.3..0.7 to 0..1. Each grain drives the existing height attenuation and a gradient-map between the yarn's dark and full color. Q-selected grain drives yarn roughness between roughnessMin and yarnRoughness; coverage mixes over backing roughness. At detailAmount=0, colors and yarn roughness become unmodulated, while the required seeds remain present. Color/roughness controls stay channel-isolated, and seeds do not move W/F/Q. Full public causality tests remain pending.

The graph's inspection aliases name W/F/Q, both pre-max heights, coverage and both grains. Normal uses final stored height; metallic remains zero. Every production pixel still comes from wgpu, with f32 arithmetic and half storage at every pass.

## Source-bound before/after review

Both runs use clean engine main `e73e2b99c85987551d11db78eb90dfbf0564100a` and the same release CLI built into `target/native-consumer`. The after recipe/plan were temporary candidate inputs, bound by hashes in `recipeIteration`; this subsequently annotated plan is not the measured input. The current graph bytes match the after graph. Historical `reviewMeasurements` remain bound to revision 1 and its older source, not revision 2.

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

Resolve the chosen revision's recipe/defaults using the mapping above. Emit `{version:1,nodes,edges}`: copy node id/type/version/resolved parameters, move each `inputs` reference `source.port` into `from:{nodeId:source,portId:port}` → `to:{nodeId:target,portId:inputName}` edges, append `material-output@1` with ID `material`, and wire all five output aliases to matching channels. Do not retain design-only fields or `$` placeholders in `.mix`. For the before recipe, use main `e73e2b9` and its original mappings. Use a fresh ignored directory and preserve stdout JSON separately from stderr and process exits.

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

# Woven fabric — MAT-03a draft design

English | [简体中文](./README.zh-CN.md)

The [contract](../../../docs/mat-03-woven-surfaces.md), [graph design](./graph-design.json) and [qualification plan](./qualification-plan.json) are **draft, not frozen**, with `runtimeImplemented: false`. No executable material, request builder or woven test command is committed in this slice. Temporary resolved documents and two Native review renders now exist under ignored `tmp/`; they are not accepted pixels or a completed material implementation. Existing node implementations are available; that does not mean this material is implemented. `$` values are caller-side placeholders, not `.mix` syntax or a runtime expression language.

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

The governing measurement is the actual plan-v3 compile-time peak after reuse: **2K fails at 570,426,320 bytes against 536,870,912 (512 MiB)**. Resolve this freeze blocker through recipe change or a separately scoped measured PERF-MAT slice, never by raising the limit. The historical retain-all texture arithmetic, 40×2048×2048×8 = 1,342,177,280 bytes, is background only; it is not the scheduled descriptor peak and does not govern the freeze decision. The 64-pass target and separate hardware/software timing targets remain draft; this review does not qualify timing or implement optimization.

## Reproduced review observations and commands

The clean measured branch was `7555c8b25558fb7a2aedd2a14adc2474b6562006`, based on main `5785068d8d3e49a503bfe30cb1d90d28f0bc548e`, on 2026-10-02. The [plan](./qualification-plan.json) retains `reviewMeasurements`: release binary/input hashes, original pre-amendment plan identity, resolved-document hashes, exact CLI arguments/exits and output hashes. These amendments were not the measured source.

To reproduce without a checked-in resolver, use that source revision's graph and plan, merge defaults/case controls and apply the mapping above recursively to parameters. Emit `{version:1,nodes,edges}`: copy each node's id/type/version/resolved parameters, convert each design `inputs` reference `source.port` to an edge `from:{nodeId:source,portId:port}` and `to:{nodeId:target,portId:inputName}`; append `material-output@1` with ID `material`, and wire every design output alias to its matching material channel. Do not leave design-only fields or placeholders in the `.mix`. Preserve the five resolved cases in a fresh ignored directory. Invoke `target/native-consumer/release/mixture.exe` as `mixture` below; keep stdout JSON separate from stderr and check exits:

```text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <case.mix> --json
mixture inspect <plain.mix> --plan --size 1024 --output baseColor,normal,roughness,metallic,height --json
mixture inspect <plain.mix> --plan --size 2048 --output baseColor,normal,roughness,metallic,height --json
mixture doctor --backend vulkan --json
mixture render <case.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
```

Validation passed for plain, varied, flat, combined-low and combined-high, each with zero diagnostics (exit 0). Plain 1K inspection compiled **40 passes, 16 physical textures, peakBytes 142,607,312 and logicalTextureBytes 335,544,320** (exit 0). Plain 2K inspection returned **MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED**, configured **536,870,912**, observed **570,426,320** (exit 2, no GPU execution). Do not treat that expected reproduction of a rejection as a material budget pass.

Plain/varied 1K five-channel renders succeeded on **NVIDIA GeForce GT 1030 / Vulkan / NVIDIA 582.66**. Agent inspection of original baseColor/normal maps shows alternating crossings and the varied 12×8 counts/unequal widths. The cross-sections read as flat-topped planks with narrow bevel edges; default directional grain is not visibly resolved at detailAmount=0.03, which only scales relief=0.025. These are unresolved decision-2 quality risks, not proof of a missing node or full independent-control causality. No new node is proposed.

The temporary resolver, generated graphs, original input snapshots, raw JSON/stderr, receipt and ten PNGs remain in ignored `tmp/mat03-review-amendment/`. This is local review retention, not durable accepted evidence; hashes identify bytes but do not ensure availability. No PBR/human acceptance, Native/browser parity, DX12/software qualification, timing gate or full structural/odd-size probe was performed.

## Evidence and next review

Read the contract's five maintainer decisions before a later freeze PR. The limited Native review now records a 2K budget rejection and visual risks; resolve them and complete the remaining public Native/browser feasibility work with source/request/build/adapter identity and failures preserved. No new-node proposal is justified merely by the recipe's size. Any demonstrated gap requires a separate minimal identity/ABI/version review. Qualification builds use `target/native-consumer`; ordinary output uses ignored `tmp/` or CI artifacts. Retain accepted images and human decisions under the [evidence policy](../../../docs/evidence-policy.md). No existing golden, manifest, runtime or Studio content changes here.

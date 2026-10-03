# Woven fabric — MAT-03a findings and node proposal (draft)

English | [简体中文](./README.zh-CN.md)

The maintainer changed direction after reviewing recipe revision 4 at ca2e98b on 2026-10-03. Four existing-node rounds traded flat crowns/grain, crossing seams, channel mismatch and bone/quilted shapes for aligned but disconnected-looking capsules. The [contract](../../../docs/mat-03-woven-surfaces.md) records bounded findings and the analytical node proposal; this is not proof of impossibility and does not authorize implementation. **draft, frozen: false, runtimeImplemented: false, materialAccepted: false** remain in force.

## Keep the baseline separate from the proposal

- [graph-design.json](./graph-design.json) remains the exact revision-4 existing-node baseline. Top-level defaults/cases/sweeps and measured resource fields in [qualification-plan.json](./qualification-plan.json) still describe that baseline. The four source-bound review records and input identities are retained; no new GPU run is claimed in this design-only change.
- [graph-proposal.json](./graph-proposal.json) is a separate draft using **proposed weave-pattern@1**, which the current catalog does not implement. Do not run it as if it were a qualified or accepted material. Its projection is not a compiled plan or render measurement.
- Node proposal: no inputs, one value: Scalar output, mode=height/coverage/warp-share; three instances share all geometry parameters and dimensions. Registry metadata permits named output lists, but current lowering/ComputePass/resource lookup is single-output. One mode-based kernel avoids a multi-output runtime redesign.
- The proposed structure separates continuous centerline lift from transverse occupancy, and derives height and coverage-weighted yarn selection from one visibility formula. Under-yarn retains full occupancy even at low height; depth is not backing blend weight. Existing seeded noise, color, roughness and height-to-normal nodes remain responsible for those channels. Exact formulas, parameters, diagnostics, periodicity, 2×2 weighted sampling and 48-byte ABI are in the contract.

| Scope | Passes | Physical textures | 1K peakBytes | 2K peakBytes |
|---|---:|---:|---:|---:|
| Revision 4, measured; all eight cases | 52 | 10 | 92,276,064 | 369,100,128 |
| Proposed graph, static projection only | 21 | 8 | Not measured | 301,990,480 projected |

The projection assumes the numeric ID order, exact-descriptor plan-v3 reuse, 592 uniform bytes and one 33,554,432-byte staging buffer at 2K: 8*33,554,432+592+33,554,432. Its logical 704,643,072 bytes are not the peak. Keep <=64 passes and <=536,870,912 bytes; actual implementation must be measured later. No timing or pixel claim follows from this calculation.

## Baseline reproduction

Merge each baseline case over top-level plan defaults. Validate the unchanged accepted control ranges; reject odd counts, unknown/non-finite/out-of-range controls. Resolve warpGap=1-warpWidth, weftGap=1-weftWidth, halfWarpCount=warpCount/2, crossingOffsetX=0.5/warpCount, profileBevel=0.19+0.5*bevel, warpDark/weftDark=respective color RGB*(1-4*detailAmount) with alpha=1, roughnessMin=yarnRoughness*(1-2*detailAmount). Other placeholders use same-named controls, including underRatio. Fixed unvaried profile/selector seeds are zero; both noise seeds remain explicit. These are caller expressions, not Core expressions.

Keep numeric node IDs. Emit ordinary .mix v1: copy node id/type/version/resolved parameters; turn inputs source.port into from/to edges; append material-output@1 with ID material and wire all five outputs. Exclude design-only fields/placeholders. Revision-4 aliases warpShape/weftShape are normalized A/B, coverage=C and surfaceOrder=D; consult the retained revision-4 section for their historical formulas. This baseline was not accepted for continuous-yarn appearance.

~~~text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <baseline-case.mix> --json
mixture inspect <baseline-case.mix> --plan --size <1024|2048> --output baseColor,normal,roughness,metallic,height --json
mixture render <plain|varied.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
~~~

Here mixture is target/native-consumer/release/mixture.exe. Prior revision-4 Native renders used NVIDIA GeForce GT 1030 / Vulkan / NVIDIA 582.66. Local review files remain under tmp/mat03-recipe-review-4/after/{plain,varied}-1024/: baseColor, normal, roughness and height PNGs; plain has matching *-crossing-256.png crops (x=64,y=64,256×256, no rescale/tone adjustment). The ignored root also holds baseline/probe inputs and receipts. Neutral C and D*C probes had zero differing pixels, but they did not detect the maintainer's continuous-yarn failure. These files are not durable acceptance evidence.

## Proposed caller mapping and pending decisions

For the proposal only, merge top-level baseline defaults, then graph-proposal.proposedDefaults, then each case override. That adds crown=0.5 and restores underRatio=0.5 / relief=0.025; it does not alter baseline defaults. Geometry bundle {warpCount,weftCount,warpWidth,weftWidth,bevel,crown,underRatio} must be identical on the three weave instances, with only mode different. The proposed bevel directly controls transverse occupancy feather, with no brick profileBevel conversion. Resolve color/roughness dark-end mappings exactly as above; other proposal placeholders directly use named controls. Existing node seeds stay explicit; the deterministic weave generator has no random output or seed. Candidate defaults and crown control require approval.

Approval questions: bounded node rationale; identity/Scalar-mode design versus separately scoped multi-output support; exact occupancy/lift/profile/visibility/sampling contract; crown/bevel/defaults; **17 types/15 kernels → 18/16** and unpublished **Rust 0.9.0 / browser 0.9.0-alpha.0**; precise node/material/PBR/human gates; explicit later implementation authorization. No manifests change; .mix/.mixpack v1 and plan/API v3 remain. Old catalogs reject the new type; no fallback or migration.

The contract's later-PR checklist covers invariant 8 and the node playbook: Core contract/validation/lowering, exhaustive kernel consumers, one WGSL, fixtures/docs/targeted tests, shader/software/GPU/browser/node/material regressions, package consumption, unchanged budgets and separate visual acceptance. No proposed-node command is currently runnable. Keep full structural, sampling, raw-half/normal, periodic/odd-size, <=1/255 Native/browser, repeat/package, timing, PBR and human freeze blockers. Retain reviewed bytes under the [evidence policy](../../../docs/evidence-policy.md). No new crate, runtime, shader, version, golden or committed tooling change is included.

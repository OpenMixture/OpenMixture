# Woven fabric — Stage A public material matrix

English | [简体中文](./README.zh-CN.md)

The material plan is frozen and the node is implemented; **materialAccepted=false**. First commit f3d0f3f froze the [plan](./qualification-plan.json) and [recipe](./graph-proposal.json), before the [ordinary .mix v1 material](./material.mix) and public caller tools. Original draft bytes remain in [qualification-plan-draft.json](./qualification-plan-draft.json) and [graph-proposal-draft.json](./graph-proposal-draft.json). [graph-design.json](./graph-design.json) and original receipts retain revision 4.

The [staged plan](../../../docs/mat-03-woven-surfaces.md) scopes this PR to A, the public Native/browser matrix. B raw-half/structural/periodic/normal-replay/stress probes, C dielectric PBR/human decision and D retained evidence/acceptance each require later PRs. This matrix does not establish structural, PBR or human acceptance.

## Exact caller controls

The [request builder](../../../scripts/woven-fabric-requests.mjs) validates and emits every public override. Three weave instances share seven geometry controls; only mode differs. Seeds are explicit. Color alpha and derived dark alpha are one. Changes to warpColor/weftColor/yarnRoughness also recompute derived endpoints. The frozen plan's bindings specify the exact public ID mapping below.

| Control | Default | Inclusive range/type | Public override mapping |
|---|---|---|---|
| warpCount | 8 | 4..32 (even; never rounded) | warpshare_warpCount → n06-weave-warp-share.warpCount; coverage_warpCount → n09-weave-coverage.warpCount; height_warpCount → n17-weave-height.warpCount |
| weftCount | 8 | 4..32 (even; never rounded) | warpshare_weftCount → n06-weave-warp-share.weftCount; coverage_weftCount → n09-weave-coverage.weftCount; height_weftCount → n17-weave-height.weftCount |
| warpWidth | 0.7 | 0.55..0.9 | warpshare_warpWidth → n06-weave-warp-share.warpWidth; coverage_warpWidth → n09-weave-coverage.warpWidth; height_warpWidth → n17-weave-height.warpWidth |
| weftWidth | 0.7 | 0.55..0.9 | warpshare_weftWidth → n06-weave-warp-share.weftWidth; coverage_weftWidth → n09-weave-coverage.weftWidth; height_weftWidth → n17-weave-height.weftWidth |
| bevel | 0.08 | 0.02..0.12 | warpshare_bevel → n06-weave-warp-share.bevel; coverage_bevel → n09-weave-coverage.bevel; height_bevel → n17-weave-height.bevel |
| relief | 0.025 | 0..0.05 | relief → n18-surfaceHeight.outputMax |
| underRatio | 0.5 | 0.25..0.75 | warpshare_underRatio → n06-weave-warp-share.underRatio; coverage_underRatio → n09-weave-coverage.underRatio; height_underRatio → n17-weave-height.underRatio |
| detailAmount | 0.08 | 0..0.1 | warpDark = warpColor.rgb*(1-4*d); weftDark = weftColor.rgb*(1-4*d); roughnessMin = yarnRoughness*(1-2*d) |
| warpSeed | 1729 | 0..4294967295 (integer) | warpSeed → n03-warpNoise.seed |
| weftSeed | 65537 | 0..4294967295 (integer) | weftSeed → n00-weftNoise.seed |
| warpColor | [0.22,0.08,0.035,1] | RGBA 0..1; alpha=1 | warpColor → n07-warpColor.colorB |
| weftColor | [0.38,0.23,0.1,1] | RGBA 0..1; alpha=1 | weftColor → n10-weftColor.colorB |
| backingColor | [0.015,0.012,0.01,1] | RGBA 0..1; alpha=1 | backingColor → n12-backingColor.value |
| yarnRoughness | 0.8 | 0..1 | yarnRoughness → n14-yarnRoughness.outputMax |
| backingRoughness | 0.95 | 0..1 | backingRoughness → n15-backingRoughness.value |
| normalStrength | 0.5 | 0..1 | normalStrength → n20-normal.strength |
| crown | 0.5 | 0..1 | warpshare_crown → n06-weave-warp-share.crown; coverage_crown → n09-weave-coverage.crown; height_crown → n17-weave-height.crown |

## Reproduction and gates

Commit implementation first to bind clean source; use fresh output directories for each run. There are 48 rows (12 cases × four sizes) and three stress rows outside the plain/varied downsample guarantee, still gated for repeats/parity. Native native.json updates after each row, retaining numeric frozen-gate failures before stopping. For each CLI row run `inspect material.mix --plan --size WxH --output baseColor,normal,roughness,metallic,height --json`, passing every request.overrides entry as `--set ID=JSON`; validate a temporary ordinary .mix with those values substituted. Do not commit the resolver or rewrite original receipts.

See the frozen plan for color/height box-filter gates, all-channel cross-runtime limits and matched-adapter timings. Software timing requires pinned SwiftShader and MIXTURE_SWIFTSHADER_COMMIT. Outputs are encoded after renderer destruction; independent constants still use the same wgpu executor. Stop on any frozen gate failure without tuning or relaxing it.

```powershell
$env:PATH='C:\Users\krapnik\AppData\Roaming\fnm\node-versions\v24.21.0\installation;'+$env:PATH
node --test scripts/woven-fabric-requests.test.mjs
node scripts/woven-fabric-requests.mjs tmp/woven-matrix/requests
$env:MIXTURE_GPU_BACKEND='vulkan' # repeat separately with dx12
$env:MIXTURE_GPU_SOFTWARE='0'
$env:MIXTURE_GPU_EXPECT_ADAPTER='NVIDIA GeForce GT 1030'
$env:MIXTURE_WOVEN_ROOT=(Get-Location).Path
$env:MIXTURE_WOVEN_REQUESTS=(Join-Path (Get-Location).Path 'tmp/woven-matrix/requests')
$env:MIXTURE_WOVEN_EVIDENCE=(Join-Path (Get-Location).Path 'tmp/woven-matrix/native-vulkan')
cargo test --release --locked --all-features --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --test woven_material -- --ignored --nocapture
$env:MIXTURE_BROWSER_CHANNEL='chrome'
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/woven-matrix/browser
node scripts/browser-runtime/check-woven.mjs tmp/woven-matrix/browser tmp/woven-matrix/comparison
cargo xtask test-consumer
cargo xtask links
cargo xtask check
```

## Historical PR #80 design record (status at that time)

## Keep the baseline separate from the proposal

- [graph-design.json](./graph-design.json) remains the exact revision-4 existing-node baseline. Top-level defaults/cases/sweeps and measured resource fields in [qualification-plan.json](./qualification-plan-draft.json) still describe that baseline. The four source-bound review records and input identities are retained; no new GPU run is claimed in this design-only change.
- [graph-proposal.json](./graph-proposal-draft.json) is a separate draft using **proposed weave-pattern@1**, which the current catalog does not implement. Do not run it as if it were a qualified or accepted material. Its projection is not a compiled plan or render measurement.
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

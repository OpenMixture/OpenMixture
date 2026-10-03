# Woven fabric — public matrix and Stage B structural probes

English | [简体中文](./README.zh-CN.md)

The material plan is frozen and the node is implemented; **materialAccepted=false**. First commit f3d0f3f froze the [plan](./qualification-plan.json) and [recipe](./graph-proposal.json), before the [ordinary .mix v1 material](./material.mix) and public caller tools. Original draft bytes remain in [qualification-plan-draft.json](./qualification-plan-draft.json) and [graph-proposal-draft.json](./graph-proposal-draft.json). [graph-design.json](./graph-design.json) and original receipts retain revision 4.

The [staged plan](../../../docs/mat-03-woven-surfaces.md) retains the merged Stage A public Native/browser matrix. This PR adds Stage B raw-half/structural/periodic/normal-replay/stress probes; C dielectric PBR/human decision and D retained evidence/acceptance require later PRs. This matrix does not establish structural, PBR or human acceptance.

Current material is [explicit recipe revision 2](../../../docs/mat-03-default-revision.md): only defaults underRatio=0.25 and crown=0 change; all frozen gates remain identical. Historical Stage A observations below describe revision 1, not qualification of revision 2; the caller-controls table describes current defaults. The maintainer accepted these defaults on 2026-10-04; residual pinch is recorded, and full PBR/human review remains Stage C. materialAccepted=false.

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
| underRatio | 0.25 | 0.25..0.75 | warpshare_underRatio → n06-weave-warp-share.underRatio; coverage_underRatio → n09-weave-coverage.underRatio; height_underRatio → n17-weave-height.underRatio |
| detailAmount | 0.08 | 0..0.1 | warpDark = warpColor.rgb*(1-4*d); weftDark = weftColor.rgb*(1-4*d); roughnessMin = yarnRoughness*(1-2*d) |
| warpSeed | 1729 | 0..4294967295 (integer) | warpSeed → n03-warpNoise.seed |
| weftSeed | 65537 | 0..4294967295 (integer) | weftSeed → n00-weftNoise.seed |
| warpColor | [0.22,0.08,0.035,1] | RGBA 0..1; alpha=1 | warpColor → n07-warpColor.colorB |
| weftColor | [0.38,0.23,0.1,1] | RGBA 0..1; alpha=1 | weftColor → n10-weftColor.colorB |
| backingColor | [0.015,0.012,0.01,1] | RGBA 0..1; alpha=1 | backingColor → n12-backingColor.value |
| yarnRoughness | 0.8 | 0..1 | yarnRoughness → n14-yarnRoughness.outputMax |
| backingRoughness | 0.95 | 0..1 | backingRoughness → n15-backingRoughness.value |
| normalStrength | 0.5 | 0..1 | normalStrength → n20-normal.strength |
| crown | 0 | 0..1 | warpshare_crown → n06-weave-warp-share.crown; coverage_crown → n09-weave-coverage.crown; height_crown → n17-weave-height.crown |

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

## Stage B frozen structural probes

The maintainer accepted material recipe revision 2 defaults on 2026-10-04; this does not accept the full material. The additive `structuralProbes` section in [the plan](./qualification-plan.json) owns the exact probes. No Stage A field, threshold, case, size, timing budget, default or retained revision-1 receipt changes. C dielectric PBR/human review and D retained evidence/acceptance remain separate; materialAccepted=false.

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


### Focused Stage B commands

The following ignored tests run through `cargo xtask gpu-smoke` automatically, using its existing serialized ignored-test selection. Set the documented explicit GPU environment for Vulkan or DX12. The reviewer has authorized the narrowly bounded sampling-origin instrumentation described above.

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_crossing_structure -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_flat -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_control_isolation -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_normal_periodic -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_stress -- --ignored --nocapture
```

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

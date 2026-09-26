# Painted metal — MAT-02a frozen design

English | [简体中文](./README.zh-CN.md)

The [graph design](./graph-design.json) and [qualification plan](./qualification-plan.json) retain the frozen MAT-02a contract. Both node identities are now implemented; complete material acceptance remains pending. The historical plan fields `runtimeImplemented: false` and `planned-mat-02a-frozen` describe its freeze state, not current implementation. The `$` recipe is not a runtime expression language or `.mix` document.

## Implemented request preparation

The [fixture request builder](../../../scripts/painted-metal-requests.mjs) validates the controls below and generates all seven presets × four sizes × five channels (140 channel comparisons per Native/browser pairing). It copies the exact executable graph from the [measured baseline](../../../docs/evidence/perf-mat-before/material.mix), preserves its topology/node versions, and emits ordinary public request overrides. Native and browser harnesses can consume the same JSON requests; no graph semantics move out of Core.

```bash
node --test scripts/painted-metal-requests.test.mjs
node scripts/painted-metal-requests.mjs tmp/mat02-requests
```

The output directory must be fresh. `requests.json` binds source bytes, the frozen plan, builder, revision and working-tree state; `material.mix` preserves the original graph bytes. CPU tests cover matrix completeness, invalid controls, endpoints, independent axis radii and snapshot isolation, and run in both existing browser CI workflows. Preparation alone neither renders pixels nor qualifies the material. Full Native/browser comparisons, raw-half relationships/causality, seams, package/repeat checks, release timing and metallic PBR/human acceptance remain required.


## Public GPU matrix candidate

The Native `painted_material` integration test and browser `painted.spec.mjs` consume the same 28 requests. They check all five channels, exact repeats, loose/package equivalence, sliced height, owned outputs after destruction and physical memory accounting. Native measures the frozen default 1K/2K cold/warm budgets in release mode and rejects unknown timing-adapter coverage. These matrix tools are candidates; their presence is not evidence of a passing run.

For a Native run, set `MIXTURE_PAINTED_ROOT` to the repository, `MIXTURE_PAINTED_REQUESTS` to the generated request directory and `MIXTURE_PAINTED_EVIDENCE` to a fresh output directory. Explicitly set `MIXTURE_GPU_BACKEND=vulkan|dx12` and `MIXTURE_GPU_SOFTWARE=0|1`; `MIXTURE_GPU_EXPECT_ADAPTER` can enforce the recorded device.

```bash
cargo test --release --locked --all-features --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --test painted_material -- --ignored --nocapture
node scripts/browser-runtime/check-painted.mjs tmp/sdk-candidate tmp/sdk-painted-comparison
```

The second command consumes a successful independent SDK candidate run: candidate mode now requires 21 browser tests, including the full painted matrix. It checks exact source/request/package/build identities and reruns the Native matrix against every browser channel with the unchanged <=1/255 limit. The existing Chromium workflow invokes this comparison and retains its output. Registry mode stays at 13 tests; the published package does not support this material. Raw-half causality, periodic seam tests, stress/PBR views and human decisions remain separate pending gates. `materialAccepted` stays false.


## Endpoint and resolution-quality gates

Both public hosts render independent constant reference graphs for intact paint, exposed substrate and fully rusted substrate. At all four frozen sizes, every pixel of all five channels must equal its constant reference, including the flat normal. The comparison requires all 60 endpoint-channel assertions. References use ordinary `.mix` nodes through the sole wgpu executor; they do not reuse the layered material topology or add a CPU renderer.

Each host also compares default 256² height/baseColor against the exact 4×4 box mean of its delivered 1024² RGB bytes. The mean is not rounded back to an integer byte before measuring absolute error. Each RGB component must stay within the frozen 4/255 limit, and the comparison rejects missing or failed measurements. Alpha is opaque and excluded from the RGB measurement. Native oracle tests cover fractional means, distinct components, rectangular indexing and truncated inputs. This executes the existing contract; it changes no shader, format, threshold or golden. Full material acceptance still requires raw-half causality, periodic seams, stress and PBR/human review.

## Public control isolation

At the frozen 257×129 size, the Native and browser matrix runners render the complete default material with eleven individual control changes: each of the three colors, each of the three roughness endpoints, normal strengths 0/0.5/1, and independently incremented macro/detail seeds. Every variant is rendered twice. Color changes must affect baseColor only; roughness changes must affect roughness only; normal strength must affect normal only. A changed control must change at least one delivered pixel, while the unchanged normal-strength case must reproduce all channels exactly. Seed variants must change pixels and repeat exactly. Both hosts retain matching control/effect receipts, and comparison rejects missing cases.

These are delivered-byte causality checks on the actual graph, not proof of internal half-float mask inequalities or exact normal replay from the stored height. Raw-half W/I/B/R relations, exposure/width monotonicity, seed isolation of W, seams, stress and PBR/human review remain required. The request manifest binds the frozen causality inputs; no shader, runtime API or material contract changes.

## Raw-half mask qualification

The ignored `graph_gpu_painted_raw_mask_relations_and_control_causality` Rust test reads the measured material and frozen plan from the source checkout. It retains all 23 pixel computations and changes only the height output edge to observe W, I, B, R, S or D. Core compiles each alias and pins its output through the ordinary allocation plan. The sole executor runs the production kernels and normal copy/map/cleanup path. A private `cfg(test)` readback format returns tightly packed half bytes before RGBA8 conversion; no raw-output public API, shader variant, global capture buffer or alternate executor is added. Published builds retain only ordinary RGBA8 readback.

At 257×129, nineteen cases check finite normalized values, I≤W, B=half(max(W−I,0)), B≤S≤W, R≤S≤W, detail bounds, exact zero-width B even at fractional W, monotonic band/exposure/rust controls, exact fill endpoints, rust-control isolation of W/I/B, detail-seed isolation of W/I/B and exact seeded repeats. Assertions inspect every raw pixel without a byte tolerance. The CPU readback probe verifies row-padding removal and preservation of a 1/4096 difference. `cargo xtask gpu-smoke` includes the test; the focused command below uses the same documented explicit GPU environment. Ordinary package unit-test compilation does not require repository fixtures; executing this ignored qualification does.

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_painted_raw_mask -- --ignored --nocapture
```

This gate does not qualify browser raw-half fields, complete metallic/roughness/height composition, normal replay, periodic seams, stress or PBR/human review. The separate public matrix and remaining material gates still apply.

## Scalar composition and normal replay

The ignored `graph_gpu_painted_composition_and_final_height_normal_replay` test covers all seven frozen presets at 257×129. It observes the actual W/R, coating/final height, coating/final roughness and metallic fields through the same raw-half instrumentation. Independent scalar assertions account for half storage between nodes, including constant rust height/roughness, check each composition and bound final height between the permitted substrate and paint endpoints. These finite per-pixel assertions are test oracles, not a CPU rendering API.

The original five-output graph also returns its final height and normal as raw half bytes. The test uploads those exact height bytes to a separate probe texture and invokes the existing production height-to-normal WGSL via the normal pipeline factory. At normal strengths 0/0.5/1, the graph's stored height must stay identical and every replayed normal byte must match the graph output. No RGBA8 height reconstruction or CPU normal algorithm is used. This is test-only GPU instrumentation, using the same shader and readback/cleanup helpers, with no public API or runtime behavior change. `cargo xtask gpu-smoke` includes the test; its focused command is:

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_painted_composition -- --ignored --nocapture
```

These checks do not replace browser raw-half evidence, other resolutions, periodic-seam/stress measurements or metallic PBR/human review. The public multi-resolution Native/browser matrix remains required.

## Metallic PBR review views

`scripts/painted-material-preview.mjs` visualizes already-rendered 1K maps from a successful clean-source Native/browser painted comparison directory. It verifies producer/contract/source identities and dimensions, reads all five channels for the seven frozen presets, and records every PNG and screenshot hash. It does not execute `.mix` or modify textures. Keep the MAT-01 dielectric renderer and its historical review bytes unchanged.

```bash
npm ci --ignore-scripts --prefix examples/browser-consumer
node scripts/painted-material-preview.mjs <painted-comparison-directory> <fresh-preview-directory> [producer-plan.json]
```

The controlled Chrome consumer uses metallic GGX shading with baseColor-driven conductor reflectance and suppressed metallic diffuse, fixed orthographic camera/key/fill lights, fixed tone mapping and an explicit ambient approximation. Each preset has plane/sphere, 1×/3× repeat and 4× close-up views plus five channel thumbnails. Height is shown but not displaced. A separate three-canvas check requires metallic 0/1 to change shading and metallic 0 to repeat exactly; these forced settings are excluded from the material screenshots. `MIXTURE_BROWSER_CHANNEL` may select an installed browser; the default is Chrome with the same recorded controlled WebGPU flags as the earlier preview.

The fresh output contains seven PBR sheets, a static `index.html` gallery and `preview.json` binding producer build/revision, preview revision/dirty status, adapter/browser, input/screenshot/renderer hashes and shading-check results. `humanAccepted` and `materialAccepted` remain false. Agent image inspection is not a human decision. Retain selected reviewed images and an actual maintainer decision under the evidence policy before claiming visual acceptance; numeric passes alone do not accept smooth/chunky wear, weak rust visibility or close-up quantization. Periodic sampling measurements and high-frequency/subpixel stress remain independent gates.

If a producer checkout used different line endings, supply its exact plan snapshot as the optional third argument. Its raw hash must match the original receipt and its parsed contract must equal the current frozen plan. The output preserves that snapshot and records both hashes; line-ending differences never rewrite historical identities.

## Exact caller controls

Unknown controls, non-finite values and values outside these ranges must be rejected by the fixture request builder. This is a fixture harness contract, not a new Core API. Merge preset controls over defaults before mapping. Explicit macro/detail seeds are u32; preserve both, including when detail is disabled. All colors are linear RGBA in [0,1] with alpha fixed at 1.

| Public control | Range | Mapping |
|---|---|---|
| `exposureAmount` | Float [0,1] | For 0<a<1, exposure levels inputMin=0.8*(1-a), inputMax=inputMin+0.2, outputMin=0, outputMax=1. At a=0 or 1 use inputMin=0, inputMax=1 and both outputs equal a. Gamma is always 1. Exact endpoints do not depend on noise extrema. |
| `exposureScale` | Integer [1,64] | Macro noise scale; three octaves and persistence 0.5 are fixed. Default quality applies to the default scale, not all high-frequency inputs. |
| `macroSeed`, `detailSeed` | u32 | Respective noise seed. Detail noise scale 32, two octaves, persistence 0.5, both bases explicitly `value` with version 2. |
| `edgeWidth` | Integer [0,8] | Reference texels at 1024; use the plan's independent per-axis radius formula. Reject axes >2048 in this qualification harness. |
| `rustAmount`, `rustFill`, `detailAmount` | Float [0,1] | Rust mask opacity, band-to-exposure interpolation weight, and detail levels outputMin=1-detailAmount respectively; outputMax=1. |
| `paintColor`, `substrateColor`, `rustColor` | Linear RGBA | Corresponding constant-color value; alpha must remain 1. |
| `paintRoughness`, `substrateRoughness`, `rustRoughness` | Float [0,1] | First two are coatingRoughness output endpoints; third is rustRoughness constant. Reversed levels output endpoints are intentional and supported. |
| `paintThickness` | Float [0,0.5] | coatingHeight outputMin=0.2+paintThickness, outputMax=0.2. |
| `rustRelief` | Float [0,paintThickness] | rustHeight constant=0.2+rustRelief; reject relief above thickness instead of silently clamping. |
| `normalStrength` | Float [0,8] | Existing height-to-normal strength. |

Output names map to ordinary material-output ports when the real `.mix` is constructed. Inspection aliases expose intermediate masks only to qualification requests; they do not add material channel types or public expression syntax. Every render starts from the same graph topology, including endpoint presets, so controls cannot hide unwanted dependencies by replacing the graph. Dependency slicing for a requested output remains Core-owned.

The conceptual formulas describe structure; actual references must honor each existing node's half-float intermediate rounding. For mask inequalities, inspect raw half values rather than infer exact values from RGBA8. Endpoint pixels are compared with independently constructed constant reference graphs through the sole wgpu executor, including its existing normal encoding/conversion. A half-float difference must not be hidden by an RGBA8-only subtraction probe.

`detailAmount` modulates rust coverage, hence downstream relief/roughness/color; it does not move W. `rustFill=0` restricts rust to the eroded edge band, while `rustFill=1` permits coverage throughout exposure. Zero edge width forces B=0; it does not force R=0 when rustFill>0. Paint remains at or above the permitted rust/substrate height endpoints. This does not claim every transition height equals one endpoint.

## Historical resource decision and freeze evidence

The explicit graph has 23 pixel nodes and one eventual structural material-output node. The 24-pass ceiling is retained. Using levels for the two coating endpoint interpolations avoids four redundant constant/interpolation passes while preserving their public controls; it changes no node implementation.

At design freeze, the executor retained every rgba16float intermediate. At 2048², 23 textures alone require **771,751,936 bytes**, above the unchanged **536,870,912-byte** descriptor ceiling, before uniforms and the sequential readback buffer. This is a static design estimate, not a compiled plan, measured GPU allocation or accepted performance result. Core also defaults to a 512 MiB transient-byte limit, so the current 2K graph is expected to be rejected before execution. Once its nodes exist, retain the actual structured 2K budget failure plus a valid 1K compiled plan and render baseline before opening a separate PERF-MAT lifetime/reuse slice. Both Core estimates and executor lifetimes must agree; do not raise the global limit to collect a passing 2K result. Do not raise the ceiling or add pass fusion/global caches to conceal the failure. The graph and quality goals stay fixed through that optimization.

The [retained existing-input measurement](../../../docs/evidence/mat-02-input-feasibility/README.md) supports keeping the 4/255 target: maximum baseColor mean error 0.158619/255 and height 0.101737/255 on recorded GT 1030 Vulkan/DX12. The JSON freezes exact probe fields, finite-set analytical oracles and ABIs. The contract selects the two new identities and 0.7 candidate at first implementation. Existing-node feasibility does not qualify unimplemented morphology or the finished material. Admission, implementation, qualification and publication remain separate.


## Final-height normal periodic boundaries

`graph_gpu_painted_normal_periodic_boundaries` captures final height and normal as raw half bytes for all seven presets at all four frozen sizes. It cyclically shifts the captured height by one pixel on each axis and by half the image on both axes, then invokes the existing production normal kernel. Every output byte must equal the identical cyclic shift of the graph normal. Moving boundary pixels into the interior (and the reverse) checks periodic derivative sampling without incorrectly requiring opposite border pixels to match. The CPU helper only permutes bytes; an asymmetric rectangular test verifies its indexing.

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_painted_normal_periodic_boundaries -- --ignored --nocapture
```

The 28 cases contain 84 exact translation comparisons per adapter. The test runs in `cargo xtask gpu-smoke` and prints adapter identity and case/size/offset receipts. This specifically covers the final-height normal derivative. It does not establish periodicity of the upstream noise/mask composition, browser raw-half equivalence, absence of visually conspicuous seams, stress quality or human acceptance. No shader, runtime API, node version or golden changes.

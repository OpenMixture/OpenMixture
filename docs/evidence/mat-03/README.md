# MAT-03 qualification — accepted within recorded scope

English | [简体中文](./README.zh-CN.md)

This Stage D record closes woven-fabric recipe revision 2 within the scope below. Fresh machine evidence binds clean main **9dc8c1a4dec12a0566f5237e05dc6862056bc819**, the merge of PR #86. The human decision binds the **original six Stage C sheets from clean 6eb52a6a4de18ae0f8081e6314cff0215e249783**, not newly generated images or a later documentation commit. Rust **0.9.0** / browser **0.9.0-alpha.0** remain unpublished. Only the recorded adapters and scopes are qualified.

## Maintainer decision and visual scope

The [human decision](./human-decision.json) records these statements verbatim, dated **2026-10-06, Asia/Shanghai**:

- **"Accept with a narrower range"** — superseded: PR #87 measurements showed narrowing underRatio alone does not remove the combined-high bow-tie. PR #87 was closed without merging.
- **"Accept now, fix in @2 later (Recommended)"** — final: accept now with the quality boundary below. Record **weave-pattern@2**, an over-yarn visibility fix so it wins across its own width at crossings, as the next follow-up. It is **not started** and requires a separate maintainer entry decision and contract/catalog/version review.

Accepted visual presets: **plain, varied, warp-seed, weft-seed, combined-low**. **Combined-high and similar high underRatio/crown/width, low-bevel combinations that produce bow-tie yarn shapes are outside the visual quality guarantee.** Passing numeric comparisons does not extend that guarantee across the whole allowed control space. The material/node range remains underRatio 0.25..0.75; recipe defaults remain underRatio=0.25 and crown=0. Every frozen gate, case, size and budget is unchanged.

[plain](./review/plain-pbr.png) · [varied](./review/varied-pbr.png) · [warp-seed](./review/warp-seed-pbr.png) · [weft-seed](./review/weft-seed-pbr.png) · [combined-low](./review/combined-low-pbr.png) · [combined-high — outside visual guarantee](./review/combined-high-pbr.png)

[preview.json](./review/preview.json) and its [index](./review/index.html) retain their pre-decision false flags and pending text. The later decision is here and in human-decision.json; frozen plans and historical receipts are not rewritten. The preview has fixed dielectric GGX lighting, orthographic plane/sphere, 1×/3× tiling, 4× close-ups and five channel thumbnails; height is shown, not displaced. The recorded roughness 0/1 sanity images differ, and identical settings repeat exactly. Agent image inspection is not the human decision.

## Machine gates at clean main

[Machine summary](./machine/summary.json), [Vulkan receipt](./machine/native-vulkan.json), [DX12 receipt](./machine/native-dx12.json), [browser matrix](./machine/woven-browser.json), [candidate consumption](./machine/browser-qualification.json) and [binding](./binding.json) carry source, request, plan, package, adapter and file identities.

- Both GT 1030 backends pass **51 rows** (48 frozen material + 3 separately scoped stress) and **255 channel comparisons** against the same clean Chrome **154.0.8037.98** candidate. Maximum component difference is **1/255 on Vulkan and DX12**, within ≤1/255. Vulkan has 251 exact comparisons and four at one; DX12 has 244 exact and eleven at one. The browser reports BrowserWebGpu with an empty adapter name; do not infer an additional GPU hardware identity from it.
- Chrome candidate consumption passes **23/23 tests**, with no skipped, unexpected or flaky tests. Exact repeats, loose/.mixpack equivalence, sliced height, owned output lifetime, endpoint references and physical allocation accounting pass.
- All rows use **21 passes / 8 physical textures**. The full five-channel 2048² descriptor peak is **301,990,480 bytes**, below **536,870,912**; the pass gate remains ≤64. Reported live bytes return to zero.
- Worst default/varied gated downsample per-component mean error across both pairs is **0.335238457/255**, on Native Vulkan, DX12 and Chrome, below **4/255**. Stress remains separately labeled and does not extend the visual or default-quality guarantee.
- All four matched-adapter timing rows pass on each backend. GT 1030 Vulkan uses NVIDIA **582.66**; DX12 records its driver in the receipt. Frozen hardware budgets are cold 1K ≤10,000 ms, warm median 1K ≤1,000 ms and 2K ≤4,000 ms; software budgets remain 60,000 / 20,000 / 80,000 ms. Timings include readback, exclude adapter creation and are not an SLA.

| Backend | Preset / size | Cold ms | Five-render warm median ms |
|---|---|---:|---:|
| vulkan | plain-1024x1024 | 257.082 | 215.408 |
| vulkan | plain-2048x2048 | 837.889 | 986.056 |
| vulkan | varied-1024x1024 | 231.922 | 241.751 |
| vulkan | varied-2048x2048 | 1564.909 | 1179.280 |
| dx12 | plain-1024x1024 | 1231.064 | 356.079 |
| dx12 | plain-2048x2048 | 2412.133 | 1033.852 |
| dx12 | varied-1024x1024 | 985.451 | 225.803 |
| dx12 | varied-2048x2048 | 1581.106 | 850.630 |

## Stage B and node gates

All **seven probes** pass their **gated** rules on GT 1030 Vulkan and DX12; [command receipts](./machine/probe-runs.json) retain actual start/end times. The [SwiftShader receipts](./machine/swiftshader/) come from the successful post-merge pinned GPU job, not an inference from local hardware.

| Probe | GT 1030 Vulkan | GT 1030 DX12 | Pinned SwiftShader Vulkan |
|---|---|---|---|
| Crossing structure, plain/varied | 8 rows / four sizes | 8 rows / four sizes | 6 rows / three sizes |
| Flat height/normal | 4 rows / four sizes | 4 rows / four sizes | 3 rows / three sizes |
| Public control isolation | 16 variants / 257×129 | 16 variants / 257×129 | 16 variants / 257×129 |
| Final-height normal cyclic replay | 48 case-size rows / four sizes | 48 case-size rows / four sizes | 36 rows / three sizes |
| Selected noise periodicity | 48 rows / four sizes | 48 rows / four sizes | 36 rows / three sizes |
| Weave translated fields | 672 gated passes + 48 observations | 672 gated passes + 48 observations | 492 gated passes + 48 observations; 180 rows notRunOnSoftware |
| Dense-thin stress structural checks | 3 frozen sizes | 3 frozen sizes | 3 frozen sizes |

Hardware full-matrix sizes are 256², 1024², 2048² and 257×129. Isolation is intentionally odd-size only; stress retains 256²/1024²/257×129. Amendment 4 omits 2048² on reported Cpu software adapters, with explicit **notRunOnSoftware**, never passed. Software does not qualify hardware pixels. The focused **cargo xtask test-node weave-pattern** passes on Vulkan, including contract fixtures, GPU node checks and the translated-field probe; [node receipt](./machine/weave-pattern.json).

## Known limitations retained with acceptance

- **Bow-tie corner:** the [original source-bound sweep](./limitations/safe-region-sweep.json) and [derived summary](./limitations/summary.json) preserve the evidence behind the superseded range decision. At the later **0.75 reporting threshold**, **156/432** grid points meet the visible-width criterion; **crown=1: 8/144**. The original experiment used 0.85 and is unchanged. The 0.75 summary is not a new gate or proof of a safe continuous region.
- **Residual default pinch:** plain recipe revision 2 measured min **0.875** for both axes at 1K. Human acceptance does not claim straight-sided yarn at all positions.
- **Amendment 3:** odd 257×129 translated warp-share and visible weight are **ungated observations**, not passes. On each recorded hardware backend, raw S reaches **27 half steps** at combined-high; max |ΔP| is **0.00048828125**, about **0.124512 of an 8-bit level**. Maximum ratio to the superseded amendment-2 inequality is **6.802104**. Power-of-two translations and zero-origin identity remain exact; odd-size height/coverage retain the one-half-step gate. These out-of-tile f32 sampling observations do not rewrite production semantics.
- **Amendment 4:** software omits 2048²; only hardware evidence covers that size. No arbitrary adapter, arbitrary control combination, cloth simulation, fiber geometry or universal antialiasing guarantee is made.
- **Unpublished:** no Rust or browser package is published by this record. MAT-04 remains planned; weave-pattern@2 is not implemented or started.

## Post-merge CI and verification

All six required checks passed on **9dc8c1a**: three-platform CPU, WASM/npm, pinned SwiftShader GPU/materials and Chromium WebGPU materials: [CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415513), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415302), [pinned SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415383), [Chromium attempt 2](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415333/attempts/2). The original Chromium attempt hit its 50-minute timeout; [attempt 1](./ci/ci-browser-materials.json) remains recorded as cancelled. The unchanged retry's [attempt-2 result](./ci/ci-browser-retry.json) is separate evidence, not a rewrite of that cancellation. No workflow timeout or frozen gate changed.

The documentation/evidence delivery additionally runs cargo xtask evidence, cargo xtask links and cargo xtask check serially. Their result is recorded in the delivery PR; those checks do not turn its documentation commit into the tested machine source.

## Reproduction and retention

Use the [fixture guide](../../../fixtures/materials/woven-fabric/README.md) for caller controls and probe commands. At the stated clean main revision, prepend Node 24.21.0 to PATH, set MIXTURE_BROWSER_CHANNEL=chrome, build with node scripts/browser-runtime/build.mjs, then run consumer.mjs candidate target/browser-runtime into a fresh directory. Run check-woven.mjs twice against it with MIXTURE_GPU_BACKEND=vulkan or dx12, MIXTURE_GPU_SOFTWARE=0 and MIXTURE_GPU_EXPECT_ADAPTER="GT 1030". The exact focused commands/environment are in the machine summary. Run each backend serially; retain heavy logs under ignored tmp/woven-acceptance/.

All 60 original Native/browser input PNGs, all nine screenshots (six sheets plus three roughness sanity images), every preview input receipt, generated HTML, review plan and index were SHA-256 verified **before copying**. No review image was regenerated or re-encoded. The [path mapping](./review/path-mapping.json) maps unchanged producer paths to retained paths. The recovered runner source matches the original runner hash, including its final CRLF. Scoped Git attributes preserve receipt bytes.

Bound inventory: **126 files / 21,973,504 bytes (excluding binding and these README files); largest file 961,337 bytes**. No file exceeds 4 MiB; no raw logs, archives or retention exception were added. Git retains the reviewed content, critical measurements, decisions and selected receipts. Complete new matrix PNG sets, ordinary logs, candidate archives and unselected CI contents remain temporary; hashes do not preserve omitted bytes. [CI artifact metadata](./ci/ci-artifacts.json) records expiry (2026-11-04 for the retained SwiftShader source artifact); no permanent external archive or full-run auditability is promised. OpenMixture maintainers own this retention.

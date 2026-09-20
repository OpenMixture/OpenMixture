# ENG-03 independent browser consumer evidence

English | [简体中文](./README.zh-CN.md)

On 2026-09-20, the candidate and exact public registry package each passed all eight browser checks (zero skipped, unexpected or flaky results). Both runs used clean consumer commit `20ad93c2cfcc6e64d9acc1349275d76403972ca8`. This later evidence/documentation commit is not the tested source. [The example](../../../examples/browser-consumer/README.md) defines the commands and coverage; [PR #28](https://github.com/OpenMixture/OpenMixture/pull/28) tracks integration and separate CI results.

| Run | Package engine revision | Build ID |
|---|---|---|
| [Candidate receipt](./candidate.json) | `20ad93c2cfcc6e64d9acc1349275d76403972ca8` | `sha256:a30e066c9b44ce1f418681d95d6508b5f5582d0492db4249ecbf3fd390cad5f8` |
| [Registry receipt](./registry.json) | `82b74707b2a8a998190e2f28b16f91fb9614486a` | `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28` |

Both packages identify runtime `0.1.0-alpha.0`; their archive and build identities differ. The candidate is unpublished. The receipts retain UTC times, source/verifier/package hashes, archive SHA-256/SHA-512, tool versions, browser options and completion counts. Exact installed dependency locks are retained as [candidate-lock.json](./candidate-lock.json) and [registry-lock.json](./registry-lock.json).

## Execution and limits

These were local Windows 11 x64 runs with explicitly selected Chrome `153.0.8010.48`, using `--enable-unsafe-webgpu --ignore-gpu-blocklist`. Browser reports expose `BrowserWebGpu` with redacted adapter name/vendor/device and `Other` device type. Hardware model and native driver/backend are unavailable; these results do not establish a hardware matrix or ordinary-profile support. CI independently uses its configured Chromium/SwiftShader environment.

The tests cover inert import and deferred WASM loading, CPU-only public operations, package-relative WASM at `/consumer/`, structured invalid-input/unavailable-GPU errors, exact checker/scalar pixels, selected channels and exposed overrides, independently owned output, busy/closing/destroy behavior, the example page and exclusion of test code from its production build. The retained [candidate render](./candidate-render.json) and [registry render](./registry-render.json) include the actual adapter/limits, allocation report, plan hash and all bytes for two 65×3 RGBA8 channels (780 bytes per channel). Exact expectations and requests remain in the source-bound browser test.

The [example screenshot](./example.png) was visually inspected by the implementing agent: the successful page shows a black/white checker and uniform roughness preview, channel labels, controls and a completion message confirming released GPU ownership. This is SDK example inspection, not human material-quality acceptance. Existing three-material goldens and the full pinned Studio qualification remain unchanged and required.

Before these accepted runs, the local cached Playwright Chromium executable failed at process launch with `spawn ...chrome.exe UNKNOWN`, before the runtime could execute. That attempt failed and is not counted above. Chrome was selected explicitly for new runs; the verifier has no silent fallback. This launch issue does not justify changing pixel tolerances or runtime behavior.

## Reproduction and retention

Use a clean checkout of the tested commit, the pinned toolchain described by the browser build guide, and fresh output paths. On this Windows host, the exact `wasm-bindgen` 0.2.128 executable was selected with `WASM_BINDGEN`; run:

```powershell
$env:MIXTURE_BROWSER_CHANNEL = 'chrome'
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

Each qualifier installs in a fresh OS temporary directory outside the checkout, verifies installed bytes, runs TypeScript checking, production build and eight browser tests, then removes successful staging. Registry consumption preserves its committed lock and fresh npm cache; it does not inherit candidate acceptance. No package was published by this work.

[files.json](./files.json) binds the retained receipts, locks, full render measurements and reviewed screenshot. Complete command logs, Playwright reports and archives are ordinary temporary local/CI output, not durably archived here. Their hashes identify bytes but cannot recover expired originals. The retained content supports this bounded acceptance; reproduction creates a new run, not the original artifact. Later CI results are bound to their own revisions and do not rewrite these local receipts.

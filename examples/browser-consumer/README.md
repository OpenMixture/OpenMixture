# Independent browser SDK consumer

English | [简体中文](./README.zh-CN.md)

ENG-03 provides a small engine-owned Vite/TypeScript example using only `@openmixture/runtime`'s public entry. It loads the included `.mix`, overrides exposed parameters, selects channels, renders through WebGPU and destroys the GPU instance in `finally`. Displayed pixels remain owned after destruction. It has no Studio dependency, engine-source import, Rust compilation step, alternate renderer or product editor.

## Run the published package

From this directory, with Node 24 and npm 11:

```sh
npm ci --ignore-scripts
npm run check
npm run build
npm run preview
```

Open the printed local address at `/consumer/`. Click **Render material**; the page loads `public/input.mix` and initializes WASM/GPU explicitly for that invocation. The committed lock installs exactly `@openmixture/runtime@0.3.0-alpha.0` from the registry, not `latest` or a source checkout. Use a secure context (localhost or HTTPS) and a WebGPU-capable browser. GPU acquisition failure is displayed with the SDK's structured diagnostics; there is no automatic fallback.

The fixture exposes `frequency` and `roughness`. The example intentionally uses these known exposed IDs; it does not duplicate the node catalog or build a general editor. `src/consumer.ts` is the short public-SDK workflow. The report shows package build identity, plan hash, effective exposed values and selected adapter. Its bigint-to-string formatting is host-owned display logic, not a replacement Mixture report schema. Scalar canvas display is a simple byte preview; it does not claim a color-managed material preview or PNG export.

## Browser checks

```sh
npx playwright install chromium
npm run check
npm run build
npm run test:browser
```

On Linux CI, install browser system dependencies with `npx playwright install --with-deps chromium`. The automated default uses the pinned Playwright Chromium and explicit `--enable-unsafe-webgpu --ignore-gpu-blocklist` flags. `MIXTURE_BROWSER_ARGS` may specify a JSON array of additional launch flags; CI uses its existing explicit Chromium SwiftShader policy. For an intentional local Chrome run, set `MIXTURE_BROWSER_CHANNEL=chrome` (PowerShell: `$env:MIXTURE_BROWSER_CHANNEL = 'chrome'`). No channel is silently retried or substituted. These are automated test profiles, not ordinary-browser or universal hardware qualification.

All 13 tests (including the resource cases below) must pass with zero skips/retries: inert import/GPU-free APIs and bundled identity; structured invalid-input diagnostics; exact 65×3 checker/scalar pixels; owned outputs through later renders/destruction; busy/closing lifecycle; explicit GPU-unavailable errors; static non-root UI rendering; and exclusion of the test host from the normal build. `test:browser` builds a separate `test-dist` containing the qualification host; the ordinary `dist` contains only the example. The analytic checker/constant expectations are test oracles, not a second material executor. Tests exercise real WASM and WebGPU; no mock pixel backend is used.

## Candidate and registry qualification

Run these producer commands from the engine root, using fresh output directories for every attempt:

```sh
node --test scripts/browser-runtime/consumer.test.mjs
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

The candidate requires a clean-source build at the current engine revision. The qualifier stages this example outside the repository in an OS temporary directory and changes only the runtime dependency in the copied manifest/lock to the supplied tarball. It preserves the frozen tool dependency graph, verifies archive SHA-256/SHA-512 and every installed package file, then compares `getBuildInfo()` from the bundled browser code with installed metadata. A stale, dirty or mismatched candidate fails. Nothing is published.

Registry mode keeps the committed manifest/lock unchanged, fetches the exact version using a fresh npm cache, verifies its archive against lock integrity, and records the published build identity separately. It does not expect the published package to have the candidate's revision or build ID. The registered bytes cannot inherit a new candidate's acceptance, or vice versa. Both modes run type checks, production builds and all browser checks without invoking Rust from the consumer. Temporary staging is outside the checkout, but is not an OS filesystem sandbox.

`qualification.json`, installed-file hashes, consumer-source hashes, copied lock, commands/logs, Playwright report, renderer/adapter evidence and screenshot are written to the requested output directory. Failed attempts stay failed and retain their staging path for diagnosis; successful staging is removed. Ordinary logs/screenshots stay in ignored local directories or finite-retention CI artifacts. The source-bound accepted summary follows the repository evidence policy.

## Coverage and release boundary

| Coverage | This consumer | Retained existing qualification |
|---|---|---|
| Exact package installation/build identity and public types | Candidate and registry, independently | Pinned Studio candidate/archive/lock checks remain |
| Basic SDK lifecycle, errors, channels, overrides and ownership | Nine direct tests, tiny checker/scalar workload | Existing wider browser contract and lifecycle tests remain |
| Pixel/material quality | Exact checker/constant expectations | Three materials, 11 cases, 44 channels, v2 gates, stress and native comparisons remain in the existing workflow |
| Deployment | Static Vite assets at `/consumer/`; test host excluded from normal build | Existing pinned product deployment/export checks remain |
| Product UX / upgrades / trials | Not covered | Owned by Studio; not assigned to this engine task |

The existing `Chromium WebGPU material matrix` job runs both independent modes **in addition to** its pinned Studio qualification. No existing check, material case or required-check name is removed. This is an SDK acceptance entry, not replacement evidence for the full supported material/browser matrix.

The npm Alpha is published within its recorded scope. Public Rust APIs are consumable from source/local Cargo archives; Rust crates remain unpublished, and this example does not introduce CLI binary distribution. A future browser release needs a new version and archive identity, engine qualification, explicit publication, then clean exact-version registry consumption. Studio may choose its own upgrade/deployment schedule. A version change requires updating this fixture's exact package/lock and compatibility expectations in a reviewed change.

ENG-04 adds a ninth test: both the candidate and exact registry 0.3.0-alpha.0 render the two-Scalar fixture at four 1K weights. Historical 0.1.0-alpha.0 unknown-type rejection evidence remains in the ENG-04 record. Only staged runtime version/archive/integrity change for candidate installation. See [ENG-04](../../docs/eng-04-scalar-blend.md) and the [new release record](../../docs/evidence/npm-030-alpha/README.md).

M6A-04 candidate mode requires 13 tests: the original nine plus four resource tests for synchronous capture, offset views, invalid buffers, lifetime and the frozen M6A-03 1K image/noise composition. Registry 0.3.0-alpha.0 also requires all 13; neither mode permits skips. `check-resources.mjs` independently compares Native with the fixed maximum component delta ≤1; passing interfaces do not override [scope and the known hardware failure](../../docs/m6a-04-browser-resources.md).

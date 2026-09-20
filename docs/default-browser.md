# Ordinary desktop browser qualification — ALPHA-03

English | [简体中文](./default-browser.zh-CN.md)

This procedure qualifies a specific browser/OS/driver and runtime archive with ordinary GPU settings. It is separate from controlled Chromium CI and from publishing an Alpha. The [recorded result](./evidence/browser-quality-v2/README.md) defines the actual supported scope; a browser version alone is not a universal hardware guarantee.

## Inputs and launch policy

Use the exact archive already qualified by ALPHA-01, including its producer receipt and SHA-256. Stage it in the pinned independent consumer with [candidate verification](./browser-materials.md), use `npm ci`, and verify installed bytes. Keep the original product commit and every unrelated lock entry fixed. The retained candidate may be older than a later documentation or verification-tool commit; do not identify those as the tested runtime.

The Windows [launcher](../scripts/browser-runtime/launch-default-browser.ps1) starts the installed desktop browser in a new disposable profile. It adds only `--user-data-dir` and a loopback CDP port, plus `about:blank`. It does not add headless, unsafe-WebGPU, blocklist, ANGLE/software, sandbox or feature overrides, and does not change a personal profile, driver or browser setting. CDP is test transport, not an end-user prerequisite. The [verifier](../scripts/browser-runtime/default-browser.mjs) checks both the observed OS command line and the browser's own command line; only empty Chromium flag-switch markers and surrounding whitespace are normalized. For Edge's internal relaunch, the verifier additionally binds the directly observed child executable/parent PID and the CDP browser PID; only its observed `--edge-skip-compat-layer-relaunch` marker is accepted, never injected. Other extra flags or a reused launch directory fail the guard.

Record the OS/build, installed browser version/executable digest, complete launch arguments, browser GPU information and exposed runtime adapter evidence. Unavailable/redacted fields remain unavailable. The observational `requestAdapter` wrapper forwards unchanged options and the original adapter; failure injection happens only in separate pages and is labeled synthetic.

## Reproduce

The consumer needs its pinned Node/npm/Playwright dependencies. A real desktop browser and desktop session are required; Linux headless CI is not a substitute for this host qualification.

1. Prepare the candidate consumer and `candidate.json` using `candidate.mjs stage`, install with `npm ci`, and run `candidate.mjs installed` as described in the material guide.
2. Prepare fresh native references from the same runtime implementation on the target host. Use an explicit native adapter/backend policy, confirm the selected adapter, and retain the existing drift check against the candidate's engine revision. For the recorded Windows NVIDIA host:

```powershell
$env:MIXTURE_GPU_BACKEND = 'dx12'
$env:MIXTURE_GPU_SOFTWARE = '0'
$env:MIXTURE_GPU_EXPECT_ADAPTER = 'NVIDIA GeForce GT 1030'
node scripts/browser-runtime/prepare-materials.mjs tmp/default-native ec571816026a945a706067769fef76e24c8398b0
./scripts/browser-runtime/launch-default-browser.ps1 -OutputDirectory tmp/default-launch
```

3. In the independent consumer, build the test entry using `npx vite build --mode browser-test`, then run `node scripts/static-server.mjs` in a separate terminal. The static server exposes only the product's `dist` at `http://127.0.0.1:4173/player/`. Loopback is the secure context used in this record; public deployment still needs the serving/security requirements in the [browser guide](./browser-runtime.md).
4. From the engine checkout, use absolute paths and new output directories:

```bash
node scripts/browser-runtime/default-browser.mjs run /absolute/product /absolute/candidate-evidence /absolute/native-reference /absolute/launch/launch.json /absolute/new-browser-output
```

The runtime gate checks actual build identity, all 11 existing 1K cases/four channels, stable plan identity, zero reported live allocation after readback, owned pixels after later renders/destruction, idempotent destruction, completion of accepted in-flight work, and rejection of new work after destruction. The same frozen `browser-material-check` enforces structure, causality, relationships and pixel tolerances. The verifier never changes a native golden, tolerance or previous report.

5. In the product, run `npm run build` to replace the test assets with normal production assets, keeping the static server running. Then run:

```bash
node scripts/browser-runtime/default-browser.mjs production /absolute/product /absolute/candidate-evidence /absolute/launch/launch.json /absolute/new-production-output
```

This checks explicit Player initialization/render/disposal, download of a 65×3 checker with exact independently decoded RGBA bytes and sRGB metadata, and absence of the test harness from production. It retains the PNG, a screenshot and a receipt. Close only the qualification browser/profile after collecting evidence; never close a personal browser session.

## Firefox and native reference configuration

Launch Firefox with `-Family firefox -BrowserPath <firefox.exe>`. Only a fresh profile, loopback BiDi port and blank page are added. The verifier binds version, headed state, profile and actual child-process PID/command line to the launch record. `run` supports Firefox; the actual download check in `production` currently requires Chromium and does not certify Firefox Player downloads.

Successful GPU acquisition/rendering followed by a pixel comparison failure does not mean WebGPU is unsupported. Native references also depend on build profile and the actual shader compiler. `prepare-materials.mjs` keeps `debug` as its default; set `MIXTURE_NATIVE_PROFILE=release` explicitly when needed. On Windows DX12, `MIXTURE_DX12_COMPILER_DIRECTORY` prepends a directory containing `dxcompiler.dll` only to the native child PATH. The manifest records profile, requested DLL path/digest, script and executable digests. Separately record the actual loaded process module; PATH configuration alone is not loading evidence. These are developer reference-tool settings; browser users do not need Rust or DXC configuration.

```powershell
$env:MIXTURE_NATIVE_PROFILE = 'release'
$env:MIXTURE_DX12_COMPILER_DIRECTORY = '<verified Firefox directory>'
node scripts/browser-runtime/prepare-materials.mjs tmp/configured-native <full-candidate-engine-revision>
```

Keep existing backend/adapter requirements and the source-drift check, and use fresh reference/browser output directories. Preserve previous failures, goldens and tolerances. A result certifies the recorded configuration pair, not bitwise agreement across every compiler.

## Failures and evidence boundaries

The negative pages separately inject absent WebGPU, a null adapter and rejected device creation. They must retain `MIX_BROWSER_WEBGPU_UNAVAILABLE`, `MIX_GPU_ADAPTER_UNAVAILABLE`/`gpuAdapter`, and `MIX_GPU_DEVICE_REQUEST_FAILED`/`gpuDevice`, respectively. Engine failures keep actionable suggestions and CPU document validation remains usable. These are controlled diagnostic probes, not natural unsupported-host coverage or a fallback pixel executor.

Keep failed runs intact. Comparing a hardware browser to references from a different software/hardware adapter is a distinct portability experiment; it must not silently redefine the same-host browser/native acceptance. Retain any such failed comparison and use fresh, source-matched references/output directories for the intended host pair. Never widen tolerances to close this gate.

The three Node guard tests run in browser CI; they do not launch or certify an ordinary desktop browser there. Run `node --test scripts/browser-runtime/default-browser.test.mjs` and `cargo xtask check` for verifier changes. A later candidate, OS/browser/driver change, different browser, custom profile/extension/policy, public deployment or full Studio saved-file workflow needs its own evidence. Follow [evidence retention](./evidence-policy.md) for accepted records, temporary full outputs and archive expiry.

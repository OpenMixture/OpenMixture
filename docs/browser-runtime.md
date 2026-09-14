# Browser runtime and first independent consumer

English | [简体中文](./browser-runtime.zh-CN.md)

**Player update, 2026-09-14:** the M5-04 parameter/preview slice is implemented and [locally verified](https://github.com/OpenMixture/Studio/blob/7370e482e2dcacb9911f5663f8ec4f9e8da6a4cc/docs/evidence/m5-04-parameters/README.md) in the independent product: Rust-metadata controls, channel selection, one active render plus one replaceable pending request, stale-preview diagnostics and lifecycle cleanup. The clean isolated consumer passed 23 Chromium checks and six Node tests, including three-material 128×128 previews. PNG export and full M5-04/M5-05 acceptance remain open; the runtime archive is unchanged and unpublished.

**Local acceptance update, 2026-09-14:** M5-02/M5-03 initial browser execution and isolated package consumption now pass the [recorded gates](./evidence/m5-02-03/README.md). The unchanged archive passes 13 Chromium checks, including controlled real-device loss, mapping cleanup and repeated independent modules/devices. M5-04/M5-05 material regression, complete Player workflows and the accepted browser CI matrix remain open; the package is unpublished.

M5-02/M5-03 now provide a thin WASM binding, a complete local `@openmixture/runtime@0.1.0-alpha.0` tarball and an independent [Studio product repository](https://github.com/OpenMixture/Studio) with a minimal Player. The package has not been published to npm. This is the checker execution/consumption slice, not full M5 acceptance or a node editor. The [M5 plan](../M5_PRS.md) and [delivery contract](./browser-sdk.md) retain the remaining scope.

## Engine build

Use the repository Rust toolchain and an exact matching wasm-bindgen CLI. This checkpoint uses Rust 1.98.1, wasm-bindgen 0.2.128, browser target `wasm32-unknown-unknown`, Node 24.20.0 and npm 11.19.0. The product locks its own Vite/TypeScript/Playwright versions.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
cargo check --locked -p mixture-wasm --target wasm32-unknown-unknown
node --test packages/runtime/test/runtime.test.mjs
node scripts/browser-runtime/build.mjs
```

`WASM_BINDGEN` may name an explicit CLI executable. The build refuses a mismatched version. It compiles the locked Rust source, generates bindings, assembles public JS/types/WASM and licenses, and runs `npm pack` under `target/browser-runtime/`. The directory contains the archive, SHA-256 sidecar and `receipt.json` identifying source content, build ID and tools. No consumer installation compiles Rust.

The generated no-modules wasm-bindgen glue is enclosed in an ESM factory. Each explicit `loadRuntime` obtains independent binding/module state; importing the public entry does not load WASM or acquire GPU state. This packaging detail does not add graph or shader semantics. Handwritten public TypeScript declarations ship from the same build and are checked by the independent consumer; they are not described as automatically generated Rust API declarations.

## Public usage

```typescript
import { loadRuntime } from '@openmixture/runtime';

const runtime = await loadRuntime();
const inspection = runtime.inspect(source, {
  size: [65, 3], channels: ['baseColor'],
});
const gpu = await runtime.createGpu();
try {
  const result = await gpu.render(source, {
    size: [65, 3], channels: ['baseColor'],
  });
  const pixels = result.channels[0].pixels;
  // The consumer owns these RGBA8 bytes, including after gpu.destroy().
} finally {
  await gpu.destroy();
}
```

The product supplies raw string/UTF-8 source; it must not parse and reserialize user files before Rust validation. `getNodeCatalog`, `getBuildInfo`, `validate` and `inspect` run after explicit WASM loading without requesting a GPU. `validate` returns diagnostics for expected document/request failures; `inspect` rejects invalid input through a structured SDK error. The initial validation result includes compiled request and binding metadata.

`loadRuntime({ wasm })` accepts a URL/string or owned byte input; omitted `wasm` uses the packaged asset URL. Each GPU instance admits one render; another concurrent request rejects with `MIX_BROWSER_RUNTIME_BUSY`. `destroy` stops new work, waits for accepted work and explicitly releases the device without invalidating returned JS pixels. Browser callback waiting yields to the event loop. Native blocking waits keep their native behavior; no browser hard deadline, automatic recovery or alternate renderer is promised.

Core/GPU diagnostics preserve their codes and context. `MIX_BROWSER_BINDING_FAILED` reports an unexpected binding/trap failure separately from invalid arguments; browser loading, build mismatch, unsupported WebGPU, busy and destroyed errors remain separate. Inspect [public declarations](../packages/runtime/src/index.d.ts) for the precise JS projection: u64/usize data use bigint, while u32 fields such as pipeline hits/misses use number. Empty optional diagnostic evidence may be absent.

## Independent product and verification

The product installs the actual archive under `vendor/` using a relative file dependency and committed lock. Its provenance receipt identifies the consumed archive and engine build. The initial vendor artifact is an unreleased build retained for standalone reproducibility, not an npm publication. Package upgrades replace the actual archive and update its integrity/receipt; a producer checkout, symlink or absolute developer path is not a runtime dependency.

The product README defines `npm ci`, `npm run check`, `npm run dev` and `npm run test:browser`. The browser command builds the test consumer, serves the production output at the non-root `/player/` base, and uses the locked full Chromium browser with explicit WebGPU flags. Missing GPU access fails the tests rather than skipping them. Browser-only tests are not silently folded into native `cargo xtask gpu-smoke`.

The initial browser checks cover GPU-free module operations, invalid raw source, loader failures, URL/byte asset loading, exact odd-sized checker pixels, request input capture, busy rejection, repeated rendering, retained output after destruction, and the visible Player file/render/dispose workflow. These complement native checker/readback/device-loss regressions; they do not certify all M5 material cases or all browsers.

Before merging engine changes, run `cargo xtask check`, the affected shader checks and explicit native GPU regression under the existing required CI policy. The [browser build workflow](../.github/workflows/browser-runtime.yml) separately verifies WASM compilation, JS wrapper tests and archive generation; it does not claim real browser GPU execution. The product owns the independent real-browser tests.

## Remaining acceptance

- Full three-material 1K browser/native comparisons and reviewed tolerances, tiling and parameter causality.
- Spontaneous device/driver loss and unexpected platform-event coverage, broader lifecycle stress and the documented browser CI acceptance environment. Controlled real-device destruction and mapping cleanup are now covered by the M5-02/M5-03 record.
- Complete the Player PNG export workflow, including independently decoded bytes and metadata. Parameters, channel selection and latest-request previews are covered by the M5-04 first-slice record above.
- Full M5 acceptance, stable support claims, registry release, Studio authoring and M6 resource packaging.

Record exact tested sources, archive identity, browser/OS/adapter, commands and limitations under the existing [evidence policy](./evidence-policy.md). A successful native run, WASM compile, archive build or development server does not substitute for browser execution.

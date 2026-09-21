# @openmixture/runtime

English | [简体中文](./README.zh-CN.md)

A browser ESM runtime built from `mixture-core` and the sole `mixture-wgpu` executor. The published Alpha is `@openmixture/runtime@0.2.0-alpha.0`; see the [release record](https://github.com/OpenMixture/OpenMixture/tree/main/docs/evidence/npm-020-alpha) for exact archive identity, tested environments and limitations. Install with `npm install --save-exact @openmixture/runtime@0.2.0-alpha.0`. It adds no TypeScript renderer, hidden device, worker or fallback.

```ts
import { loadRuntime } from '@openmixture/runtime';

const module = await loadRuntime();
const validation = module.validate(source);
if (!validation.ok) throw validation.diagnostics;
const gpu = await module.createGpu({ powerPreference: 'high-performance' });
try {
  const result = await gpu.render(source, { size: [65, 3], channels: ['baseColor'] });
  const { pixels, encoding } = result.channels[0];
  // pixels is owned Uint8Array RGBA8; encoding describes its transfer function.
  console.log(result.plan.hash, result.report.allocations.liveBytes, pixels, encoding);
} finally {
  await gpu.destroy();
}
```

`import` only evaluates inert facade constants. `loadRuntime()` imports packaged glue and fetches one package-relative `wasm/mixture_wasm_bg.wasm` asset, then instantiates an independent WASM module. Vite rewrites the static asset URL in production. Supply `{ wasm: new URL('/assets/runtime.wasm', location.href) }` or `{ wasm: bytes }` to override it; bytes cause no loader fetch. URL strings resolve against the page URL. The loader buffers fetched bytes and uses WebAssembly byte instantiation, so it does not require streaming's `application/wasm` MIME; correct HTTP, CORS and a CSP permitting WebAssembly are still required. It never retries or fetches materials. Browser WebGPU requires a supported browser and secure context.

After loading, `validate(source, request?)`, `inspect(source, request?)`, `getNodeCatalog()` and `getBuildInfo()` are synchronous and GPU-free. `validate` returns `{ ok: false, diagnostics }` for core failures; `inspect` throws. JS representation errors throw `MixtureRuntimeError`; async initialization/render errors reject with it. The error exposes `operation`, `code`, unchanged engine `diagnostics`, and separate `browserFailure`/`evidence` when applicable. Browser codes are `MIX_BROWSER_WASM_LOAD_FAILED`, `MIX_BROWSER_BUILD_MISMATCH`, `MIX_BROWSER_WEBGPU_UNAVAILABLE`, `MIX_BROWSER_INVALID_ARGUMENT`, `MIX_BROWSER_RUNTIME_BUSY` and `MIX_BROWSER_RUNTIME_DESTROYED` and `MIX_BROWSER_BINDING_FAILED` (unexpected binding traps/failures).

Source is a string or ordinary `Uint8Array`; raw UTF-8 reaches Rust without JS JSON parsing. Strings reject unpaired surrogates; bytes are checked by the Rust UTF-8 parser. Inputs and arrays are captured before asynchronous work. Options/overrides require plain objects and own enumerable data properties; no getters, class instances, undefined values, sparse arrays or coercion. Requests support `size: [width, height]`, unique `channels`, exposed-ID `overrides` and partial `limits`. Size defaults to `[64,64]`, channels to `['baseColor']`. Every explicitly supplied limit is a nonnegative u64 `bigint`; omitted fields use defaults obtained from the Rust binding, with no duplicate JS policy table. JS integer overrides are serialized as integer tokens, while original `.mix` numeric tokens remain strict. Rust decides ranges, bindings, all-branch validation and cross-parameter constraints.

Successful validation/inspection includes the plan, material channels and exposed binding metadata: the Rust parameter contract, resolved source value and effective override value. Catalog ports distinguish inputs from outputs by their list; an output with `default: null` is not a required input. No metadata is returned for invalid documents.

Each GPU instance accepts one render at a time. Busy calls reject. `destroy()` stops acceptance immediately, awaits the accepted render's settlement and cleanup, and releases the device; repeated calls share a promise. It does not cancel GPU work or detach previous results. Requested channels are returned completely or the render fails. Pixels are copied from Rust into independent JS-owned arrays, tightly packed top-left RGBA8 with straight alpha. Color RGB is sRGB; scalar and encoded-normal bytes are linear data. Product PNG export must preserve these encodings.

Typed Rust u64 and usize report fields project as `bigint`; typed u32 fields and parameter values remain JS numbers. For example, `plan.estimates.peakBytes` and `report.allocations.liveBytes` are bigint, while dimensions, plan versions and pipeline cache hit/miss counts are numbers. Plain `JSON.stringify` cannot serialize bigint; a product may encode them explicitly in its own logs. The exact public surface is in `src/index.d.ts`; JS, declarations, WASM and `build-info.json` share a checked build identity.

Producer commands, run from the engine repository:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
node --test packages/runtime/test/runtime.test.mjs
node scripts/browser-runtime/build.mjs
```

Set `WASM_BINDGEN` to an explicit CLI path when needed. The build requires the repository-pinned Rust, Cargo.lock and wasm-bindgen 0.2.128. It emits a real `target/browser-runtime/openmixture-runtime-0.2.0-alpha.0.tgz`, SHA-256 sidecar and receipt with exact tool versions and source build identity. `engineRevision` identifies HEAD and `engineDirty` records whether source changes were present; the build ID additionally covers source, repository compilation settings, the actual compiler/binding versions and explicit target/profile flags. The source tree package is not the distributable: generated JS/WASM/build metadata exist only in the staged archive. Consumer installation uses `npm install ./vendor/openmixture-runtime-0.2.0-alpha.0.tgz` with no Rust, engine source, compilation hook or network CDN dependency. Keep the consumer's package lock.

The Node tests use a fake low-level binding solely for public request/lifecycle checks. A successful build or these tests do not demonstrate real WebGPU rendering, native/browser pixel equivalence, PNG export fidelity, device-loss delivery or all-browser support. Those require the independent product's served production tests and retained evidence. Node GPU/SSR rendering, an editor, resource packaging, cancellation and zero-copy textures remain unsupported.

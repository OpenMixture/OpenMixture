# M6B-04 — CLI and browser asset adapters

English | [简体中文](./m6b-04-adapters.zh-CN.md)

**Current status:** this implementation is merged into main with M6-B; M6B-05 has completed [comprehensive qualification](./m6b-05-qualification.md) within its recorded scope. Follow-up tasks mentioned in this implementation-slice account were the plan at that checkpoint; see [release status](./release.md) for versions and hardware scope.

Implemented on the review branch, 2026-09-22. The [shared CPU codec](./m6b-03-cpu-assets.md) now serves explicit local-file CLI commands and public browser byte APIs. Versions remain unpublished Rust 0.5.0 / browser 0.5.0-alpha.0. M6B-05 owns combined pixel qualification and milestone closeout; this slice does not change shaders, the source format, package v1, or loose-input APIs.

## CLI files

```bash
mixture asset pack material.mix --size 65x3 --image Input input.rgba --out material.mixpack --json
mixture asset inspect material.mixpack --json
mixture asset inspect material.mixpack --plan --size 65x3 --output height --json
mixture asset render material.mixpack --size 65x3 --output height --out rendered --json
```

`pack` accepts repeated `--image <logical-id> <file>` bindings for the full default document resource set, including unused images. Files contain exact, tightly packed `rgba8-linear` bytes at the same positive `--size`; there is no PNG decoder, adjacent-file discovery, or network lookup. Default size is 64×64. Source bytes are preserved. Duplicate IDs, missing/extra bindings and incorrect lengths fail. Output uses exclusive creation, so an existing asset is never overwritten. A failed write can leave a partial new file; inspect it before use.

Input must be an explicit regular file, not a directory or symlink. The adapter bounds metadata length before allocation, then detects truncation/growth during reading. Archive entries are never extracted. Commands are explicit: extensions do not choose behavior, and moving/renaming an asset does not change its identity. Use `--` before an input path starting with a dash.

`inspect` validates the whole package without acquiring a GPU. `--plan` additionally prepares the selected Core plan; `--size`, `--output` and `--set id=JSON` require it. Default output is `baseColor`. `render` prepares first, releases transport storage, then uses the existing wgpu executor and PNG writer. Existing backend, software and power-preference flags apply only to render. A malformed asset fails before GPU acquisition or output directory creation. Resource-reference overrides, even unchanged values, fail with `MIX_PACKAGE_RESOURCE_OVERRIDE`; valid ordinary overrides retain Core semantics.

All three commands accept lower-only unsigned decimal `--package-bytes`, `--manifest-bytes` and `--package-buffer-bytes`. Defaults/ceilings are 67 MiB, 64 KiB and 202 MiB. CLI loading charges actual archive capacity plus source/manifest scratch and selected captured resources. Authoring also reserves caller-retained raw image buffers before invoking the shared writer. These are byte-buffer budgets, not process RSS limits.

JSON asset envelopes use schema 1 with `operation`, `input`, `ok`, `diagnostics`, and nullable `asset`, `plan`, `render`. Pack adds `output` and `writtenBytes`; render nests the existing schema-2 rendering report, including adapter, PNG and partial-write diagnostics. JSON u64 values remain numbers; JavaScript consumers needing exact u64 values should use the browser API. Exit codes: 0 success, 1 I/O/GPU failure, 2 invalid invocation/package/request. Invocation syntax errors go to stderr as in existing commands. Core diagnostic codes/stages are preserved; package failures have stage `package` and typed evidence (including the resource override ID array).

## Browser bytes

```typescript
import { loadRuntime } from '@openmixture/runtime';
const runtime = await loadRuntime();
// The host obtains bytes; the SDK never resolves asset paths or URLs.
const asset = runtime.inspectPackage(bytes, {
  packageLimits: { packageBytes: 10n * 1024n * 1024n },
});
const gpu = await runtime.createGpu();
try {
  const result = await gpu.renderPackage(bytes, {
    size: [65, 3], channels: ['height'],
    packageLimits: { packageBufferBytes: 32n * 1024n * 1024n },
  });
  // result.channels owns its pixels, also after destroy().
} finally { await gpu.destroy(); }
```

`RuntimeModule.inspectPackage(Uint8Array, PackageOptions?)` is synchronous and CPU-only after `loadRuntime()`. `PackageOptions` contains optional `limits`, `resourceLimits`, `packageLimits`; it does not accept render options. `GpuRuntime.renderPackage(Uint8Array, PackageRenderRequest?)` accepts existing size/channels/overrides/limits/resourceLimits and packageLimits, but never loose `resources`. Partial package limits are merged with Rust-authoritative defaults. All package byte limits and inspection u64 fields are `bigint`; the API schema remains 2 and the package report schema is 1.

Inspection includes exact package/source SHA-256 and byte lengths, source version, sorted resource identities/dimensions/row sizes/hashes, and `buffers.jsPackageBytes`, `rustPackageBytes`, `chargedBytes`. It validates all stored resources without selected Core pixel capture. No GPU acquisition occurs, even when `navigator.gpu` throws.

Options must be plain objects with own enumerable data properties and known fields. Bytes must be an ordinary Uint8Array over a fixed ArrayBuffer; shared, resizable, detached, proxy and subclass views are rejected. Accessor/own overrides of relevant view properties are rejected; irrelevant properties are not read. Offset views copy only their visible range. Acceptance snapshots bytes and options synchronously before yielding; later caller mutation cannot affect rendering. The package and loose render APIs share one busy slot, rejection recovery and idempotent destroy. Closing/busy calls fail before reading inputs.

The JS snapshot and Rust Vec are both real copies. A Uint8Array binding avoids an additional implicit wasm-bindgen slice copy. The ledger is `JS P + Rust capacity(P) + D + M + selected S`; policy and `2P` are checked before copying, then actual capacity and scratch/capture are checked by Rust. Transfer storage is released before asynchronous GPU rendering. `AssetLimits::with_retained_bytes` reserves adapter buffers in the same codec policy; `AssetView::loading_buffer_bytes` reports loading charges. Structured package failures preserve their code, stage and evidence through `MixtureRuntimeError`; browser argument/busy/destroy codes remain unchanged. Importing the normal npm bundle stays inert.

## Reproduction and scope

```bash
cargo test --locked -p mixture-asset
cargo xtask test-consumer
cargo xtask check
npm test --prefix packages/runtime
node --test scripts/browser-runtime/consumer.test.mjs
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/check-assets.mjs
# Requires a clean, matching source/archive identity and a fresh output directory:
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/m6b04-browser-consumer
```

The CPU WASM verifier imports only the public built package, tests all 22 codec corpus entries and exact transfer budgets, and can accept additional archive paths after the package-directory argument for larger copy measurements. Its report is `tmp/m6b04-wasm-assets.json`. The independent browser fixture is [asset.mix](../examples/browser-consumer/public/asset.mix) with [asset.mixpack](../examples/browser-consumer/public/asset.mixpack): 65×3 raw pixels repeating `[128,37,91,255]`, authored by the CLI command above with ID `Input`. Height output must repeat `[128,128,128,255]`; package and loose plan hashes must match.

Candidate browser qualification runs 15 tests, adding CPU package inspection and real WebGPU lifecycle/ownership/budget rendering. Registry 0.3.0-alpha.0 qualification retains its 13 tests because that published version has no package API. Native source and isolated archive CLI consumers author/move/inspect the asset and reject bad bytes before GPU acquisition; the existing explicit GPU consumer suite additionally verifies package pixels. Existing material gates remain required. M6B-05 still owns the full four-weight, rectangular-control, repeated-load, Native/browser package matrix and all-six-check closeout. No publication, Studio changes, node migration or platform support expansion is included.

Local results and their exact tested revision: [verification receipt](./evidence/m6b-04/README.md).

# M6A-04 — Browser resource interface

English | [简体中文](./m6a-04-browser-resources.zh-CN.md)

The unpublished `0.3.0-alpha.0` candidate/API schema 2 now accepts image resources in `validate`, `inspect` and GPU `render`. This implements the browser boundary of the [resource contract](./m6a-resource-contract.md), depending on [M6A-03 execution](./m6a-03-native-resources.md). [M6A-05](./evidence/m6a-05/README.md) now retains bounded software qualification; publication remains separate; the published `0.2.0-alpha.0` package has no resource entry point.

## Public request

```ts
const request = {
  size: [2, 2] as [number, number],
  channels: ['height', 'normal'] as ('height' | 'normal')[],
  resources: [{
    id: 'heightSource', width: 2, height: 2,
    format: 'rgba8-linear' as const, bytesPerRow: 8,
    data: new Uint8Array([0,11,22,0, 64,33,44,1, 128,55,66,2, 255,77,88,3]),
  }],
  resourceLimits: { resourceCount: 8n, resourcePixels: 16777216n, resourceBytes: 67108864n },
};
const inspection = runtime.inspect(source, request);
const gpu = await runtime.createGpu();
try {
  const pending = gpu.render(source, request);
  request.resources[0].data.fill(0); // The accepted render already owns its snapshot.
  const result = await pending;
  // result.plan.hash equals inspection.plan.hash; pixels remain owned after destroy.
} finally { await gpu.destroy(); }
```

`runtime` is the result of public `loadRuntime`; `source` is the [height fixture](../fixtures/nodes/image-input/height.mix). The [installed browser tests](../examples/browser-consumer/tests/resources.spec.mjs) contain executable examples. Resources default to an empty array; resource limits merge with Rust defaults. Limits are u64 bigint values, while dimensions and `bytesPerRow` are exact safe JS integers. Resource metadata in returned plans uses bigint for its Rust u64 stride. The array preserves duplicate IDs for Core rejection.

Only ordinary `Uint8Array` views over non-shared, non-resizable, attached storage are accepted. Copy only the view's offset/length. Unsupported request keys, accessor properties, holes, class instances and invalid numeric representations fail through `MIX_BROWSER_INVALID_ARGUMENT`. Busy/closing checks precede resource access. Core owns ID/format/size/stride/length, selected completeness, overrides, budgets, deterministic identity and diagnostics. `validate` returns semantic failures; `inspect` throws and `render` rejects with the same engine diagnostics. A failed preparation performs no GPU work and leaves the instance available.

## Synchronous capture and lifetime

The TypeScript facade captures data descriptors and byte views without copying their pixels. The synchronous WASM preparation invokes Core `prepare_from` with `AdapterImageBinding`/`ImageData`. This small adapter surface declares length and copies into a Core-allocated destination; all resource validation completes before any selected pixel copy. Unused bindings are checked but never copied. Core computes the digest from the resulting bytes and creates the same opaque `PreparedRender` used by Native. It does not trust caller digests or expose mutable snapshots.

For the browser implementation, each selected resource requires one engine-owned packed copy, below the contract's two-copy ceiling. `Uint8Array.copy_to` writes directly into the Core destination; no serialized pixel arrays or intermediate transfer Vec are built. No JS callbacks or asynchronous yields occur between validation and capture. Later caller mutation, detachment or metadata replacement cannot alter the accepted request. The internal WASM preparation handle is consumed by the asynchronous render and never exposed through the public package. Per-render snapshots and GPU descriptors are released on success/error; `destroy` waits for active work, remains idempotent, and preserves returned owned pixels.

## Verification

Run `npm ci --prefix packages/runtime --ignore-scripts`, `npm test --prefix packages/runtime`, `cargo xtask test-core`, `cargo xtask test-plan`, `cargo xtask check`, and the Native image/lifetime gates from M6A-03. Build the clean candidate with `node scripts/browser-runtime/build.mjs`, then:

```text
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/check-resources.mjs tmp/sdk-candidate tmp/sdk-resource-comparison
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

Output directories must be fresh. Candidate consumption requires all 13 browser tests, including four resource tests; exact registry consumption retains its nine existing tests. Neither mode permits skipped/flaky/failed tests. The installed candidate exercises source-identical Native/browser 1K height/normal at weights 0, 0.25, 0.5, 1, literal byte/orientation tests, 1×1/1×256/256×1/65×3 inputs, alpha independence, offset views, mutation/detachment after acceptance, rejection recovery, busy/destruction and descriptor cleanup. The Native comparison requires maximum component delta ≤1 for all eight channels; existing Scalar/material gates are unchanged. Node tests also reject SharedArrayBuffer independently of browser cross-origin-isolation availability.

Normal and height PNGs produced by the test canvas are comparison artifacts, not a new public encoding API. Full routine reports/images live in local ignored outputs or CI artifacts under the [evidence policy](./evidence-policy.md). M6A-05 owns final retained release acceptance and wider platform claims. No CLI image decoder, Studio change, resource cache, new shader, package publication or second executor is included.

## Qualification boundary

The Linux pinned-software CI matrix is the integration gate. Local Windows Chrome public-interface tests pass, but that does **not** qualify its hardware path for the fixed Native/browser component limit: the frozen M6A-03 input produced normal maxima of 4 at weight 0.5 and 8 at weight 1 on the measured Native NVIDIA GT 1030/Vulkan versus Chrome WebGPU route. Height maxima were ≤1 and imported-only weight 0 matched exactly. Weight 1 also equals the direct procedural endpoint in each runtime, so this failure is present without imported pixels contributing to the output. It must remain a failed comparison, not be absorbed by material tolerances or a new baseline. M6A-05 must resolve or explicitly scope this hardware precision limitation before claiming that route; no universal cross-adapter qualification is asserted here.

[Bounded software success and hardware failure receipts](./evidence/m6a-04/README.md) are retained separately; neither substitutes for the other.

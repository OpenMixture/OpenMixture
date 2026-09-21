# M6A-03 — Native image upload and execution

English | [简体中文](./m6a-03-native-resources.zh-CN.md)

M6A-03 implements Native execution under the [resource contract](./m6a-resource-contract.md), on top of [M6A-02](./m6a-02-core-resources.md). Its PR's six required checks gate integration. Browser resource arguments remain M6A-04; cross-platform image qualification remains M6A-05. No package is published here.

## Public API and ownership

Call `mixture_core::prepare` synchronously with borrowed packed RGBA8 bytes, then `renderer.render_prepared(&prepared).await`. Core owns immutable snapshots paired with the plan. Mutation or disposal of caller bytes after preparation cannot affect rendering. Retaining a prepared request intentionally retains CPU snapshots. Each render uploads fresh resources; the renderer and returned output do not retain snapshots. Output pixels survive request and renderer destruction. The [independent consumer](../examples/native-consumer/tests/resources.rs) is a complete example.

Ordinary `render(&plan)` rejects selected image invocations before allocation, pipeline lookup or submission with `MIX_RESOURCE_MISSING`, resource ID, plan hash and adapter evidence. No late binding, trusted caller digest, mutable snapshot, image cache or fallback executor is introduced.

## GPU execution and accounting

Each selected ID creates one non-sRGB `rgba8unorm` texture and one explicitly owned staging buffer. Packed rows are copied into 256-byte-aligned mapped staging without another packed CPU copy. Checked copy submission completes before compute. Several nodes can bind the same image; unused bindings are not uploaded. The per-call guard retains uploads and explicitly destroys them on success, failure or future drop, preserving the conservative retain-all schedule.

The sole image WGSL kernel uses integer `textureLoad`, reads R and stores `(value,0,0,1)` through `mixture_half4`. Its reserved zero uniform is part of the fixed ABI and referenced by the entry point. There is no sampler, gamma transfer, alpha multiplication, resampling or seam repair. Existing wrapped height-to-normal differences apply. The cache is bounded by eleven kernel identities, independent of pixel content.

Upload failures preserve the first structured GPU error/source chain and add `operation=resourceUpload` and `resourceId`. Existing adapter/loss/cleanup evidence remains. Device unavailability before upload retains the existing early-loss diagnostic.

Actual `allocations` adds `resourceCount`/`resourceUploadBytes` for successfully submitted IDs/packed bytes and `resourceTextureBytes`/`resourceStagingBytes` for allocated descriptors. The latter are subsets of existing texture/staging totals. Texture counts include pass and image textures; staging counts include upload and readback buffers. Resource-free values remain unchanged. Driver memory and incomplete failed allocation batches remain excluded under the existing report contract. For 65×3, packed/texture bytes are 780 and upload staging is 1,536 bytes. Cleanup requires zero live bytes; Core estimates cover the conservative descriptor lifetime.

## Verification

[Frozen tests](../crates/mixture-wgpu/tests/support/image_probe.rs) and the [source fixture](../fixtures/nodes/image-input/height.mix) cover:

- 1×1, 1×256, 256×1, asymmetric 2×2, 65×3 and 1K; all 256 red codes round-trip exactly with opaque scalar output.
- G/B/A changes preserve pixels but change identity; caller mutation, same-ID replacement, shared references and bounded cache behavior.
- Nonperiodic edge preservation and wrapped normal directions; a periodic 1K height tile at weights 0/0.25/0.5/1, direct imported/procedural endpoint equality, intermediate causality within one byte.
- Repeated success/rejection, partial readback failure/recovery after upload, device destruction, cleanup and independently owned output.
- Public Rust consumption from source and isolated Cargo archives, without private imports or producer fixtures.

Run `cargo xtask test-node image-input`, `shader-check`, `test-consumer`, `gpu-smoke` and `check`. Node execution JSON and four 1K height/normal RGBA pairs go to `tmp/node-tests/<backend>/`; gpu-smoke uses `tmp/gpu-smoke/nodes/`. GPU smoke includes existing nodes, Scalar, lifecycle and packaged consumers. CI additionally retains unchanged three-material goldens, 2K trace and browser checks.

Selected evidence belongs in the [Native evidence record](./evidence/m6a-03/README.md); ordinary repeated logs stay ignored or in CI artifacts. Native results do not certify browser image execution. CLI decoding, browser resource APIs, pooling, new dependencies, Studio changes and publication are out of scope.

**Later implementation:** [M6A-04](./m6a-04-browser-resources.md) now supplies the browser resource interface; this page retains the M6A-03 boundary.

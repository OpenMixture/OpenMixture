# ADR 0009: Per-render transient texture reuse

English | [简体中文](./0009-transient-texture-reuse.zh-CN.md)

Status: design accepted through PR #58, 2026-09-23. The [0.8 working implementation](../perf-mat-texture-reuse.md) applies the atomic Core/wgpu contract; integration and full qualification of that candidate remain pending. The original design PR changed no executable versions or allocation behavior.

## Context

The frozen MAT-02 default graph now compiles and renders five channels at 1024. Its 23 pass textures remain resident under plan v2. The actual 2048 request fails `MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED`: 805,306,832 descriptor bytes exceed the unchanged 536,870,912-byte ceiling. [Retained input, reports and 1K pixels](../evidence/perf-mat-before/README.md) establish PERF-MAT entry. This is a compiler budget failure, not an attempted hardware out-of-memory allocation.

The existing plan guide explicitly ties v2 estimates to retain-all allocation. Changing only the executor would leave compilation rejecting the selected material; changing only estimates would let the executor exceed the advertised budget. Core and wgpu must implement the same reviewed schedule in one vertical change.

## Decision and ownership

Core computes a deterministic, per-render physical-slot assignment after the existing dependency slicing and stable pass ordering. Logical resource IDs, kernels, uniforms, node versions, dispatch sizes and output provenance remain unchanged. wgpu allocates and executes that assignment through its existing sole pixel path. CLI, WASM and JS project the Rust contract; they do not calculate lifetimes. No new crate, global pool, alternate renderer, pass fusion, shader rewrite or reordering is introduced.

For `P` passes numbered `0..P-1`, initialize each logical resource's last use to its producer pass. Visit all typed input bindings, including repeated bindings, and take the maximum consumer index. Pin every requested output resource to sentinel `P`, including aliases. Unrequested branches were already sliced away. External image textures are separately owned inputs and never enter this pool.

Process producers in pass order. A prior slot is available only when its occupant's last use is **strictly less** than the current pass index. Its complete `TextureDesc` must match: dimensions, format and kind (Scalar/Color/Normal), with the existing fixed one-mip, one-layer, sample-count-one and usage policy. Pick the lowest available compatible slot ID; otherwise append a slot. Associate the output resource with that slot and its computed last use. Never alias an input with its output in the same dispatch. Retain pinned outputs until all readbacks finish.

All physical slots and per-pass uniform buffers may still be created before encoding. Each pass gets views of its assigned output slot and its logical inputs' slots. Keep distinct compute-pass boundaries in the existing command encoder, allowing wgpu to order storage writes and sampled reads. Reuse storage without calling `destroy` on a texture referenced by queued commands. Destroy each owned physical texture exactly once after completion or cleanup. Bind groups, uniforms, uploaded image textures and upload buffers keep their current explicit per-render lifetime. No cross-render texture retention is added.

This is logical last-consumer release plus compatible storage reuse, not a promise of immediate driver deallocation or process-RSS reduction. Error, loss and destruction cleanup must preserve the first structured error and release all owned allocations; a later render on a usable context starts with a fresh slot set.

## Plan, estimates and compatibility

The first implementation selects **RenderPlan v3**, hash domain `mixture-render-plan-v3\0`, and **browser API schema 3**. Graph `inspect --plan` and `render` envelopes become schema 3; doctor, fixed checker and asset envelopes remain schema 1, and `validate` remains unversioned. Nested plan/render payloads expose their new version independently. `.mix` v1, `.mixpack` v1, node semantics, raw pixel encodings and resource capture/limits remain unchanged.

The plan adds a required typed `allocation` object: `slots` in ascending physical ID order, `resourceSlots` indexed by logical resource ID, and `lastUses` with the sentinel rule above. Slots contain the full texture descriptor. Core owns construction; serialized plans still cannot be deserialized into executable plans. Root hash-field order becomes `version`, `documentVersion`, `size`, `materialOutput`, `passes`, `outputs`, `allocation`, `estimates`, `imageResources`. All schedule fields are hashed. Policy ceilings, adapter identity and timing remain outside the hash.

Keep `textureBytes` as allocated pass-texture descriptor bytes, now the sum across physical slots, and add `logicalTextureBytes` for the sum across logical pass results plus `textureCount` for physical pass slots. `uniformBytes`, readback layout, per-channel sequential readback and external-image accounting retain their definitions. For a resource-free equal-size graph, with `N` physical slots, `T=W*H*8`, uniform sum `U`, per-readback padded bytes `R`, and `O` requested channels:

- `textureBytes = N*T`; `logicalTextureBytes = P*T`; `textureCount = N`.
- `peakBytes = N*T + U + R`; `cumulativeBytes = N*T + U + O*R`.
- Keep the existing conservative addition of selected external image texture/upload-staging bytes to the peak, and existing cumulative/upload fields. Do not optimize external uploads in this slice.
- Checked arithmetic and the exact-limit acceptance boundary remain mandatory. Defaults and caller ceilings do not increase.

The GPU allocation report counts actual successfully created physical textures (including separate uploaded images) and their descriptor bytes. `reusedBytes` counts pass-result bytes served by an already allocated slot, excluding its first use and external images. Uniform/staging counts remain actual allocations. At successful completion `liveBytes=0`, and released/cumulative bytes reconcile. Bind-group/driver overhead and CPU outputs remain outside descriptor accounting. The fixed checker must still report its real one-texture allocation under its unchanged envelope.

Target the next unpublished minor, Rust 0.8.0 / browser 0.8.0-alpha.0, at first implementation after checking source/registry availability. All peer versions and build identities move together. Consumers recompile Rust and explicitly accept schema 3; persisted plan/hash caches are invalidated and `.mix` requests are recompiled. There is no v2 serialized-plan loader, dual executor or document rewrite. Candidate consumers update their schema expectations; exact old registry consumption stays pinned to its own contract. Existing historical source/archive evidence remains immutable. This design alone changes no manifest, lockfile, wire response or published package.

## Alternatives

Raising the memory ceiling violates the selected material gate and leaves the expression-cost gap. Reducing resolution, channels or material nodes changes the requested outcome. Destroy/reallocate after each pass would require extra submission/lifetime machinery and could increase driver churn; fixed per-call slots preserve the current encoder path. Sharing across kinds could reduce slots further but is unnecessary: a conservative design simulation gives 14 slots and 503,316,944 bytes at 2K, below the frozen ceiling. This estimate is not implementation evidence. A general optimizer, graph reorder, precision specialization, global pool and pass fusion are unnecessary for this failure.

## Delivery and verification

1. **PERF-MATa:** retain the actual failure and valid baseline, review this ADR and compatibility decision, synchronize architecture, roadmap and both Agent Guides; links and full repository checks pass. No runtime claim.
2. **PERF-MATb:** implement Core schedule/estimates/hash, the wgpu slot executor and thin schema/version projections together. A compiled plan must never advertise pooled costs to a retain-all executor. Add independent expected schedules for chains, diamonds, repeated inputs, same-pass exclusion, descriptor-kind separation, output aliases/pinning, sliced requests, defaults and external resources; test deterministic ordering/hashes, exact-limit acceptance, one-byte-under rejection and checked overflow. Do not derive every expected result from the allocator itself.
3. **PERF-MATc:** prove exact before/after 1K pixels on the same measured adapter, actual 2K five-channel success within 512 MiB and at most 24 passes, matching physical estimates/counters, nonzero reuse and zero live bytes. Exercise multi-size/seed/endpoint requests, partial outputs, repeated requests, independent renderers, upload/package paths, failure cleanup and owned results after destruction. Measure cold/warm timings with the frozen MAT-02 targets and adapter identities; a successful CPU plan is insufficient.
4. Run shader validation (no shader changes expected), original and explicit-v2 material regressions, Native/browser exact candidate and package consumption, affected cross-runtime pixel comparisons, all six required checks and post-merge checks. Preserve historical v1 numerical failures. Update deliberate v3 hash/wire snapshots, never pixel goldens just to pass. Keep MAT-02 material causality/seam/PBR/human acceptance separate; publication remains separate too.

If the conservative assignment or current command path cannot meet the unchanged budget/pixel gates, preserve the failure and review the smallest further change. Do not silently expand into a general allocator or lower the material target.

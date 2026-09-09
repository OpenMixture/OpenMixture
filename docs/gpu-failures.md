# GPU failure reasons and context lifetime

English | [简体中文](./gpu-failures.zh-CN.md)

PR-013 makes device loss and typed GPU out-of-memory failures observable through the public Rust API and existing diagnostic JSON. It also fixes a reproduced readback-cleanup panic after device destruction. The [independent consumer](../examples/native-consumer/tests/device_loss.rs) verifies the public contract using real device destruction. [Local evidence](./evidence/pr-013/README.md) distinguishes those GPU runs from synthetic OOM classification tests.

## Public contract

| API | Meaning |
|---|---|
| `GpuContext::device_loss()` | Borrow the first delivered `DeviceLoss`, or `None` if no notification has been recorded. This getter does not poll or block. |
| `DeviceLoss::reason` | Typed `DeviceLossReason::Unknown` or `Destroyed`, translated directly from the native callback. |
| `DeviceLoss::message` | Original callback text, including an empty message for explicit destruction. |
| `GpuOperationError::reason()` | Primary `GpuFailureReason::DeviceLost`, `OutOfMemory`, or `Other`. |
| `GpuOperationError::device_loss()` | Owned loss evidence attached to this failure, including loss observed alongside another primary error. It survives renderer/context drop. |
| `GpuOperationError::adapter()` | Owned selected-adapter evidence when the error came from the executor. |
| `GpuOperationError::allocations()` | Successful per-call descriptor accounting after cleanup; absent for failures before entering the executor. |
| `GpuOperationError::diagnostic()` | Existing structured code, stage, context, scalar evidence and original source chain. |

The reason enums are non-exhaustive. `Other` includes validation/internal faults, timeouts, host pixel-allocation failures and other operations without a primary typed OOM/loss cause. A driver string containing “out of memory” or “device lost” does not change classification.

`reason()` describes the **primary failure**. Always inspect `device_loss()` before considering context reuse: an OOM or mapping error can remain primary while the attached loss record says that context is unusable. Conversely, an absent notification is not a health guarantee.

## Device-loss lifecycle

Each explicit context owns an `Arc<OnceLock<DeviceLoss>>`. Its registered callback retains only that record, without GPU handles or a reference back to the context. The first notification wins and cannot be cleared. There is no global state, background worker, automatic reacquisition, retry or alternate executor.

The acquisition `ContextReport` remains an immutable snapshot. Its original `unverified` verdict does not change when loss is recorded; use `device_loss()` for current delivered loss evidence. Likewise, a previously healthy doctor report is not a promise about future device health.

For a render request that passes the existing size/device-limit checks, the executor checks recorded loss, performs one nonblocking native `Poll`, then checks again before cache lookup or allocation. This delivers available destruction callbacks, including the tested cold/warm-cache cases. If loss is already recorded, the native poll and new GPU work are skipped. Input/request validation remains ahead of GPU work.

The executor also checks for loss at allocation, post-submission and readback boundaries and before returning completed pixels. A delivered loss prevents success. It clears retained pipelines on an observed loss and returns no partial `RenderOutput`. Subsequent valid calls on that same context fail with zero new descriptor allocations. Existing CPU-owned outputs remain usable.

Native notification delivery is backend-driven and can require polling. A callback arriving after a checkpoint is observed at a later checkpoint or call; the getter does not synchronously query driver health. The retained raw `device()` escape hatch still permits callback replacement, which disables Mixture's loss tracking and is outside this contract. Normal consumers should keep the registered callback intact. See the pinned [wgpu callback API](https://docs.rs/wgpu/30.0.1/wgpu/struct.Device.html#method.set_device_lost_callback) and [dependency-exposure policy](./native-sdk.md).

## Failure selection and preserved evidence

| Observation | Primary diagnostic |
|---|---|
| Loss recorded before further work, without an earlier operation error | `MIX_GPU_DEVICE_LOST`, at the checkpoint's operation stage. A blocked new render uses `gpuExecution`. |
| Scoped `wgpu::Error::OutOfMemory` | `MIX_GPU_OUT_OF_MEMORY`, retaining shader/pipeline/execution/readback stage and operation message. |
| Another operation fails before loss is attached | Keep that operation's code, stage, message and source; attach the loss record separately. |
| Unmap cleanup fails after mapping/access/conversion already failed | Keep the first failure; attach the cleanup code/message/driver evidence. |
| Cleanup alone fails after otherwise successful readback | Return the cleanup failure, never successful pixels. |

All three native error scopes are popped before awaiting their results. Error scopes expose no chronological ordering across filters. If the same scoped action produces multiple categories, PR-013 selects **OOM, then validation, then internal** and retains other captured driver messages. Without OOM, the existing validation-before-internal precedence remains. This avoids reporting a secondary invalid-resource error as the only evidence of allocation failure. An error already selected by an operation is never replaced merely because loss or cleanup failure appears later.

The [typed wgpu error](https://docs.rs/wgpu/30.0.1/wgpu/enum.Error.html) and its nested native source remain reachable through `Error::source()`. The selected scope also records `driverMessage` and, when supplied, `driverCause`. A primary loss error has the owned `DeviceLoss` as its source. Native source objects are not serialized.

Additional JSON evidence uses the existing scalar-only map:

| Key | Value |
|---|---|
| `failureReason` | `deviceLost` or `outOfMemory` for the corresponding primary codes. |
| `deviceLost` | Boolean `true` whenever a delivered loss is attached, even with another primary code. |
| `deviceLostReason`, `deviceLostMessage` | `unknown`/`destroyed` and the original callback text. |
| `adapterName`, `backend` | Selected adapter name and native backend. |
| `planHash` | Exact compiled plan hash for graph-render failures. Existing pass/node context remains when available. |
| `allocationCumulativeBytes`, `allocationPeakBytes`, `allocationReleasedBytes`, `allocationLiveBytes` | Exact unsigned descriptor-byte counters after cleanup. |
| `allocationTextureCount`, `allocationUniformCount`, `allocationStagingCount` | Exact unsigned counts of successful recorded allocations. |
| `secondaryValidationMessage`, `secondaryInternalMessage` | Other errors captured by the same scoped action, when present. |
| `cleanupCode`, `cleanupMessage`, `cleanupDriverMessage` | Secondary cleanup failure evidence, when present. |

An OOM plus a loss notification therefore retains `MIX_GPU_OUT_OF_MEMORY` / `failureReason: outOfMemory`, with `deviceLost: true` and its separate reason/message. A mapping error followed by loss retains `MIX_READBACK_FAILED` with the same loss fields. Consumers need not parse driver prose for either decision.

## Resource and readback cleanup

The same per-render resource guard is explicitly finished on success and error. Textures/uniforms are destroyed, staging guards release their buffers, and errors receive the resulting allocation report. A failure after one completed readback returns an error with released resources and no partial pixel result.

Readback always attempts unmapping under native error scopes, after dropping any mapped view. This includes map/poll/access/conversion failures. The regression reproduced a native validation panic when a device-destroyed buffer was unmapped outside a scope; the fixed path returns the original mapping error and records the unmap failure as secondary evidence. Cleanup does not depend on the context still being usable.

Counters cover successful descriptor batches retained by Mixture. Temporary handles from an incomplete failed allocation batch are dropped and are not counted as completed allocations. Counters exclude driver granularity, pipeline/bind-group overhead and CPU pixel/PNG memory. `live_bytes == 0` means recorded resources were destroyed, not immediate physical memory reclamation. The retain-all resource schedule and lack of pooling remain unchanged.

## JSON and compatibility boundaries

The two diagnostic codes are an intentional addition to the non-exhaustive vocabulary. Known typed OOM/loss cases now use these specific codes; generic operation failures retain their existing codes. No `.mix`, node or plan version changes. CLI envelopes, optional/null rules and exit codes remain unchanged: GPU errors return `1`, with no completed graph execution or PNG outputs. Existing acquired-context and plan evidence survives. See the [CLI contract](./cli-contract.md).

The core diagnostic decoder rejects unknown code strings. A strict older decoder therefore needs the updated vocabulary to consume these new failures; this is not a promise that pre-alpha diagnostic vocabularies are interchangeable. Full package/release compatibility remains PR-015.

Classification is based on public typed runtime signals. The pinned wgpu `RequestDeviceError` does not expose its category publicly; acquisition failures continue using `MIX_GPU_DEVICE_REQUEST_FAILED` with native evidence. Host allocation failures are not labeled GPU OOM. No private wgpu-core dependency or driver-message parser is added.

## Verification and scope

```bash
cargo test --locked -p mixture-core --test diagnostics
cargo test --locked --all-features -p mixture-wgpu operation
cargo test --locked --all-features -p mixture-wgpu context
cargo test --locked --all-features -p mixture-cli
cargo xtask test-consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

Prepare the software loader using the [GPU guide](./gpu-context.md). Default checks compile the independent loss test but do not run a GPU. Explicit smoke runs it with a fresh evidence file linked by `deviceLossEvidence` in `native-consumer/status.json`; missing, skipped or incomplete tests cannot satisfy that gate.

CPU tests construct typed synthetic OOM/internal/validation errors to check classification, precedence, serialization and source chains without exhausting memory. Real GPU tests destroy devices with cold/warm caches, reject repeated renders, keep another context operational, release staging after destruction, and release all recorded descriptors after a late readback failure. The independent consumer uses only public Rust imports and its own source input. These tests cover notification/lifecycle behavior, not spontaneous driver loss or physical memory exhaustion.

**Out of scope:** automatic recovery, fallback, new resource pools, shader/pixel changes, cancellation/stale-result scheduling, physical memory exhaustion, packaged consumption, publication and deferred remote platform CI.

# PR-013 GPU failure evidence

English | [简体中文](./README.zh-CN.md)

**Local acceptance, 2026-09-09:** device loss and typed execution OOM have distinct structured codes; context-owned loss tracking, cache invalidation and scoped readback cleanup preserve the original failure. The implementing revision is the commit containing this record on `codex/pr-013-gpu-failures`, parent `8ce6c0a75eec7af14e5080519ad110802e11a5dd`.

The [failure contract](../../gpu-failures.md) documents notification timing, public accessors, allocation evidence and compatibility. No shader, node, document/plan format, dependency or lockfile changed. CLI envelopes and exit codes are unchanged; strict diagnostic-code decoders need the two new variants.

## Verification

| Check | Evidence |
|---|---|
| Cold/warm loss before the fix | [Before](./checks/device-loss-before.log): both expected regressions fail with generic execution errors. |
| Cleanup defect before the fix | [Before](./checks/readback-loss-before.log): unmapping a destroyed staging buffer triggers an uncaptured validation panic. [Doctor afterward](./checks/doctor-after-readback-failure.json) verifies a fresh context. |
| Typed OOM, first-error preservation and source chains | [Operation tests](./checks/operation-tests.log); OOM is synthetic, not physical exhaustion. |
| Diagnostic vocabulary and tooling guards | [Core](./checks/core-diagnostics.log), [consumer tooling](./checks/consumer-tooling.log). |
| CPU consumption and CLI compatibility | [Consumer](./checks/test-consumer.log), [CLI](./checks/cli-tests.log). |
| Fixed destruction/readback regressions | [Device loss](./checks/device-loss-metal.log), [readback](./checks/readback-metal.log). |
| Metal full GPU smoke | [Command](./gpu-smoke/metal/command.log), [GPU tests](./gpu-smoke/metal/gpu-tests.stdout.log), [doctor](./gpu-smoke/metal/doctor.json), [consumer completion](./gpu-smoke/metal/consumer/status.json). |
| Pinned software full GPU smoke | [Command](./gpu-smoke/software/command.log), [GPU tests](./gpu-smoke/software/gpu-tests.stdout.log), [doctor](./gpu-smoke/software/doctor.json), [consumer completion](./gpu-smoke/software/consumer/status.json). |
| Final repository check | [check.log](./checks/check.log). |
| Bilingual executable blocks | [Pair check](./checks/doc-pairs.json); narrative reviewed separately. |

Both full smokes pass with Apple M5/Metal and SwiftShader Device (LLVM 10.0.0)/Vulkan respectively. The software source was clean at pin `694585a05946e1ed49b6bd577ca6537cbb57f025`, with the loader under `tmp/pr-004/swiftshader-build/bin`. Each run includes existing GPU regressions, public-Rust output ownership, ten CLI GPU cases and the independent device-loss contract. Local macOS execution does not satisfy the deferred remote platform CI gate.

The independent consumer destroys cold and warm devices, checks two identical repeated failures per case with zero new allocations and an empty pipeline cache, retains diagnostics after renderer drop, and consumes correct pixels from another live context. `deviceLossEvidence` in each completion status points to its fresh two-case JSON receipt. The staging destruction regression preserves the initial mapping error and reports scoped unmap failure as secondary evidence; allocation reports show zero live descriptor bytes. A separate late-readback regression verifies release after work has already completed and subsequent healthy reuse.

[runs.json](./runs.json) locates the selected active captures. Raw streams, original absolute paths, inputs and PNGs are preserved. [capture-manifest.json](./capture-manifest.json) inventories sources and archived bytes; [pre-smoke source hashes](./checks/gpu-source-hashes.json) match the final source set. These hashes record local capture identity and are not embedded build attestations.

## Reproduction and limits

Use the exact CPU and adapter commands in the [failure guide](../../gpu-failures.md#verification-and-scope) and the [independent consumer](../../../examples/native-consumer/README.md#device-loss-contract-pr-013). Run `cargo xtask check` for the final CPU/docs gate. Prepare the pinned loader using the [GPU guide](../../gpu-context.md).

Typed synthetic OOM tests cover classification across operation stages, serialization, secondary error scopes and source chains. Actual GPU tests use explicit device destruction; they do not establish spontaneous driver-loss or physical-exhaustion behavior. Acquisition errors remain `MIX_GPU_DEVICE_REQUEST_FAILED` when the public dependency exposes no typed cause. Error strings are not classification heuristics.

Out of scope: automatic recovery, alternate execution, resource pooling, cancellation/stale-result scheduling, packaged consumption, publication and remote CI. PR-014 owns stale results and bounded retained outputs.

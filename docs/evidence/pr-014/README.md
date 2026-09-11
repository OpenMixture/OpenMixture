# PR-014 stale-result and retention evidence

English | [简体中文](./README.zh-CN.md)

**Local acceptance, 2026-09-11:** the independent consumer publishes only the newest generation, bounds work to one active/one replaceable pending request, and retains at most the displayed CPU output plus the completion being processed. The implementing revision is the PR-014 commit containing this record on `codex/pr-014-stale-results`, parent `6246647`.

The [contract](../../stale-results.md) describes the state transitions, deterministic driver, directory cleanup and memory limits. Product runtime code, shaders, node/format/plan semantics, dependencies and lockfiles are unchanged. A library target inside the existing independent example shares application state with its executable and tests; this is not a new product package.

## Checks and observations

| Verification | Evidence |
|---|---|
| CPU state and CLI compatibility | [Consumer command](./checks/test-consumer.log); final [consumer test output](./checks/consumer/tests.stdout.log) includes six new state/lifetime tests alongside the existing checks. |
| Reject skipped/incomplete evidence | [Tooling tests](./checks/tooling-tests.log) reject wrong publication generations, leaked outputs and missing CLI cases. |
| Focused cache sequence | [Metal log](./checks/cache-metal.log): all nine kernels, 22 changed cases, each repeated; explicit clear, device destruction and independent renderer. |
| Real Rust consumer | [Metal](./gpu-smoke/metal/consumer/latest.stdout.log), [software](./gpu-smoke/software/consumer/latest.stdout.log): started `[1,2,4,5]`, published `[1,5]`, request 3 replaced, request 4 failed compilation. |
| Metal full smoke | [Command](./gpu-smoke/metal/command.log), [doctor](./gpu-smoke/metal/doctor.json), [GPU tests](./gpu-smoke/metal/gpu-tests.stdout.log), [cache reports](./gpu-smoke/metal/cache-bound.json), [completion](./gpu-smoke/metal/consumer/status.json). |
| Pinned software full smoke | [Command](./gpu-smoke/software/command.log), [doctor](./gpu-smoke/software/doctor.json), [GPU tests](./gpu-smoke/software/gpu-tests.stdout.log), [cache reports](./gpu-smoke/software/cache-bound.json), [completion](./gpu-smoke/software/consumer/status.json). |
| Final repository check | [check.log](./checks/check.log): format, dependency boundaries, strict Clippy, tests, independent consumer, Rustdoc and local links. |
| Bilingual documentation | [Pair check](./checks/doc-pairs.json); executable/configuration blocks match, narrative reviewed separately. |

Both full smokes pass on Apple M5/Metal and SwiftShader Device (LLVM 10.0.0)/Vulkan. The software source was clean at pin `694585a05946e1ed49b6bd577ca6537cbb57f025`, using the loader in `tmp/pr-004/swiftshader-build/bin`. Existing checker/graph regressions, PR-011 ownership, PR-012 ten-call CLI checks and PR-013 destruction/cleanup checks remain included. This is local macOS evidence; remote platform CI remains deferred.

The new Rust driver checks literal pixels and metadata after renderer drop. A superseded success is released, a replaced pending override is never compiled/rendered, and the invalid newest override keeps the original display explicitly stale until generation 5 succeeds. The final four-channel 65×3 output retains 3,120 pixel bytes; the two-output bound is structural, verified by drop-count tests, not an OS/VRAM measurement. Reports retain fixed-size evidence for the bounded sequence, not historical pixel buffers.

Each new CLI sequence performs five calls: initial success, obsolete success, invalid override (exit `2`), partial write (exit `1`) and final success. It verifies generation isolation, unchanged old display, final decoded normal bytes, changed checker PNG, removal of generations 1–4 and preservation of unrelated data. Only `generation-5` remains. The final 65×3 baseColor was visually inspected; no shader or golden pixels changed.

[runs.json](./runs.json) locates each active capture through `latestEvidence`. Its receipt contains raw child reports; final PNGs and owned input are retained beside it. Obsolete/partial PNGs were intentionally deleted by the tested cleanup, so their original report paths no longer exist. Original absolute paths and raw stdout/stderr are preserved, including the earlier ten-case CLI and device-loss receipts.

The cache sequence reaches exactly nine entries while each report shows zero live/reused descriptor bytes. Repeating each request gives identical allocation counters and pixels with no new pipeline misses. Clearing drops entries to zero; a subsequent render misses once. After destroying that renderer's device, its next call returns zero-allocation failure and clears the cache. A separately acquired renderer still produces matching pixels; output data remains accessible after the first renderer is dropped. The existing late-readback failure regression verifies cleanup after allocations, beyond this preflight-loss case.

## Reproduction and limits

Use [the exact commands](../../stale-results.md#verification) and the [independent example guide](../../../examples/native-consumer/README.md#latest-requests-pr-014). Run `cargo xtask test-consumer` for deterministic CPU transitions, explicit `gpu-smoke` for both real consumers and the GPU sequence, then `cargo xtask check` for the repository gate. The new tests contain no sleep-based races.

[Pre-smoke source hashes](./checks/gpu-source-hashes.json) were rechecked after the final repository check. [capture-manifest.json](./capture-manifest.json) records source/build identity and archive hashes, excluding itself. These are capture evidence, not embedded build attestations.

Already submitted GPU work may finish. No hard interruption, background runtime, global state, resource pooling, last-consumer optimizer, UI, daemon, package verification or remote release action is included. CPU retention, GPU descriptor budgets and disk cleanup are separate boundaries; ignored cleanup errors or extra host-held copies invalidate the example's bound. PR-015 remains the next step for packaged consumption and compatibility/release documentation; no release-ready claim is made.

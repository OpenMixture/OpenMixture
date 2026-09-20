# Latest requests and bounded consumer state

English | [简体中文](./stale-results.zh-CN.md)

PR-014 adds application-owned scheduling in the [independent native consumer](../examples/native-consumer/README.md), with a runnable `latest` verification mode. Product core, renderer and CLI behavior are unchanged. The [local acceptance record](./evidence/pr-014/README.md) includes CPU transition tests, real Rust/CLI outputs and the nine-kernel cache sequence.

## Ownership and transitions

[Latest](../examples/native-consumer/src/latest.rs) is an example library target in the existing consumer package, not a new product crate or public Mixture SDK API. The host owns one renderer and the worker/event loop. The state holds one active generation, one replaceable pending request, one displayed output and at most one latest failure. It never starts a worker, cancels a future or clones output pixels.

| Event | Effect |
|---|---|
| `request(request)` | Checked monotonic generation increment, replacing/dropping previous pending work. Existing display immediately becomes stale; previous failure is cleared. Register before compilation so a failed newer request also supersedes older work. |
| `start()` | Transfers pending input to the caller and marks it active. Returns none while any request is active; no concurrent render is launched. |
| Active completion, still latest | Successful output replaces display; the old output is returned for immediate release. Failure retains the old display, explicitly stale, and records the latest failure. |
| Active completion, superseded | Clears the active slot and returns obsolete successful output for release; stale errors do not become the latest failure. |
| Unknown, pending or duplicate completion | Cannot clear active work, replace display or change latest failure; any supplied output is returned for release. |
| `displayed()` | Returns the display's generation, borrowed output and freshness boolean. A plan hash is semantic identity, not request freshness. |

Generation overflow returns an error without modifying existing state. Callers must immediately drop returned obsolete CPU output or explicitly clean its owned directory. `take_displayed()` allows the host to release its retained display. Consumers that keep additional copies or completed-result history are outside this bound.

The `latest` mode uses deterministic event delivery: start request 2, inject requests 3 and 4 before completing 2, discard 2, skip replaced request 3, reject invalid request 4, then publish request 5. Published generations are `[1,5]`; actual started generations are `[1,2,4,5]`. It checks real pixels after renderer drop. This is a reproducible driver of the scheduling state, not an interactive service or a demonstration of parallel GPU execution. A responsive application supplies its own explicitly managed worker and delivers events to this state; Mixture's native polling can block that worker.

## CLI directory ownership

[latest_cli.rs](../examples/native-consumer/tests/latest_cli.rs) exercises the same state with a prebuilt CLI and consumer-owned source. Each started generation exclusively creates a new `generation-N` directory. It never adopts or overwrites an existing directory. Publication selects the current completed directory in consumer state; it does not rename it into a shared output target or claim atomic file export.

The five CLI calls cover initial success, an obsolete success, a newer invalid override (exit `2`), a partial PNG-write failure (exit `1`), and final success. Obsolete success, failed/partial directories and the replaced display are explicitly removed. Only generation 5 is retained; unrelated files remain untouched. No files are allocated for replaced pending requests. The tests compare completed PNGs, decode the final neutral normal, verify the old display is unchanged until replacement, and reject stale publication after a newer failure.

`OutputDirectory::remove` returns I/O errors; no destructor silently hides cleanup failure. The example assumes exclusive ownership of its fresh directory tree. If cleanup fails, the caller must report the error and stop accepting more work until it resolves the retained output; proceeding regardless would invalidate the disk bound. The test harness fails with an incomplete receipt in that case.

## Memory and cache bounds

Retained CPU pixels are bounded by the displayed output plus the current completion during replacement. For output size `W × H` and `C` channels, tightly packed output pixels occupy `4 × W × H × C` bytes. With different requests the sum is `B_display + B_completion`; for the 65×3 four-channel probe this is at most 6,240 bytes, returning to 3,120 bytes after completion handling. The report's `maximumCoexistingOutputs: 2` is a structural consumer bound checked by drop-counter tests, not measured OS memory.

Per-render `SafetyLimits::transient_bytes` and `AllocationReport` describe GPU descriptors, not this aggregate CPU retention. The host must additionally budget input/plans, readback conversion, PNG encoding, allocator overhead, drivers and pipeline state. Pending input is bounded by count and must also have a host-enforced byte limit if accepting arbitrary data; this example queues only integer overrides for bounded owned input. There is one active renderer call, no output history, texture pool or hidden global cache.

The focused [GPU regression](../crates/mixture-wgpu/tests/nodes.rs) renders the first two available cases of each of twelve node contracts at 33×3, repeats each for identical pixels and per-call counters, and reaches all ten pixel kernels. Cache entries equal distinct encountered kernels, never exceed ten, remain independent between renderers, and clear explicitly or after observed loss. Drop releases ownership; descriptor reports after success/failure show `liveBytes == 0` and reuse remains zero. These are ownership/counter assertions, not immediate physical VRAM reclamation measurements. Existing late-readback failure tests continue to run in smoke.

## Verification

```bash
cargo xtask test-consumer
cargo test --locked -p xtask consumer
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo test --locked --all-features \
  -p mixture-wgpu --test nodes \
  graph_gpu_cache_is_bounded_across_all_kernels_and_request_changes \
  -- --ignored --exact --nocapture
cargo run --locked --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- latest metal hardware 'Apple M5'
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

Prepare the pinned loader using the [GPU guide](./gpu-context.md). CPU checks run six state/lifetime tests and compile the ignored CLI test. Explicit smoke runs Rust `latest` and the five-call CLI sequence; `native-consumer/status.json` references `latestEvidence` for the fresh CLI receipt, alongside `latest.stdout.log` for Rust evidence. Missing/skipped/incomplete receipts fail the gate. No timing sleeps are needed.

Already submitted GPU work may finish. Dropping a render future or reaching a per-poll timeout does not promise cancellation or an overall deadline. Hard interruption, a runtime job framework, daemon/IPC, UI controls, resource pooling, packaged consumption and deferred remote CI are out of scope. PR-015 now provides [package verification](./package-consumption.md) and [compatibility/release documentation](./release.md).

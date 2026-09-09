# M4 implementation train — stable native consumption

English | [简体中文](./M4_PRS.zh-CN.md)

**Status:** started from the [M3 review](./docs/m3-review.md) of `e9dd03b`, 2026-09-08. PR-011 through PR-013 are locally implemented and verified; PR-014 and PR-015 remain planned. This train instantiates [M4](./ROADMAP.md#m4--stable-native-sdk) without changing architecture or expanding the node vocabulary. Remote platform CI remains deferred; M4 planning and local work do not close it.

The consumer needs the existing decode → validate → compile → wgpu → owned-output path. Release measurements support debounced previews and show no startup blocker warranting native bindings or a daemon. The 2K descriptor peak fits the current budget without pooling. Work below stabilizes observable behavior and independently verifies consumption.

## Sequence and shared rules

```text
PR-011 Public Rust API + independent consumer
   -> PR-012 CLI JSON/exit-code contract + diagnostic context
      -> PR-013 Device loss / out-of-memory contract
         -> PR-014 Stale results + bounded consumer/cache lifetime
            -> PR-015 Packaged consumer + compatibility/release checks
```

Each PR is a separate reviewable local commit or PR, with paired English/Chinese documentation, focused tests, and `cargo xtask check`. Keep `mixture-core` GPU-free, `mixture-wgpu` the sole pixel executor, CLI thin, GPU state explicit, and the source review bundle unchanged. No new runtime crate is justified. A separate consumer Cargo fixture is an actual dependency/compilation boundary and must not join the product workspace by accident.

`cargo xtask test-consumer` is **implemented** by PR-011 and included in `check`; it remains CPU-only. `cargo xtask package-check` is **proposed** for PR-015 and is not implemented. GPU execution remains explicit through the existing `gpu-smoke` policy; no GPU requirement is added to ordinary `check`. Preserve existing CLI schema/exit behavior while adding tests; any intentional incompatible change needs an explicit compatibility decision first.

## PR-011 — `feat(sdk): verify public Rust consumption end to end`

**Completed locally, 2026-09-08:** commit `244b384` follows `f56bffe` on `codex/pr-011-native-consumer`. See the [public API contract](./docs/native-sdk.md), [independent application](./examples/native-consumer/README.md), and [checks, source hashes and paired release evidence](./docs/evidence/pr-011/README.md). Raw wgpu getters remain documented escape hatches. Metal and pinned SwiftShader execute and inspect owned outputs after renderer drop; no product runtime behavior or dependency was changed. Packaged consumption remains PR-015.

### Outcome and evidence

An independent Rust program owns its `.mix`, uses public imports, requests exposed overrides/channels, renders and consumes owned pixels and reports. The [M3 probe](./docs/reviews/m3/native-consumer/README.md) already proves CPU compilation and the GPU call shape; it has not run that GPU path or consumed packaged crates.

### Scope

- Review the public decode/validate/compile/request/render/error/result types against that program. Prefer documentation and focused corrections over a new facade or builder.
- Decide and document compatibility of raw `GpuContext` wgpu getters and re-exported dependency types before calling the API stable. Do not silently remove existing accessors; document escape-hatch limits where justified.
- Add a consumer-owned input and independent Cargo manifest, with no private imports or repository-relative runtime resources. Correct stale six-node/future-compiler Rustdoc and replace its source-relative input example.
- Execute explicit native GPU acquisition/render in the consumer. Inspect requested channels, color/scalar/normal encodings, connected/default provenance, dimensions/byte lengths, adapter, plan hash and metrics; prove returned pixels remain usable after renderer/context drop.
- Add `test-consumer` for CPU errors, stable override/channel compilation and compilation of the complete GPU path. Add the explicit GPU consumer invocation to `gpu-smoke`.
- Measure the same 1K material via public Rust with context creation, first render and a reused-renderer call recorded separately. Compare correctness before comparing timing with the CLI. Return pixels directly; the example need not encode PNG or implement pixel semantics.

### Acceptance and verification

Run `cargo xtask test-consumer`, `cargo xtask test-plan`, `cargo xtask check`, then explicit `cargo xtask gpu-smoke` on local Metal and pinned SwiftShader. The independent program must consume actual owned outputs, reject an invalid exposed override with the expected parameter context, and report the selected adapter. Record source/build/adapter/resolution/channels and metric definitions for measurements. Do not claim packaged-crate acceptance from path dependencies.

**Out of scope:** new API abstraction layers without a consumer failure; N-API, daemon/IPC, browser/UI work; packaging/publication; shader changes or performance optimization.

## PR-012 — `fix(cli): preserve diagnostic context and verify consumer reports`

**Completed locally, 2026-09-08:** commit `8ce6c0a` follows `244b384` on `codex/pr-012-cli-contract`. A shared CLI formatter preserves document/stage/node/port/parameter context, including the missing displacement and render-override defects. The [CLI contract](./docs/cli-contract.md), independent CPU/GPU process tests, and [recorded evidence](./docs/evidence/pr-012/README.md) verify existing envelopes, exit codes, completed PNGs and partial writes. JSON shapes and versions are unchanged; `validate` retains its unversioned diagnostic envelope. No GPU/core semantics or product dependencies changed.

### Outcome and evidence

A separate program can invoke the built CLI and reliably distinguish success, source/request failure and operational failure. [M3 probes](./docs/reviews/m3/diagnostics/README.md) demonstrate a concrete defect: human `inspect` and `render` drop the missing warp input's `displacement` port while JSON and human `validate` preserve it.

### Scope

- Fix context omission in human reporting, sharing only the small formatting helper justified by duplication. Keep semantics and typed errors in their owning libraries.
- Document and test current versioned JSON reports for `validate`, `inspect --plan`, `doctor`, and `render`: required/optional fields, deterministic ordering where promised, diagnostics and measured-evidence types.
- Preserve exit `0` for success, `2` for invocation/source/request errors, `1` for operational errors. State the current usage-error exception: `--json` does not turn usage errors into stdout JSON. Keep JSON stdout parseable, and document stderr behavior.
- Run the built executable from an independent working directory using consumer-owned input/output paths. Cover public overrides, requested outputs, adapter/plan identity, encoding metadata and completed output files.
- Cover failures before GPU acquisition and partial PNG-write failure, including the list of files already written. Do not promise atomic output-directory replacement; callers use per-request directories in PR-014.

### Acceptance and verification

Add CLI integration/contract assertions covering the same invalid warp fixture through validate/inspect/render in human and JSON modes, malformed input, invalid override, missing source file, explicit disabled-backend acquisition, successful reports, and partial writes. Assert semantic fields and exit codes, not volatile timing/adapter snapshots. Run `cargo xtask test-consumer`, the focused CLI tests and `cargo xtask check`; exercise successful render/partial-write GPU paths via `cargo xtask gpu-smoke`.

**Out of scope:** a new document format or JSON schema version without need; changing usage-error policy incidentally; a general serialization framework; transactional export; bindings or remote services.

## PR-013 — `fix(gpu): classify device loss and out-of-memory failures`

**Completed locally, 2026-09-09:** the PR-013 commit containing this record follows `8ce6c0a` on `codex/pr-013-gpu-failures`. [GPU failure contracts](./docs/gpu-failures.md) define context-owned loss records, typed OOM/loss codes, first-error precedence and cleanup evidence. Real destruction tests cover cold/warm caches and independent contexts; synthetic typed errors cover OOM without exhausting memory. The reproduced destroyed-buffer unmap panic is fixed with scoped cleanup. See [checks and evidence](./docs/evidence/pr-013/README.md). No shader, dependency, format, fallback or recovery change is included.

### Outcome and evidence

Library and JSON consumers can branch on device loss and out-of-memory without parsing driver strings. Before PR-013, error scopes captured OOM but mapped it only by operation stage, and the context had no runtime device-loss record. The earlier destroyed-device tests proved failure without a stable reason/lifecycle contract.

### Scope

- Define typed failure reasons and stable externally visible diagnostics for device loss and OOM while preserving operation stage, first failure, source chain, selected adapter and plan evidence where available.
- Keep loss state owned by each explicit `GpuContext`; define when it becomes observable and whether subsequent renders fail immediately. Do not reacquire a device or silently retry elsewhere.
- Preserve resource/readback cleanup on failure and avoid converting an earlier useful failure into a generic wrapper. Define precedence for loss recorded during another failed operation.
- Extend existing destroyed-device tests for cold and populated pipeline caches, no successful outputs, released per-call resources and an independent context still operating.

### Acceptance and verification

Test classification and serialization deterministically without exhausting physical memory. Exercise real device destruction/loss notification through explicit GPU tests; do not describe synthetic OOM classification as hardware exhaustion coverage. Run targeted context/operation/readback tests, CLI report tests, `cargo xtask test-consumer`, `cargo xtask check`, and `cargo xtask gpu-smoke` on both local policies. Retain driver evidence and distinguish classification coverage from backend behavior.

**Out of scope:** automatic recovery, fallback executors, process-global error state, hardware memory exhaustion, new resource pools, a generalized retry framework.

## PR-014 — `feat(consumer): reject stale renders and bound retained state`

### Outcome and evidence

Parameter changes during a long render cannot publish an old result over a newer request. M3 measured 0.4–0.7 second full-output CLI medians for wood/leather; the current per-wait 30-second timeout is neither cancellation nor an overall deadline. M4 explicitly permits stale-result handling instead of cancellation.

### Scope

- Add a monotonically increasing request generation in the independent consumer. Publish only the latest requested generation; a newer failed request must not cause an older success to be mislabeled current.
- Bound work with one active render and one replaceable pending request in the example. Keep freshness and scheduling in the consumer; retain explicit renderer ownership.
- Use generation-specific directories when exercising CLI output. Promote/consume only the current result and clean obsolete owned outputs; an older request must never write into the newest directory.
- State that GPU work already submitted may finish and that dropping an async future or hitting a poll timeout does not promise GPU cancellation. Avoid adding an abort API without an actual need.
- Verify the existing renderer pipeline cache stays bounded by the nine pixel kernels across parameter/channel changes, supports explicit clear/drop, and does not leak per-render descriptor resources. Keep resource reuse at zero unless new measured evidence requires a change.
- Explain per-render versus aggregate memory bounds and retain at most the displayed CPU result plus the current completion during replacement.

### Acceptance and verification

Use deterministic consumer tests for out-of-order completion, newer-request failure, replacing pending work, duplicate completion, isolated output paths and cleanup. Use a bounded GPU sequence to verify cache growth/clear, independent renderers, per-call counter reset and `liveBytes == 0` after completion/failure. Run `cargo xtask test-consumer`, focused GPU/resource tests, `cargo xtask check` and `cargo xtask gpu-smoke`. No sleeps or timing races should be necessary for freshness tests.

**Out of scope:** hard GPU interruption, an async job framework inside core, daemon/IPC, unbounded parallel rendering, pooling/last-consumer optimization, UI controls.

## PR-015 — `test(release): verify packaged native consumption and compatibility`

### Outcome and evidence

An independent consumer can build from the intended package contents and use the built CLI without access to repository internals. Current workspace crates have `publish = false`, library dependencies use source paths, and documentation references outer example files. Source-path probes cannot establish this exit gate.

### Scope

- Introduce `cargo xtask package-check` to stage and verify actual local package contents in an isolated location, resolve the core/wgpu dependency as a local package, and build the independent consumer without repository asset paths. Record the exact package-resolution method; listing archives alone is insufficient.
- Review package includes, licenses, Rustdoc/README resources and dependency version metadata. Keep actual publication disabled; adjust local verification metadata only as needed and avoid incidental lockfile updates.
- Include package verification and CPU consumer checks in appropriate repository/CI checks, with GPU consumption explicit. Test the built CLI from an isolated working directory as the alternate native boundary.
- Write paired compatibility and release documentation covering `.mix`/node versions, plan hash changes, public Rust types and dependency exposure, CLI schema/exit codes, output encoding, diagnostic evolution, release notes and reproducible verification.
- Record all remaining external platform gates and the tested host/backend matrix. Re-run the three golden-material checks and 2K trace for the final train, without overwriting accepted pixels.

### Acceptance and verification

Run `cargo xtask package-check`, `cargo xtask test-consumer`, `cargo xtask check`, explicit `cargo xtask gpu-smoke`, `cargo xtask golden check`, and `cargo xtask trace-2k` on the intended local adapter policies. The packaged consumer must render and consume outputs through public APIs in the explicit GPU checks. A missing runtime asset must fail the packaging test. M4 is locally ready only when PR-011–015 evidence, public docs and behavior agree; any deferred remote gates remain explicitly open and block a release-ready claim.

**Out of scope:** publishing to crates.io, remote push/merge, binary distribution/installers, WebAssembly, a node editor, engine-specific exports, a thirteenth node, or copying future tasks from the legacy repository.

## Completion record

Update each section with the implementing revision and reproducible evidence when it lands; do not mark a proposed command implemented before its owner PR introduces it. Use the [PR template](./.github/pull_request_template.md), state intentionally excluded scope, and preserve separate commits in dependency order. After PR-015, assess M4 exit criteria and outstanding platform evidence before defining the next train.

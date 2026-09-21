# Public native Rust consumption

English | [简体中文](./native-sdk.zh-CN.md)

**M6A-02 update:** [M6A-02 Core implementation](./m6a-02-core-resources.md) now provides resource references, immutable prepared requests and content-bound plan v2. Rust source is 0.3.0; the unpublished browser candidate is 0.3.0-alpha.0/API schema 2, with schema 2 inspect/graph-render reports. The [M6A-03 Native path](./m6a-03-native-resources.md) now executes prepared images; [M6A-04 browser resources](./m6a-04-browser-resources.md) now add synchronous capture and public rendering. Final cross-platform qualification remains M6A-05. Read historical version descriptions below in that context.

PR-011 verifies the existing public Rust path with an [independent application](../examples/native-consumer/README.md). It adds no renderer facade, runtime crate, node, shader, document version or dependency to the product crates. The project remains pre-alpha: PR-012 separately verifies the [CLI contract](./cli-contract.md), PR-013 defines [device-loss/OOM classification and cleanup](./gpu-failures.md), PR-014 adds [consumer-owned freshness](./stale-results.md), and PR-015 verifies [actual local package consumption](./package-consumption.md). See the [M4 exit/release assessment](./release.md).

## Reviewed API path

| Consumer action | Public API and ownership |
|---|---|
| Decode source with explicit limits | `MaterialDocument::decode(bytes, &limits)` returns a source document or `DocumentError`; no GPU access. |
| Validate the graph | `into_validated(&limits)` returns immutable `ValidatedDocument`; validation does not repair or populate the source. |
| Select parameters, size and channels | `CompileRequest`, `compile(&validated, &request)` return an immutable `RenderPlan` or `CompileError`. Overrides affect compilation, not the source document. |
| Acquire the requested device | `GpuContext::request(options).await` owns its instance/adapter/device/queue and returns structured requested/actual context evidence. |
| Render | `Renderer::new(context)` consumes the context; `render(&plan).await` returns owned `RenderOutput` or `GpuOperationError`. |
| Render captured images | `prepare` captures caller bytes synchronously; `render_prepared(&prepared).await` consumes that immutable pairing without retaining input snapshots. See [M6A-03](./m6a-03-native-resources.md). |
| Consume data and reports | `channels()`, `pixels()` and `report()` borrow CPU-owned data from the result, which outlives the renderer/context. |
| Inspect GPU failure | `GpuOperationError::reason()`, `device_loss()`, `adapter()` and `allocations()` retain the primary classification, delivered loss, selected adapter and cleanup evidence. |

Core also publicly exposes `BUILT_INS`, `node_contract`, node/port/parameter types and versioned defaults. Consumers do not need a second catalog, parser, compiler or pixel implementation. [Core Rustdoc](../crates/mixture-core/src/lib.rs) now uses complete inline source examples instead of repository-relative files; it accurately describes all eleven contracts and implemented validation/compilation.

The independent example owns its input and Cargo manifest, uses only public crate imports, compiles separately, and runs outside the producer's working directory. Its path dependencies prove source-level API consumption. They do **not** prove that future published archives contain every required asset; PR-015 separately verifies actual local archives with [isolated package resolution](./package-consumption.md). The historical [M3 compile-only probe](./reviews/m3/native-consumer/README.md) remains unchanged as earlier evidence.

## Data and lifetime contract

Requests may select supported channels in any order; returned channels follow material-contract order. Duplicate channels, invalid exposed overrides, malformed graphs and exceeded limits return typed diagnostics before GPU acquisition. Paths, adapter identity and wall times are not plan-hash inputs. The consumer checks deterministic serialization/hashing and that requesting only roughness prunes the unused checker branch.

`RenderedChannel` contains `channel`, `kind`, `source`, `size`, `encoding` and tightly packed, top-left, row-major RGBA8 bytes. Source retains a connected endpoint or versioned default. Color RGB is sRGB, alpha is straight and linear; scalar values and already encoded tangent-space normals use linear bytes. A scalar is replicated to RGB with opaque alpha. Read the encoding and kind rather than guessing from filenames.

The 65×3 consumer fixture deliberately distinguishes these cases:

| Channel | Source | Literal bytes checked after renderer drop |
|---|---|---|
| baseColor | Connected checker | `[137,188,225,255]` and `[255,0,0,128]`, including straight alpha |
| normal | Default | `[128,128,255,255]` |
| roughness | Connected scalar 0.25 | `[64,64,64,255]` |
| height | Default | `[0,0,0,255]` |

These are literal test expectations for the application-owned input; no CPU color conversion or checker algorithm is added. Two public `repeat` overrides must produce different baseColor bytes and different plan hashes. Both outputs are inspected after dropping the renderer and its context, demonstrating actual ownership rather than only a type signature.

`RenderReport` provides actual adapter, plan hash, pass count, size, pipeline-cache hits/misses/entries, allocation estimates/accounting, transferred bytes and wall timings. Descriptor counters exclude driver/pipeline/bind-group and CPU-output overhead. `live_bytes == 0` on a completed render means per-call descriptors were destroyed; it does not promise immediate OS memory reclamation. Retaining two CPU results increases host memory independently of that per-render GPU budget.

## Explicit GPU state and dependency exposure

Normal consumer code needs `GpuContextOptions`, `ContextReport`, `Renderer` and the result/error types. It does not need a direct `wgpu` dependency or the raw handle accessors. Acquisition success is `unverified` until a checker health probe is run; completing a material render does not rewrite the acquisition snapshot into a doctor verdict.

The existing public `instance()`, `adapter()`, `device()` and `queue()` getters are **retained** as advanced escape hatches. Their signatures expose the crate's wgpu major version, currently 30, and are not a facade independent of that dependency. The `wgpu::Limits` fields in `RequestedPolicy`, `AdapterDiagnostics` and `DeviceDiagnostics` have the same coupling. Public serde traits and core `serde_json::Value` parameters likewise expose their respective dependency types. A dependency upgrade that changes any public signature or serialized limit shape requires an explicit compatibility review, tests and documentation; it is not hidden behind unchanged Mixture method names.

Shared references to wgpu handles do not imply immutable GPU state: a caller can submit work, destroy the device or change callbacks through them. Such interference is the caller's responsibility and may make subsequent Mixture operations fail. External submissions and allocations are outside Mixture's reported timings/accounting. Normal consumer acceptance does not certify arbitrary raw-wgpu interoperation. PR-011 removes no existing accessors and makes no new blanket stability promise; the [compatibility record](./compatibility.md) and [release status](./release.md) define the reviewed scope.

Although `Renderer::render` returns a future and its native types satisfy `Send`, native polling may block the executing thread. A responsive application should own the renderer on an explicitly managed worker. Individual waits are bounded to 30 seconds, not an overall render deadline. Dropping a future is not a GPU cancellation contract. PR-014 adds consumer-owned freshness state; it introduces no thread, recovery or implicit alternate execution.

## Diagnostics and verification

PR-013 adds context-owned `DeviceLoss` records and `GpuFailureReason` without changing acquisition snapshots. Replacing the raw device's registered loss callback disables that tracking. An earlier OOM/mapping error remains primary when loss is also observed; inspect the attached loss before considering reuse. The [failure contract](./gpu-failures.md) defines notification timing, scoped unmap cleanup, new codes and strict-decoder compatibility.

`DocumentError::report`, `CompileError::report`, `GpuContextError::report`/`diagnostic` and `GpuOperationError::diagnostic` preserve Mixture's structured errors. The example's render-error wrapper retains the selected context and original source chain rather than replacing the first GPU error. The CPU probe confirms an invalid `repeat=0` names `pattern`, `cellsX` and public ID `repeat`; process tests cover disabled backend acquisition without constructing wgpu state.

```bash
cargo xtask test-consumer
cargo xtask test-plan
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

`test-consumer` is CPU-only and included in `check`. `gpu-smoke` explicitly builds and runs the same independent application, alongside the existing checker, graph and GPU regressions. Its environment policy is passed as explicit consumer arguments; standalone consumer commands do not read that policy implicitly. The [example guide](../examples/native-consumer/README.md) gives direct and pinned-software commands. [PR-011 evidence](./evidence/pr-011/README.md) records local results and paired release measurements; remote CI was deferred at that batch. Later [remote CI acceptance](./evidence/remote-ci/README.md) records the passing CPU/software-GPU matrix; hardware limits remain in the [release record](./release.md).

PR-013 additionally runs the independent `device_loss` integration test with explicit harness environment variables. It destroys cold/warm devices, verifies repeated failures without new allocations and consumes another context's correct outputs. Its fresh JSON receipt is linked by `deviceLossEvidence` in the smoke consumer status. See [PR-013 evidence](./evidence/pr-013/README.md).

[PR-014 freshness state](./stale-results.md) lives entirely in the independent consumer. It accepts generations before compilation, limits active/pending work and retains at most displayed pixels plus a current completion. A stale display stays labeled stale after newer failure. The host still owns responsive worker scheduling.

## ENG-04 compatibility and unpublished versions

The source packages advance to Rust 0.2.0 because adding ScalarBlend to the exhaustive public KernelId/KernelInvocation enums may break downstream exhaustive matches. No non_exhaustive retrofit or other API redesign is made. The browser candidate advances to 0.2.0-alpha.0; API schema 1, .mix version 1 and plan version/hash domain remain unchanged. Serialized existing variants and old plan hash snapshots remain unchanged. Public npm 0.1.0-alpha.0 stays pinned in the registry consumer and must reject scalar-blend with MIX_NODE_UNKNOWN_TYPE. Candidate installation changes only the staged runtime archive/version/integrity; frozen tool dependencies and the pinned disposable Studio source remain intact. Rust packages and the new browser candidate are unpublished; this work does not authorize publication.

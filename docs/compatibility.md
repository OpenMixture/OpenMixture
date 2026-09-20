# Compatibility record

English | [简体中文](./compatibility.zh-CN.md)

**2026-09-20 gate redesign:** New runtime comparisons use [profile v2](./browser-quality.md): bounded amplitude, local bias and channel-specific responses. Runtime and Studio material comparisons share this profile; current reports omit superseded sparse-pixel verdicts. Native goldens and exact checker checks remain unchanged. New browser support still needs source-bound qualification.

**ALPHA-03, 2026-09-20:** [Recorded Windows Chrome/Edge/Firefox runs](./evidence/browser-quality-v2/README.md) each pass 11 cases / 44 channels under v2 on the recorded GT 1030 host and existing archive. Fresh PR packages are separately qualified in Linux CI. This is not a universal browser/driver guarantee; npm remains unpublished.

**M5 acceptance, 2026-09-15:** [Browser acceptance](./evidence/m5-05/README.md) closes M5-05 for the recorded macOS/Linux Chromium matrix: each environment passes 11 post-freeze 1K cases/44 channel comparisons, semantics/quality gates and 12 additional lifecycle renders, alongside independent product isolation, 28 browser contracts and normal production static deployment. Complete pixels and provenance are retained. Alpha readiness is bounded to tested coverage; npm remains unpublished and Studio/M6 require separate decisions. Earlier dated entries below retain their historical status.

This records the tested pre-alpha `0.1.0` boundaries after PR-011–015. It consolidates existing contracts and their review requirements; it does not introduce a 1.0 support guarantee, a binary ABI, an older-format migration promise or automatic GPU recovery. All packages remain unpublished. Several local implementation commits share version `0.1.0`, so retain the implementing commit, lockfile and archive hashes when identifying a build. [PR-015 evidence](./evidence/pr-015/README.md) identifies this one.

## Version and data boundaries

| Boundary | Current contract | Change review |
|---|---|---|
| Source `.mix` | Strict UTF-8 JSON, document version `1`; unknown/duplicate fields, unsupported versions and invalid values fail explicitly. | Never reinterpret a field under the same version. Format changes follow the architecture's version/migration tests and bilingual guide requirements. No legacy format decoder or migration exists. |
| Nodes | Eleven built-ins, independent node version `1`; nine pixel kernels. IDs, ports, parameter types/ranges/defaults and explicit seeds come from core. | Semantic changes require deliberate node/format version decisions, contracts, fixtures and golden evidence; no duplicate catalog or silent default change. |
| RenderPlan | Plan version `1`; immutable compiled semantics and deterministic canonical ordering. | Review serialization and exact hash snapshots. A changed compiler representation or semantic input may invalidate saved plans/hashes; `.mix` remains the source of truth. |
| CLI reports | `schemaVersion: 1` where implemented; `validate` retains its original unversioned `{ok, diagnostics}` envelope. | Preserve each command's presence/null/omission rules and exits; update independent process tests and docs with deliberate wire changes. Do not invent a version field for existing validate output. |
| Diagnostics | Stable code/stage/context vocabulary for this build; messages and native sources remain descriptive evidence. | Additions require vocabulary/consumer review. Strict older decoders can reject new code strings even when the enclosing CLI schema version stays `1`. |
| Rust packages | Source APIs at `0.1.0`, exact peer-package requirements; compilation against the recorded dependency lock. | Review source compatibility and public dependency exposure. Package metadata alone cannot certify arbitrary dependency upgrades or a cross-version binary ABI. |

The [format guide](./file-format.md), [node contracts](./node-contracts.md), [plan guide](./render-plan.md), [CLI contract](./cli-contract.md) and [diagnostic vocabulary](./diagnostics.md) define the detailed executable behavior. Existing tests/stable public behavior retain the authority order in [AGENTS.md](../AGENTS.md).

A plan hash identifies normalized semantics, not original source spelling, package bytes, GPU pixels or request freshness. It includes retained nodes/versions, effective parameters, requested outputs, dimensions and plan structure; source ordering, unused branches, paths, adapters and timing fields do not add semantic identity. Persist `.mix` plus explicit compile requests, and record implementation/shader revision and adapter evidence alongside cached pixels. [Consumer generations](./stale-results.md) remain separate even for identical plans.

## Rust API and dependencies

Core owns parse/validate/compile and GPU-free diagnostics. The renderer owns its context and pipelines, accepts an immutable plan, and returns CPU-owned pixels/reports that survive renderer/context drop. No hidden global device, alternate renderer or implicit acquisition is introduced. Use public methods and typed errors as shown in the [native SDK guide](./native-sdk.md); private modules are not consumer APIs.

Advanced context getters expose wgpu 30 instance/adapter/device/queue types, and diagnostic limits expose wgpu limits. Public serde traits and `serde_json::Value` also expose dependency types. These are documented source dependencies, not an independent opaque ABI. Recompile and review signatures and wire limit fields when upgrading them. A newer dependency satisfying a manifest range is not automatically part of this run's verified matrix.

Some public enums are non-exhaustive; Rust consumers should handle future variants where required by the type. The diagnostic deserializer still rejects unknown string codes. A generic JSON consumer may retain unknown fields/code strings for reporting and reject unsupported semantics explicitly; it must not silently convert an unknown failure into success. Input `.mix` remains strict regardless of a consumer's report-decoding strategy.

PR-013 intentionally added `MIX_GPU_DEVICE_LOST` and `MIX_GPU_OUT_OF_MEMORY`. `GpuOperationError::reason()` retains the first classification; attached loss and allocation evidence are separate. A replaced native loss callback disables context tracking. Acquisition errors whose category is opaque remain `MIX_GPU_DEVICE_REQUEST_FAILED`, rather than being inferred from driver wording. See [GPU failures](./gpu-failures.md).

## Processes and outputs

Product CLI exit codes remain `0` for completed commands, `2` for invocation/source/compile errors, and `1` for operational failures. Usage errors are text on stderr even with `--json`; report transport failure can leave incomplete stdout. Keep streams separate and check process completion before parsing/consuming files. Human messages, timing values, absolute paths and driver text are not portable snapshots.

Outputs are top-left, row-major, tightly packed RGBA8. Color RGB is sRGB with linear straight alpha; scalar red is replicated across RGB with opaque alpha; already encoded tangent-space normals use linear bytes. Reports retain selected channels, default/connected provenance and canonical channel order. Do not guess encoding from filenames or confuse compressed PNG bytes with raw output-buffer bytes.

PNG writes are sequential, not atomic: only completed writes appear in `outputs`, and a failed file can be partially written. PR-014's consumer uses fresh generation directories and explicit cleanup so an old request never writes into the latest directory. Only the newest requested generation may publish; newer failure leaves the retained display explicitly stale. The consumer bounds active/pending work and output retention; it does not add a runtime scheduler to core.

A returned future is not a cancellation guarantee. Native polling can block; the per-wait timeout is not an overall render deadline. Submitted GPU work may finish after a consumer supersedes its request. Per-render descriptor budgets exclude aggregate CPU outputs, encoding, driver overhead and pipeline memory. Nine cache entries and `liveBytes == 0` are ownership/accounting evidence, not promises of immediate physical VRAM reclamation.

## Pixels, releases and review

Accepted ceramic, leather and wood appearances remain unchanged in this train. Pinned SwiftShader comparisons and tolerance-based hardware checks cover the recorded fixtures; they do not guarantee identical pixels on every driver/device. Shader or dependency changes require the existing targeted tests, appropriate material comparisons and reviewed visual evidence. Golden updates remain a separate guarded acceptance action and never an automatic repair for failed tests.

PR-015 verifies normalized local archive contents, exact peer versions and package-local assets through an isolated consumer. It does not certify registry publication, binary installers or all native platforms. [Package resolution](./package-consumption.md) documents the deliberately omitted archive locks, separate pinned verification lock and lack of registry fallback. [Release status](./release.md) records local and remote M4 acceptance and the remaining distribution and untested-hardware limits.

Before a later change, identify affected source/node/plan/report/Rust boundaries; update the smallest decisive tests and both doc languages, explain any compatibility impact and migration/recompile needs, and record the implementation revision. Architecture changes or new compatibility promises still require the existing ADR process. No automatic next implementation train or expanded node vocabulary is approved by this record.

## ENG-04 compatibility and unpublished versions

The source packages advance to Rust 0.2.0 because adding ScalarBlend to the exhaustive public KernelId/KernelInvocation enums may break downstream exhaustive matches. No non_exhaustive retrofit or other API redesign is made. The browser candidate advances to 0.2.0-alpha.0; API schema 1, .mix version 1 and plan version/hash domain remain unchanged. Serialized existing variants and old plan hash snapshots remain unchanged. Public npm 0.1.0-alpha.0 stays pinned in the registry consumer and must reject scalar-blend with MIX_NODE_UNKNOWN_TYPE. Candidate installation changes only the staged runtime archive/version/integrity; frozen tool dependencies and the pinned disposable Studio source remain intact. Rust packages and the new browser candidate are unpublished; this work does not authorize publication.

# Roadmap

English | [简体中文](./ROADMAP.zh-CN.md)

**Current status (2026-09-20):** M0–M5 and M4.1 are complete within their recorded acceptance scope. The [npm Alpha publication and exact registry consumption](./docs/evidence/npm-alpha/README.md) are complete for `@openmixture/runtime@0.1.0-alpha.0`; Rust crates remain unpublished. This status uses retained evidence, not a fresh registry query or GPU run. Historical acceptance does not certify new sources, packages or untested environments.

This page owns current engine priorities. [Alpha closeout](./docs/browser-alpha.md) retains the first delivery, support limits and defect handoff. [INITIAL_PRS](./INITIAL_PRS.md), [M4_PRS](./M4_PRS.md) and [M5_PRS](./M5_PRS.md) retain historical implementation plans; their former next steps are not current execution instructions.

## Current direction — Post-Alpha

OpenMixture independently chooses scope, priority, acceptance and release cadence. Work may originate from approved milestones, maintainer-defined engine use cases, measurements, regressions or external issues. Downstream requests are planning inputs, not a prerequisite for engine work. Studio owns its upgrades, product acceptance, deployment and user trials under the [project boundary](./AGENTS.md).

The next outcome is an independently usable and verifiable SDK, followed by one bounded material-expression increment. Maintain one main feature increment plus necessary maintenance. This batch implements ENG-01 followed by ENG-02; their documentation and rule/test changes are in place. ENG-03 now provides the independent SDK entry below; ENG-04 implements the bounded Scalar composition increment; acceptance is bound to its implementation evidence. Completion claims remain bound to each work item’s evidence.

| Item | Scope and sequence | Exit criteria |
|---|---|---|
| ENG-01 — Post-Alpha roadmap | Implemented: current plan, paired navigation and first-Alpha closeout; separate completed history from future work. | One current planning entry; no downstream product task controls engine progress; resource input and packaging have independent entry conditions. |
| ENG-02 — Graduate stage rules | Implemented after ENG-01: agent/architecture guidance and node-catalog tests. | Replace the expired M3 count gate with explicit use-case admission; retain reviewed catalog identities, contract checks, explicit seeds and all material gates. No node added. |
| ENG-03 — Independent browser SDK entry | Implemented [independent browser consumer](./examples/browser-consumer/README.md), public-package example and separate candidate/registry identity checks. Acceptance is bound to each tested revision; existing Studio coverage remains. | Load .mix, override parameters, select channels, render, diagnose and destroy through public APIs without Studio knowledge. Verify exact published-version consumption separately from the current candidate tarball, recording each identity. |
| ENG-04 — One material-expression increment | Implemented [Scalar composition](./docs/eng-04-scalar-blend.md): two Scalar height fields combined before height-to-normal, with focused native/browser acceptance. New packages remain unpublished. | Approve a concrete use case and input/weight/range/precision/version contract before adding a minimal scalar-blend. Native/browser execution, boundary/tiling/causality tests and existing material regressions pass. |

ENG-03 must preserve the existing pinned Studio consumer checks and their historical evidence. Map existing coverage to the new host before proposing any replacement in a separate change; do not build another full Player or modify Studio. ENG-04 does not authorize a math-node collection or changes to existing blend semantics. New-node documents are not promised to work on older runtimes; old documents retain their behavior and unsupported types must fail explicitly.

## Verification and release ownership

- Engine contracts, pixels, material quality and lifecycle checks gate engine releases.
- Engine-owned qualification of a pinned disposable consumer remains required according to the declared compatibility scope. Intentional versioned API changes require an explicit compatibility/test update, not permanent support for every historical host.
- Studio's current product branch, deployment and user trials do not automatically gate engine releases. Existing six required checks remain intact; this plan does not remove or rename them.
- Changed package content requires a new version and frozen archive identity, qualification, explicit publication and clean exact-version registry consumption. A prior release does not certify new bytes. Browser npm delivery, native Rust source APIs and CLI distribution have separate support/distribution decisions; this batch publishes none.

Follow [governance](./docs/governance.md), [release status](./docs/release.md) and [evidence retention](./docs/evidence-policy.md). Use targeted verification before `cargo xtask check`; GPU changes also require the relevant GPU evidence. Native goldens, exact checker and [browser v2 quality gates](./docs/browser-quality.md) stay distinct. The documented warp precision limitation remains; closed research did not fix it.

## M6 — Resources and Portable Packaging

These are independent, unscheduled candidates, not a single implementation batch.

| Direction | Entry decision | Bounded outcome |
|---|---|---|
| M6-A — External image input | Approve an engine-owned use case that requires external pixels, with a minimal resource contract and testable budgets. No container or Studio resource-panel requirement. | Caller supplies resources; engine validates identity, size, format and budgets, uploads and manages GPU lifetime. Caller owns network/files/permissions; CLI decoding stays an adapter concern. |
| M6-B — Portable asset packaging | Demonstrate a distribution/loading problem that plain .mix plus external files cannot adequately solve. Decide independently of M6-A. | Define inspectable resource identities, bounded loading and path safety; consider .mixpack only when justified. |

Neither direction starts in ENG-01/02. Resource caches, deduplication, incremental uploads and legacy conversion need separate measured or demonstrated problems. No built-in URL downloader, marketplace, general resource manager or editor state in runtime documents is authorized.

## Continuing engineering and later candidates

Measure parsing/validation, compilation, first/reused rendering, readback/conversion and peak resource counts on stable workloads before proposing a local optimization. A complete performance program is not a prerequisite for ENG-04 or M6-A. Numerical semantic changes need their own version/compatibility decision and must not be hidden in an unrelated feature.

Subgraphs, presets, custom shaders/plugins, specialized formats, GPU interop, engine export profiles and native package/binary distribution remain separate decisions. Product editors, 3D preview, accounts and collaboration remain outside this engine batch. CPU/WebGL renderers are forbidden second pixel executors, not ordinary backlog candidates.

## Stop rules

1. New nodes require an approved bounded engine use case, explicit catalog/version review, complete vertical-slice evidence and existing material regressions. ENG-02 graduates the completed M3 count gate; it adds no node.
2. New scope requires an approved milestone or bounded engine use case with acceptance criteria; defects and measured blockers also remain valid work sources.
3. No binary format before a real packaging or loading problem is measured.
4. No second pixel backend or implicit semantic fallback.
5. No generalized optimizer before pass count, memory or runtime data proves it necessary.
6. No new crate without a real boundary.
7. No golden update without reviewable before/after evidence; non-empty pixels alone are not success.
8. No host/engine-specific concern in mixture-core unless it is a universal material semantic.
9. No milestone closes while the primary documented command path or required CI is red.
10. Public behavior, active docs and their Chinese counterparts must agree in the same PR.

## Completed milestones — bounded historical acceptance

These outcomes remain regression obligations, not tasks to repeat or universal support claims.

### M0 — Clean Foundation

Workspace, deterministic repository checks and platform CI completed; the [remote CI record](./docs/evidence/remote-ci/README.md) closes previously deferred clean-checkout gates.

### M1 — Headless wgpu Vertical Slice

Explicit context, structured doctor diagnostics, fixed compute checker and readback completed through the sole wgpu executor. See the [initial train](./INITIAL_PRS.md).

### M2 — .mix v1 Graph MVP

Strict decoding, validation, deterministic plans, graph execution and CLI workflows completed. See the [format contract](./docs/file-format.md) and [graph rendering](./docs/graph-rendering.md).

### M3 — Material MVP and Quality Gate

Three scoped materials accepted with eleven nodes; [M3 review](./docs/m3-review.md) and subsequent [remote CI](./docs/evidence/remote-ci/README.md) retain quality/platform evidence. ENG-02 explicitly supersedes the review's decision to retain the initial count gate; it does not rewrite that historical decision or accept new pixels.

### M4 — Stable Native SDK

Public Rust/CLI consumption, failures, freshness, lifetime and isolated package checks completed within the [M4 release assessment](./docs/release.md). Rust publication and CLI binary distribution are separate.

### M4.1 — Repository Maintenance

PR governance and native required checks completed; [governance](./docs/governance.md) also records subsequent browser enforcement. Tracked ruleset files alone do not prove remote enforcement.

### M5 — WebAssembly and Browser WebGPU

Thin WASM binding, complete npm runtime and independent consumer accepted in the [recorded browser matrix](./docs/evidence/m5-05/README.md). Later ordinary-browser and first-publication evidence lives in [Alpha closeout](./docs/browser-alpha.md). Player/Studio implementation history does not assign new product work to this repository.

## Historical checkpoints (status at the time)

The following paragraphs retain their original dates and next-step assessments; current status and ownership follow the Post-Alpha plan above.

**Studio MVP, 2026-09-16:** the independent product now passes its [saved-file qualification](./docs/evidence/studio-qualification/README.md) in the recorded macOS matrix. Engine work here adds detached CLI reference/comparison tooling only; runtime semantics and native goldens are unchanged. PR integration, publication and M6 remain separate.

**M5 acceptance, 2026-09-15:** [Browser acceptance](./docs/evidence/m5-05/README.md) closes M5-05 for the recorded macOS/Linux Chromium matrix: each environment passes 11 post-freeze 1K cases/44 channel comparisons, semantics/quality gates and 12 additional lifecycle renders, alongside independent product isolation, 28 browser contracts and normal production static deployment. Complete pixels and provenance are retained. Alpha readiness is bounded to tested coverage; npm remains unpublished and Studio/M6 require separate decisions. Earlier dated entries below retain their historical status.

**Player export update, 2026-09-15:** the M5-04 open → edit → channel preview → PNG download workflow now passes local acceptance in the independent product. The clean isolated consumer passed 28 Chromium checks and nine Node tests. Twelve 128×128 channel PNGs from three materials decode to the exact public-runtime bytes with correct sRGB/linear metadata. Additional checks cover eight-channel 65×3 downloads, stale-export suppression and encoding failure. [Product evidence](https://github.com/OpenMixture/Studio/blob/77f00deb5410b73140220fabe31f4f8599b3279d/docs/evidence/m5-04-export/README.md) binds the exact source and unchanged runtime archive. M5-05 1K cross-runtime quality, stress, formal browser CI and deployment/compatibility qualification remain open; nothing is published.

**Player update, 2026-09-14:** the M5-04 parameter/preview slice is implemented and [locally verified](https://github.com/OpenMixture/Studio/blob/7370e482e2dcacb9911f5663f8ec4f9e8da6a4cc/docs/evidence/m5-04-parameters/README.md) in the independent product: Rust-metadata controls, channel selection, one active render plus one replaceable pending request, stale-preview diagnostics and lifecycle cleanup. The clean isolated consumer passed 23 Chromium checks and six Node tests, including three-material 128×128 previews. PNG export and full M5-04/M5-05 acceptance remain open; the runtime archive is unchanged and unpublished.

**Browser checkpoint, 2026-09-14:** [M5-02/M5-03 local acceptance](./docs/evidence/m5-02-03/README.md) closes initial browser execution and isolated tarball consumption on the recorded Chromium/macOS environment. Next: M5-04 Player MVP, then M5-05 full browser acceptance.

**Current milestone:** M4, M4.1 and [bounded M5 browser acceptance](./docs/evidence/m5-05/README.md) are complete. The Studio MVP is accepted in its recorded environment; npm publication and engine M6 have not started.

**Implementation status:** PR-001 through PR-015 are implemented. The `.mix` graph path, eleven nodes and three accepted 1K materials have local Metal and pinned SwiftShader evidence; the [M3 review](./docs/m3-review.md) records quality, release measurements and bounded 2K allocation. The [M4 train](./M4_PRS.md) verifies public Rust/CLI contracts, failures, stale results, bounded retention and actual package consumption. [Remote CI acceptance](./docs/evidence/remote-ci/README.md), completed on `8b43c84`, now adds clean-checkout Linux/macOS/Windows CPU checks and Linux pinned SwiftShader smoke, packaged consumers, three 1K materials and 2K trace. The previously deferred M0/M1 platform gates are closed for this matrix. See [release status](./docs/release.md) for compatibility and untested hardware limits. Packages remain unpublished. The thin browser binding and independent product repository now exist; see the browser start guide for the implemented checker slice and remaining M5 gates.

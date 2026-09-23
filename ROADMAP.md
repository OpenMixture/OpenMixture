# Roadmap

English | [简体中文](./ROADMAP.zh-CN.md)

**Current integration state:** M6-B and NUM-01, including prerequisite PRs #39–45, are merged into main with all six combined checks passing. MAT-01 is accepted within its [recorded scope](./docs/evidence/mat-01/README.md) on the unpublished 0.6 candidate. MAT-01 nodes and qualification tooling are integrated through PRs #50–52 ([integration record](./docs/evidence/mat-01/integration/README.md)); human visual acceptance and post-merge checks are complete, with accepted source manifests at 0.6.0 / 0.6.0-alpha.0; the current MAT-02b working candidate is 0.7.0 / 0.7.0-alpha.0, not yet qualified; the published browser version is 0.3.0-alpha.0. See [release status and hardware scope](./docs/release.md).

**Historical checkpoint (2026-09-21):** M0–M5 and M4.1 are complete within their recorded acceptance scope; ENG-01–04 are implemented. [Browser Alpha 0.2.0 publication and exact registry consumption](./docs/evidence/npm-020-alpha/README.md) deliver Scalar composition. Rust crates remain unpublished. Historical acceptance does not certify new sources, packages or untested environments.

**M6A-05 qualification, 2026-09-21:** [Retained acceptance](./docs/evidence/m6a-05/README.md) closes comprehensive qualification for the recorded Linux software matrix: eight resource channel comparisons are byte-exact, Scalar and three-material regressions and all six required checks pass. Windows hardware parity remains failed and outside accepted coverage; the ≤1 gate is unchanged. Subsequently integrated into main and [published as browser 0.3.0-alpha.0](./docs/evidence/npm-030-alpha/README.md); Rust crates remain unpublished.

This page owns current engine priorities. [Alpha closeout](./docs/browser-alpha.md) retains the first delivery, support limits and defect handoff. [INITIAL_PRS](./INITIAL_PRS.md), [M4_PRS](./M4_PRS.md) and [M5_PRS](./M5_PRS.md) retain historical implementation plans; their former next steps are not current execution instructions.

**Historical Windows investigation, before NUM-01 (2026-09-22):** [Reproduction and isolation](./docs/evidence/windows-numerics/README.md) locate the recorded resource-normal failure upstream in noise arithmetic/half rounding; identical-height normal replay is exact. An FMA-only experiment still fails a legal scale and is not a runtime fix. Hardware qualification remains open; a numerical/version decision is the next prerequisite for a semantic repair.

## Current direction — Post-Alpha

**NUM-01, implemented and qualified within recorded scope:** the authorized [stable value-noise migration](./docs/stable-noise.md) implements `fractal-noise@2` in the integrated, unpublished 0.5 candidate. [Retained evidence](./docs/evidence/stable-noise/README.md) closes the measured Windows Vulkan/DX12-versus-Chrome resource/Scalar regression (max difference 0), passes the migrated software material matrix and all six CI checks, and retains before/after visual review. This does not publish a package, qualify old 0.3 hardware pixels, or close unrelated cellular/warp precision limits.

**M6B-05 qualification complete:** [Retained acceptance](./docs/evidence/m6b-05/README.md) closes M6-B implementation/qualification for the recorded Linux software matrix: 16 package channel comparisons are exact, independent source/archive consumers and existing materials pass, and all six required checks succeed. The frozen v1 Windows hardware normal comparison remains failed; NUM-01 separately qualifies explicit v2 value-noise inputs. Rust 0.5.0 / browser 0.5.0-alpha.0 remain unpublished; the stack is integrated and release remains separate.

**M6-B started, 2026-09-22:** the maintainer selected portable asset packaging as the next main increment. [M6B-01](./docs/m6b-portable-assets.md) establishes the measured offline-delivery gap and the M6B-02–05 format/implementation/qualification sequence. This kickoff does not implement a loader, select a binary format or publish a package.

OpenMixture independently chooses scope, priority, acceptance and release cadence. Work may originate from approved milestones, maintainer-defined engine use cases, measurements, regressions or external issues. Downstream requests are planning inputs, not a prerequisite for engine work. Studio owns its upgrades, product acceptance, deployment and user trials under the [project boundary](./AGENTS.md).

Maintain one main feature increment plus necessary maintenance. ENG-01–04 are complete; their outcomes and acceptance boundaries are retained under completed milestones below, not queued for another implementation round.

| Planning state | Current content |
|---|---|
| Available baseline | Published `@openmixture/runtime@0.3.0-alpha.0`: external image resources, API schema 2, plan v2; `.mix v1` unchanged. Qualified Rust 0.6.0 source is consumable; MAT-01 is qualified within its recorded scope; crates remain unpublished. [Exact archive and registry qualification](./docs/evidence/npm-030-alpha/README.md). |
| Active increment | MAT-02b morphology and saturating-subtraction implementations under the integrated frozen contract (PR #55). MAT-01 nodes, fixture and tools are integrated and accepted within [recorded scope](./docs/evidence/mat-01/README.md). Both nodes are working candidates; their qualification/integration and material qualification remain pending. Publication remains separate. |
| Current qualification | [MAT-01](./docs/evidence/mat-01/README.md): frozen brick matrix, recorded software and GT 1030 Native/browser comparisons, retained human decision and six passing post-merge checks. Previous v1 Windows failures remain historical failures; no publication or general hardware guarantee. |
| Planned sequence | MAT-01 structure → MAT-02 layered weathering → MAT-03 woven surfaces → MAT-04 graph reuse. Each stage enters implementation only with its bounded contract and catalog/version review. PERF-MAT is measurement-triggered support, not a prerequisite program. |

Follow the selected contract task sequence, recording design integration, implementation and accepted evidence separately. Studio upgrades, deployment and product acceptance are not prerequisites for engine planning or release.

## Verification and release ownership

- Engine contracts, pixels, material quality and lifecycle checks gate engine releases.
- Engine-owned qualification of a pinned disposable consumer remains required according to the declared compatibility scope. Intentional versioned API changes require an explicit compatibility/test update, not permanent support for every historical host.
- Studio's current product branch, deployment and user trials do not automatically gate engine releases. Existing six required checks remain intact; this plan does not remove or rename them.
- Changed package content requires a new version and frozen archive identity, qualification, explicit publication and clean exact-version registry consumption. A prior release does not certify new bytes. Browser npm delivery, native Rust source APIs and CLI distribution have separate support/distribution decisions; the [0.2.0 Alpha delivery](./docs/evidence/npm-020-alpha/README.md) publishes only the browser npm package.

Follow [governance](./docs/governance.md), [release status](./docs/release.md) and [evidence retention](./docs/evidence-policy.md). Use targeted verification before `cargo xtask check`; GPU changes also require the relevant GPU evidence. Native goldens, exact checker and [browser v2 quality gates](./docs/browser-quality.md) stay distinct. The documented warp precision limitation remains; closed research did not fix it.

## M6 — Resources and Portable Packaging

M6-A and M6-B are implemented, integrated and qualified within their recorded software matrices. The table below retains their scope boundaries.

| Direction | Entry decision | Bounded outcome |
|---|---|---|
| M6-A — External image input | M6A-01 selects the [external-height contract](./docs/m6a-resource-contract.md) for design integration. M6A-02–05 now deliver implementation and bounded qualification on main; no container or Studio resource-panel requirement. | Caller supplies resources; engine validates identity, size, format and budgets, uploads and manages GPU lifetime. Caller owns network/files/permissions; CLI decoding stays an adapter concern. |
| M6-B — Portable asset packaging | [M6B-01](./docs/m6b-portable-assets.md) records the portable binding gap and single-asset offline use case; M6B-02 selects USTAR/ownership; M6B-03–05 implement, qualify and integrate it. | Shared Native/browser loading, inspectable resource identity, bounded memory and path safety; preserve loose-input semantics and qualify the exact candidate in M6B-05. |

M6-B starts through the maintainer's explicit decision, not automatically from M6-A qualification. Format, loader, adapters and acceptance remain separate work items. Resource caches, deduplication, incremental uploads and legacy conversion need separate measured or demonstrated problems. No built-in URL downloader, marketplace, general resource manager or editor state in runtime documents is authorized.

## Continuing engineering and later candidates

Measure parsing/validation, compilation, first/reused rendering, readback/conversion and peak resource counts on stable workloads before proposing a local optimization. A complete performance program is not a prerequisite for M6-A or another bounded feature. Numerical semantic changes need their own version/compatibility decision and must not be hidden in an unrelated feature.

Subgraphs and presets follow MAT-04 below; their format and implementation remain undecided. Custom shaders/plugins, specialized formats, GPU interop, engine export profiles and native package/binary distribution remain separate decisions. Product editors, 3D preview, accounts and collaboration remain outside this engine batch. CPU/WebGL renderers are forbidden second pixel executors, not ordinary backlog candidates.

## Material capability roadmap — staged delivery

The 2026-09-23 planning decision selects material-expression breadth as the next direction. The target is reusable procedural texture materials, not parity with the complete Substance 3D suite. Keep one feature increment active. MAT-01 is accepted within its recorded scope and MAT-02b node implementation is active; later stages are ordered planning commitments, not blanket approval for new nodes or schemas. Track contract acceptance, implementation, pixel qualification, integration and publication separately. No package version or release date is assigned here.

The planning baseline had thirteen node types, image input, public parameters, eight material output channels and portable assets. MAT-01 adds two accepted node types for structured brick materials; later stages still need the processing vocabulary specified below. A channel slot is not a channel generator; a portable archive is not a reusable subgraph. Existing ceramic/leather/wood acceptance remains bounded and mandatory.

| Stage / entry | Bounded outcome and candidate capabilities | Exit criteria / explicit exclusions |
|---|---|---|
| MAT-01 — structure; accepted within recorded scope | One parameterized brick/paving family: shape/profile generation, staggered repetition, explicit seeded per-element variation, mortar mask, edge softening and shared height/color/roughness structure. Prefer the smallest reusable primitives over a universal Tile Sampler. | Independently control element count/aspect, mortar width, row offset, height/color variation and edge softness; changing seed preserves layout controls. Qualify default, regular, staggered and varied cases. No general scatter, arbitrary region segmentation or universal shape library. |
| MAT-02 — layered weathering; after MAT-01 | Painted metal with exposed metal and rust. Add only demonstrated missing Scalar arithmetic, spatial mask composition, local filtering/morphology, remapping and height/normal composition. Evaluate height-derived AO/curvature only where the selected material needs them. | A common wear mask consistently controls baseColor, metallic, roughness and height/normal; test intact paint, exposed substrate and rust, plus mask endpoints and transitions. Existing scalar-blend semantics remain unchanged. No mesh baking, physically simulated corrosion or mandatory new material-layer type. |
| MAT-03 — woven surfaces; after MAT-02 | One woven fabric family with warp/weft structure, independent thread dimensions, over/under height order, directional detail and scale-aware sampling. Admit additional directional patterns/noise or transforms only against failed material cases. | Plain and varied weave cases retain correct crossings, periodic seams and coherent height/normal at multiple resolutions; quantify high-frequency aliasing and document limits. No cloth simulation, fiber geometry, arbitrary weave catalog or universal antialiasing promise. |
| MAT-04 — reusable recipes; after MAT-03, or explicitly reprioritized when duplication blocks earlier work | Extract shared recipes from at least two accepted material families into graph instances with typed inputs/outputs, public parameter mapping, deterministic seed handling and versioned dependencies; add a small preset corpus. | Demonstrate reuse in two independent parent graphs, instance isolation, equivalent flattened results, deterministic hashing and actionable cycle/missing-version/type errors. Decide dependency closure and package compatibility before implementation. No implicit filesystem/network lookup, marketplace, user scripting or automatic document migration. |
| PERF-MAT — measurement-triggered support during any stage | Record cold/warm execution, compilation, readback, pass count and peak accounted buffers on that stage's real graph. When a frozen budget fails, prefer last-consumer release and compatible texture reuse; consider partial recomputation only after measuring repeated edits. | Fix numeric time/memory targets and hardware identity before optimization; show before/after measurements with unchanged pixel gates. No speculative general optimizer, global cache or second executor. |

These capabilities are candidates, not final node names or a node-count target. Region IDs/local coordinates, distance fields, richer gradients/noise and vector/color transforms enter only when one of the bounded cases demonstrates a gap. If solving a case requires a larger seam, split and review its contract rather than silently expanding the stage. High-precision image I/O and GPU interop require independent consumer evidence and compatibility decisions.

### MAT-01 implementation sequence

Work-item IDs below are not GitHub PR numbers. Each implementation seam uses an actual PR and both language versions of affected documentation.

| Item | Deliverable | Dependency / completion |
|---|---|---|
| MAT-01a — contract and acceptance design | [Selected contract](./docs/mat-01-structured-materials.md) and [frozen matrix](./fixtures/materials/brick-paving/qualification-plan.json) specify brick-pattern@1 and scalar-mask-blend@1, parameters, coordinates, seeds, sampling, precision and budgets. | Design, implementations and bounded material acceptance are complete; see the retained MAT-01 evidence. Any format/architecture change needs its own decision. |
| MAT-01b — structural generation | Implement the approved shape/repetition/variation seam through Core lowering and the sole wgpu executor; add focused fixtures and public contract documentation. | After MAT-01a; shader/node tests, invalid inputs, deterministic seed behavior, boundary and odd-size cases pass. Preserve explicit catalog review. |
| MAT-01c — material composition | Add only the approved missing mask/edge-processing seam; construct the material's coherent baseColor, roughness, height and normal outputs with exposed controls. | After MAT-01b; independent parameter causality, seam and height/normal tests plus reviewable channel and PBR contact sheets. Avoid duplicating a kernel or disguising a broad math library as one node. |
| MAT-01d — qualification and handoff | Exercise Native and browser public APIs and the exact candidate package; retain source/archive-bound receipts, visual review and measured costs. Record support limits and remaining gaps. | After MAT-01c; common gates below pass, all six required CI checks pass, both Agent Guides agree. Integration and any subsequent publication have separate receipts. |

MAT-02/03/04 use the same contract → minimal implementation seams → material evidence → qualification sequence. Break them into actual work items at entry, using lessons from the previous stage; do not assign speculative node IDs or shipping versions now.

[MAT-02a selected contract](./docs/mat-02-layered-weathering.md) freezes the morphology/subtraction identities, caller graph/control mappings and [qualification plan](./fixtures/materials/painted-metal/qualification-plan.json), supported by source-bound existing-input feasibility. It selects an unpublished 0.7 candidate at first implementation; the working node implementations select 0.7 and add both reviewed identities; qualification/integration remain pending. MAT-01 exit is satisfied. After contract integration, MAT-02b implements each node through a separate PR; the [actual 2K failure and valid 1K baseline](./docs/evidence/perf-mat-before/README.md) now activate PERF-MAT without relaxing the frozen budget. [ADR 0009](./docs/decisions/0009-transient-texture-reuse.md) selects the bounded allocation/version change: PERF-MATa retains evidence and contract; PERF-MATb implements Core scheduling, wgpu reuse and thin version projections together; PERF-MATc qualifies pixels, costs, lifecycle and public consumers. Plan/API v3 and the 0.8 candidate are selected for first implementation, not implemented by this design. MAT-02 remains the single active material increment; its full material gates and MAT-03/04 remain required.

### Common material acceptance and ownership

- Before implementation, bind a material brief and cases to measurable structure, independent parameter effects, output-channel relationships, periodic boundary behavior and numeric tolerances. Include default, endpoint, combined-parameter and at least two-seed cases for randomized behavior. Do not loosen gates after seeing failed candidates.
- Validate at 256, 1024 and 2048 square sizes plus an odd rectangular case chosen in the contract. Evaluate seamless repetition using the periodic sampling convention, not identical opposite border bytes. Include low-resolution/high-frequency stress and finite-value/range checks. Record unsupported combinations explicitly.
- Retain channel sheets and controlled PBR comparison views at fixed lighting, scale and camera, including close-up and tiled views. Keep engine-owned evidence separate from Studio product implementation. Human visual acceptance complements structural/numerical checks; a default screenshot or nonempty output alone cannot close a stage.
- Run the owning targeted tests from the Agent Guide, shader validation for kernels, material regressions, public Native/browser candidate consumption and affected cross-runtime comparisons. Preserve original and explicitly migrated material matrices, frozen v1 failures and all six required checks. Hardware claims are limited to recorded adapter/backend results; software passes do not qualify arbitrary hardware.
- Core owns contracts, validation, deterministic lowering and hashes; wgpu owns all pixel processing and GPU resources. The asset codec owns any accepted dependency-package changes; CLI/WASM/JS remain thin. No new crate without an actual boundary. `.mix v1`, package v1 and current plan/API schemas stay unchanged by this planning document. New types are additive only where existing schemas can represent them; old runtimes must reject unsupported identities, and old nodes must not change meaning.
- Retain accepted evidence under the [evidence policy](./docs/evidence-policy.md), with source/lock/package identities, adapters, commands, machine results and visual-review verdicts. Update both Agent Guides for major changes. Publication follows the separate [release process](./docs/release.md); engine work does not authorize Studio upgrades or deployment.

Photo-to-material reconstruction, mesh-aware painting/baking, editor UX, production 3D preview and material marketplaces are separate product or research tracks. This roadmap does not promise all physical material models, including transmission, subsurface scattering or anisotropic shading, through the current output contract.

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

### ENG-01–04 — Post-Alpha independent SDK and Scalar composition

These work items are complete. The retained exit criteria describe their acceptance scope, not instructions to restart them. Browser delivery is bound to the [0.2.0-alpha.0 publication record](./docs/evidence/npm-020-alpha/README.md); Rust distribution and platform support claims remain unchanged.

| Item | Scope and sequence | Exit criteria |
|---|---|---|
| ENG-01 — Post-Alpha roadmap | Implemented: current plan, paired navigation and first-Alpha closeout; separate completed history from future work. | One current planning entry; no downstream product task controls engine progress; resource input and packaging have independent entry conditions. |
| ENG-02 — Graduate stage rules | Implemented after ENG-01: agent/architecture guidance and node-catalog tests. | Replace the expired M3 count gate with explicit use-case admission; retain reviewed catalog identities, contract checks, explicit seeds and all material gates. No node added. |
| ENG-03 — Independent browser SDK entry | Implemented [independent browser consumer](./examples/browser-consumer/README.md), public-package example and separate candidate/registry identity checks. Acceptance is bound to each tested revision; existing Studio coverage remains. | Load .mix, override parameters, select channels, render, diagnose and destroy through public APIs without Studio knowledge. Verify exact published-version consumption separately from the current candidate tarball, recording each identity. |
| ENG-04 — One material-expression increment | Implemented [Scalar composition](./docs/eng-04-scalar-blend.md): two Scalar height fields combined before height-to-normal, with focused native/browser acceptance. [Browser 0.2.0-alpha.0 is published](./docs/evidence/npm-020-alpha/README.md); Rust packages remain unpublished. | Approve a concrete use case and input/weight/range/precision/version contract before adding a minimal scalar-blend. Native/browser execution, boundary/tiling/causality tests and existing material regressions pass. |

ENG-03 must preserve the existing pinned Studio consumer checks and their historical evidence. Map existing coverage to the new host before proposing any replacement in a separate change; do not build another full Player or modify Studio. ENG-04 does not authorize a math-node collection or changes to existing blend semantics. New-node documents are not promised to work on older runtimes; old documents retain their behavior and unsupported types must fail explicitly.

## Historical checkpoints (status at the time)

The following paragraphs retain their original dates and next-step assessments; current status and ownership follow the Post-Alpha plan above.

**Studio MVP, 2026-09-16:** the independent product now passes its [saved-file qualification](./docs/evidence/studio-qualification/README.md) in the recorded macOS matrix. Engine work here adds detached CLI reference/comparison tooling only; runtime semantics and native goldens are unchanged. PR integration, publication and M6 remain separate.

**M5 acceptance, 2026-09-15:** [Browser acceptance](./docs/evidence/m5-05/README.md) closes M5-05 for the recorded macOS/Linux Chromium matrix: each environment passes 11 post-freeze 1K cases/44 channel comparisons, semantics/quality gates and 12 additional lifecycle renders, alongside independent product isolation, 28 browser contracts and normal production static deployment. Complete pixels and provenance are retained. Alpha readiness is bounded to tested coverage; npm remains unpublished and Studio/M6 require separate decisions. Earlier dated entries below retain their historical status.

**Player export update, 2026-09-15:** the M5-04 open → edit → channel preview → PNG download workflow now passes local acceptance in the independent product. The clean isolated consumer passed 28 Chromium checks and nine Node tests. Twelve 128×128 channel PNGs from three materials decode to the exact public-runtime bytes with correct sRGB/linear metadata. Additional checks cover eight-channel 65×3 downloads, stale-export suppression and encoding failure. [Product evidence](https://github.com/OpenMixture/Studio/blob/77f00deb5410b73140220fabe31f4f8599b3279d/docs/evidence/m5-04-export/README.md) binds the exact source and unchanged runtime archive. M5-05 1K cross-runtime quality, stress, formal browser CI and deployment/compatibility qualification remain open; nothing is published.

**Player update, 2026-09-14:** the M5-04 parameter/preview slice is implemented and [locally verified](https://github.com/OpenMixture/Studio/blob/7370e482e2dcacb9911f5663f8ec4f9e8da6a4cc/docs/evidence/m5-04-parameters/README.md) in the independent product: Rust-metadata controls, channel selection, one active render plus one replaceable pending request, stale-preview diagnostics and lifecycle cleanup. The clean isolated consumer passed 23 Chromium checks and six Node tests, including three-material 128×128 previews. PNG export and full M5-04/M5-05 acceptance remain open; the runtime archive is unchanged and unpublished.

**Browser checkpoint, 2026-09-14:** [M5-02/M5-03 local acceptance](./docs/evidence/m5-02-03/README.md) closes initial browser execution and isolated tarball consumption on the recorded Chromium/macOS environment. Next: M5-04 Player MVP, then M5-05 full browser acceptance.

**Historical M5 checkpoint:** M4, M4.1 and [bounded M5 browser acceptance](./docs/evidence/m5-05/README.md) are complete. The Studio MVP is accepted in its recorded environment; npm publication and engine M6 have not started.

**Historical M4 implementation status:** PR-001 through PR-015 are implemented. The `.mix` graph path, eleven nodes and three accepted 1K materials have local Metal and pinned SwiftShader evidence; the [M3 review](./docs/m3-review.md) records quality, release measurements and bounded 2K allocation. The [M4 train](./M4_PRS.md) verifies public Rust/CLI contracts, failures, stale results, bounded retention and actual package consumption. [Remote CI acceptance](./docs/evidence/remote-ci/README.md), completed on `8b43c84`, now adds clean-checkout Linux/macOS/Windows CPU checks and Linux pinned SwiftShader smoke, packaged consumers, three 1K materials and 2K trace. The previously deferred M0/M1 platform gates are closed for this matrix. See [release status](./docs/release.md) for compatibility and untested hardware limits. Packages remain unpublished. The thin browser binding and independent product repository now exist; see the browser start guide for the implemented checker slice and remaining M5 gates.

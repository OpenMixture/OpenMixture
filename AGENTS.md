# Agent Guide

English | [简体中文](./AGENTS.zh-CN.md)

This file is the operational contract for coding agents and contributors working on Mixture.

Read [ARCHITECTURE.md](./ARCHITECTURE.md) before changing boundaries and [ROADMAP.md](./ROADMAP.md) before adding scope. [INITIAL_PRS.md](./INITIAL_PRS.md) and [M4_PRS.md](./M4_PRS.md) retain the completed implementation batches. [M5_PRS.md](./M5_PRS.md) retains the completed browser implementation train; the [browser SDK contract](./docs/browser-sdk.md) defines its public contract. Current work follows the Post-Alpha roadmap; distinguish planning, implementation and accepted evidence. Follow [repository governance](./docs/governance.md) for new branches, actual GitHub pull requests, and required checks, and [evidence retention](./docs/evidence-policy.md) when recording results. A tracked ruleset file alone does not prove that remote protection is active.

## Current implementation essentials

Baseline reviewed 2026-09-22: M6-A, M6-B and NUM-01 are integrated. [Release status](./docs/release.md) owns current publication and qualified-platform claims; source manifests own build versions. The working PERF-MAT candidate is Rust 0.8.0 / browser 0.8.0-alpha.0, unpublished and not yet qualified; the last qualified MAT-01 baseline is 0.6 within the recorded software and GT 1030 scope; the recorded published browser package is 0.3.0-alpha.0. Integration, qualification and publication are distinct. Do not treat historical milestone plans as the current backlog.

- **Version boundaries:** `.mix` remains v1; `RenderPlan` and browser API schema are v3. The plan hash domain is `mixture-render-plan-v3\0`, including selected external-image identities. Graph `render` and `inspect --plan` reports use schema 3; doctor, fixed checker and asset envelopes use schema 1; `validate` stays unversioned. See [compatibility](./docs/compatibility.md).
- **Node semantics:** seventeen node types lower to fifteen pixel kernels. `scalar-blend@1` and `image-input@1` are implemented. Resolve explicit type/version pairs: `fractal-noise@1` and `@2` coexist. Counts describe this snapshot, not admission targets or permanent limits.
- **External resources (M6-A):** callers supply tightly packed `rgba8-linear` image bytes; Core validates identities, dimensions and budgets, captures selected pixels synchronously, and returns immutable `PreparedRender`. Wgpu uploads and executes them. No implicit path/URL lookup, PNG decoding, resampling or mutable caller-buffer retention. See [resource contract](./docs/m6a-resource-contract.md).
- **Portable assets (M6-B):** optional `mixture-asset` owns the shared CPU-only `.mixpack v1` reader/writer using a strict canonical uncompressed USTAR profile. Preserve exact `.mix` bytes and node versions, validate hashes and the full resource closure before slicing, and never extract archive paths. Package v1 rejects all `resourceRef` overrides, including unchanged values, with `MIX_PACKAGE_RESOURCE_OVERRIDE`; ordinary overrides retain Core semantics. See [format/ownership](./docs/m6b-package-format.md) and [codec](./docs/m6b-03-cpu-assets.md).
- **Package memory:** default ceilings are 67 MiB archive, 64 KiB manifest and 202 MiB accounted byte buffers; callers may only lower them. Charge retained Rust capacity, source/manifest scratch, selected Core snapshots and adapter copies. Browser accounting includes both its synchronous JS snapshot and Rust copy; release transport buffers before asynchronous GPU execution. This is a byte-buffer policy, not a process-RSS guarantee. Keep [adapter rules](./docs/m6b-04-adapters.md) and executable limits synchronized.
- **Browser boundary:** `mixture-wasm` and `packages/runtime` are thin bindings over the same Rust logic. Import is inert; `loadRuntime` and GPU creation are explicit. Snapshot accepted bytes/options before yielding, reject busy/closing calls before copying, share one busy slot across loose/package rendering, and retain owned outputs after idempotent destruction. `inspectPackage` is CPU-only; `renderPackage` uses the sole wgpu executor.
- **Numerical qualification (NUM-01):** explicit v2 **value** noise uses Q0.24 arithmetic. Recorded GT 1030 Vulkan/DX12-versus-Chrome resource/Scalar height and normal comparisons have maximum difference 0; frozen v1 resource/asset fixtures retain the historical normal difference 8/255 against the unchanged ≤1/255 gate. Do not auto-migrate documents, reset goldens, or generalize this repair to cellular, warp, arbitrary graphs or all hardware. See [stable noise](./docs/stable-noise.md) and its source-bound evidence.
- **Delivery gates:** main requires all six checks: Linux/macOS/Windows CPU checks, pinned SwiftShader GPU/package consumption, WASM/npm packaging, and Chromium material qualification. Preserve both original and explicitly migrated material matrices. Candidate archives and exact registry consumption are separate evidence; passing CPU/interface/software checks alone does not qualify Windows hardware pixels. Follow [governance](./docs/governance.md) and [evidence retention](./docs/evidence-policy.md).

## Current material planning rule

The [material capability roadmap](./ROADMAP.md) records MAT-01 brick/paving acceptance and selects MAT-02 layered-weathering material qualification with measured PERF-MAT support next, followed by MAT-03 woven surfaces and MAT-04 graph reuse. Later capabilities remain planned, not implemented catalog entries. Freeze each bounded contract, catalog/version review and acceptance cases before implementation; add only primitives justified by the selected material. PERF-MAT starts from measured cost failures. This sequence addresses expression gaps while retaining one active feature increment.

The [MAT-01a contract](./docs/mat-01-structured-materials.md) selects brick-pattern@1 and scalar-mask-blend@1 and freezes [acceptance cases/budgets](./fixtures/materials/brick-paving/qualification-plan.json). The working candidate implements brick-pattern and scalar-mask-blend; the four-channel brick fixture is accepted within its frozen scope. MAT-01 qualification is complete with the recorded human decision and six passing post-merge checks. The candidate Native matrix harness is documented in the [fixture guide](./fixtures/materials/brick-paving/README.md); the candidate browser CI now requires its twenty-case/eighty-channel comparison. The harness also checks package roundtrips and matched-adapter timing budgets; `test-node brick-pattern` checks periodic-origin shifts and raw f16 range; `test-material brick-paving` runs the Native matrix, including measured 64×64-cell aliasing limits. Both brick entry points explicitly route Cargo builds to `target/native-consumer`; qualification must not rely on private Git excludes or weaken clean-source checks. See the [CI routing failure](./docs/evidence/mat-01/ci-failure-4bf/README.md) and isolated regression. Fixed WebGPU PBR review tools are consumer visualization only. [Windows candidate evidence](./docs/evidence/mat-01/README.md) retains complete selected pixels and source-bound receipts; the separate [human decision](./docs/evidence/mat-01/human-decision.json) accepts these views. Nodes and qualification tooling are integrated through PRs #50–52; the [integration record](./docs/evidence/mat-01/integration/README.md) separates passing pre-merge checks from post-merge verification. The human decision and recorded machine gates together close this bounded stage; integration alone does not assert acceptance. Core owns semantics/lowering, wgpu owns pixels, and adapters stay thin; Studio work remains separately owned. The contract schedules a 0.6 candidate at first implementation because exhaustive Rust kernel matches change; this design alone does not change formats, node behavior, manifests or migration policy. Each stage requires the roadmap's multi-resolution, parameter-causality, seam, PBR visual and public Native/browser evidence, existing material regressions and all six required checks. Preserve historical failures and qualify only measured hardware. Publication stays separate.

The [MAT-02a selected contract](./docs/mat-02-layered-weathering.md) selects scalar-morphology@1 and scalar-subtract@1 for the bounded painted-metal use case; Both identities are integrated through PRs #56–57; node evidence does not accept the full material. The [subtraction fixtures](./fixtures/nodes/scalar-subtract/README.md) verify required Scalar inputs, saturating endpoints and exact half-representable differences with one reserved-zero 16-byte uniform; Core owns the contract and lowering and wgpu the sole pixel operation. [Recorded subtraction evidence](./docs/evidence/mat-02-subtract-node/README.md) has 64 exact Native/browser cases per Windows Vulkan/DX12 backend; PR #57 merged as cbeb367261bf09b3b7acd2540b4efbee3000e7e7 with all six pre/post-merge checks passing (see the [integration summary](./docs/perf-mat-texture-reuse.md)). Its [node fixtures](./fixtures/nodes/scalar-morphology/README.md) cover typed validation, raw half extrema, periodic support sets, degenerate dimensions, axis commutation and radius monotonicity; [Recorded Windows public-consumer evidence](./docs/evidence/mat-02-morphology/README.md) has 192 exact Native/browser cases per Vulkan/DX12 backend; PR #56 passed all six required checks and merged as 313074451cd6ddb5e5f82cce933c3d4fe3b4ed38; the combined PR #57 main matrix also passes all six checks. Core owns semantics/lowering and wgpu the sole kernels. The [frozen design and plan](./fixtures/materials/painted-metal/README.md) specify caller controls, 23 planned passes, seven presets/four sizes/five channels, exact probe fields/ABIs and unchanged 4/255 downsample, 24-pass and 512 MiB targets. [Existing-input feasibility](./docs/evidence/mat-02-input-feasibility/README.md) and the [subtraction counterexample](./docs/evidence/mat-02-subtraction/README.md) support the design, not new-node qualification. [Actual PERF-MAT entry evidence](./docs/evidence/perf-mat-before/README.md) retains the 2K rejection (805,306,832 > 536,870,912 bytes) and valid 23-pass, five-channel 1K render. [ADR 0009](./docs/decisions/0009-transient-texture-reuse.md) selects Core-owned deterministic last-use/physical-slot planning and wgpu-owned per-render reuse, without changing kernels or pass order. The [working reuse implementation](./docs/perf-mat-texture-reuse.md) updates Core estimates and wgpu slots together, selects plan/hash/API/graph-report v3 and unpublished 0.8, and preserves .mix/.mixpack v1 and budgets. Recompile source requests and invalidate plan/hash caches. Full clean-candidate pixel/cost/lifecycle/public-consumer qualification and all six checks remain required. The exact eleven-case v2-to-v3 hash migration preserves kernels and pixel goldens; trace-2k now verifies physical-slot accounting. No global pool or shader change is introduced. First node implementation selects Rust 0.7.0/browser 0.7.0-alpha.0 for exhaustive kernel changes; downstream exhaustive matches must handle ScalarMorphology and ScalarSubtract. No format, automatic migration, second executor or publication change. The contract is integrated through [PR #55](https://github.com/OpenMixture/OpenMixture/pull/55); preserve six gates and paired documentation.

## Mandatory maintenance of this guide

Every major adjustment **must update `AGENTS.md` and `AGENTS.zh-CN.md` in the same PR**. This is a completion requirement, not an optional follow-up. Major adjustments include architecture/ownership changes; public APIs, commands, formats, node semantics, versions or migration policy; resource/memory/lifecycle rules; required checks or qualification scope; milestone integration/publication that changes the working baseline; and cross-project or delivery-policy changes.

Update the relevant current rule, map, command or test entry rather than accumulating contradictory status banners. Record the change, its reason, the owning boundary, compatibility/migration consequences, and the required verification or evidence link. Keep detailed design in its focused guide/ADR, current release state in the release guide, and source-bound results in evidence records. Do not rewrite historical failures as passes or mark planned work implemented. Routine refactors, typo fixes and repeated runs need no new milestone entry unless they alter these operational rules.

Before marking a PR ready, explicitly review its impact on this guide. For a major adjustment, list the updated sections and supporting links in the PR description; for other work, record that no operational rule changed. If code or accepted decisions disagree with this guide, follow the authority order below and correct the drift in the same change. A major adjustment with a stale or unsynchronized guide is not complete.

## Mission

Mixture is a Rust material-graph compiler and headless texture renderer with exactly one pixel execution path: `wgpu` compute shaders.

The critical path is:

```text
.mix -> parse -> validate -> compile -> RenderPlan -> wgpu -> texture outputs
```

Prefer the smallest change that makes this path more correct, observable, and useful to a real consumer.

## Non-negotiable invariants

Do not violate these rules without an accepted architecture decision record and an explicit roadmap change.

1. `mixture-core` must not depend on `wgpu`, browser APIs, CLI libraries, or UI frameworks.
2. `mixture-wgpu` is the only pixel executor. Do not add a CPU renderer or a second TypeScript/WebGL renderer.
3. Do not silently fall back to another semantic executor. Return a structured error and report the selected adapter.
4. GPU state must be explicit. Do not introduce a process-global adapter, device, queue, renderer, or cache.
5. `.mix` is the source of truth. Do not place editor layout, thumbnails, transient UI state, or target-engine configuration in v1 documents.
6. Graph traversal and serialization must be deterministic. Never rely on hash-map iteration order for externally visible behavior.
7. Every randomized node requires an explicit seed.
8. Every built-in node requires a contract, one WGSL implementation, focused fixtures, documentation, and targeted tests.
9. Golden outputs may not be overwritten merely to make a failing test pass.
10. Generated bindings and adapters must remain thin. Runtime behavior belongs in normal typed Rust modules.
11. Do not add a new crate unless an actual compilation, publication, runtime, or dependency boundary requires it.
12. Every new built-in node requires an approved, bounded engine use case and explicit catalog/version review. M3 acceptance graduated the initial count gate under ENG-02; node count is neither a goal nor a permanent ceiling. Existing material gates remain required.

## Authority order

When sources disagree, use this order:

1. executable tests and stable public behavior;
2. `ARCHITECTURE.md` invariants;
3. accepted architecture decision records;
4. `ROADMAP.md` scope and stop rules;
5. issue or pull-request descriptions;
6. comments and historical notes.

Do not preserve accidental behavior solely because it exists in an old Mixture repository. Legacy code is research evidence, not a compatibility contract.

## Quick reference

The implemented repository commands are listed in [docs/development.md](./docs/development.md). `cargo xtask check`, `fmt`, `clippy`, `test`, `test-core`, `test-format`, `test-plan`, `test-consumer`, `package-check`, `test-node <id>`, `test-material <id>`, `golden check`, guarded `golden update <id> --accept`, `doc`, `deps`, `links`, `trace-2k`, `shader-check`, and the explicit `gpu-smoke` work today. The CLI implements CPU-only `validate` and `inspect --plan`, verified `doctor` with `--skip-probe`, and graph `render`/`render-builtin checker`, in human and JSON modes.

Common implemented commands are:

```bash
# Fast repository checks used during normal development
cargo xtask check

# Focused checks
cargo xtask test-core
cargo xtask test-node <node-id>
cargo xtask test-material <material-id>
cargo xtask test-format
cargo xtask test-plan
cargo xtask test-consumer
cargo xtask package-check
cargo xtask shader-check
cargo xtask golden check
cargo xtask gpu-smoke

# CLI workflows
cargo run -p mixture-cli -- doctor --json
cargo run -p mixture-cli -- validate <file.mix> --json
cargo run -p mixture-cli -- inspect <file.mix> --plan --json
cargo run -p mixture-cli -- render <file.mix> --size 512 --out ./out

# Portable asset workflows (raw tightly packed rgba8-linear input)
cargo run -p mixture-cli -- asset pack material.mix --size 65x3 --image Input input.rgba --out material.mixpack --json
cargo run -p mixture-cli -- asset inspect material.mixpack --json
cargo run -p mixture-cli -- asset render material.mixpack --size 65x3 --output height --out ./out --json

# Shared codec and public browser package checks
cargo test --locked -p mixture-asset
npm test --prefix packages/runtime
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh-evidence-directory>
```

Keep introduced command meanings stable. Label any future command proposal as unimplemented until it exists. Do not silently replace a documented command with an unrelated one.

## Repository map

The map below identifies current runtime ownership. Historical PR-001–015 implementation batches live in [INITIAL_PRS](./INITIAL_PRS.md) and [M4_PRS](./M4_PRS.md); browser implementation is retained in [M5_PRS](./M5_PRS.md). Current resource/asset/noise rules are summarized above. Introduce modules for actual ownership, without empty stubs.

```text
crates/mixture-core/
  src/document.rs       .mix data model and decoding
  src/validation.rs     format, graph, parameter, and budget validation
  src/registry.rs       built-in node contracts
  src/compiler/allocation.rs  deterministic last-use and physical-slot planning
  src/compiler.rs       document -> RenderPlan
  src/resources.rs      image identities, resource limits and PreparedRender
  src/plan.rs           backend-neutral execution plan
  src/error.rs          stable diagnostic codes and structured errors
  src/limits.rs         explicit safety limits and measured limit failures
  src/nodes/            one small module per built-in node contract

crates/mixture-asset/
  src/archive.rs        strict byte-only USTAR loading
  src/writer.rs         deterministic package writing
  src/manifest.rs       manifest/closure validation
  src/limits.rs         explicit byte-buffer accounting

crates/mixture-wgpu/
  src/context.rs        explicit adapter/device/queue acquisition
  src/executor.rs       RenderPlan execution
  src/resources.rs      textures, buffers, and lifetime management
  src/readback.rs       GPU texture readback
  src/diagnostics.rs    adapter and execution evidence
  shaders/nodes/        one WGSL implementation per pixel kernel

crates/mixture-cli/
  src/main.rs           argument parsing and command dispatch
  src/commands/doctor.rs
  src/commands/validate.rs
  src/commands/inspect.rs
  src/commands/render.rs
  src/commands/asset.rs

crates/mixture-wasm/    thin Rust browser bindings; src/assets.rs package adapter
packages/runtime/      public ESM loader, TypeScript API and lifecycle bridge
scripts/browser-runtime/  candidate/registry and material qualification
fixtures/packages/     portable asset acceptance and rejection corpus

fixtures/nodes/         focused node input/output fixtures
fixtures/materials/     golden materials and acceptance evidence
examples/               small readable user examples
xtask/                   repository automation only
```

## Key ownership rules

### `mixture-core`

Owns:

- document decoding and version checks;
- graph and budget validation;
- node IDs, ports, parameters, defaults, and versions;
- output dependency slicing and stable topological ordering;
- parameter override validation;
- compilation to `RenderPlan`;
- stable plan hashing;
- external-image identity, resource budgets, synchronous pixel capture and immutable `PreparedRender`;
- structured diagnostics that do not depend on a GPU.

Must not own:

- adapter selection;
- GPU resource allocation;
- WGSL module compilation;
- PNG file I/O;
- command-line output formatting;
- browser or Node.js bindings.

### `mixture-wgpu`

Owns:

- `wgpu::Instance`, adapter, device, and queue lifetime;
- mapping every `KernelId` to exactly one WGSL implementation;
- pipeline creation and caching;
- compute pass encoding;
- texture allocation, reuse, and release;
- readback and execution metrics;
- GPU-specific structured diagnostics.

Must not own:

- `.mix` parsing;
- graph repair;
- node default values;
- parameter override semantics;
- a second copy of the node catalog;
- engine-specific texture naming or packaging.

### `mixture-asset`

Owns byte-only package format, deterministic writing, borrowed/owned loading, full resource closure and package budgets; reuses Core validation, resource identity and preparation. Must not own file/network I/O, GPU state, browser objects, pixel execution or duplicate node semantics.

### `mixture-wasm` and `packages/runtime`

Own thin bindings, explicit loading, synchronous input capture, type/error transfer and public lifecycle; package budgeting calls the shared Rust codec. Do not reimplement graph semantics, the package parser or rendering in JS/bindings.

### `mixture-cli`

Owns only:

- file and directory I/O;
- command-line parsing;
- calling public APIs from `mixture-core`, `mixture-asset` and `mixture-wgpu`;
- PNG encoding and JSON/human-readable reports;
- process exit codes.

Keep command modules thin. If logic is useful outside the CLI or needs direct unit tests, move it into the appropriate library crate.

## Cross-project task boundary

An OpenMixture task owns engine code, public runtime contracts, packages, producer-side qualification and release documentation. Studio owns its dependency upgrades, product acceptance, deployment and user trials. A dependency, shared evidence or access to another checkout does not authorize taking over that project's work. Unless the user explicitly assigns cross-project work, do not edit Studio, run its product delivery tasks, or manage its issues/PRs from an engine task.

Studio reports suspected runtime defects by opening an upstream issue in OpenMixture. The issue should include the exact package version/build identity, a minimal `.mix`, request/overrides, browser/OS/adapter, reproduction steps, expected versus actual behavior and the first structured error or failed current gate. Triage ownership before implementation; reproduce engine defects here, add focused regression coverage and deliver the fix through an engine PR and qualified release. Report the fix/version and engine verification in the upstream issue; Studio independently upgrades and accepts the product. A product-only failure stays with Studio. Locally discovered engine regressions and failing engine checks remain valid work triggers without a Studio issue.

Existing producer-owned CI may use a pinned, disposable Studio consumer as its test host. This does not authorize changes to the Studio repository or hosted product. Preserve required checks and historical cross-repository evidence; their scope is qualification, not cross-project task ownership.

Work may also originate from approved milestones, maintainer-defined engine use cases and measurements under the [roadmap](./ROADMAP.md). External issues are one planning input; OpenMixture owns prioritization and acceptance. This does not expand authority to edit consumer projects.

## Standard workflow

For every task:

1. Read the relevant document and nearby implementation before editing.
2. Identify the owning crate and avoid cross-boundary shortcuts.
3. Add or update the smallest focused test that expresses the intended behavior.
4. Implement the smallest coherent change.
5. Run targeted checks first.
6. Inspect generated artifacts or rendered images when behavior is visual.
7. Run `cargo xtask check` before declaring the change ready.
8. Update public docs in the same pull request when behavior or commands change.
9. Apply the mandatory guide-maintenance rule: update both Agent Guides for major adjustments in the same PR and record guide impact in the PR description.

Keep English documentation and its paired `*.zh-CN.md` version synchronized in the same pull request. See [documentation maintenance](./docs/README.md) for language and source-bundle conventions.

Do not begin by running the full GPU matrix for a local one-line parser change. Do not finish a GPU or format change after running only a narrow unit test.

## Task-to-test map

| Changed area | Minimum targeted verification |
|---|---|
| fixed built-in checker before the node registry | `cargo xtask shader-check`, focused `checker`/`readback` tests, and `cargo xtask gpu-smoke` |
| `.mix` decoding or versioning | `cargo xtask test-format` |
| graph validation | `cargo xtask test-core` plus focused validation test |
| compiler or plan hashing | `cargo xtask test-plan` |
| M2 contracts before graph pixel execution (PR-005) | `cargo test --locked -p mixture-core --test registry` and `cargo xtask test-format` |
| node contract with a graph pixel executor (PR-007 onward) | `cargo xtask test-node <id>` |
| WGSL kernel | `cargo xtask shader-check` and `cargo xtask test-node <id>` |
| resource lifetime or readback | focused `mixture-wgpu` tests plus `cargo xtask gpu-smoke` |
| CLI command | command snapshot/integration test; `test-consumer` for public report/exit contracts |
| golden material | `cargo xtask test-material <id>` and visual evidence review |
| repository automation | the exact affected `xtask` test plus `cargo xtask check` |
| Resource identity/preparation | `cargo xtask test-core`, `test-plan`, independent Native/browser resource consumption; affected cross-runtime comparisons for pixel changes |
| Shared asset codec/package format | `cargo test --locked -p mixture-asset`, `test-consumer`, `package-check`; `gpu-smoke` and package cross-runtime comparisons for pixel-path changes |
| WASM/JS lifecycle or package adapter | `npm test --prefix packages/runtime`, clean WASM/npm build, independent candidate consumption and affected resource/Scalar/asset pixel comparisons |
| Documentation/Agent Guide | Check source/evidence, synchronize languages, `cargo xtask links`, `cargo xtask check`; serialize xtask on Windows to avoid rewriting a running launcher |
| public API | downstream example or external-consumer fixture |

CI owns the full platform and GPU matrix. Local development should prefer the smallest decisive command.

For browser environment/adapter setup and cross-runtime pixel comparison, follow the full recipes in [browser resources](./docs/m6a-04-browser-resources.md), [asset qualification](./docs/m6b-05-qualification.md) and [stable noise](./docs/stable-noise.md). Use fresh evidence directories; interface tests do not replace pixel qualification.

## Adding a built-in node

A node addition is a vertical slice, not only a registry entry.

Required steps:

1. Link an approved roadmap item or bounded engine use case with acceptance criteria. Define compatibility and version behavior before changing the reviewed catalog; downstream product work is not a prerequisite.
2. Add a small node contract module under `mixture-core/src/nodes/`.
3. Define stable type ID, node version, ports, parameter types, defaults, ranges, and validation.
4. Add or extend the exhaustive `KernelId` mapping.
5. Add exactly one WGSL implementation under `mixture-wgpu/shaders/nodes/`.
6. Add focused fixtures for defaults, boundaries, invalid parameters, and at least one non-trivial case.
7. Add an agent-readable node document describing inputs, outputs, parameters, tiling behavior, and known precision limits.
8. Run shader validation and node tests on the pinned software adapter, existing material regressions, and relevant Native/browser public-consumer checks.
9. Verify that the node solves the approved material-expression use case or documented consumer failure. Update the explicit type/version catalog expectation in the same PR; do not replace it with a count-only or self-derived assertion.

A node is not complete when it compiles. It is complete when the contract, shader, fixtures, diagnostics, and visual evidence agree.

Prefer obvious static Rust data and explicit matches. A larger catalog alone does not justify a procedure macro; require demonstrated repetition before proposing abstraction.

## Changing `.mix`

Format work must be versioned and deliberate.

Required steps:

1. State whether the change is additive, migratable, or breaking.
2. Update `document.rs` and validation rules.
3. Update `ARCHITECTURE.md` and the focused file-format guide.
4. Add decode, reject, round-trip, and deterministic serialization tests.
5. Update examples and fixtures only through an explicit migration.
6. Preserve old-version decoding when the roadmap promises it; otherwise fail with a specific unsupported-version error.

Never reinterpret an existing field without changing a version. Never let the CLI silently repair an invalid graph and then save different semantics.

## Changing a shader

Shader changes are semantic changes.

Required steps:

1. Read the node contract and all focused fixtures.
2. Run the existing node golden before editing and retain the result for comparison.
3. Make the smallest shader change.
4. Run shader parsing/validation before GPU execution.
5. Compare default and boundary parameter variants.
6. Inspect tiling seams, non-finite values, and output range.
7. Produce a before/after contact sheet when golden pixels change.
8. Explain in the pull request why the new result is more correct.

Do not update a golden in the same command that renders it. The update path must require an explicit acceptance flag and must be disabled in CI.

## Golden update policy

The implemented commands are:

```bash
cargo xtask golden check
cargo xtask golden update <material-id> --accept
```

`golden update` must:

- refuse to run in CI;
- require `--accept`;
- write new files to a reviewable location first;
- report per-channel metrics and changed-pixel ratios;
- generate a before/after/difference contact sheet;
- never commit or stage files;
- leave a machine-readable acceptance report.

A reviewer must be able to distinguish a correct visual change from an accidental baseline reset.

## GPU debugging workflow

When GPU execution fails:

1. Run `mixture doctor --json` and save its complete output.
2. Record the requested adapter policy and the actual selected adapter/backend.
3. Reproduce with the smallest built-in or node fixture.
4. Distinguish adapter acquisition, device creation, shader compilation, pipeline creation, execution, and readback stages.
5. Preserve the first structured GPU error; do not replace it with a generic wrapper message.
6. Run on the pinned software adapter to separate hardware/driver behavior from graph semantics.
7. Add a focused regression test or diagnostic probe before broad refactoring.

A successful fallback is not a fix because Mixture has no alternate semantic renderer.

## Error and diagnostic rules

Library errors must be structured and stable enough for CLI, WebAssembly, and agents to consume.

Each externally visible failure should provide, where applicable:

- stable `code`;
- `stage`;
- concise `message`;
- document path or node/port/parameter ID;
- evidence or measured value;
- actionable `suggestion`;
- source error chain for human diagnostics.

Use typed errors in library crates. `anyhow` is acceptable at the CLI orchestration boundary, not as the public library error model.

Do not expose raw driver messages as the only error contract. Preserve them as evidence beneath a stable Mixture code.

## Rust and dependency rules

- Prefer safe Rust. Any `unsafe` block requires a safety comment, a focused test, and an architecture justification.
- Avoid `unwrap`, `expect`, and indexing panics in library paths that can be reached by untrusted documents or device behavior.
- Keep dependencies minimal and purpose-specific.
- Do not add a dependency when the standard library or an existing dependency is sufficient.
- Do not update the lockfile incidentally during an unrelated task.
- Keep `Cargo.lock` committed and run CI with `--locked`.
- Treat shader parsers, image decoders, and serialization dependencies as input-boundary security dependencies.
- Avoid hidden background threads and process-global caches.

## Determinism rules

- Use stable ordering for nodes, edges, outputs, diagnostics, and serialized plans.
- Include document version, node versions, requested outputs, resolution, parameter overrides, and relevant resource identities in plan hashes.
- Do not include adapter names, timestamps, absolute paths, or non-semantic logging fields in plan hashes.
- Reject NaN and infinite document parameters unless a node explicitly defines them, which v1 nodes should not.
- Random behavior must be derived from explicit integer seeds and documented coordinate conventions.

## Performance work

Measure before optimizing.

Performance changes must begin with:

- a reproducible fixture;
- adapter and backend evidence;
- resolution and requested outputs;
- wall time, GPU time when available, peak estimated bytes, and pass count;
- a correctness baseline.

Prefer, in order:

1. dependency slicing so unused outputs are not compiled;
2. releasing intermediates after their last consumer;
3. reusing compatible physical textures;
4. pipeline caching;
5. reducing readback and encoding work;
6. only then considering pass fusion or more complex compilation.

Do not introduce a general optimizer, SSA, shader AST, or precision inference system to solve a single unmeasured case.

## Pull-request rules

- Integrate new changes through actual GitHub pull requests following [repository governance](./docs/governance.md). Keep milestone work-item IDs separate from GitHub PR numbers; historical `PR-001` through `PR-015` identify implementation batches.
- One architectural seam per pull request.
- Keep adapters and generated bindings thin.
- Avoid unrelated renames and formatting churn.
- Add tests before or with behavior changes.
- Every pull request must state what is intentionally out of scope.
- Large changes should be stacked in dependency order and remain independently reviewable.
- Do not bypass required checks.
- Do not merge a pull request that leaves the documented primary command path broken.
- Retain accepted results and their required review content under the [evidence policy](./docs/evidence-policy.md); preserve historical records and keep ordinary repeated output in CI artifacts or ignored local directories.

## Definition of done

A task is complete only when:

- ownership boundaries remain intact;
- targeted tests pass;
- decisive repository checks pass;
- structured diagnostics remain meaningful;
- public behavior and docs agree;
- major adjustments are recorded in both Agent Guides with rationale, owning boundary, compatibility impact and verification links;
- visual changes have reviewable evidence;
- no hidden fallback or global state was introduced;
- out-of-scope work is not smuggled into the change;
- the next agent can discover how to reproduce and verify the result from repository files alone.

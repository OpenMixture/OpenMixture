# Roadmap

English | [简体中文](./ROADMAP.zh-CN.md)

**Current milestone:** M4 and [M4.1 repository maintenance](./docs/evidence/m4-1/README.md) are accepted. [M5-01 planning](./M5_PRS.md) is in progress, 2026-09-12; browser implementation and acceptance have not started.

**Implementation status:** PR-001 through PR-015 are implemented. The `.mix` graph path, eleven nodes and three accepted 1K materials have local Metal and pinned SwiftShader evidence; the [M3 review](./docs/m3-review.md) records quality, release measurements and bounded 2K allocation. The [M4 train](./M4_PRS.md) verifies public Rust/CLI contracts, failures, stale results, bounded retention and actual package consumption. [Remote CI acceptance](./docs/evidence/remote-ci/README.md), completed on `8b43c84`, now adds clean-checkout Linux/macOS/Windows CPU checks and Linux pinned SwiftShader smoke, packaged consumers, three 1K materials and 2K trace. The previously deferred M0/M1 platform gates are closed for this matrix. See [release status](./docs/release.md) for compatibility and untested hardware limits. Packages remain unpublished. M5-01 prepares the browser delivery contract; the planned runtime, independent product and browser acceptance remain unimplemented.

This roadmap is organized by verifiable outcomes, not dates, quarters, node counts, or feature volume. A milestone is complete only when all exit criteria pass in a clean environment.

## Roadmap rules

1. Work on the current milestone before pulling later scope forward.
2. Every milestone must leave the primary command path green.
3. Engineering evidence does not substitute for visual material quality or external-consumer evidence.
4. A feature that is not required by a milestone exit criterion is out of scope unless it fixes a blocking defect.
5. Stop rules at the end of this document override attractive but speculative follow-up work.

---

## M0 — Clean Foundation

### Outcome

A small Rust repository that a human or coding agent can clone, understand, verify, and extend without inheriting the old Mixture architecture.

### Scope

- Rust workspace with:
  - `mixture-core`;
  - `mixture-wgpu`;
  - `mixture-cli`;
  - private `xtask` tooling.
- `README.md`, `AGENTS.md`, `ARCHITECTURE.md`, `ROADMAP.md`, and `INITIAL_PRS.md`.
- fixed Rust toolchain and edition;
- formatting, Clippy, unit-test, docs, and dependency-policy commands;
- CI for Linux, macOS, and Windows non-GPU checks;
- dual MIT/Apache-2.0 license files;
- initial ADR directory and architecture decisions;
- empty but documented fixture and example directories.

### Exit criteria

```bash
cargo xtask check
```

passes from a clean clone on supported development platforms, and:

- no crate contains placeholder runtime behavior presented as implemented;
- all documented repository paths exist;
- CI uses the committed lockfile with `--locked`;
- public mission, non-goals, and ownership boundaries are consistent across documents.

### Out of scope

- adapter acquisition;
- WGSL;
- `.mix` parsing;
- rendering;
- browser work.

---

## M1 — Headless `wgpu` Vertical Slice

### Outcome

The project proves that it can acquire an explicit GPU context, execute one offscreen compute pass, read pixels back, and explain failures.

### Scope

- explicit `GpuContext` with caller-visible options;
- adapter, backend, device-limit, and feature diagnostics;
- stable GPU error codes and stages;
- `mixture doctor` in human and JSON modes;
- one built-in 64x64 checker compute shader;
- offscreen texture allocation;
- texture-to-buffer readback;
- PNG encoding in the CLI;
- a dedicated GPU smoke workflow using a pinned software adapter;
- one real-hardware validation record.

### Exit criteria

```bash
mixture doctor --json
mixture render-builtin checker --size 64 --out checker.png
```

must demonstrate:

- an actual selected adapter and backend;
- successful compute dispatch and readback;
- a deterministic checker image on the pinned software adapter;
- a structured, actionable failure when no adapter is available;
- no window surface, frame loop, graph format, or hidden fallback.

### Out of scope

- `.mix` documents;
- node registry;
- resource pooling;
- WebAssembly.

---

## M2 — `.mix` v1 Graph MVP

### Outcome

A small `.mix` graph can be parsed, validated, compiled into a deterministic `RenderPlan`, inspected, and rendered through the existing `wgpu` path.

### Scope

- `.mix` v1 JSON model;
- strict decoding and conservative budgets;
- nodes, typed ports, parameters, versions, edges, and exposed parameters;
- cycle detection and stable topological ordering;
- output dependency slicing;
- typed `RenderPlan` and stable plan hash;
- `validate`, `inspect --plan`, and graph-aware `render` commands;
- initial built-in nodes:
  1. `constant-scalar`;
  2. `constant-color`;
  3. `checker`;
  4. `levels`;
  5. `blend`;
  6. `material-output`;
- focused fixtures and shader checks for each node;
- raw and PNG output for requested channels, including connected/default source status.

### Exit criteria

At least three readable examples must complete:

```text
.mix -> validation -> RenderPlan -> wgpu -> requested PNG outputs
```

with:

- deterministic plan hashes;
- stable diagnostics for malformed documents;
- requested-output pruning proven by tests;
- no GPU initialization during core-only tests;
- no duplicate graph or node semantics in the CLI;
- no general optimizer or binary format.

### Out of scope

- realistic material claims;
- image resources;
- subgraphs;
- presets;
- browser bindings.

---

## M3 — Material MVP and Quality Gate

### Outcome

Mixture demonstrates that its small node set can produce three useful, accepted materials at 1K and can handle at least one representative 2K workload without unbounded intermediate growth.

### Scope

Add only the nodes required by the golden materials, keeping the total at twelve or fewer:

- `fractal-noise`;
- `gradient-map`;
- `transform-2d`;
- `warp`;
- `height-to-normal`;
- one reserved node only if a documented material blocker proves it necessary.

Establish three golden materials:

1. **Glazed ceramic/checker** — graph fundamentals, color, roughness, documented neutral-normal default, height, and seamless tiling; it may later adopt generated normals without changing its acceptance intent.
2. **Leather-like surface** — micro-height, parameter causality, roughness, and normal response.
3. **Anisotropic wood-like surface** — directional structure, transform, warp, and scale behavior.

Add:

- guarded golden-check and golden-update workflow;
- software-adapter expected outputs;
- tolerant real-adapter comparisons;
- tiling seam metrics;
- non-finite and output-range checks;
- parameter causality variants;
- human visual-review records;
- 1K acceptance for all three materials;
- 2K allocation trace for the most demanding material;
- last-consumer release and compatible texture reuse only if measurement requires them;
- peak-byte and pass-count reporting in `inspect` and render reports.

### Exit criteria

All three materials must:

- validate and compile without warnings that hide missing behavior;
- render requested baseColor, normal, roughness, and height channels;
- pass the pinned software-adapter golden;
- pass tolerance-based real-adapter checks;
- pass tiling and non-degeneracy gates;
- show meaningful output changes for documented parameter variants;
- have an accepted human-review note;
- render at 1024x1024 through the one `wgpu` path.

At least one material must render at 2048x2048 under the documented transient-memory budget or produce evidence that justifies the minimum required lifetime optimization.

No thirteenth built-in node may be added before every M3 gate is green.

### Out of scope

- a node editor;
- embedded images;
- custom shaders;
- engine profiles;
- public marketplace content.

---

## M4 — Stable Native SDK

### Outcome

An independent native consumer can use Mixture without repository-private imports or source-relative coupling.

### Scope

- reviewed public Rust API for parse, validate, compile, render, and diagnostics;
- reviewed stable CLI JSON schemas and exit codes;
- explicit cancellation or stale-result handling appropriate for long renders;
- device-lost and out-of-memory diagnostics;
- bounded pipeline/resource caches;
- package-consumer tests using only public crates or the built CLI;
- one independent Node.js or Rust consumer example;
- release checklist and compatibility policy.

Node.js integration should start by spawning the CLI. N-API, a daemon, or custom IPC requires profiling evidence that process startup is a real blocker.

### Exit criteria

A separate consumer fixture can:

```text
load .mix
-> validate
-> override exposed parameters
-> request channels
-> render
-> consume outputs and structured metrics
```

without importing workspace internals.

Public docs, examples, CLI JSON, and library behavior must agree.

### Out of scope

- WebAssembly;
- editor UI;
- binary packaging;
- engine-specific exporters in core.

---

## M4.1 — Repository Maintenance

**Accepted implementation:** `cc98dc9298add5ed172e0c1752d8a768b61a0eb5`, through [PR #1](https://github.com/OpenMixture/OpenMixture/pull/1). The [completion record](./docs/evidence/m4-1/README.md) binds the passing bootstrap/PR/main checks, active protection, default branch and retained evidence. Later documentation changes require their own PR and main checks.

### Outcome

The accepted M4 implementation is discoverable from a protected default `main`, and subsequent changes have real PR records and durable acceptance evidence. This maintenance scope was selected on 2026-09-12; it does not reopen M4 acceptance or authorize M5 runtime work.

### Scope

- Establish `main` from the accepted history without force pushes or deleting historical branches, verify its CI, and make it the default branch.
- Apply the [reviewable branch ruleset](./.github/main-ruleset.json): PR required, resolved conversations, no force push/deletion, four required GitHub Actions checks, and zero required approvals for the single-maintainer workflow.
- Run CI for PRs, pushes to `main`, and manual dispatch, preserving GPU serial execution, driver pin/cache verification, and every acceptance gate.
- Distinguish historical implementation identifiers `PR-001` through `PR-015` from actual GitHub PR numbers.
- Synchronize current English/Chinese status, adopt [evidence retention](./docs/evidence-policy.md), and describe verified environments without inventing support guarantees.

### Exit criteria

- Live GitHub state confirms the correct default branch and active protection, and the current integrated revision has all four required checks passing.
- A real PR integrates the workflow/documentation change through the protected path; its current checks, merge revision, and resulting main runs are recorded.
- `cargo xtask check` passes, active documentation agrees with current evidence, and historical source bundles, goldens, human receipts, and bound visual evidence remain unchanged.
- A compact completion receipt identifies branches, revisions, runs, applied rules, and any descriptive source checkpoint tag; no package publication is claimed.

The [governance guide](./docs/governance.md) defines activation order and verification. Runtime/API changes, new nodes, M5 implementation, M6, large tooling refactors, and package distribution are out of scope.

---

## M5 — WebAssembly and Browser WebGPU

**Planning entry, 2026-09-12:** [M5-01](./M5_PRS.md) records the selected delivery direction after M4/M4.1 acceptance. This change prepares the paired plan and [browser SDK contract](./docs/browser-sdk.md); it does not implement bindings, create the product repository, publish a package or claim browser acceptance. M5-02 through M5-05 remain planned work.

### Outcome

An independent Player installs one complete browser runtime package and uses the same `.mix`, Rust node contracts, compiler, RenderPlan semantics and WGSL as native consumers.

### Scope

- Keep the engine and browser package build in this repository; add `mixture-wasm` as a thin binding over `mixture-core` and the sole `mixture-wgpu` executor.
- Adapt browser target features, asynchronous execution/readback and failure delivery while preserving native behavior.
- Ship one package, provisionally `@openmixture/runtime`, containing the JS facade, TypeScript declarations and WASM from one build; consumers need no Rust toolchain.
- Provide explicit WASM/GPU initialization, GPU-free validation and node-contract queries, existing exposed-parameter overrides, requested channels, owned RGBA8 and structured diagnostics.
- Use one independent product repository, provisionally `OpenMixture/Studio`, with Player first and Studio deferred. It owns files, controls, 2D preview, latest-request handling and export through the public package.
- Verify real tarball installation and one pinned Vite consumption recipe, including production static deployment and configurable WASM resource paths.
- Add three-material browser/native regression, lifecycle/failure evidence and an Alpha readiness assessment. Registry publication is a separate distribution action.

The [implementation plan](./M5_PRS.md) orders M5-01 contracts → M5-02 browser execution → M5-03 packaged consumer → M5-04 Player MVP → M5-05 acceptance. Proposed names do not establish repository creation or package ownership. The independent Player fulfills the external Web consumer criterion; another full demo is not required.

### Exit criteria

An isolated product consumer can complete:

```text
install the runtime tarball
-> load .mix bytes
-> validate and apply exposed parameter overrides
-> request outputs
-> render through browser WebGPU
-> display and export textures
```

It must use the package's public entry without producer-private imports, source-relative assets, Rust installation or compiling postinstall scripts. Production-built assets must run under static serving, including a non-root base path. JS, declarations and WASM must match the recorded archive/build.

Rust remains the semantic authority. Native and browser plans must be semantically equivalent for identical requests. All three golden materials and their existing acceptance variants run at 1K, with reviewed per-channel pixel tolerances, tiling, non-degeneracy and parameter causality; native baselines remain protected. Returned pixels survive later renders and runtime destruction. Invalid input, unsupported environments, concurrent calls, device loss, cleanup and stale results have tested outcomes.

Acceptance records identify both repositories, archive/lock/fixture identities, exact browser/OS/toolchain and observable adapter/backend, required limits/features/flags, comparison criteria, commands and gate results. The browser environment and numerical tolerances are fixed during implementation before acceptance; skipped GPU execution is not a pass. Existing native required checks remain intact, and browser build/regression must be reproducible in the documented CI environment. M5 cannot close on planning documents or native GPU evidence alone.

### Out of scope

- another pixel executor, WebGL2 or CPU fallback;
- a separately authored TypeScript node registry/compiler;
- node-authoring UI, intermediate-node preview and 3D preview;
- SSR, Node.js GPU, all-bundler adapters and GPU texture interop;
- separate SDK/Player repositories beyond the engine and shared product repository;
- M6 resource/container formats, new nodes and package publication without a release decision.

After M5 acceptance, Studio MVP may be planned as a product milestone with standard `.mix` output consumed by Player and the native CLI. It does not rename or start engine M6.

---

## M6 — Resources and Portable Packaging

### Entry condition

Do not start M6 until a real consumer requires image resources or a portable single-file asset and plain `.mix` plus external files is demonstrably insufficient.

### Outcome

Mixture can reference, validate, and distribute bounded external image resources without weakening the core trust boundary.

### Possible scope

- image-input node;
- content-addressed resources and hashes;
- safe relative paths;
- resource budgets and decode limits;
- optional `.mixpack` container;
- incremental resource caching;
- legacy conversion tool isolated from `mixture-core`.

### Exit criteria

At least one real consumer uses the resource workflow, and all resource bytes, identities, limits, and failure modes are inspectable and testable.

### Out of scope

- proprietary project import;
- arbitrary executable shaders;
- embedding editor state into the runtime document.

---

## Later candidates

These are not scheduled milestones:

- node editor;
- subgraphs and reusable functions;
- presets;
- custom WGSL or plugins;
- specialized texture formats;
- pass fusion or generalized compiler optimization;
- CPU renderer;
- WebGL2 renderer;
- Unity, Unreal, Godot, or glTF export profiles;
- marketplace, accounts, collaboration, or cloud rendering;
- 3D baking;
- AI-assisted material generation.

Each candidate requires a new roadmap decision based on a real consumer or measured blocker.

## Stop rules

1. No thirteenth node before all M3 golden-material gates pass.
2. No WebAssembly work before the native CLI and `.mix` contract are stable enough for M4 consumer use.
3. No node editor before a web consumer successfully uses the thin M5 API.
4. No binary format before a real packaging or loading problem is measured.
5. No second pixel backend.
6. No implicit semantic fallback.
7. No generalized optimizer before pass count, memory, or runtime data proves it necessary.
8. No new crate without a real boundary.
9. No golden update without reviewable before/after evidence.
10. No feature may claim success solely because it produces non-empty pixels.
11. No engine-specific concern may move into `mixture-core` unless it is a universal material semantic.
12. No milestone may close while the primary documented command path or required CI is red.

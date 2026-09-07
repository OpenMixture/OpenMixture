# Roadmap

**Current milestone:** M0 — Clean Foundation.

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

## M5 — WebAssembly and Browser WebGPU

### Outcome

The same `.mix`, node contracts, compiler, RenderPlan semantics, and WGSL run in a browser through a thin WebAssembly binding.

### Scope

- add `mixture-wasm` as a thin binding crate;
- compile `mixture-core` and `mixture-wgpu` for the browser WebGPU target;
- generated TypeScript declarations;
- explicit browser GPU initialization and diagnostics;
- raw pixel or image-ready output transfer;
- minimal viewer/demo with no node editor;
- browser regression for all three golden materials;
- first real integration with `Procedural_Texture_Online` or another independent web consumer.

### Exit criteria

An external web consumer can:

```text
load .mix bytes
-> apply parameter overrides
-> request outputs
-> render through browser WebGPU
-> display or export textures
```

while Rust remains the owner of graph and node semantics.

Native and browser plans must be semantically equivalent. Pixel comparison may use documented tolerances rather than universal byte identity.

### Out of scope

- WebGL2-specific renderer;
- TypeScript node registry;
- node authoring UI;
- server rendering.

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

# Initial Pull Request Train

English | [简体中文](./INITIAL_PRS.zh-CN.md)

This file defines the first implementation train for the greenfield Mixture repository. Each pull request should be independently reviewable, green, and narrow enough for a coding agent to execute without inventing adjacent scope.

The sequence covers M0 through M3. M4 and later work should be scoped only after these pull requests produce real evidence.

## Working rules for the train

- Land pull requests in dependency order.
- Prefer stacked branches, but rebase each next branch after its dependency merges.
- One architectural seam per pull request.
- Every pull request must include tests and documentation for the behavior it introduces.
- Every description must contain an explicit **Out of scope** section.
- Do not combine dependency upgrades, formatting churn, or unrelated refactors with feature work.
- Use conventional commit-style titles shown below unless repository policy chooses another explicit standard.
- A pull request may split into smaller pieces when review reveals two independent ownership seams. Do not merge adjacent pull requests merely to reduce count.

## Dependency graph

```text
PR-001 Foundation
  ├── PR-002 Diagnostics contract
  └── PR-003 Explicit GpuContext
         └── PR-004 Built-in checker execution

PR-002 + PR-004
  └── PR-005 .mix v1 decoding and validation
         └── PR-006 RenderPlan and graph inspection
                └── PR-007 Graph MVP node execution
                       └── PR-008 Golden harness + ceramic
                              └── PR-009 Leather material pipeline
                                     └── PR-010 Wood material + 2K evidence
```

---

## PR-001 — `chore(repo): establish the greenfield Rust workspace`

**Repository status:** Implemented as the initial foundation. Local verification uses `cargo xtask check`; remote cross-platform CI evidence is still required before closing M0. See [development instructions](./docs/development.md).

### Goal

Create a clean, honest repository that can be cloned and verified before any runtime behavior exists.

### Suggested branch

```text
chore/foundation
```

### Required changes

- Add workspace members:
  - `crates/mixture-core`;
  - `crates/mixture-wgpu`;
  - `crates/mixture-cli`;
  - `xtask`.
- Add minimal crate roots that compile but do not claim unimplemented behavior.
- Add:
  - `README.md`;
  - `AGENTS.md`;
  - `ARCHITECTURE.md`;
  - `ROADMAP.md`;
  - `INITIAL_PRS.md`.
- Add `.cargo/config.toml` with the repository-local `cargo xtask` alias.
- Add `rust-toolchain.toml`, workspace lint policy, formatting configuration, and committed `Cargo.lock`.
- Add `LICENSE-MIT` and `LICENSE-APACHE`.
- Add directories and short readmes for:
  - `fixtures/nodes`;
  - `fixtures/materials`;
  - `examples`;
  - `docs/decisions`.
- Add initial ADRs:
  - Rust owns graph semantics;
  - `wgpu` is the only pixel backend;
  - `.mix` v1 is a single readable DAG;
  - no implicit semantic fallback.
- Implement `cargo xtask check` to run, at minimum:
  - format check;
  - Clippy with warnings denied;
  - workspace tests;
  - rustdoc build;
  - basic repository-document link checks.
- Add non-GPU CI on Linux, macOS, and Windows using `--locked`.

### Tests and verification

```bash
cargo xtask check
```

### Acceptance criteria

- A clean clone passes the command above.
- All documented paths exist.
- No `wgpu` adapter is requested.
- No placeholder command prints a false success.
- Crate dependency direction matches `ARCHITECTURE.md`.

### Out of scope

- GPU dependencies and adapter acquisition;
- `.mix` data structures;
- shaders;
- image encoding;
- releases or package publication.

---

## PR-002 — `feat(core): define stable diagnostics and safety limits`

**Repository status:** Implemented with public Rust APIs, JSON snapshots, deterministic reports, native source chains, and seven explicit upper bounds. See [the diagnostic contract](./docs/diagnostics.md). The M0 remote CI gate remains pending; no GPU or runtime CLI implementation is claimed.

### Goal

Establish the structured error and limit vocabulary before parsing or GPU code begins returning ad hoc strings.

### Suggested branch

```text
feat/core-diagnostics
```

### Required changes

- Add typed diagnostic model to `mixture-core`:
  - stable code;
  - stage;
  - severity;
  - message;
  - optional node/port/parameter IDs;
  - evidence map or typed evidence;
  - suggestion;
  - source chain for human reports.
- Define initial code families from `ARCHITECTURE.md`.
- Add `SafetyLimits` with conservative v1 defaults:
  - decoded bytes;
  - nodes;
  - edges;
  - exposed parameters;
  - output dimension;
  - requested output count;
  - estimated transient bytes.
- Add deterministic ordering rules for multiple diagnostics.
- Add JSON serialization used later by CLI and WebAssembly.
- Document exit-code mapping policy without implementing all CLI commands.

### Tests and verification

```bash
cargo test -p mixture-core diagnostics
cargo test -p mixture-core limits
cargo xtask check
```

### Acceptance criteria

- Diagnostic JSON has snapshot coverage.
- Evidence order is deterministic.
- Limits report configured and observed values.
- Public errors do not require `wgpu` or CLI types.

### Out of scope

- `.mix` decoding;
- graph validation;
- GPU errors beyond reserving stable code families;
- automatic remediation.

---

## PR-003 — `feat(gpu): add an explicit headless GpuContext and doctor command`

**Repository status:** Implemented: explicit context ownership, stable acquisition failures, human/JSON doctor, and opt-in GPU smoke. Local Apple M5/Metal evidence passed; the pinned SwiftShader CI job awaits its remote run. See [GPU context documentation](./docs/gpu-context.md). Compute/readback and `healthy` remain PR-004 scope.

### Goal

Acquire and report a `wgpu` adapter/device explicitly, without rendering or global state.

### Suggested branch

```text
feat/gpu-context-doctor
```

### Required changes

- Add `wgpu` dependency only to `mixture-wgpu`.
- Implement `GpuContextOptions` with explicit adapter/backend preference fields.
- Implement `GpuContext::request` owning instance, adapter, device, and queue.
- Capture:
  - adapter name;
  - device type;
  - backend;
  - supported limits;
  - selected features;
  - requested policy.
- Add typed GPU diagnostics using PR-002 codes.
- Add thin CLI `doctor` command with human and `--json` output.
- Define preliminary verdicts such as:
  - `unverified` when adapter/device acquisition succeeds but no compute/readback probe has run yet;
  - `unhealthy` when acquisition fails.
- Reserve `healthy` for PR-004, when the doctor can prove compute execution and readback.
- Add a dedicated GPU-smoke CI job with a pinned software adapter environment.
- Ensure ordinary core tests do not require a GPU.

### Tests and verification

```bash
cargo test -p mixture-wgpu context
cargo test -p mixture-cli doctor
cargo run -p mixture-cli -- doctor --json
cargo xtask gpu-smoke
```

### Acceptance criteria

- `doctor --json` reports requested and actual adapter evidence.
- Missing-adapter behavior is structured and actionable.
- No global device or cache exists.
- No render pass or window surface is created.

### Out of scope

- shaders;
- readback;
- `.mix`;
- fallback to CPU or another runtime.

---

## PR-004 — `feat(gpu): render and read back a built-in checker`

**Repository status:** Implemented: one rgba16float checker compute pass, aligned readback, RGBA8 output, CLI PNG encoding, verified doctor with explicit skip, reviewed SwiftShader golden, and stage/cleanup regressions. Local Metal and pinned SwiftShader Vulkan smoke pass; remote CI remains pending. See [the checker contract and evidence](./docs/builtin-checker.md).

### Goal

Prove the complete headless compute path before introducing graph complexity.

### Suggested branch

```text
feat/builtin-checker
```

### Required changes

- Add one WGSL checker compute shader.
- Add offscreen `rgba16float` texture creation.
- Add minimal uniform/parameter upload.
- Add compute pipeline creation and command submission.
- Add texture-to-buffer readback with row-alignment handling.
- Add conversion to an image-ready RGBA buffer.
- Add PNG encoding at the CLI boundary.
- Add command:

```bash
mixture render-builtin checker --size 64 --out checker.png
```

- Add execution report containing adapter, size, pass count, timings where available, and readback bytes.
- Add pinned software-adapter golden for the checker.
- Extend `doctor` with the checker compute/readback probe so a successful full probe reports `healthy`.
- Keep an explicit skip mode that reports `unverified`, never `healthy`.
- Add explicit cleanup/drop tests where practical.

### Tests and verification

```bash
cargo test -p mixture-wgpu checker
cargo test -p mixture-wgpu readback
cargo run -p mixture-cli -- render-builtin checker --size 64 --out ./tmp/checker.png
cargo xtask gpu-smoke
```

### Acceptance criteria

- The checker uses actual `wgpu` compute execution.
- Readback dimensions and bytes are validated.
- Pinned software-adapter output is deterministic.
- GPU errors retain their exact stage.
- The implementation creates no graph model and no generalized renderer framework beyond what the checker requires.

### Out of scope

- node registry;
- `.mix` parser;
- resource pooling;
- browser target.

---

## PR-005 — `feat(format): implement strict .mix v1 decoding and graph validation`

**Repository status:** Implemented: bounded strict decoding, six versioned contracts, deterministic graph/parameter/public-binding validation, read-only validated documents, CLI `validate`, and focused fixtures. Local format/core/workspace checks pass; remote CI remains pending. See [the source schema and verification](./docs/file-format.md).

### Goal

Create the versioned source document and reject invalid inputs before any GPU work.

### Suggested branch

```text
feat/mix-v1-validation
```

### Required changes

- Add serde data structures for:
  - document version;
  - nodes;
  - node versions;
  - parameters;
  - ports and edges;
  - exposed parameters.
- Implement strict bounded decoding.
- Add initial node contract types without pixel execution.
- Register contracts for the M2 nodes:
  - `constant-scalar`;
  - `constant-color`;
  - `checker`;
  - `levels`;
  - `blend`;
  - `material-output`.
- Implement validation for:
  - unique IDs;
  - known node type/version;
  - parameter types and ranges;
  - known and compatible ports;
  - duplicate/single-input edges;
  - cycle detection;
  - exactly one material output;
  - required connected `baseColor`;
  - documented defaults for optional unconnected material channels;
  - exposed parameter targets;
  - safety limits.
- Add CLI `validate <file.mix> [--json]` as a thin adapter.
- Add valid and invalid fixtures.
- Add focused file-format documentation.

### Tests and verification

```bash
cargo xtask test-format
cargo xtask test-core
cargo run -p mixture-cli -- validate examples/checker.mix --json
```

### Acceptance criteria

- Malformed and over-budget inputs cannot reach `mixture-wgpu`.
- Diagnostics are stable and identify node/port/parameter where applicable.
- Cycle and type-mismatch tests are explicit.
- Validation does not modify or repair input semantics.
- Core tests run without GPU access.

### Out of scope

- RenderPlan;
- graph execution;
- presets, layout, resources, or subgraphs;
- legacy import.

---

## PR-006 — `feat(core): compile validated graphs into deterministic RenderPlan values`

**Repository status:** Implemented: immutable override/default normalization, backward slicing, lexical topological ordering, typed plans/resources/output mappings, checked cumulative/peak estimates, SHA-256, CLI `inspect --plan`, and plan/hash snapshots. Local `test-plan`, `test-core`, and workspace checks pass. Graph pixel execution remains PR-007; remote CI remains pending. See [the plan contract and verification](./docs/render-plan.md).

### Goal

Define the smallest backend-neutral plan required by the one `wgpu` executor.

### Suggested branch

```text
feat/render-plan
```

### Required changes

- Add compile request containing:
  - output size;
  - requested channels;
  - exposed parameter overrides;
  - safety limits.
- Add immutable normalized document view after validated overrides.
- Implement backward output dependency slicing.
- Implement stable topological ordering.
- Add typed:
  - `RenderPlan`;
  - `ComputePass`;
  - `KernelId`/`KernelInvocation`;
  - logical resource IDs and texture descriptions;
  - output mappings;
  - cumulative/peak estimates.
- Add stable plan hash.
- Add CLI `inspect <file.mix> --plan --json`.
- Add snapshot tests for plans and hashes.
- Prove that unrequested branches do not appear in the plan.

### Tests and verification

```bash
cargo xtask test-plan
cargo run -p mixture-cli -- inspect examples/checker.mix --plan --json
cargo xtask check
```

### Acceptance criteria

- The same semantic input produces the same plan and hash.
- Hashes exclude paths, timestamps, and adapter details.
- The plan is typed and simple; it is not a generic shader AST.
- `mixture-wgpu` still does not parse `.mix`.

### Out of scope

- full graph execution;
- optimizer passes;
- resource pooling;
- image resources.

---

## PR-007 — `feat(nodes): execute the six-node graph MVP through wgpu`

**Repository status:** Implemented: exhaustive kernel/WGSL mapping, shared fixed/graph executor, renderer-owned four-pipeline cache, graph CLI render, encoded requested-channel PNGs and provenance, six focused fixture families, and three readable examples. Shader/node/plan/workspace checks and full Metal/SwiftShader smoke pass locally; original pixel goldens are unchanged. Remote CI remains pending. See [graph rendering and evidence](./docs/graph-rendering.md).

### Goal

Complete M2 by mapping every initial `KernelInvocation` to one WGSL implementation and rendering real `.mix` graphs.

### Suggested branch

```text
feat/graph-mvp-execution
```

### Required changes

- Implement exhaustive `KernelId` to WGSL mapping.
- Add WGSL and parameter encoding for:
  - `constant-scalar`;
  - `constant-color`;
  - `checker`;
  - `levels`;
  - `blend`;
  - `material-output` handling or output binding as appropriate.
- Generalize the PR-004 executor only as required to execute `RenderPlan`.
- Add simple renderer-owned pipeline cache.
- Add graph-aware CLI `render` command.
- Support requested-channel readback and deterministic file naming.
- Add focused node fixtures for defaults, boundaries, and invalid values.
- Add shader validation command.
- Add three small readable examples.

### Tests and verification

```bash
cargo xtask shader-check
cargo xtask test-node checker
cargo xtask test-node levels
cargo xtask test-node blend
cargo run -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo xtask gpu-smoke
```

### Acceptance criteria

- `.mix -> RenderPlan -> wgpu -> PNG` works end to end.
- Every M2 node has one pixel implementation and focused fixtures.
- Requested-output slicing changes executed pass count as expected.
- Actual adapter and plan hash appear in the report.
- No CPU or TypeScript pixel implementation is introduced.

### Out of scope

- realistic material quality claims;
- golden update tooling;
- noise, warp, normals;
- 2K optimization.

---

## PR-008 — `test(materials): add protected golden tooling and glazed ceramic acceptance`

**Repository status:** Protected `golden check` / separate guarded `golden update`, `test-material`, the acceptance schema, and 1K glazed ceramic default/fine-tiles/matte cases are implemented without new nodes. Machine evidence and contact sheets are retained. Following user feedback on the flat checker display, controlled PBR evidence now shows actual exported channels on a sphere and planar sample. The user explicitly accepted the appearance comparison; PR-008 local implementation and acceptance are complete. Remote CI is explicitly deferred by the user and no milestone is closed. See [material goldens](./docs/material-goldens.md).

### Goal

Make visual correctness reviewable before more sophisticated nodes are added.

### Suggested branch

```text
test/golden-ceramic
```

### Required changes

- Implement:
  - `cargo xtask golden check`;
  - `cargo xtask golden update <id> --accept`.
- Enforce update safeguards:
  - CI refusal;
  - explicit acceptance flag;
  - no Git staging/commit;
  - metrics report;
  - before/after/difference contact sheet.
- Define material fixture layout and `acceptance.json` schema.
- Add first golden material using only M2 nodes:
  - glazed ceramic/checker;
  - baseColor, normal-neutral or simple normal, roughness, and height where supported;
  - at least two exposed parameter variants.
- Add tiling seam, non-finite, range, and non-degeneracy checks.
- Add human review record.
- Add one real-hardware tolerant comparison record.

### Tests and verification

```bash
cargo xtask golden check
cargo xtask test-material glazed-ceramic
cargo xtask check
```

### Acceptance criteria

- A baseline cannot be replaced by an ordinary test command.
- Material reports explain exactly what changed.
- The first material has both machine and human acceptance evidence.
- No new node is introduced in this pull request.

### Out of scope

- noise and height-derived normals;
- resource lifetime optimization;
- web viewer.

---

## PR-009 — `feat(materials): add the noise-to-normal pipeline and leather golden`

**Repository status:** Implemented locally: three contracts/kernels, required u32 seed, focused fixtures, five-pass 1K leather with three exposed parameters and detail min/default/max plus coarse-grain cases. Pinned SwiftShader and Metal node/material/smoke evidence, protected first baselines, spatial/normal causality and controlled PBR views are recorded. Human acceptance of leather is recorded; remote CI is deferred and M3 remains open. See [leather evidence](./fixtures/materials/leather/README.md).

### Goal

Prove micro-surface material behavior without expanding beyond the minimum required nodes.

### Suggested branch

```text
feat/leather-material
```

### Required changes

- Add node contracts and WGSL for:
  - `fractal-noise`;
  - `gradient-map`;
  - `height-to-normal`.
- Require explicit deterministic seed for `fractal-noise`.
- Document coordinate, tiling, octave, and normal conventions.
- Add focused node fixtures including boundary and seed cases.
- Add leather-like golden material with:
  - baseColor;
  - height;
  - normal;
  - roughness;
  - at least three meaningful exposed parameters;
  - minimum/default/maximum variants for one representative parameter.
- Add parameter-causality metrics and visual review.
- Verify 1K rendering through the actual adapter path.

### Tests and verification

```bash
cargo xtask test-node fractal-noise
cargo xtask test-node gradient-map
cargo xtask test-node height-to-normal
cargo xtask test-material leather
cargo xtask golden check
cargo xtask gpu-smoke
```

### Acceptance criteria

- Seed stability is proven on the pinned adapter.
- `height-to-normal` has a documented tangent-space convention.
- Parameter variants cause expected channel changes.
- Leather acceptance does not rely only on histogram or non-empty output checks.
- Built-in node count remains within the M3 budget.

### Out of scope

- transform/warp;
- clearcoat, sheen, AO, curvature, or scatter;
- 2K optimization unless a failure blocks this material.

---

## PR-010 — `feat(materials): add directional warp, wood golden, and 2K memory evidence`

**Repository status:** implemented locally in `e9dd03b`; wood is user-accepted, eleven 1K cases pass on Metal and pinned SwiftShader, and the 2K peak fits 512 MiB without lifetime optimization. See [M3 review](./docs/m3-review.md). Remote CI remains deferred.

### Goal

Complete M3 with a directional material and measured high-resolution resource behavior.

### Suggested branch

```text
feat/wood-and-2k-evidence
```

### Required changes

- Add node contracts and WGSL for:
  - `transform-2d`;
  - `warp`.
- Document wrap/tiling behavior and coordinate conventions.
- Add focused transform and warp fixtures, including border/seam cases.
- Add anisotropic wood-like golden material with:
  - directional structure;
  - baseColor, height, normal, and roughness;
  - at least three exposed parameters;
  - seam and directionality measurements;
  - human visual review.
- Add 1024 acceptance for all three golden materials.
- Add a 2048 trace for the most demanding material, reporting:
  - pass count;
  - cumulative allocation estimate;
  - bounded peak estimate;
  - actual allocated/reused bytes where measurable;
  - render and readback time;
  - adapter evidence.
- Measure the naive lifetime model first.
- Implement last-consumer release and compatible texture reuse only if the measured plan exceeds the documented budget or fails.
- Add regression coverage for any lifetime change.
- Enforce the node-count stop rule in repository checks.

### Tests and verification

```bash
cargo xtask test-node transform-2d
cargo xtask test-node warp
cargo xtask test-material wood
cargo xtask golden check
cargo xtask gpu-smoke
cargo xtask check
```

### Acceptance criteria

- All M3 golden-material gates pass.
- The wood material is seamless under its declared tiling contract.
- The 2K result is either within budget or accompanied by the smallest evidence-driven lifetime fix that brings it within budget.
- No generalized optimizer, SSA, pass fusion, or texture-format matrix is introduced.
- Built-in node count is twelve or fewer.

### Out of scope

- M4 API stabilization;
- N-API or daemon work;
- WebAssembly;
- node editor;
- embedded resources.

---

## After PR-010

Do not immediately begin browser or editor work.

First perform an M3 review that answers:

1. Are all three golden materials actually acceptable to a human reviewer?
2. Is the public `.mix` vocabulary understandable without repository internals?
3. Are diagnostics sufficient for another developer to fix invalid graphs and GPU setup?
4. Does 1K performance support interactive consumer workflows?
5. Is 2K memory bounded by a simple lifetime model?
6. Which API does an independent native consumer actually need?

Use those answers to write the M4 implementation train. Do not copy speculative M4 tasks from the old Mixture repository.

**Review status:** completed in [M3 review](./docs/m3-review.md). The evidence-driven [M4 train](./M4_PRS.md) defines PR-011–015; implementation has not started.

## Pull request description template

```markdown
## Goal

What single outcome does this pull request establish?

## Why now

Which current milestone exit criterion or blocking defect requires it?

## Design

Describe ownership, data flow, and important trade-offs.

## Evidence

- targeted tests:
- repository checks:
- GPU adapter/backend:
- visual artifacts, if applicable:
- before/after metrics, if applicable:

## Risks

What could regress, and how is it covered?

## Out of scope

List adjacent work that this pull request deliberately does not perform.
```

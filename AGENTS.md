# Agent Guide

This file is the operational contract for coding agents and contributors working on Mixture.

Read [ARCHITECTURE.md](./ARCHITECTURE.md) before changing boundaries, [ROADMAP.md](./ROADMAP.md) before adding scope, and [INITIAL_PRS.md](./INITIAL_PRS.md) when implementing the initial repository.

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
12. Do not add the thirteenth built-in node before all three golden materials pass the M3 acceptance gates.

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

The commands currently implemented in M0 are listed in [docs/development.md](./docs/development.md). `cargo xtask check`, `fmt`, `clippy`, `test`, `test-core`, `doc`, `deps`, and `links` work today. Other commands below are the target interface and must not be presented as already implemented.

The intended repository commands are:

```bash
# Fast repository checks used during normal development
cargo xtask check

# Focused checks
cargo xtask test-core
cargo xtask test-node <node-id>
cargo xtask test-material <material-id>
cargo xtask test-format
cargo xtask test-plan
cargo xtask shader-check
cargo xtask golden check
cargo xtask gpu-smoke

# CLI workflows
cargo run -p mixture-cli -- doctor --json
cargo run -p mixture-cli -- validate <file.mix> --json
cargo run -p mixture-cli -- inspect <file.mix> --plan --json
cargo run -p mixture-cli -- render <file.mix> --size 512 --out ./out
```

During the first implementation train, a command may not exist until the pull request that introduces it. Once introduced, keep its meaning stable. Do not silently replace a documented command with an unrelated one.

## Repository map

The following is the planned runtime-module ownership map. M0 contains only crate roots and repository tooling; modules are introduced by their owning implementation PR, without empty runtime stubs. Existing roots and commands are linked from [the development guide](./docs/development.md).

```text
crates/mixture-core/
  src/document.rs       .mix data model and decoding
  src/validation.rs     format, graph, parameter, and budget validation
  src/registry.rs       built-in node contracts
  src/compiler.rs       document -> RenderPlan
  src/plan.rs           backend-neutral execution plan
  src/error.rs          stable diagnostic codes and structured errors
  src/nodes/            one small module per built-in node contract

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

### `mixture-cli`

Owns only:

- file and directory I/O;
- command-line parsing;
- calling public APIs from `mixture-core` and `mixture-wgpu`;
- PNG encoding and JSON/human-readable reports;
- process exit codes.

Keep command modules thin. If logic is useful outside the CLI or needs direct unit tests, move it into the appropriate library crate.

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

Do not begin by running the full GPU matrix for a local one-line parser change. Do not finish a GPU or format change after running only a narrow unit test.

## Task-to-test map

| Changed area | Minimum targeted verification |
|---|---|
| `.mix` decoding or versioning | `cargo xtask test-format` |
| graph validation | `cargo xtask test-core` plus focused validation test |
| compiler or plan hashing | `cargo xtask test-plan` |
| node contract only | `cargo xtask test-node <id>` |
| WGSL kernel | `cargo xtask shader-check` and `cargo xtask test-node <id>` |
| resource lifetime or readback | focused `mixture-wgpu` tests plus `cargo xtask gpu-smoke` |
| CLI command | command snapshot/integration test |
| golden material | `cargo xtask test-material <id>` and visual evidence review |
| repository automation | the exact affected `xtask` test plus `cargo xtask check` |
| public API | downstream example or external-consumer fixture |

CI owns the full platform and GPU matrix. Local development should prefer the smallest decisive command.

## Adding a built-in node

A node addition is a vertical slice, not only a registry entry.

Required steps:

1. Confirm the node is permitted by the current roadmap and node-count stop rule.
2. Add a small node contract module under `mixture-core/src/nodes/`.
3. Define stable type ID, node version, ports, parameter types, defaults, ranges, and validation.
4. Add or extend the exhaustive `KernelId` mapping.
5. Add exactly one WGSL implementation under `mixture-wgpu/shaders/nodes/`.
6. Add focused fixtures for defaults, boundaries, invalid parameters, and at least one non-trivial case.
7. Add an agent-readable node document describing inputs, outputs, parameters, tiling behavior, and known precision limits.
8. Run shader validation and node tests on the pinned software adapter.
9. Verify that the node changes a real material or solves a documented consumer failure.

A node is not complete when it compiles. It is complete when the contract, shader, fixtures, diagnostics, and visual evidence agree.

Do not introduce a procedure macro for node declarations during the initial twelve-node vocabulary. Prefer obvious static Rust data and explicit matches until repetition is proven.

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

The intended commands are:

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

- One architectural seam per pull request.
- Keep adapters and generated bindings thin.
- Avoid unrelated renames and formatting churn.
- Add tests before or with behavior changes.
- Every pull request must state what is intentionally out of scope.
- Large changes should be stacked in dependency order and remain independently reviewable.
- Do not bypass required checks.
- Do not merge a pull request that leaves the documented primary command path broken.

## Definition of done

A task is complete only when:

- ownership boundaries remain intact;
- targeted tests pass;
- decisive repository checks pass;
- structured diagnostics remain meaningful;
- public behavior and docs agree;
- visual changes have reviewable evidence;
- no hidden fallback or global state was introduced;
- out-of-scope work is not smuggled into the change;
- the next agent can discover how to reproduce and verify the result from repository files alone.

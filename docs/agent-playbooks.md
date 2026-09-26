# Agent playbooks

English | [简体中文](./agent-playbooks.zh-CN.md)

Task-specific procedures for coding agents and contributors. The [Agent Guide](../AGENTS.md) owns the rules that apply to every task, including invariants, the authority order, ownership and the task-to-test map. Read the section below that matches your change before editing. Terms are defined in the [glossary](./glossary.md).

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

- Integrate new changes through actual GitHub pull requests following [repository governance](./governance.md). Keep milestone work-item IDs separate from GitHub PR numbers; historical `PR-001` through `PR-015` identify implementation batches.
- One architectural seam per pull request.
- Keep adapters and generated bindings thin.
- Avoid unrelated renames and formatting churn.
- Add tests before or with behavior changes.
- Every pull request must state what is intentionally out of scope.
- Large changes should be stacked in dependency order and remain independently reviewable.
- Do not bypass required checks.
- Do not merge a pull request that leaves the documented primary command path broken.
- Retain accepted results and their required review content under the [evidence policy](./evidence-policy.md); preserve historical records and keep ordinary repeated output in CI artifacts or ignored local directories.

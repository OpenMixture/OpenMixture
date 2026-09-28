# Agent Guide

English | [简体中文](./AGENTS.zh-CN.md)

The operational contract for coding agents and contributors. It holds only rules that apply to every task and links to the documents that own details:

| Need | Owner |
|---|---|
| Boundaries, invariants, data flow | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| Priorities, active increment, stop rules | [ROADMAP.md](./ROADMAP.md) |
| Versions, qualified scope, publication | [release status](./docs/release.md) and source manifests |
| Node, `.mix`, shader, golden, GPU, error, performance and PR procedures | [agent playbooks](./docs/agent-playbooks.md) |
| Milestone and work-item IDs (MAT-02b, NUM-01, M6-A…) and evidence terms | [glossary](./docs/glossary.md) |
| Branches, required checks, CI scope | [governance](./docs/governance.md) and [evidence retention](./docs/evidence-policy.md) |

Do not copy counts, versions or dated status into this guide; link the owner. [INITIAL_PRS](./INITIAL_PRS.md), [M4_PRS](./M4_PRS.md) and [M5_PRS](./M5_PRS.md) are completed history, not backlog. Integration, qualification and publication are distinct states.

## Mission

Mixture is a Rust material-graph compiler and headless texture renderer with exactly one pixel execution path: `wgpu` compute shaders.

```text
.mix -> parse -> validate -> compile -> RenderPlan -> wgpu -> texture outputs
```

Prefer the smallest change that makes this path more correct, observable, and useful to a real consumer.

## Non-negotiable invariants

Do not violate these without an accepted architecture decision record and an explicit roadmap change.

1. `mixture-core` must not depend on `wgpu`, browser APIs, CLI libraries, or UI frameworks.
2. `mixture-wgpu` is the only pixel executor. No CPU renderer or second TypeScript/WebGL renderer.
3. No silent fallback to another semantic executor. Return a structured error and report the selected adapter.
4. GPU state is explicit. No process-global adapter, device, queue, renderer, or cache.
5. `.mix` is the source of truth. No editor layout, thumbnails, transient UI state, or target-engine configuration in v1 documents.
6. Traversal and serialization are deterministic. Never rely on hash-map iteration order for externally visible behavior.
7. Every randomized node requires an explicit seed.
8. Every built-in node requires a contract, one WGSL implementation, focused fixtures, documentation, and targeted tests.
9. Golden outputs may not be overwritten merely to make a failing test pass.
10. Generated bindings and adapters stay thin. Runtime behavior belongs in typed Rust modules.
11. No new crate without an actual compilation, publication, runtime, or dependency boundary.
12. Every new built-in node requires an approved, bounded engine use case and explicit catalog/version review. Node count is neither a goal nor a ceiling; existing material gates remain required.

## Authority order

When sources disagree: (1) executable tests and stable public behavior; (2) `ARCHITECTURE.md` invariants; (3) accepted ADRs; (4) `ROADMAP.md` scope and stop rules; (5) issue or PR descriptions; (6) comments and historical notes. Legacy Mixture code is research evidence, not a compatibility contract.

## Boundary essentials

- **Versions:** `.mix` and `.mixpack` are v1; `RenderPlan` and the browser API are v3. Resolve explicit node type/version pairs (`fractal-noise@1` and `@2` coexist); never auto-migrate documents. The catalog lives in `registry.rs` and [node contracts](./docs/node-contracts.md). See [compatibility](./docs/compatibility.md).
- **External resources:** callers supply packed `rgba8-linear` bytes; Core validates, captures synchronously and returns immutable `PreparedRender`; wgpu uploads. No implicit path/URL lookup, PNG decoding or resampling. See the [resource contract](./docs/m6a-resource-contract.md).
- **Portable assets:** `mixture-asset` is the only `.mixpack` codec (canonical USTAR, CPU-only). Validate hashes and the full resource closure before slicing; never extract paths; reject every `resourceRef` override. Ceilings may only be lowered. See [format](./docs/m6b-package-format.md) and [adapters](./docs/m6b-04-adapters.md).
- **Browser:** `mixture-wasm` and `packages/runtime` are thin. Import is inert; loading and GPU creation are explicit; snapshot inputs before yielding; one busy slot per instance. See the [browser SDK contract](./docs/browser-sdk.md).
- **Numerics:** do not reset goldens or generalize a measured repair beyond its recorded nodes and hardware. See [stable noise](./docs/stable-noise.md) and the bounded [gamma-one correction](./docs/levels-linear-correction.md); retain node contracts while binding changed pixels to implementation builds.
- **Delivery:** `main` requires six checks. Documentation-only PRs may skip the GPU/browser jobs; a skipped check is not qualification evidence. CPU or software passes never qualify hardware pixels.
- **Evidence:** keep ordinary run output in CI artifacts or ignored `tmp/`. `cargo xtask check` rejects raw logs, archives and per-run folders in evidence areas, and any file above 4 MiB, unless the directory is listed in `docs/evidence/retention-exceptions.txt` with a PR justification. See [evidence retention](./docs/evidence-policy.md).
- **Historical attachments:** repository tooling owns explicit [archive restore and verification](./docs/evidence/archives/README.md). Only the listed snapshots use the maintainer-selected evidence Release; retain original receipts and current goldens/human-review images, verify published bytes before removal, and never treat archive storage as product publication or new qualification. The CPU workflow runs offline restore tests; normal builds never fetch archives.

## Current work

Follow the [roadmap](./ROADMAP.md)'s active increment: MAT-02b under the [MAT-02 contract](./docs/mat-02-layered-weathering.md), with PERF-MAT per [ADR 0009](./docs/decisions/0009-transient-texture-reuse.md) (the [working implementation](./docs/perf-mat-texture-reuse.md) changes Core estimates and wgpu execution together; full material acceptance remains separate). Freeze each stage's contract and acceptance cases before implementation. Material qualification builds into `target/native-consumer` and must not weaken clean-source checks. Caller control mapping and the public Native/browser matrix are owned by the [painted-metal fixture guide](./fixtures/materials/painted-metal/README.md); preserve exact source/request/package identity, frozen pixel/time limits and separate structural/PBR/human gates. The fixture guide also owns constant-reference, resolution-quality and public control-isolation gates, plus test-only raw-half mask/composition observation through the same executor and production-normal replay/periodic boundary checks plus unwrapped sampling of the selected v2 noise inputs, plus producer-bound metallic PBR views and separate human decisions; cross-runtime parity alone does not prove those properties. Recipe revision 2 uses zero-relative height to improve half precision; preserve revision 1 inputs and rerun all material gates under the [revision contract](./docs/mat-02-relative-height.md).

## Quick reference

All implemented repository commands are in [development](./docs/development.md).

```bash
cargo xtask check                      # before declaring any change ready
cargo xtask test-core | test-format | test-plan | test-consumer | package-check
cargo xtask test-node <node-id>
cargo xtask test-material <material-id>
cargo xtask shader-check
cargo xtask golden check               # golden update <id> --accept is guarded
cargo xtask gpu-smoke
cargo test --locked -p mixture-asset
npm test --prefix packages/runtime
cargo run -p mixture-cli -- doctor --json
cargo run -p mixture-cli -- validate <file.mix> --json
cargo run -p mixture-cli -- inspect <file.mix> --plan --json
cargo run -p mixture-cli -- render <file.mix> --size 512 --out ./out
cargo run -p mixture-cli -- asset inspect <file.mixpack> --json
```

Keep command meanings stable; label proposals as unimplemented until they exist.

## Repository map and ownership

| Path | Owns | Must not own |
|---|---|---|
| `crates/mixture-core` | decoding, validation, node contracts (`src/nodes/`), compilation, plan hashing, resource identity, diagnostics | adapters, GPU resources, WGSL, file I/O, CLI formatting, bindings |
| `crates/mixture-wgpu` | context, one WGSL kernel per `KernelId` (`shaders/nodes/`), pipelines, textures, readback, GPU diagnostics | `.mix` parsing, defaults, override semantics, a second catalog |
| `crates/mixture-asset` | `.mixpack` bytes, deterministic writing, closure, package budgets | file/network I/O, GPU state, node semantics |
| `crates/mixture-cli` | argument parsing, file I/O, PNG encoding, reports, exit codes | reusable logic (move it into a library) |
| `crates/mixture-wasm`, `packages/runtime` | bindings, explicit loading, input capture, error transfer, lifecycle | graph semantics, package parsing, rendering |
| `xtask/`, `scripts/` | repository automation and qualification | runtime behavior |
| `fixtures/`, `examples/` | node/material/package acceptance corpora; small user examples | — |

Introduce modules for actual ownership, without empty stubs.

## Cross-project boundary

OpenMixture tasks own engine code, runtime contracts, packages, producer-side qualification and release docs. Do not edit Studio or manage its issues/PRs unless the user explicitly assigns it. Studio reports engine defects upstream (version, minimal `.mix`, environment, first structured error); CI's pinned Studio consumer is a test host, not ownership.

## Standard workflow

1. Read the relevant document, the matching [playbook](./docs/agent-playbooks.md) section and nearby code.
2. Identify the owning crate; avoid cross-boundary shortcuts.
3. Add the smallest focused test, then the smallest coherent change.
4. Run the targeted checks below, inspect rendered output for visual changes, then `cargo xtask check`.
5. Update public docs and their `*.zh-CN.md` pairs in the same PR.

Do not run the full GPU matrix for a one-line parser change, and do not finish a GPU or format change after only a narrow unit test.

## Task-to-test map

| Changed area | Minimum targeted verification |
|---|---|
| `.mix` decoding or versioning | `cargo xtask test-format` |
| graph validation | `cargo xtask test-core` plus focused validation test |
| compiler or plan hashing | `cargo xtask test-plan` |
| node contract | `cargo xtask test-node <id>` |
| WGSL kernel | `cargo xtask shader-check` and `cargo xtask test-node <id>` |
| resource lifetime or readback | focused `mixture-wgpu` tests plus `cargo xtask gpu-smoke` |
| CLI command | command integration test; `test-consumer` for public report/exit contracts |
| golden material | `cargo xtask test-material <id>` and visual evidence review |
| resource identity/preparation | `test-core`, `test-plan`, Native/browser resource consumption; cross-runtime comparisons for pixel changes |
| asset codec/package format | `cargo test --locked -p mixture-asset`, `test-consumer`, `package-check`; `gpu-smoke` for pixel paths |
| WASM/JS lifecycle or package adapter | `npm test --prefix packages/runtime`, clean WASM/npm build, candidate consumption, affected pixel comparisons |
| repository automation or CI | the affected `xtask` test or workflow lint, plus `cargo xtask check` |
| documentation | check sources, synchronize languages, `cargo xtask links`; serialize xtask on Windows |
| evidence records | `cargo xtask evidence` and `cargo xtask links`; archive changes also run `python scripts/evidence/test_restore.py` and a real restore/member verification per the [evidence policy](./docs/evidence-policy.md) |
| public API | downstream example or external-consumer fixture |

CI owns the full platform and GPU matrix. Cross-runtime pixel recipes: [browser resources](./docs/m6a-04-browser-resources.md), [asset qualification](./docs/m6b-05-qualification.md), [stable noise](./docs/stable-noise.md).

## Maintaining this guide

A major adjustment updates `AGENTS.md` and `AGENTS.zh-CN.md` in the same PR: ownership, public APIs/commands/formats/node semantics/versions, lifecycle rules, required checks or qualification scope, delivery policy. Edit the affected rule in place and link its owner; never add dated status banners here. Every PR description records its guide impact or states that no operational rule changed. Fix drift against code or accepted decisions in the same change.

## Definition of done

- Ownership boundaries remain intact; no hidden fallback or global state.
- Targeted tests and decisive repository checks pass; structured diagnostics stay meaningful.
- Public behavior, docs and both languages agree; visual changes have reviewable evidence.
- Major adjustments are reflected in both Agent Guides; out-of-scope work is not smuggled in.
- The next agent can reproduce and verify the result from repository files alone.

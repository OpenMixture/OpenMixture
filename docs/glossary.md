# Glossary

English | [简体中文](./glossary.zh-CN.md)

Short definitions of the identifiers and evidence terms used across OpenMixture documents. Each entry links the document that owns the details; this page never states current versions or qualification status. For those, see [release status](./release.md) and the [roadmap](../ROADMAP.md).

## Work identifiers

Work-item IDs are planning labels, not GitHub pull request numbers. Actual PRs are written as links or `#N`.

| ID | Meaning | Owner |
|---|---|---|
| `PR-001`–`PR-015` | Historical implementation batches for M0–M4, named before GitHub PRs were used | [INITIAL_PRS](../INITIAL_PRS.md), [M4_PRS](../M4_PRS.md) |
| M0–M3 | Foundation, headless wgpu slice, `.mix` v1 graph MVP, and the three-material quality gate | [roadmap](../ROADMAP.md#completed-milestones--bounded-historical-acceptance) |
| M4, M4.1 | Stable native SDK; repository governance and required checks | [release](./release.md), [governance](./governance.md) |
| M5, `M5-01`–`M5-05` | WebAssembly binding, npm runtime and browser qualification | [M5_PRS](../M5_PRS.md), [browser runtime](./browser-runtime.md) |
| `ALPHA-01`–`ALPHA-07` | First browser Alpha: candidate qualification, required checks and npm delivery | [browser Alpha](./browser-alpha.md) |
| ENG-01–04 | Post-Alpha engineering items; ENG-02 replaced the node-count gate with use-case admission, ENG-04 added `scalar-blend@1` | [roadmap](../ROADMAP.md#eng-0104--post-alpha-independent-sdk-and-scalar-composition) |
| M6-A, `M6A-01`–`05` | Caller-supplied external image resources and plan v2 | [resource contract](./m6a-resource-contract.md) |
| M6-B, `M6B-01`–`05` | Portable `.mixpack` assets and the `mixture-asset` codec | [portable assets](./m6b-portable-assets.md), [package format](./m6b-package-format.md) |
| NUM-01 | Versioned stable value noise (`fractal-noise@2`, Q0.24 arithmetic) | [stable noise](./stable-noise.md) |
| MAT-01…MAT-04 | Staged material-expression increments: structure, layered weathering, woven surfaces, reusable recipes. A suffix letter is a sub-step: `a` contract, `b`/`c` implementation, `d` qualification | [roadmap](../ROADMAP.md#material-capability-roadmap--staged-delivery) |
| PERF-MAT (`a`/`b`/`c`) | Measurement-triggered cost work: evidence and contract, implementation, qualification | [ADR 0009](./decisions/0009-transient-texture-reuse.md) |

## Engine terms

| Term | Meaning |
|---|---|
| `.mix` | Versioned JSON material graph; the source of truth. See [file format](./file-format.md). |
| `.mixpack` | Canonical uncompressed USTAR package holding exact `.mix` bytes and raw images. See [package format](./m6b-package-format.md). |
| `RenderPlan` | Backend-neutral compiled plan with a stable hash, produced only by Core. See [render plan](./render-plan.md). |
| `PreparedRender` | Immutable plan plus synchronously captured resource snapshots, ready for wgpu. |
| `KernelId` | The pixel operation a plan pass runs; each maps to exactly one WGSL file. |
| node identity `type@version` | Nodes resolve by explicit type and version; versions coexist and never change meaning. See [node contracts](./node-contracts.md). |
| golden | Accepted expected output from the pinned software adapter. See [material goldens](./material-goldens.md). |
| pinned software adapter | The source-verified SwiftShader Vulkan build used for reproducible GPU checks. |
| Studio | The separately owned product repository; a pinned copy is a CI test host only. |

## Delivery and evidence terms

| Term | Meaning |
|---|---|
| candidate | A source revision and its built archives before qualification or publication. |
| integration | Merged into `main` through a PR with required checks passing. |
| qualification | Recorded machine gates, plus human review where required, for an exact candidate on named adapters. It never generalizes beyond the recorded scope. |
| publication | A separate, explicitly authorized registry release of exact archive bytes. |
| six required checks | The ruleset's CPU (three OS), SwiftShader GPU, WASM/npm and Chromium material jobs. See [governance](./governance.md). |
| source-bound | Evidence tied to a full commit SHA, clean-tree state and relevant input hashes. See [evidence retention](./evidence-policy.md). |
| receipt | A small machine-readable record binding a result to its source, run and artifacts. |
| recorded scope | The exact hosts, adapters, cases and versions a result covers; nothing else is claimed. |

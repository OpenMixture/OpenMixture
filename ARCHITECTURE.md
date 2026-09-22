# Architecture

English | [简体中文](./ARCHITECTURE.zh-CN.md)

**Integration review (2026-09-22):** M6-B is now integrated into main. NUM-01 is being reconciled with that baseline under PR #40, retaining unpublished Rust 0.5.0 / browser 0.5.0-alpha.0. Its earlier 0.4 receipts remain historical. Loose resource/Scalar qualification explicitly selects noise v2; portable asset regression keeps a separate v1 fixture. No stored document or asset is automatically migrated. Fresh combined checks are required before merge.

**Status:** architecture contract for the greenfield implementation.

This document defines what Mixture owns, how data moves through the system, and which designs are intentionally excluded. It is normative for the initial roadmap.

## 1. Architectural goal

Mixture turns a small, versioned material graph into texture outputs through one execution path:

```text
untrusted .mix bytes
  -> parse
  -> validate
  -> normalize
  -> compile requested outputs
  -> deterministic RenderPlan
  -> wgpu compute execution
  -> texture readback
  -> caller-selected encoding
```

The architecture optimizes for:

- one semantic owner;
- one pixel implementation per node;
- native headless rendering first;
- future browser WebGPU without a second graph runtime;
- explicit state and observable failures;
- small, reviewable modules that coding agents can modify safely;
- material-quality evidence rather than capability-count growth.

It does not optimize for editor features, maximum node count, multiple rendering backends, or broad engine integrations.

## 2. System context

```text
                         +----------------------+
                         |  .mix author/tool    |
                         |  human or consumer   |
                         +----------+-----------+
                                    |
                                    | JSON bytes + overrides + requested outputs
                                    v
+-------------------+      +--------+---------+      +----------------------+
| mixture-cli       | ---> | mixture-core     | ---> | mixture-wgpu         |
| file I/O          |      | parse / validate |      | explicit GPU context |
| report formatting |      | compile / hash   |      | compute / readback   |
| PNG encoding      | <--- | RenderPlan       | <--- | metrics / outputs    |
+-------------------+      +------------------+      +----------------------+

Browser:
+-------------------+
| mixture-wasm      |  thin browser binding over the same core and wgpu crates
+-------------------+
```

Consumers may choose how to display, package, or export textures. Core Mixture does not own a 3D preview scene, target-engine naming conventions, ZIP packaging, or editor state.

## 3. Workspace boundaries

The initial repository has three product crates and one private tooling crate.

### 3.1 `mixture-core`

`mixture-core` is a pure Rust library with no GPU or platform dependency.

It owns:

- `.mix` versioned data structures;
- decoding and deterministic serialization helpers;
- graph, port, parameter, and budget validation;
- built-in node contracts and versions;
- exposed-parameter override validation;
- requested-output dependency slicing;
- stable topological ordering;
- compilation to `RenderPlan`;
- plan hashes and non-GPU diagnostics.

It does not own:

- `wgpu` types;
- adapter selection;
- shader compilation;
- texture allocation;
- image file encoding;
- CLI behavior;
- browser bindings.

### 3.2 `mixture-wgpu`

`mixture-wgpu` is the only pixel execution implementation.

It owns:

- explicit `GpuContext` acquisition and lifetime;
- adapter/device/queue diagnostics;
- the exhaustive mapping from `KernelId` to WGSL;
- compute pipeline construction and caching;
- bind groups and parameter upload;
- logical resource to physical texture mapping;
- last-consumer release and compatible texture reuse when M3 requires them;
- command submission;
- output readback;
- GPU execution metrics and typed GPU failures.

It does not own:

- `.mix` parsing;
- graph validation or repair;
- node defaults or public parameter ranges;
- a duplicate node catalog;
- engine-specific output profiles;
- a CPU rendering fallback.

### 3.3 `mixture-cli`

`mixture-cli` is a thin executable over public library APIs.

It owns:

- file and directory I/O;
- CLI argument parsing;
- JSON and human-readable report formatting;
- PNG encoding;
- process exit codes.

It must not reimplement format, graph, compiler, or rendering semantics.

### 3.4 `xtask`

`xtask` is private repository automation, not a product crate.

It may orchestrate:

- formatting, Clippy, tests, and docs checks;
- targeted node and material tests;
- shader validation;
- golden comparisons and guarded updates;
- GPU smoke tests;
- fixture generation that is deterministic and reviewable.

`xtask` must call public or test-support APIs. It must not become a hidden second implementation of document or rendering behavior.

### 3.5 `mixture-wasm`

M5 introduces `mixture-wasm` as the browser binding crate.

It must remain a thin binding that:

- accepts `.mix` bytes or strings and structured options;
- calls `mixture-core` for parsing and compilation;
- calls `mixture-wgpu` for browser WebGPU execution;
- returns raw pixels, image-ready buffers, metrics, and structured diagnostics;
- generates or ships TypeScript declarations from the Rust public API.

It must not maintain a TypeScript node registry, graph validator, compiler, or shader implementation.

The [M5 plan](./M5_PRS.md) selects one browser distribution, `@openmixture/runtime`, built here from the same JS facade, declarations and WASM build. A separate product repository consumes that package for Player first and Studio later. Product controls, request freshness, previews, file export and editor layout remain consumer-owned. No separate SDK repository or additional semantic executor is introduced.

The [browser SDK contract](./docs/browser-sdk.md) defines explicit WASM/GPU initialization, GPU-free validation/catalog access, asynchronous completion, owned RGBA8 output and disposal. Browser adapters must preserve native semantics while adapting platform waiting and error delivery; compiling a native path to WASM is not browser acceptance. The [initial runtime implementation](./docs/browser-runtime.md) provides this binding and packaged loading path. [Bounded M5 browser acceptance](./docs/evidence/m5-05/README.md) is complete; the native API contract is unchanged. Current work follows the [Post-Alpha roadmap](./ROADMAP.md).

## 4. Dependency direction

```text
mixture-core
    ^
    |
mixture-wgpu
    ^
    |
mixture-cli

xtask -> public/test-support APIs from all three crates
mixture-wasm -> mixture-core + mixture-wgpu
```

More precisely:

- `mixture-core` depends only on general-purpose Rust libraries.
- `mixture-wgpu` depends on `mixture-core`.
- `mixture-cli` depends on both libraries.
- No library depends on the CLI.
- No core module imports a platform adapter.
- No adapter owns substantial runtime behavior.

Cycles between product crates are forbidden.

## 5. `.mix` v1 document model

PR-005 implements the first strict source schema in [the format guide](./docs/file-format.md). Document fields are `version`, `nodes`, `edges`, and optional `exposedParameters`; nodes require `id`, `type`, and `version`, with optional `parameters`. Endpoints use `nodeId`/`portId`; exposed bindings use `id`/`nodeId`/`parameterId`. IDs follow the documented ASCII grammar. Duplicate keys and undeclared fields are rejected; no metadata fields are currently allowed. This supersedes prospective README examples, not a previously shipped format.

### 5.1 Source-of-truth rules

`.mix` v1 is UTF-8 JSON and contains only material semantics required to compile and render a graph.

It includes:

- top-level format version;
- nodes with stable IDs, type IDs, node versions, and parameters;
- directed edges between named ports;
- exposed parameter bindings;
- optional material metadata that does not affect execution, if explicitly allowed by the schema.

It does not include:

- node coordinates or viewport state;
- comments represented as UI widgets;
- thumbnails or rendered previews;
- presets;
- engine-specific export targets;
- embedded binary resources;
- subgraphs or function graphs;
- arbitrary user WGSL.

A future editor should store layout separately, for example in `material.mix.layout.json` or consumer-owned local state.

### 5.2 Stable identifiers

- Node IDs are document-local strings.
- Node type IDs are lowercase kebab-case, for example `fractal-noise`.
- Ports and parameter IDs are stable camelCase strings.
- PBR output IDs are stable camelCase strings such as `baseColor` and `ambientOcclusion`.
- Node versions are independent from the top-level format version.

Changing the meaning of an existing node or field requires a version change and migration policy.

### 5.3 Port kinds and material channels

The v1 graph keeps data kinds deliberately small:

- `Scalar` — one logical value per pixel, used for masks, height, roughness, metallic, AO, and opacity;
- `Color` — four-component color data;
- `Normal` — encoded tangent-space normal data.

Channel meaning is assigned at `material-output`; it is not represented by a large hierarchy of nearly identical gray/height/roughness port kinds. Connections require exact kind compatibility. Conversions are explicit nodes, such as `gradient-map` from `Scalar` to `Color` and `height-to-normal` from `Scalar` to `Normal`.

The v1 `material-output` contract is:

| Channel | Port kind | Connection policy | Default when unconnected |
|---|---|---|---|
| `baseColor` | `Color` | required | none |
| `normal` | `Normal` | optional | neutral tangent-space normal |
| `roughness` | `Scalar` | optional | `1.0` |
| `metallic` | `Scalar` | optional | `0.0` |
| `height` | `Scalar` | optional | `0.0` |
| `ambientOcclusion` | `Scalar` | optional | `1.0` |
| `opacity` | `Scalar` | optional | `1.0` |
| `emissive` | `Color` | optional | black |

Render and inspection reports must state whether each requested output was connected or supplied by a documented default.

### 5.4 Initial limits

The initial implementation should enforce conservative defaults:

| Limit | v1 default |
|---|---:|
| decoded JSON bytes | 2 MiB |
| nodes | 128 |
| edges | 512 |
| exposed parameters | 64 |
| requested output dimension | 2048 per axis |
| requested material outputs | 8 |
| estimated transient GPU bytes | 512 MiB |

PR-002 provides an explicit `SafetyLimits` object with these defaults and upper-bound checks. Limits must never be raised implicitly. Every rejected limit reports the configured limit and observed value. PR-004 checks checker request lower bounds; PR-005 enforces byte/collection limits and validates source graphs. PR-006 checks compile requests and estimated peak allocation; see [the safety-limit contract](./docs/diagnostics.md).

Embedded resources are unsupported in v1, so their budget is zero.

## 6. Validation pipeline

Validation occurs before any GPU work.

### 6.1 Decode validation

Checks:

- valid UTF-8 and JSON;
- bounded decoded size;
- supported top-level version;
- required fields and field types;
- rejection of unknown fields where forward-compatible policy does not explicitly allow them;
- finite numeric values.

### 6.2 Node validation

Checks:

- unique node IDs;
- known node type and supported node version;
- complete and type-correct parameters;
- parameter ranges and enum values;
- required explicit seeds for randomized nodes.

### 6.3 Edge validation

Checks:

- known source and destination nodes;
- known source and destination ports;
- output-to-input direction;
- compatible port kinds;
- at most one connection to a single-input port;
- no duplicate edge identity;
- no graph cycles.

### 6.4 Material validation

Checks:

- exactly one `material-output` node in v1;
- `baseColor` is connected;
- requested outputs are connected or have a documented material-output default;
- exposed parameters target real mutable parameters;
- overrides use declared public IDs and valid values;
- graph and estimated resource budgets are within limits.

Validation returns all safe, independently discoverable diagnostics in stable order where possible. It does not silently repair the document.

## 7. Node contract model

A built-in node contract defines:

- stable type ID and node version;
- human-readable label and concise description;
- typed input and output ports;
- parameter types, defaults, ranges, and enum values;
- whether an explicit seed is required;
- `KernelId` used by the compiled plan;
- tiling and coordinate expectations;
- precision and range notes.

Use explicit Rust modules and static data. Propose procedure macros only after demonstrating real, stable repetition; catalog size alone is insufficient.

PR-005 registers the six [M2 contracts](./docs/node-contracts.md) in small static modules, without pixel executors or speculative KernelId stubs. `ValidatedDocument` resolves versioned defaults and exposes connected/default input sources without changing the source. PR-006 implements overrides and typed kernel lowering. PR-007 implements the exhaustive WGSL mapping and all six node pixel fixtures through the shared [graph executor](./docs/graph-rendering.md). The fixed checker and graph checker use one shader and dispatch path.

### 7.1 One pixel implementation

There is no CPU mirror of node pixels.

For each pixel-producing `KernelId`, `mixture-wgpu` contains exactly one WGSL implementation. Rust defines the contract and typed invocation; WGSL defines the pixel computation. The mapping must be exhaustive so a new `KernelId` cannot compile without an executor mapping.

Tests, fixtures, and golden materials are the behavioral contract between Rust node definitions and WGSL kernels.

### 7.2 Reviewed node catalog and admission

ENG-02 graduates the pre-M3 twelve-node budget after the [M3 material review](./docs/m3-review.md) and [subsequent remote acceptance](./docs/evidence/remote-ci/README.md). This explicitly supersedes the review's historical decision to retain that gate; it does not change accepted pixels or runtime semantics.

The current reviewed catalog remains:

1. `constant-scalar`
2. `constant-color`
3. `checker`
4. `levels`
5. `blend`
6. `material-output`
7. `fractal-noise`
8. `gradient-map`
9. `transform-2d`
10. `warp`
11. `height-to-normal`

Node count is neither an expansion target nor a permanent ceiling. A new type requires an approved bounded engine use case, version/compatibility decision, contract, exactly one WGSL implementation where pixels are produced, focused fixtures, paired documentation and Native/browser evidence as applicable. Existing material regressions remain required. Update the explicit type/version expectations in the [registry tests](./crates/mixture-core/tests/registry.rs) in the same reviewed PR; do not bypass catalog review by deleting these assertions. The [roadmap](./ROADMAP.md) and [node workflow](./AGENTS.md#adding-a-built-in-node) govern admission. ENG-02 adds no node and does not authorize ENG-04 implementation.

PR-009 extends the catalog additively to nine node types with `fractal-noise`, `gradient-map` and `height-to-normal`. Source shape and document/node version 1 remain unchanged. The noise seed is required explicitly: `ParameterContract::default: Option<ParameterDefault>` uses `None` for required parameters. A missing seed fails source validation even on an unused branch. Typed plans preserve all u32 seed bits; the sole WGSL mapping owns each new pixel formula. [Node conventions](./docs/node-contracts.md) and the [leather fixture](./fixtures/materials/leather/README.md) define coordinates, precision and consumer evidence.

## 8. Compilation model

The compiler accepts:

- a validated `MaterialDocument`;
- output size;
- requested material channels;
- exposed parameter overrides;
- safety limits.

It produces a deterministic `RenderPlan`.

### 8.1 Compilation steps

1. Apply validated overrides to an immutable normalized document view.
2. Resolve requested output ports on the `material-output` node.
3. Walk dependencies backward and discard unrelated nodes.
4. Produce a stable topological order.
5. Lower each required node to one typed compute pass.
6. Assign logical texture resources and descriptions.
7. Record output-resource mappings.
8. Estimate cumulative and peak resource pressure.
9. Compute a stable plan hash.

The first compiler is intentionally simple. It does not perform general expression fusion, SSA construction, global precision inference, or shader-AST optimization.

### 8.2 RenderPlan shape

PR-006 implements immutable `RenderPlan` with shared-reference getters. Concrete types are defined in [plan.rs](./crates/mixture-core/src/plan.rs); a runnable consumer example is tested in [core rustdoc](./crates/mixture-core/src/lib.rs).

```rust
let request = CompileRequest::default();
let plan = mixture_core::compile(&validated_document, &request)?;
let passes: &[ComputePass] = plan.passes();
let outputs: &[PlanOutput] = plan.outputs();
let estimates: &PlanEstimates = plan.estimates();
let hash: &PlanHash = plan.hash();
```

`KernelInvocation` is a typed enum carrying its input resource bindings. Passes also retain source/default provenance; outputs map connected/default sources to real resources. Optional defaults lower to constant passes. Plan version 1 uses rgba16float textures, f32 arguments, 8×8 workgroups, and a conservative allocation model without pooling. See [the exact plan and hash contract](./docs/render-plan.md).

### 8.3 Plan hash

PR-006 hashes the domain-separated compact plan body with SHA-256. Only the requested dependency slice and its effective parameters enter the hash: unused branches/exposure metadata and policy ceilings do not. Source order, explicit defaults, no-op overrides, and request order normalize identically; retained node IDs are significant.

The plan hash includes semantic inputs such as:

- format and plan versions;
- node type and node versions;
- normalized parameters and overrides;
- requested outputs;
- output size;
- pass ordering;
- logical resource descriptions.

It excludes:

- absolute file paths;
- timestamps;
- adapter names;
- log formatting;
- wall-clock metrics.

## 9. `wgpu` execution model

### 9.1 Explicit context

GPU initialization is explicit:

```rust
let context = GpuContext::request(options).await?;
let mut renderer = Renderer::new(context);
let result = renderer.render(&plan).await?;
```

`GpuContext` owns the `wgpu::Instance`, selected adapter, device, and queue. `Renderer` owns execution caches and reusable resources associated with that context.

No public render API initializes a hidden global device.

PR-013 records the first native device-loss notification inside each context, separately from its acquisition snapshot. Observed loss prevents further GPU work and clears that renderer's pipelines; it never triggers recovery. Typed OOM/loss diagnostics retain the operation stage and first failure, and scoped readback cleanup preserves secondary unmap errors. Failure reports include tracked descriptor release evidence. See [GPU failures and lifetime](./docs/gpu-failures.md).

PR-003 implements context acquisition; its immutable snapshot remains `unverified`. PR-004 adds `GpuContext::render_checker` and a verified doctor probe: actual compute/readback and pixel checks yield `healthy`, explicit skip yields `unverified`, and failures yield `unhealthy` with their exact stage. PR-007 implements the shared `Renderer` for immutable core plans; the fixed checker and graph path use the same executor. See [GPU ownership](./docs/gpu-context.md) and [the checker contract](./docs/builtin-checker.md).

### 9.2 Headless compute only

Initial rendering uses compute pipelines and offscreen textures only.

The core renderer does not create:

- a window surface;
- swapchain configuration;
- frame loop;
- mesh, camera, light, or PBR scene;
- UI event handling.

Each pass reads sampled or storage inputs, uploads typed parameters, and writes one storage texture.

### 9.3 Baseline texture representation

The initial intermediate representation should prefer one portable format, expected to be `rgba16float`, for both scalar and color data.

Conventions:

- scalar/gray values use the red component and defined filler components;
- color uses RGBA;
- normals use a documented encoded tangent-space convention;
- output color-space metadata is separate from intermediate storage.

A more compact format may be introduced only after profiling proves that format specialization materially improves a golden material or consumer workload.

### 9.4 Resource lifetime

M1 and early M2 may allocate straightforwardly for clarity. Before M3 exits, the renderer must measure and, where needed, implement:

- last-consumer analysis from the plan;
- release of dead logical intermediates;
- reuse of compatible physical textures;
- explicit output pinning until readback completes;
- bounded pipeline and resource caches.

Do not implement a general allocator before a 2K golden-material trace establishes the requirement.

### 9.5 Pipeline cache

Pipeline caching is keyed only by semantic GPU inputs such as kernel, shader version, texture format, and relevant device capabilities.

Cache ownership belongs to `Renderer`, not global state. PR-010 retains at most nine pipelines keyed by `KernelId`, with one fixed device/shader ABI/format/workgroup policy per renderer. Explicit cache clear and renderer drop release them; request dimensions and parameters do not expand the cache.

### 9.6 Adapter policy and fallback

Callers may express an adapter/backend preference through `GpuContextOptions`. The selected adapter and native backend are always reported.

If acquisition or execution fails, Mixture returns a typed failure. It does not switch to a CPU renderer or another graph runtime.

`wgpu` may choose among available native API adapters according to the explicit policy, but this is adapter selection inside the one `wgpu` execution model, not semantic fallback.

## 10. Results and output ownership

A render result contains:

- raw texture outputs or readback pixel buffers;
- per-channel dimensions, encoding metadata, and `connected`/`default` source status;
- actual adapter/backend evidence;
- pass count and execution timings where available;
- allocation and peak-byte metrics;
- plan hash;
- warnings that did not invalidate the result.

PNG encoding initially belongs to the CLI or a small shared non-semantic helper. The core compiler does not know file names, ZIP layouts, Godot ORM packing, or browser download behavior.

PR-007 execution follows the PR-006 naive allocation model without a resource pool. [Graph output encoding](./docs/graph-rendering.md) converts linear color RGB to sRGB, keeps alpha linear, exports scalar red as replicated grayscale, and keeps encoded normals linear. The CLI writes per-channel PNGs and reports provenance, actual adapter, plan hash, passes, estimates, and cache/timing evidence.

## 11. Determinism and numerical parity

Mixture separates semantic determinism from byte-identical floating-point output.

### 11.1 Deterministic semantic inputs

The project guarantees stable:

- document decoding behavior for a version;
- validation order;
- dependency slicing;
- topological ordering;
- seed conventions;
- RenderPlan content and hash for the same semantic inputs.

### 11.2 Pixel evidence

A pinned software adapter is used for byte-level or very tight golden comparisons when practical.

Different hardware, drivers, native APIs, and browser WebGPU implementations may use tolerance-based comparison. Cross-adapter tests should measure:

- maximum and mean absolute error;
- mismatched-pixel ratio above a small threshold;
- non-finite pixels;
- output range;
- tiling seam error;
- material-specific structural measures when useful.

Do not claim universal byte-identical GPU output.

## 12. Error and diagnostics model

Externally visible failures use stable Mixture codes.

PR-002 implements `Diagnostic`, `DiagnosticReport`, and native source chains in `mixture-core`. Reports sort diagnostics deterministically and derive `ok` from severity. Source objects remain available through `std::error::Error::source` and are not serialized automatically. The exact JSON fields, ordering, reserved codes, and shared CLI exit-code policy are documented in [the diagnostic contract](./docs/diagnostics.md).

Example:

```json
{
  "ok": false,
  "diagnostics": [
    {
      "code": "MIX_PORT_TYPE_MISMATCH",
      "stage": "validation",
      "severity": "error",
      "message": "Cannot connect a color output to a scalar input used as height.",
      "nodeId": "normal",
      "portId": "height",
      "evidence": {
        "sourceKind": "color",
        "targetKind": "scalar"
      },
      "suggestion": "Connect a scalar output or add an explicit supported conversion node."
    }
  ]
}
```

Initial code families should include:

- `MIX_PARSE_*`
- `MIX_FORMAT_*`
- `MIX_LIMIT_*`
- `MIX_NODE_*`
- `MIX_PORT_*`
- `MIX_GRAPH_*`
- `MIX_PARAMETER_*`
- `MIX_COMPILE_*`
- `MIX_GPU_ADAPTER_*`
- `MIX_GPU_DEVICE_*`
- `MIX_GPU_OUT_OF_MEMORY`
- `MIX_GPU_SHADER_*`
- `MIX_GPU_EXECUTION_*`
- `MIX_READBACK_*`
- `MIX_ENCODING_*`

Driver messages are preserved as evidence but are not the only API contract.

## 13. Testing architecture

### 13.1 Core tests

Run without a GPU and cover:

- decode and rejection cases;
- format versions;
- graph cycles and port mismatches;
- parameter ranges and overrides;
- budgets;
- dependency slicing;
- stable plan ordering and hashing.

### 13.2 Shader checks

Every WGSL module is parsed and validated without requiring a live adapter where possible. Binding and layout expectations are tested against Rust-side parameter encoding.

### 13.3 Node fixtures

Each node has focused cases for:

- defaults;
- minimum and maximum values;
- invalid input;
- deterministic seed behavior when relevant;
- tiling behavior when claimed;
- one representative non-trivial output.

### 13.4 Golden materials

Each material fixture contains:

```text
fixtures/materials/<id>/
├── material.mix
├── README.md
├── acceptance.json
├── variants/
├── expected/
└── reports/
```

Acceptance includes fixed software-adapter outputs, tolerant hardware checks, tiling measures, non-degeneracy, parameter causality, and human review.

### 13.5 GPU smoke tests

A small dedicated CI job installs or uses a pinned software GPU stack and verifies:

- adapter acquisition;
- a built-in compute pass;
- texture readback;
- one graph render;
- structured diagnostics.

GPU absence in a non-GPU unit-test job must not be misreported as a graph failure.

### 13.6 External-consumer tests

Before M4 exits, an independent consumer must use only the public CLI or Rust API. Source-relative imports or workspace-private internals do not count.

## 14. Security and trust boundaries

`.mix` files are untrusted inputs.

The system must:

- enforce decode and graph budgets before allocation;
- reject non-finite and out-of-range parameters;
- avoid panics on malformed documents;
- prevent path traversal in CLI output names;
- bound readback and image-encoding allocations;
- preserve source errors without leaking secrets or unrelated environment data;
- avoid executing arbitrary user shader source in v1;
- keep dependency versions locked in CI.

## 15. Rejected initial designs

The following designs are explicitly rejected for the initial roadmap:

### Multiple pixel backends

Rejected because they recreate semantic drift and multiply node-maintenance cost.

### CPU renderer as an oracle

Rejected because it becomes a second pixel implementation. A pinned software GPU adapter exercises the real WGSL path instead.

### TypeScript-owned graph runtime

Rejected because Native and WebAssembly must share one compiler and one node contract.

### Generic compiler IR or shader AST

Rejected until real profiling shows that simple typed compute passes cannot meet the golden-material requirements.

### Binary `.mixar` in v1

Rejected because readable JSON is easier to inspect, version, migrate, test, and modify by humans and agents. Packaging is deferred until a real resource-distribution need exists.

### Editor-first development

Rejected because a UI can hide an unstable engine contract and consume most project capacity before rendering quality is proven.

## 16. Architecture change process

A change requires an architecture decision record when it introduces or changes:

- a product crate;
- a dependency direction;
- a public file-format field or version;
- a second runtime or pixel path;
- a global cache or background execution model;
- arbitrary shader execution;
- a major resource-lifetime strategy;
- a compatibility promise.

An ADR must include context, decision, alternatives, consequences, migration, and verification. It must also update this document when the accepted decision changes a normative rule.

## M6B-03 portable input boundary

[ADR 0008](./docs/decisions/0008-portable-assets.md) selects [canonical uncompressed USTAR assets](./docs/m6b-package-format.md) and authorizes an optional `mixture-asset` public CPU codec crate in M6B-03. It depends on Core; CLI and WASM depend on the codec, while Core/wgpu never do. The codec owns bounded archive bytes, manifest/integrity and package diagnostics, not graph/resource semantics, filesystem extraction or pixel execution. Core supplies reusable resource metadata/digest/reference helpers; wgpu remains unchanged. M6B-03 implements the optional crate and shared Core helpers. [M6B-04](./docs/m6b-04-adapters.md) connects CLI file I/O and WASM byte transfer to that codec; both prepare Core-owned snapshots and reuse wgpu. Adapter-retained bytes share the codec budget, including both JS and Rust package copies. This narrowly enables producer-owned input transport outside Core; existing exclusions of editor/export/ZIP responsibilities from Core remain.

## PR-010 measured resampling slice

The catalog now has eleven node types and nine kernels. Scalar `transform-2d` and `warp` use explicit wrapped bilinear texture loads before color/normal derivation; document and node versions remain 1. Their additive typed payloads are documented in [node contracts](./docs/node-contracts.md). `RenderReport.allocations` records successful resource descriptor bytes separately from core estimates. The naive retain-all-pass-textures schedule remains in place; `trace-2k` ranks all three materials and their cases by peak estimate, then measures the largest against the existing 512 MiB budget. Counters exclude driver overhead and CPU buffers; destruction is not a claim of immediate physical memory reclamation. See [development and trace semantics](./docs/development.md). No new crate, format, dependency, cache, optimizer or renderer is introduced.

PR-014 introduces only application-owned freshness state in the independent consumer: one active render, one replaceable pending input and one retained display, with at most two CPU outputs during replacement. It does not add a core scheduler, global worker, cancellation API or resource pool. See [consumer lifetime boundaries](./docs/stale-results.md).

PR-015 adds local archive verification in repository tooling only. The independent consumer is staged with normalized public package contents in a disposable external workspace; local version patches resolve those extracted crates, never producer-private modules. Package licenses/README/unit assets are self-contained, committed locks remain unchanged and publication remains disabled. The [compatibility record](./docs/compatibility.md) consolidates existing boundaries; [M4 release status](./docs/release.md) records completed local and remote acceptance, remaining distribution decisions, and untested hardware limits.

M5 browser start adds the actual `mixture-wasm` compilation boundary and keeps npm assets in this engine repository. `mixture-wgpu` now adapts browser callback completion and timing; native waiting remains separate. The [browser runtime guide](./docs/browser-runtime.md) identifies the initial implemented slice. The diagrams above include the implemented browser boundary; no second compiler or pixel executor is added.

Browser runtime qualification uses the bounded texture-agreement profile defined by [ADR 0006](./docs/decisions/0006-browser-quality-gates.md). Native pinned-software goldens remain exact regression evidence; cross-browser near-byte identity is not a support promise. Current comparator receipts separate semantic, structural, numerical and historical regression verdicts.

## ENG-04 catalog extension

ENG-04 adds scalar-blend v1: twelve node types map to ten WGSL kernels. Renderer-owned cache capacity is now ten kernel identities. Existing source semantics and serialized plans remain unchanged. See [the contract](./docs/eng-04-scalar-blend.md).

## M6A-01 selected resource design — not implemented

[ADR 0007](./docs/decisions/0007-external-image-resources.md) and the [minimal resource contract](./docs/m6a-resource-contract.md) select logical resource references inside existing node parameters, caller-owned linear RGBA8 input, Core-owned immutable capture/content identity and wgpu-owned upload/lifetime. Integration accepts the design; runtime implementation remains pending. This narrowly extends ADR 0003's resource-reference exclusion without adding embedded resources or changing the sole pixel executor. The implementation will introduce image-input v1, plan/hash v2 and API schema 2; existing architecture descriptions above remain the implemented baseline until their owning implementation changes land.

M6A-02 implements Core resource semantics and plan v2. [M6A-03](./docs/m6a-03-native-resources.md) adds public prepared-resource execution, per-render RGBA8 uploads and accounting through the sole wgpu executor. Existing twelve-node pixel semantics remain unchanged. [M6A-04](./docs/m6a-04-browser-resources.md) adds synchronous browser capture through Core adapter byte sources and the same prepared executor. Final image qualification remains M6A-05.

M6A-05 [qualification](./docs/evidence/m6a-05/README.md) closes the recorded Linux software matrix, with an explicit ADR 0007 scope addendum. The retained Windows hardware parity failure remains unresolved; no pixel semantics or numerical gates change.

## NUM-01 versioned value-noise arithmetic

[Stable value noise](./docs/stable-noise.md) adds fractal-noise v2 with Q0.24 value evaluation in the existing WGSL kernel. The catalog still has thirteen types/eleven kernels; v1 and cellular behavior remain available. Explicit node version and StableValue lowering distinguish hashes without changing .mix v1 or plan/API schema 2. The unpublished 0.4 candidate and migrated material sources require their own qualification.

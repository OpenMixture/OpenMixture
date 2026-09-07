# Architecture

English | [简体中文](./ARCHITECTURE.zh-CN.md)

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

Future:
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

### 3.5 Future `mixture-wasm`

`mixture-wasm` may be added only in M5.

It must remain a thin binding that:

- accepts `.mix` bytes or strings and structured options;
- calls `mixture-core` for parsing and compilation;
- calls `mixture-wgpu` for browser WebGPU execution;
- returns raw pixels, image-ready buffers, metrics, and structured diagnostics;
- generates or ships TypeScript declarations from the Rust public API.

It must not maintain a TypeScript node registry, graph validator, compiler, or shader implementation.

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
future mixture-wasm -> mixture-core + mixture-wgpu
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

PR-002 provides an explicit `SafetyLimits` object with these defaults and upper-bound checks. Limits must never be raised implicitly. Every rejected limit reports the configured limit and observed value. Request lower bounds and document validation remain for their owning PRs; see [the safety-limit contract](./docs/diagnostics.md).

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

The initial implementation should use explicit Rust modules and static data. Avoid procedure macros until the first twelve nodes expose real, stable repetition.

### 7.1 One pixel implementation

There is no CPU mirror of node pixels.

For each pixel-producing `KernelId`, `mixture-wgpu` contains exactly one WGSL implementation. Rust defines the contract and typed invocation; WGSL defines the pixel computation. The mapping must be exhaustive so a new `KernelId` cannot compile without an executor mapping.

Tests, fixtures, and golden materials are the behavioral contract between Rust node definitions and WGSL kernels.

### 7.2 Initial node budget

The graph MVP and material MVP may introduce at most twelve built-in node types before M3 acceptance:

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
12. one reserved slot, only if a golden material proves it necessary

The reserved slot is not permission to add a speculative node. It exists to avoid redesigning the stop rule for one demonstrated blocker.

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

The exact Rust types may evolve during M2, but the contract should resemble:

```rust
pub struct RenderPlan {
    pub version: u32,
    pub size: [u32; 2],
    pub passes: Vec<ComputePass>,
    pub outputs: Vec<PlanOutput>,
    pub estimates: PlanEstimates,
    pub hash: PlanHash,
}

pub struct ComputePass {
    pub id: PassId,
    pub kernel: KernelInvocation,
    pub inputs: Vec<ResourceId>,
    pub output: ResourceId,
    pub output_desc: TextureDesc,
    pub dispatch: [u32; 3],
}
```

`KernelInvocation` should be a typed enum or equivalent typed representation, not arbitrary JSON and not a generic shader language.

### 8.3 Plan hash

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

PR-003 implements context acquisition and human/JSON `doctor`. It reports requested policy, actual adapter capabilities, and enabled device features/limits. Success is `unverified`; acquisition failure is `unhealthy`. No compute or readback has run, and `Renderer` in the example remains planned for PR-004. See [the GPU context contract](./docs/gpu-context.md).

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

Cache ownership belongs to `Renderer`, not global state. Caches must have explicit cleanup and bounded growth.

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

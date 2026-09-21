# Deterministic RenderPlan compilation

English | [简体中文](./render-plan.zh-CN.md)

**M6A-02 update:** [M6A-02 Core implementation](./m6a-02-core-resources.md) now provides resource references, immutable prepared requests and content-bound plan v2. Rust source is 0.3.0; the unpublished browser candidate is 0.3.0-alpha.0/API schema 2, with schema 2 inspect/graph-render reports. The [M6A-03 Native path](./m6a-03-native-resources.md) now executes prepared images; [M6A-04 browser resources](./m6a-04-browser-resources.md) now add synchronous capture and public rendering. Final cross-platform qualification remains M6A-05. Read historical version descriptions below in that context.

PR-006 implements CPU-only compilation and `inspect --plan`. [The compiler](../crates/mixture-core/src/compiler.rs) owns override semantics, dependency slicing, ordering, typed lowering, allocation estimates, and hashing. [RenderPlan](../crates/mixture-core/src/plan.rs) owns the backend-neutral vocabulary. PR-007 [graph execution](./graph-rendering.md) implements the exhaustive WGSL mapping. The fixed checker pixels remain unchanged; its shared 48-byte uniform is documented there.

## Run and inspect

```bash
cargo xtask test-plan
cargo run --locked -p mixture-cli -- inspect examples/checker.mix --plan --json
cargo run --locked -p mixture-cli -- inspect examples/checker.mix --plan \
  --size 65x3 --output roughness,baseColor --set 'frequency=16'
cargo run --locked -p mixture-cli -- inspect fixtures/format/valid/all-m2.mix \
  --plan --json --output roughness
cargo xtask test-core
cargo xtask check
```

`--plan` is required. Defaults are 64×64, `baseColor` only, no overrides, and standard `SafetyLimits`. `--size N` is square; `--size WxH` is rectangular. `--output` accepts unique comma-separated channel names in any order: `baseColor`, `normal`, `roughness`, `metallic`, `height`, `ambientOcclusion`, `opacity`, `emissive`. Names are case-sensitive; an empty or duplicate channel is invalid. `--set` can repeat for distinct declared exposed IDs and takes exact JSON values. For an exposed enum, quote its JSON string, for example `--set 'mode="screen"'`. Public names must exist in that particular document. Node parameters are not directly addressable unless exposed.

`inspect` shares the validator's bounded source loader: at most 2 MiB plus one detection byte under the default policy. It performs full source validation before compilation, including unrequested branches. It never acquires an adapter or writes the input. Options can precede the path; use `--` before a path beginning with `-`.

JSON mode writes one report with `schemaVersion: 1`, `plan`, `ok`, and `diagnostics`. Success contains the plan and an empty diagnostic array. Failure contains `plan: null` and ordered shared diagnostics, including the supplied path. Human mode shows pass origins, kernels, resource mappings, connected/default output sources, hash, and estimates. Exit `0` means compiled, `2` means invalid invocation/source/request, and `1` means file/report I/O failure. Malformed CLI syntax, duplicate flags/override IDs, or malformed override JSON write usage to stderr; semantic failures use the chosen report mode.

## Public API and normalized source

```rust
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile, normalize};

let mut request = CompileRequest::default();
request.size = [65, 3];
request.outputs = vec![OutputChannel::BaseColor, OutputChannel::Roughness];
request.overrides.insert("frequency".into(), serde_json::json!(16));
let document = MaterialDocument::decode(source_bytes, &request.limits)?
    .into_validated(&request.limits)?;
let normalized = normalize(&document, &request)?;
let plan = compile(&document, &request)?;
assert_eq!(plan.passes().len(), 2);
assert_eq!(plan.estimates().peak_bytes, 5488);
```

The executable form is covered by [core doctests](../crates/mixture-core/src/lib.rs). `CompileRequest` explicitly owns dimensions, typed channels, a public-ID-to-JSON override map, and limits. `normalize` and `compile` accept `&ValidatedDocument`. They recheck graph collection limits under the supplied policy, reject zero dimensions, check each axis and requested-output/override counts, and never raise limits. Source byte limits apply during decoding; a validated document does not retain its original byte length. Callers decoding with a stricter policy must supply it at that boundary too.

All override targets must be declared, and each value must satisfy the existing parameter contract. Integer parameters require integer JSON tokens; floats/colors must be finite and within range; enum values must match exactly. Overrides are applied together to a cloned document, then cross-parameter constraints are validated on the resulting state. Even overrides on pruned branches are checked. Compilation never repairs or clamps a value.

`NormalizedDocument` exposes only a shared reference to a full source view: defaults are explicit, float/color values have f64 JSON representation, signed zero is positive zero, and nodes/edges/public bindings have stable lexical order. Unused nodes remain in this view. Neither normalization nor compilation mutates the original `ValidatedDocument`. Full normalization is separate from plan slicing.

## Ordering, kernels, and resources

The compiler starts at the requested material inputs and walks connected dependencies backward. Lexical Kahn ordering chooses the smallest ready node ID after each node. Repeated bindings from one producer are one ordering dependency. Requests are emitted in material-contract channel order regardless of CLI/API request order.

Each selected pixel node emits one pass. An unconnected optional input emits a constant pass immediately before its consumer, in contract input order. Requested unconnected material inputs emit constants after graph passes, in channel order. `material-output` is an output mapping, not a pixel pass. Distinct defaults are not deduplicated. A producer shared by several bindings/channels is compiled once; channels may map to the same resource.

`PassId` and `ResourceId` are zero-based consecutive indices in emitted pass order. Every pass produces one logical 2D, one-mip, one-layer `rgba16float` texture at the requested size. Its `TextureDesc` also carries `Scalar`, `Color`, or `Normal`. Scalar storage is `[value, 0, 0, 1]`; normal storage is `[x, y, z, 1]` with the contract's encoded tangent-space default. Dispatch is `[ceil(width/8), ceil(height/8), 1]` for local workgroups `[8, 8, 1]`.

| KernelId / invocation | Source and typed arguments | Planned uniform bytes |
| --- | --- | --- |
| `constant` | `constant-scalar`, `constant-color`, or synthesized input/channel defaults; `[f32; 4]` | 16 |
| `checker` | `[u32; 2]` cells and two `[f32; 4]` colors | 48 |
| `levels` | scalar input `ResourceId`, five f32 parameters | 32 |
| `blend` | color `a`/`b` and scalar `mask` resource IDs, typed `BlendMode`, f32 opacity | 16 |
| `fractalNoise` / `FractalNoise` | u32 `seed`, `scale`, `octaves`; f32 `persistence`; typed `NoiseBasis` | 32 |
| `gradientMap` / `GradientMap` | scalar `input: ResourceId`, two linear RGBA `[f32; 4]` endpoints | 32 |
| `heightToNormal` / `HeightToNormal` | scalar `input: ResourceId`, f32 `strength` | 16 |
| `transform2d` / `Transform2d` | `transform-2d` source node; `input: ResourceId`, `scale: [u32; 2]`, `quarter_turns: u32`, `offset: [f32; 2]` | 32 |
| `warp` / `Warp` | `warp` source node; `input: ResourceId`, `displacement: ResourceId`, `strength: [f32; 2]` | 16 |

`KernelInvocation::id()` is exhaustive; `inputs()` exposes typed bindings in binding order. There is no arbitrary parameter JSON or second untyped input list in the plan. `PassOrigin` retains node ID/type/version or the owner and port of a synthesized default. `PlanOutput` retains channel kind, the connected endpoint or explicit default, and the actual logical resource. `RenderPlan` is immutable through its public API and cannot be deserialized or publicly constructed with forged references/hashes.

Source f64 values are rounded to f32 during lowering; the plan records those effective parameters. Distinct decimals that round to the same f32 have the same plan. `levels` input bounds must remain strictly increasing after conversion; collapsed bounds fail at compile stage. Checker coordinate multiplication and allocation arithmetic are checked before returning a plan. Device-specific limits, shader binding layouts, actual GPU allocation, f16 texture precision, and pixel verification remain the executor's responsibility in PR-007.

`KernelInvocation::Transform2d` lowers source `scaleX`/`scaleY` to `scale`, `quarterTurns` to `quarter_turns`, and `offsetX`/`offsetY` to `offset`. Scale components remain exact u32 integers in `[1, 64]`, and quarter turns remain an exact u32 in `[0, 3]`; neither passes through a float. Defaults are `scale: [1, 1]`, `quarter_turns: 0`, and `offset: [0.0, 0.0]`. Offsets lower from finite source f64 values in `[-1, 1]` to f32. The invocation specifies inverse quarter rotation about the tile center, then scaling along the source axes, then a source UV offset; `quarter_turns` describes the visible clockwise rotation in top-left image coordinates.

`KernelInvocation::Warp` lowers `strengthX`/`strengthY` to `strength: [f32; 2]`, with default `[0.05, 0.0]`. Each component lowers from a finite source f64 value in `[-1, 1]`. Both invocations require a `Scalar` connection at source port `in` and produce a `Scalar` resource. Warp also requires a `Scalar` `displacement` connection; field value `0.5` is neutral. `inputs()` returns `[input]` for transform and `[input, displacement]` for warp. Repeated warp bindings remain in this list even when they refer to one shared producer pass; required connections never generate synthesized defaults. See [the node contracts](./node-contracts.md) for repeat-bilinear sampling and coordinate semantics.

The executor binds the uniform at `0`, output texture at `1`, and `input` at `2`; warp adds `displacement` at `3`. Transform's 32-byte uniform contains four u32 words `[scale[0], scale[1], quarter_turns, 0]`, followed by four f32 words `[offset[0], offset[1], 0.0, 0.0]`. Warp's 16-byte uniform contains four f32 words `[strength[0], strength[1], 0.0, 0.0]`. The plan's `uniform_bytes()` includes this padding; resource IDs are typed bindings, not uniform payload values.

## Allocation estimate contract

Plan version 1 assumes a deliberately simple execution schedule: retain every pass texture and uniform until execution/readback finish; read requested channels sequentially, allocating and releasing one staging buffer for each. There is no early release or resource pool. The estimates are logical GPU allocation bytes, not driver measurements, CPU memory, PNG allocations, shader/pipeline memory, or device allocation granularity.

Let `W`/`H` be dimensions, `P` be pass count, `O` be requested channel count, `U` be total padded uniform bytes, `T = W × H × 8`, and `R = ceil(W × 8 / 256) × 256 × H`:

| Report field | Definition |
| --- | --- |
| `textureBytes` | `P × T`; also peak resident texture bytes |
| `uniformBytes` | `U`, sum of the table's uniform sizes |
| `paddedBytesPerRow` | `ceil(W × 8 / 256) × 256` |
| `readbackBufferBytes` | `R`, one resident staging buffer |
| `readbackBytes` | `O × T`, tight raw rgba16float output bytes |
| `cumulativeReadbackBytes` | `O × R`, total staging allocation across readbacks |
| `cumulativeBytes` | `P × T + U + O × R` |
| `peakBytes` | `P × T + U + R`; checked against `limits.transient_bytes` |

Aliases still read once per requested channel. Arithmetic uses checked u64 operations, with a checked u32 staging stride. A limit equal to `peakBytes` is accepted. At 65×3, checker plus default roughness has 2 passes, a 768-byte staging row, 5488 peak bytes, and 7792 cumulative bytes. Last-consumer release, alias-readback deduplication, pooling, and driver-specific overhead are outside this version; changes to this model require an explicit plan-version decision.

## Stable hash contract

`plan.version` is `1`. `hash` is `sha256:` plus 64 lowercase hexadecimal digits. Hash input is exactly the bytes `mixture-render-plan-v1` followed by a NUL byte, then compact UTF-8 JSON of the plan body, excluding `hash`. `RenderPlan::hash_input()` returns those bytes for consumers. Root field order is `version`, `documentVersion`, `size`, `materialOutput`, `passes`, `outputs`, `estimates`; nested ordering is fixed by the typed serializer and the [checked-in snapshots](../crates/mixture-core/tests/snapshots/). This is a versioned serialization contract, not generic alphabetically sorted JSON. Changing key order, number formatting, lowering, or memory semantics needs review against this version.

The body includes source document version, selected node/owner IDs, types and versions, effective typed parameters (including override results), output provenance, requested channels, dimensions, pass/resource identities, descriptions, dispatch, and deterministic estimates. Node IDs are semantically retained for diagnostics/order; renaming nodes is not graph-isomorphism normalization.

Whitespace, source key/node/edge/public-binding order, requested-channel order, omitted versus explicit defaults, signed zero, and no-op overrides do not change the plan/hash. Unrequested branches, valid overrides affecting only those branches, unused exposure metadata, and safety-policy ceilings that admit the same plan do not enter the body. Paths, timestamps, adapters, logs, and timings are absent. Two sources with identical effective plans share a hash. The full source remains the source of truth; this hash identifies a compiled request, not every byte of a `.mix` document.

`sha2` is the sole new direct core dependency, used for SHA-256 rather than a process-dependent standard hash or a custom cryptographic implementation. Its dependency closure is deliberately added to `Cargo.lock`; previously resolved package versions are preserved. See [the dependency policy](./development.md).

## Verification and next step

[Core plan tests](../crates/mixture-core/tests/plan.rs) cover four plan/hash snapshots, permutation/default/no-op equivalence, effective overrides, full-source immutability, unrequested branches, shared producers, optional defaults, typed bindings, odd sizes, exact budgets, large sliced graphs, arithmetic overflow, and f32-bound collapse. [CLI integration tests](../crates/mixture-cli/tests/inspect.rs) cover real process reports, path independence, no GPU/source mutation, usage and semantic failures, bounded reads, and exit codes. `cargo xtask test-plan` runs both suites; `test-core` and `check` also include public API doctests.

The four new snapshots are checker/baseColor, checker/all defaults, all-M2/baseColor+roughness, and all-M2/roughness only. SHA-256 values were independently reproduced with Python hashlib from the compact bodies. For all-M2, baseColor+roughness has 5 passes; roughness only has one `mask` constant pass. Existing GPU pixel goldens were not changed.

PR-006's local focused and workspace checks passed on the pinned toolchain; remote cross-platform CI was still pending at that time. The documented [remote CI gates](./evidence/remote-ci/README.md) are now closed. PR-007 added [execution of these typed invocations](./graph-rendering.md) through the sole `wgpu` graph renderer, with all six M2 node pixel fixtures. PR-008 added [material golden tooling](./material-goldens.md); the completed [M3 review](./m3-review.md) records human acceptance of all three materials.

PR-009 adds typed `FractalNoise`, `GradientMap` and `HeightToNormal` invocations without changing plan version 1 or old plan/hash snapshots. Noise uploads all u32 seed bits and hashes seed, basis, scale, octave and persistence semantics; the compiler retains typed Scalar/Color/Normal connections and slices new branches normally. Public [M3 API tests](../crates/mixture-core/tests/m3_nodes.rs) verify defaults, required seed, lowering, branch slicing and hash sensitivity.

PR-010 adds typed `Transform2d` and `Warp` invocations as an additive extension to the v1 source node catalog. Document and plan versions remain `1`; the existing hash prefix, old plan/hash snapshots, and existing material pixel baselines are unchanged. The new invocations use the same typed serialization, effective-parameter hashing and dependency slicing rules. Public [resampling API tests](../crates/mixture-core/tests/resampling.rs) cover lowered defaults, integer fields, ordered and repeated scalar bindings, uniform sizes, unrequested-branch slicing, parameter hash sensitivity, source-order equivalence, and rejection of missing or mistyped connections.

## ENG-04 compatibility and unpublished versions

The source packages advance to Rust 0.2.0 because adding ScalarBlend to the exhaustive public KernelId/KernelInvocation enums may break downstream exhaustive matches. No non_exhaustive retrofit or other API redesign is made. The browser candidate advances to 0.2.0-alpha.0; API schema 1, .mix version 1 and plan version/hash domain remain unchanged. Serialized existing variants and old plan hash snapshots remain unchanged. Public npm 0.1.0-alpha.0 stays pinned in the registry consumer and must reject scalar-blend with MIX_NODE_UNKNOWN_TYPE. Candidate installation changes only the staged runtime archive/version/integrity; frozen tool dependencies and the pinned disposable Studio source remain intact. Rust packages and the new browser candidate are unpublished; this work does not authorize publication.

# M6A-01 — Minimal external image resource contract

English | [简体中文](./m6a-resource-contract.zh-CN.md)

Status: design selected for review under the user-requested M6A-01 task, 2026-09-21. Integration of this document and [ADR 0007](./decisions/0007-external-image-resources.md) accepts the bounded design; it does not implement or qualify it. Published runtime `0.2.0-alpha.0` cannot consume these resources. M6A-02 through M6A-05 remain implementation and acceptance work. Names and examples below are planned contracts, not currently callable APIs.

## Use case and node

Combine a caller-supplied linear height image with `fractal-noise` (seed 29, scale 32), using the existing `scalar-blend@1` crossfade at weights 0, 0.25, 0.5 and 1; derive both height and normal. A constant Color still supplies required `material-output.baseColor`. No Studio work is needed.

Select exactly one new type, `image-input@1`: no graph inputs; output `value: Scalar`; required parameter `resourceId`, with no default. Add the genuine parameter kind `resourceRef`, represented by a JSON string matching `[A-Za-z][A-Za-z0-9_-]{0,63}`. IDs are case-sensitive logical names in a separate request resource namespace, never paths or URLs. They may be exposed and overridden through the existing public-parameter mechanism; validate override syntax even on a sliced-out branch. Do not encode IDs as enum members. Multiple nodes may reference the same ID.

Planned node fragment (not a runnable document on the current release):

```json
{"id":"sourceHeight","type":"image-input","version":1,"parameters":{"resourceId":"heightSource"}}
```

Keep the existing `.mix v1` envelope and node `parameters` object. There is no top-level resource manifest, embedded pixel data, file name, URI or editor field. Full document validation checks all node contracts and resource-ID syntax without requiring pixels. Resource completeness is checked when compiling a requested dependency slice. Browser `validate`, which currently also compiles, continues to require the selected resources; native document validation and CLI `validate` remain source-only.

## Input representation and pixels

Each request binding has exactly `id`, `width`, `height`, `format`, `bytesPerRow`, and `data`. The public browser representation is an entry array so duplicate IDs remain detectable; omission means an empty array. Rust exposes equivalent typed entries. Only `format: "rgba8-linear"` is supported. `data` contains tightly packed unsigned bytes, row-major from the top-left; `bytesPerRow = width * 4`, byte length exactly `bytesPerRow * height`. No row padding, trailing bytes, pixel offsets, mipmaps, layers or negative strides are accepted. A browser Uint8Array view is copied only within its declared byte offset/length, not its entire backing buffer.

Both dimensions must be positive u32 values, within the request output-dimension limit, and exactly equal to requested output dimensions. Browser values must be finite safe integers, without coercion. Reject mismatches: no resize, resampling or implicit color conversion. G, B and A may contain any byte and do not affect the scalar, including when A is zero; no premultiplication or alpha unpremultiplication occurs.

The sole new WGSL kernel reads the matching integer texel from an uploaded non-sRGB `rgba8unorm` texture. The scalar is the normalized R value `R / 255`, stored as `(value, 0, 0, 1)` through the existing `mixture_half4` rgba16float convention. There is no sampler/filtering, vertical flip or gamma transfer. Existing RGBA8 readback rounding remains in force; input-to-output exact byte identity must be tested, not assumed from the half-float intermediate. The node preserves supplied seams; it does not make an arbitrary image periodic. Existing wrapped `height-to-normal` behavior applies and can expose an input boundary discontinuity.

## Validation, capture and ownership

1. Validate source, overrides and request metadata; apply overrides and determine the selected dependency slice. Enumerate resources in stable lexical ID order. Missing resources are required only for reachable image nodes. The referenced-ID set for rejecting unknown bindings includes all nodes after overrides, including unused branches.
2. Reject duplicate binding IDs before constructing a map; reject IDs not referenced by any node. Supplied but sliced-out bindings still undergo format, dimension, length and budget validation. They are not uploaded or included in the plan hash. Omitted resources for sliced-out nodes are allowed. At weight endpoints, both `scalar-blend` inputs remain dependencies.
3. Check counts and lengths before copying or hashing. Use checked u64 arithmetic and checked host-size conversions before allocation. Reject overflow rather than wrapping, truncating or raising a limit. Resource validation finishes before render GPU allocation or submission; an already explicit GPU context may exist.
4. Capture owned immutable pixel snapshots. Native preparation synchronously copies borrowed bytes before returning a prepared request. Browser render accepts only non-shared, non-resizable Uint8Array storage, rejects detached buffers and accessor-bearing/unsupported request objects, and copies bytes plus metadata before the first asynchronous yield. SharedArrayBuffer is rejected; no concurrent-write snapshot is promised. Busy/closing requests reject before pixel copying. Later caller mutation or release cannot alter an accepted request.
5. Core prepares an opaque immutable pairing of the compiled plan and selected owned snapshots. The executor consumes this pairing; callers cannot substitute bytes or provide a trusted digest. Plain `Renderer::render(plan)` remains usable for resource-free plans and explicitly rejects resource-bearing plans lacking their prepared resources. The new resource-aware public path must support Native and browser consumption without private imports. No unresolved executable plan or late ID-only binding is accepted.
6. wgpu owns uploads and GPU textures per render, uploading each selected ID once even if several nodes reference it. No cross-render resource cache or content deduplication is introduced. All per-render resource descriptors and owned snapshots are released on completion, rejection, GPU failure or renderer destruction; returned output pixels remain owned. Retaining a Native prepared request explicitly retains its CPU snapshots until the caller drops it. Browser instances retain no accepted input after cleanup. Existing first-error and destroy-waits-for-active-render rules remain.

## Explicit budgets

Add a separate typed resource-limit policy to resource-aware requests; retain existing SafetyLimits and its strict decoding. Omission selects the defaults below; zero is a valid ceiling, equality passes, and no limit is raised implicitly. Expose u64 policy values as browser bigint, as with existing limits.

| Budget | Default | Accounting |
|---|---:|---|
| `resourceCount` | 8 | All supplied entries, checked before capture; repeated references do not add entries |
| Per-axis size | Existing `outputDimension`, default 2048 | Every supplied image, plus exact output-size equality |
| `resourcePixels` | 16,777,216 | Sum of width × height over all supplied entries |
| `resourceBytes` | 67,108,864 (64 MiB) | Sum of exact packed lengths over all supplied entries; also caps copied/hash input bytes |
| GPU transient bytes | Existing `transientBytes`, default 512 MiB | Existing plan/readback/uniform estimate plus uploaded textures and conservative upload staging for selected resources |

For each selected resource reserve `4*w*h` GPU texture bytes plus `align_up(4*w,256)*h` upload-staging bytes in the estimate, even if the upload path uses less staging. Retain the existing conservative lifetime schedule; do not introduce pooling. Report unique uploaded resource count, packed upload bytes and resource texture/staging estimates separately from existing output/pass counts. Actual descriptor counters must include new owned GPU allocations; estimates do not claim driver allocation sizes.

Allow at most two full engine-owned packed copies during browser JS-to-WASM transfer, bounded by `2 * resourceBytes`; discard the transfer copy once Rust owns the snapshot. Native preparation requires one owned copy. These bounds exclude caller-owned storage, document/metadata and existing output buffers. They are per request/instance, not a global cap across independent renderers. Allocation failure is a structured failure, not a guarantee that every policy-admitted allocation succeeds.

## Identity and compatibility

Compute the digest internally from captured bytes. Resource content digest is SHA-256 over the concatenation of UTF-8 `mixture-image-rgba8-linear-v1`, a zero byte, width and height as unsigned 32-bit little-endian integers, and all packed RGBA bytes. Store lowercase hex. Hash all channels even though the node reads only R, conservatively identifying exact input content. Dimensions determine stride; no external supplied digest is trusted.

The plan contains a lexical-ID-ordered table of selected resources: ID, format, width, height, bytesPerRow and content digest. Image invocations reference those IDs. Include this table and upload estimates in canonical plan serialization and the plan hash; exclude raw bytes. Binding order, unrelated branches and unused bindings do not change the plan hash; changing selected content, IDs, dimensions or references does. Same-ID replacement must never reuse an old executable resource pairing. Pipeline caching remains keyed by kernel/device ABI, not pixel digest; the new kernel adds one reviewed cache identity, not one per image.

| Contract | Decision for implementation |
|---|---|
| `.mix` | Stay at v1; additive new node/parameter kind within existing source shape; no reinterpretation of old fields |
| Node catalog | Add only `image-input@1`; preserve all twelve existing identities and semantics; update exhaustive catalog and kernel tests |
| RenderPlan | Advance globally to version 2 and hash domain `mixture-render-plan-v2\0`, including resource-free plans, because the resource/estimate model changes |
| Existing plan hashes | Intentionally invalidate v1 hashes; preserve old source and pixel behavior. Keep v1 historical receipts; add independently calculated v2 snapshots rather than relabelling old acceptance |
| Rust packages | Target 0.3.0 for the implementation's exhaustive enum/public API changes; keep registry publication a separate decision |
| Browser package/API | Target 0.3.0-alpha.0 and API schema 2 for new request, catalog, plan and metrics representations; reject mixed JS/WASM/schema artifacts |
| Native CLI JSON | Advance plan-bearing inspect/render report schema to 2; source-only validate and doctor schemas remain unchanged |

This documentation PR changes no package version, executable schema or snapshot. Existing `.mix` files need no migration. Old runtimes must reject the new node with `MIX_NODE_UNKNOWN_TYPE`, including on unused branches. Old requests without resources retain pixel semantics on the new runtime, but consumers must adopt schema 2/new catalog kinds and invalidate v1 plan caches. No persisted-plan loading/migration feature is introduced. Preserve exact 0.2.0-alpha.0 registry qualification separately from the new candidate. Keep all six required checks; any producer-owned disposable consumer schema/catalog adjustment must be explicit and retain its original source/digests, without editing Studio.

## Diagnostics

Reserve (not yet implement) `MIX_RESOURCE_INVALID_BINDING`, `MIX_RESOURCE_DUPLICATE_ID`, `MIX_RESOURCE_UNKNOWN_ID`, `MIX_RESOURCE_MISSING`, `MIX_RESOURCE_FORMAT_UNSUPPORTED`, `MIX_RESOURCE_SIZE_MISMATCH`, `MIX_RESOURCE_LENGTH_MISMATCH`, `MIX_RESOURCE_IDENTITY_MISMATCH`, and `MIX_LIMIT_RESOURCE_COUNT_EXCEEDED`, `MIX_LIMIT_RESOURCE_PIXELS_EXCEEDED`, `MIX_LIMIT_RESOURCE_BYTES_EXCEEDED`. Resource request/identity failures use stage `compile`; malformed node parameters retain existing `validation` codes. Overflow uses invalid-binding with operation/operand evidence. Existing GPU device/limit/OOM/execution codes retain the first upload failure, with operation `resourceUpload` and resource ID evidence, rather than a generic wrapper or silent fallback.

Include applicable nodeId/parameterId, resource ID in evidence, expected/observed format/dimensions/length, configured/observed budgets, and an actionable suggestion. Never include raw pixel bytes. Preserve deterministic diagnostic ordering and the native source chain. Browser argument/capture failures use the existing structured request-error boundary; stable resource codes are supplied by Rust for semantically valid typed requests. Missing-resource diagnostics identify each affected reachable node in stable order.

## Acceptance tasks and stopping conditions

| Task | Required evidence before completion |
|---|---|
| M6A-02 — Core | Decode/reject/round-trip resourceRef; exposed overrides; full-node validation vs selected completeness; duplicate/unknown/missing bindings; exact and exceeded budgets, overflow; capture immutability; deterministic sorting/digests/v2 hashes independently reproduced; old source behavior |
| M6A-03 — Native | One production image WGSL kernel; integer sampling at 1×1, 1×N, N×1, 65×3 and 1K; independent known-value expectations; upload accounting and bounded repeated success/failure/destruction; public Rust prepared-resource consumption |
| M6A-04 — Browser | Same bytes/graph/requests through installed candidate; mutation after call, offset views, busy/destroy, shared/resizable/detached rejection; typed schema/catalog consumption; Native/browser pixel comparison and owned output after destroy |
| M6A-05 — Qualification | New resource tests plus existing Scalar and three-material regressions, six protected checks, source/archive identities, reviewed contact sheets and retained acceptance evidence |

Freeze test inputs before implementation acceptance: asymmetric top-left 2×2 values R = [0,64;128,255] with deliberately differing G/B/A; all 256 R codes; odd-width padding traps; periodic height tiles and deliberately nonperiodic edges. Require input orientation/channel correctness, finite in-range output, opaque scalar output alpha, exact direct-height roundtrip for the byte ramp, and unchanged R-derived pixels when only G/B/A change (while the digest changes). Weight 0 must equal direct imported height/normal; weight 1 must equal direct procedural height/normal. Use 0.25/0.5 for causality and nondegeneracy. No CPU graph renderer is added: literal expected cases and algebraic relationships test the production shader.

Same-source Native/browser height and normal RGBA8 comparisons must have maximum component delta ≤1. Do not automatically apply existing material-specific v2 structural thresholds to arbitrary imported data; use the frozen resource fixtures' endpoint, orientation, gradient, seam-preservation and causality expectations, and review 1K normal/height contact sheets. Existing material gates remain unchanged. Fix discrepancies or explicitly revise the contract with evidence; never fit thresholds or overwrite goldens merely to pass.

Run existing `test-format`, `test-core`, `test-plan`, `shader-check`, `test-consumer`, `gpu-smoke`, `cargo xtask check` as applicable. `cargo xtask test-node image-input` is a future target requiring fixtures/tooling registration; it is not implemented by this document. A CLI image-loading flag or decoder is not needed for first acceptance; use the public Rust consumer and browser bytes. `inspect --plan` without required resources fails explicitly; do not invent placeholder content or hashes.

URL downloading, file decoding frameworks, color/HDR inputs, arbitrary strides, automatic resampling, spatial masks, additive blending, `.mixpack`, caching/deduplication, GPUTexture imports, zero-copy, extra crates, Studio UI and publication are out of scope. The next task is implementation of this contract, not a broader resource system.

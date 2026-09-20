# Browser SDK delivery contract — M5

English | [简体中文](./browser-sdk.zh-CN.md)

**Current status (2026-09-20):** Native M4/M4.1, bounded M5, the recorded Studio MVP and ordinary Windows Chrome/Edge/Firefox qualification are complete within their recorded scope. The npm Alpha `@openmixture/runtime@0.1.0-alpha.0` is published, and exact registry-version consumption passes the recorded Studio gates. Rust crates remain unpublished; both browser checks are required on main. The [Post-Alpha roadmap](../ROADMAP.md) owns current work; first-delivery evidence remains in Alpha closeout. M6 remains unscheduled.

The dated checkpoints below retain their status at the time; they are not the current backlog.

**M5 acceptance, 2026-09-15:** [Browser acceptance](./evidence/m5-05/README.md) closes M5-05 for the recorded macOS/Linux Chromium matrix: each environment passes 11 post-freeze 1K cases/44 channel comparisons, semantics/quality gates and 12 additional lifecycle renders, alongside independent product isolation, 28 browser contracts and normal production static deployment. Complete pixels and provenance are retained. Alpha readiness is bounded to tested coverage; npm remains unpublished and Studio/M6 require separate decisions. Earlier dated entries below retain their historical status.

**Player export update, 2026-09-15:** the M5-04 open → edit → channel preview → PNG download workflow now passes local acceptance in the independent product. The clean isolated consumer passed 28 Chromium checks and nine Node tests. Twelve 128×128 channel PNGs from three materials decode to the exact public-runtime bytes with correct sRGB/linear metadata. Additional checks cover eight-channel 65×3 downloads, stale-export suppression and encoding failure. [Product evidence](https://github.com/OpenMixture/Studio/blob/77f00deb5410b73140220fabe31f4f8599b3279d/docs/evidence/m5-04-export/README.md) binds the exact source and unchanged runtime archive. M5-05 1K cross-runtime quality, stress, formal browser CI and deployment/compatibility qualification remain open; nothing is published.

**Local acceptance update, 2026-09-14:** M5-02/M5-03 initial browser execution and isolated package consumption now pass the [recorded gates](./evidence/m5-02-03/README.md). The unchanged archive passes 13 Chromium checks, including controlled real-device loss, mapping cleanup and repeated independent modules/devices. M5-04/M5-05 material regression, complete Player workflows and the accepted browser CI matrix remain open; the package is unpublished.

**Implementation update, 2026-09-12:** `mixture-wasm` and the local `@openmixture/runtime@0.1.0-alpha.0` tarball now implement the initial checker consumer slice. [Build/API usage and remaining verification](./browser-runtime.md) describe the actual checkpoint. This contract describes the implemented public boundary and the acceptance requirements that remain open, including full material regression, broader browser failure/stress coverage and the complete Player export workflow. No npm registry publication or full M5 acceptance is claimed.

The M5 outcome is one complete browser runtime package built in the engine repository and consumed by an independent Player. The [OpenMixture/Studio product repository](https://github.com/OpenMixture/Studio) now exists and contains the initial Player consumer. Player precedes Studio authoring, and the two may later share product modules. See the [roadmap](../ROADMAP.md), [native SDK](./native-sdk.md), [format](./file-format.md), [plan](./render-plan.md) and [rendering](./graph-rendering.md) contracts for existing engine behavior.

## Ownership and distribution

| Owner | Contract |
|---|---|
| `mixture-core` | Authoritative `.mix` decoding, node/port/parameter contracts, versioned defaults, validation, exposed overrides, normalization, compilation, plan ordering and hashing. Remains GPU-free. |
| `mixture-wgpu` | The sole pixel executor, with the same WGSL, explicit context, pipeline cache, execution, output conversion and resource cleanup. Browser completion is adapted here where it is executor behavior. |
| `mixture-wasm` | A thin browser binding over normal typed Rust APIs. It owns representation conversion and the browser-facing call boundary, not a second catalog or graph implementation. |
| `@openmixture/runtime` | One public package containing compatible browser ESM, TypeScript declarations, the WASM binary and loader assets from one build. JavaScript owns explicit loading, argument marshalling and owned-result transfer. |
| Independent Player / later Studio | File selection, graph drafts, controls, debounce, result freshness, 2D preview, downloads and deployment. Later editor layout, undo/redo, camera and lighting remain product state. |

The engine repository builds and versions the npm package. There is no separate JS SDK repository or mandatory family of public `core`/`wgpu`/`wasm`/`schema` packages. Publishing the Rust crates to a Cargo registry is not a prerequisite: the engine's locked source build supplies the complete npm artifact. The installed product does not require Rust, a Cargo checkout, a compilation `postinstall`, or producer-private imports.

`.mix` remains material semantics. Layout and application preferences must not be added to v1; a later product can own a separate layout file. UI hints may use runtime metadata, but Rust remains the authority on legality and meaning.

## Loading and explicit GPU initialization

The following operations are implemented by the local package. Exact signatures and JS types are in the [public declarations](../packages/runtime/src/index.d.ts), with executable usage and build commands in the [browser start guide](./browser-runtime.md). Consumer tests must continue to verify this boundary; implemented methods alone do not satisfy the remaining M5 acceptance gates.

| Public operation | Observable behavior |
|---|---|
| Import the package | Loads JavaScript definitions only. Does not fetch/instantiate WASM, read a document, request an adapter/device, render, start a worker or install a frame loop. |
| `loadRuntime` | Explicitly loads/instantiates the packaged WASM and returns a module handle. `loadRuntime({ wasm })` accepts a URL/string or `Uint8Array`; omitting `wasm` uses the package-relative asset URL. The single option selects either a location or supplied bytes. No GPU is requested. |
| `getBuildInfo`, `getNodeCatalog` on the module | Read owned version/build metadata and the Rust node catalog after loading. No GPU is required. |
| `validate` on the module | Decodes and validates source and compiles its request under explicit limits. Returns the shared diagnostic result and, on success, the plan and resolved public-binding/channel metadata. Invalid drafts return diagnostics; they are not repaired into renderable graphs. |
| `inspect` on the module | Validates and compiles source plus a render request without acquiring a GPU. Returns a read-only plan description/hash and estimates for cross-target checks; it cannot manufacture an executable plan from arbitrary JS data. |
| `createGpu` on the module | Explicitly acquires an independent browser GPU context and returns a renderer instance plus requested/actual capability evidence. Only this phase requests the adapter/device. |
| `render` on an instance | Validates/compiles the complete source and request through core, executes the resulting immutable plan, then returns owned pixels and reports asynchronously. Invalid input performs no render allocations or submissions. |
| `destroy` on an instance | Stops accepting renders immediately, waits for an accepted render's cleanup, releases instance-owned GPU resources and resolves asynchronously. Repeated calls are idempotent. |

Loading has no implicit network retry or CDN fallback. Supplied bytes cause no loader fetch. The loader resolves omitted input against the packaged asset URL and supplied URL strings against the page URL. It buffers fetched bytes before WebAssembly instantiation, so it does not require the streaming API's `application/wasm` MIME. URL resolution, incorrect path, HTTP, CORS, compilation and instantiation failures remain actionable structured failures; serving requirements are documented in the [browser start guide](./browser-runtime.md). No application-controlled material URL is fetched by `validate` or `render`: the product supplies document content. Successful WASM loading and successful GPU acquisition are distinct states.

M5 targets browser WebGPU in a secure serving context. GPU initialization reports an unavailable API/context or rejected acquisition without selecting a WebGL/CPU executor. Browser options expose only meaningful browser choices, such as power preference; they must not pretend that callers can force a native Metal/Vulkan backend or exact adapter name. Evidence records requested preferences, actual values available from the browser, device limits and missing capability information honestly. Unavailable/redacted adapter fields remain unavailable, not invented identities. Neither acquisition success nor a feature check counts as a successful render.

## Source, requests and metadata

| Input | Required behavior |
|---|---|
| `.mix` source | Accept a JS string or UTF-8 `Uint8Array`; capture the request before asynchronous work so caller mutations cannot change an accepted render. Preserve raw source parsing in Rust. Do not `JSON.parse` then reserialize incoming files: that loses duplicate keys and numeric-token distinctions checked by core. |
| Strings and byte limits | Apply the existing decoded-byte ceiling to UTF-8 bytes, not JS character count. Reject malformed UTF-8 bytes; reject unpaired UTF-16 surrogates in string input rather than silently replacing them. Bound conversion/copying before allocating from an oversized input. |
| Size | Width and height are finite positive safe integers accepted by the core u32 and safety/device limits. Fractional, negative, zero, overflow and non-finite values fail; no truncation or bitwise wrapping. Omission follows `CompileRequest` defaults: 64×64. |
| Requested channels | Omission means `baseColor`. Accept a nonempty list of unique supported case-sensitive channel IDs. Empty, unknown or duplicate entries fail. Returned channels follow material-contract order, independent of request order. |
| Exposed overrides | A public-ID-to-value request, never direct node addressing. Capture own keys/values without executing accessors; reject unsupported JS shapes and duplicate entries if an entry-list representation is exposed. Apply all overrides together using core, including on pruned branches; do not mutate source or clamp values. |
| JS values | Reject `undefined`, functions, symbols, cyclic objects, class instances and non-finite numbers instead of relying on JSON serialization to omit or rewrite them. Integer parameters require `Number.isSafeInteger`, nonnegative u32 range and node-specific bounds before conversion; booleans and numeric strings are not numbers. Colors have exactly four finite components in range, and enums match the Rust contract exactly. |
| Safety limits | Use the same core defaults and explicit caller policy at both decoding and compilation. Do not silently raise a ceiling to fit a request. Rust u64 ceilings cross the JS boundary as nonnegative `bigint` values checked against u64 range. Browser/device limits may reject a request already admitted by core. |

JavaScript numbers do not retain whether a caller wrote `8` or `8.0`; both are the same integer-valued JS number. Accepted integer overrides must therefore be marshalled into Rust integer values, not f64 JSON tokens. In a raw `.mix` file, the existing distinction remains: integer parameter/version token `8.0` is invalid. The bridge must test u32 endpoints, fractional values, negative values, unsafe integers, non-finite values and invalid object shapes without coercion. String-source validation and JS request validation are separate boundaries.

Catalog entries come directly from the versioned Rust registry: type ID, node version, label/description, named input/output kinds, required/optional input defaults, parameter types, inclusive ranges, enum members and parameter defaults. Output ports must not be mislabeled as required inputs merely because they have no default. Catalog ordering is deterministic and delivered data must not be mutable aliases of Rust state.

For a validated document, public-binding metadata joins each `exposedParameters` ID to its node ID/type/version, parameter ID/contract, explicit source value or resolved default, and effective override value when inspecting a request. A required parameter without a default remains required. Material-channel metadata includes logical kind and connected endpoint or explicit versioned default. Invalid drafts must not expose an apparently validated binding list. Player can build controls from this data without duplicating defaults, constraints or node semantics in TypeScript; cross-parameter validity is still decided by Rust.

## Owned output and version evidence

An accepted render returns all requested channels or an error with no partial successful pixel result. Each channel contains `channel`, `kind`, `source`, `size`, `encoding` and an owned `Uint8Array` of tightly packed RGBA8 pixels. Rows are top-left, row-major, with exactly `width × height × 4` bytes and no staging padding. Alpha is straight.

| Kind / channels | Returned bytes | Product preview and PNG responsibility |
|---|---|---|
| Color: `baseColor`, `emissive` | RGB has the engine's linear-to-sRGB transfer; alpha remains linear. | Interpret color RGB as sRGB and avoid applying the transfer a second time. PNG exports preserve sRGB semantics. |
| Scalar: `roughness`, `metallic`, `height`, `ambientOcclusion`, `opacity` | Scalar replicated into RGB, opaque alpha; linear bytes. | Treat as data. Export must preserve numeric bytes and linear metadata, matching the native convention of gAMA = 1.0 with no sRGB chunk. |
| Normal: `normal` | Already encoded tangent-space XYZ in RGB, opaque alpha; linear bytes. | Do not gamma-correct the data or renormalize it as an implicit export step. Preserve linear metadata and numeric bytes. |

M5 returns the established RGBA8 representation, not lossless internal `rgba16float` textures. Product PNG encoding is outside the engine renderer. Canvas display/download behavior alone is not evidence that data-channel PNGs preserve bytes or metadata; acceptance must decode exports and inspect the relevant chunks. A preview may have an explicitly display-only treatment, but exported data must retain its documented channel meaning.

Returned pixels are copied into JS-owned storage before resolution and never alias short-lived WASM memory or a mapped GPU buffer. They remain valid after another render, WASM memory growth and instance destruction. Mutating one returned channel must not mutate another channel, a retained prior result or the engine. GC of JS-owned buffers is the consumer's responsibility; `destroy` does not detach retained results. The first version has no GPUTexture handoff or zero-copy promise.

Results preserve the compiled plan hash/version, document version, requested dimensions/channels, effective plan description available to inspection, actual context evidence and applicable execution metrics. Build info identifies the npm runtime version, binding/API schema version, Rust engine versions/revision and one build identifier shared by JS, declarations and WASM. Missing provenance is explicit; a build cannot invent a commit identifier. Package/binding version checks reject mixed artifacts before normal execution.

The runtime/package version, `.mix` format version, node versions, plan version and product release version are separate contracts. A product UI release does not require a format change. Plan hashes retain the existing core semantics; package versions, adapter names, timings, URLs and product generations are not new hash inputs. Same-build Native/browser comparison uses the exact same source, overrides, dimensions, channels and policy.

Rust u64 and usize values in reports, estimates and unsigned diagnostic evidence are represented as `bigint` in SDK objects; typed u32 fields such as dimensions, versions and pipeline-cache hit/miss counts remain exact JS numbers. Empty optional diagnostic evidence may be absent, as reflected in the public declarations. This is a lossless JS projection, not a change to the existing native JSON schema. Plain `JSON.stringify` is not a serializer for this projection. A product that records JSON logs must explicitly encode bigint values without rounding and must not feed that product log format to the strict native diagnostic decoder. M5-03 must test these types against the actual declarations and runtime.

## Structured failures and asynchronous readback

Core and GPU failures retain the existing `MIX_*` code, stage, severity, message, optional node/port/parameter/document context, ordered diagnostics, scalar evidence and suggestion described in [diagnostics](./diagnostics.md) and [GPU failures](./gpu-failures.md). Browser wrappers do not replace an earlier meaningful failure with a generic promise rejection. Where supplied by the caller, a document label is diagnostic context only; it does not cause file access or enter the plan hash. Available adapter, plan, device-loss and cleanup evidence survives failure.

The public failure boundary uses `MixtureRuntimeError`, a structured SDK error carrying the invoked operation, the unchanged engine diagnostic list when present, and a separate browser failure when the error occurs outside Rust's diagnostic stages. Validation returns its expected diagnostic result; failed asynchronous loading, acquisition and rendering reject with that structured error. Known failures must not require parsing driver prose. Native source objects do not cross WASM; selected browser/driver cause text may be retained as evidence without claiming it is a native typed source chain.

The following **browser-only codes are implemented by the package** and are not entries in core `DiagnosticCode`. Consumer tests must preserve their exact envelope, field presence and spelling; native diagnostic codes remain unchanged. Browser failures carry `code`, `operation`, `message` and optional `evidence`/`suggestion`; they do not invent an existing core `stage` for WASM loading.

| Browser code | Use |
|---|---|
| `MIX_BROWSER_WASM_LOAD_FAILED` | URL resolution, fetch, compilation or instantiation failed; preserve the phase and useful URL/HTTP/cause evidence. |
| `MIX_BROWSER_BUILD_MISMATCH` | JS/WASM/binding build or API identifiers disagree. |
| `MIX_BROWSER_WEBGPU_UNAVAILABLE` | Browser preflight cannot access the required WebGPU context/API; actual adapter/device failures retain engine GPU codes where provided. |
| `MIX_BROWSER_INVALID_ARGUMENT` | JS representation cannot be safely marshalled, before semantic core validation. |
| `MIX_BROWSER_RUNTIME_BUSY` | Another render is already accepted by this instance. |
| `MIX_BROWSER_RUNTIME_DESTROYED` | A render is attempted after destruction starts. |
| `MIX_BROWSER_BINDING_FAILED` | An unexpected binding failure or trap cannot be represented by an existing engine diagnostic or a more specific browser failure; preserve its cause text. |

OOM and device loss must use typed browser/wgpu signals where available; do not classify by matching driver text or pretend native callback/error-scope behavior has already been proved on browsers. A delivered loss makes an instance unusable for further renders; new GPU state requires another explicit acquisition. If OOM/readback failure was already selected and loss/cleanup failure is observed later, retain the primary failure and attach the secondary evidence. No automatic recovery or fallback is implied.

Native `Device::poll(Wait)` and blocking callback retrieval are not the browser completion contract. The [wgpu WebGPU documentation](https://docs.rs/wgpu/latest/wasm32-unknown-unknown/wgpu/struct.Device.html#method.poll) states that `poll` has no effect on that backend. Browser execution must await event-loop-driven submission/map/error completion, balance error scopes and attempt cleanup on each success/failure path. It must not busy-wait or block the event loop waiting for a callback. Native 30-second individual waits remain native-only. The implemented browser path awaits callbacks and error-scope completion without adding a hard timeout or GPU cancellation promise; tab termination and undelivered platform events may prevent settlement. M5-02 still requires observable browser failure and completion evidence.

## Instance lifetime and product scheduling

Each explicitly created instance owns its context/cache; there is no process-global GPU instance. The minimal contract is one accepted render at a time per instance. A second render rejects as busy rather than creating an unbounded queue. Input capture and the busy/closing state transition happen before any asynchronous yield. Independent instances do not share mutable GPU state, though several instances still consume the same physical GPU resources and no throughput benefit is promised.

Calling `destroy` marks the instance closing immediately. New render calls reject. A render accepted before that transition completes with its normal success or structured failure and performs cleanup; `destroy` awaits that settlement before releasing remaining GPU state. Concurrent/repeated destroy calls await the same completion. Destruction does not cancel submitted GPU work, guarantee immediate OS memory reclamation, or invalidate previously returned pixels. The module's CPU validation/catalog functions remain usable. A lost instance may still be destroyed for cleanup.

The product's `runtime-client` owns debounce, one active request, one replaceable latest pending request and a monotonically increasing generation. A completion may update the display only if its generation is still current. Stale completions are discarded; an older displayed result stays visibly stale when the latest request fails. Product teardown stops pending scheduling and awaits destruction. These are consumer rules, consistent with [native freshness handling](./stale-results.md), not a scheduler added to core or the runtime. The product bounds retained CPU outputs; GPU allocation counters exclude these JS copies.

M5-02 must verify no outstanding render/destroy promise is abandoned by wrapper logic after a delivered completion or failure. Browser/tab termination and undelivered platform events are not covered by an invented hard deadline. Worker use and cancellation can be reviewed later if measurement shows they are needed; import or initialization must not silently create background workers.

## Package and consumer acceptance

The [browser start guide](./browser-runtime.md) records the current Rust 1.98.1, wasm-bindgen 0.2.128, `wasm32-unknown-unknown`, Node 24.20.0 and npm 11.19.0 build recipe, package exports and asset layout. The independent product locks its TypeScript/Vite/browser-test dependencies and provides `npm run test:browser` against production assets at `/player/`. M5-03 acceptance must bind the actual archive and product lock to the recorded run; the selected recipe is not a claim of complete M5 or all-bundler compatibility. Additional browser/platform claims require their own records.

| Gate | Required evidence before M5 acceptance |
|---|---|
| Real package contents | Build once, create a real npm tarball, inspect included public JS/types/WASM/license/readme assets, and record runtime version, build identifier and tarball digest. No producer-source imports or absolute developer paths. |
| Clean independent consumption | Install that tarball into the separate consumer with its own lockfile and no usable producer checkout/Rust toolchain. Type-check public imports, build production assets and run the served production output. A source link, dev server or `npm pack --dry-run` alone does not pass. |
| Loading and deployment | Prove default packaged asset resolution and a configured WASM URL/byte input. Test a non-root deployment base, missing asset, incompatible artifact and documented serving failures. Record actual HTTP/asset behavior. |
| CPU-only contract | Prove import has no loading/GPU effects and validation/catalog/inspection work after WASM load without requesting GPU. Invalid source/overrides retain duplicate-key, integer, limit and node-specific diagnostics. |
| Explicit browser rendering | First complete checker acquisition, dispatch, odd-sized/padded readback and destruction. Then exercise requested channels, overrides, output encoding, owned buffers, repeated renders, busy calls, in-flight destruction and available device-loss/failure paths. |
| Product workflow | Player opens `.mix`, generates exposed controls from metadata, switches 2D channels, reports failures, applies latest-generation display rules and exports correctly encoded PNGs through the public package. |
| Cross-target correctness | Compare Native and browser plans and all three ceramic/leather/wood materials with documented parameter variants. Record channels, resolution, fixture revisions, build, browser/OS/adapter and per-channel pixel metrics. |
| Honest support matrix | Record exact tested browser builds, OS versions, hardware/software adapters, serving context and pass/fail/skipped outcomes. Missing WebGPU or an unrun test cannot satisfy a rendering gate. Native CI success alone is not browser evidence. |

Browser pixel tolerances must be measured and frozen in a reviewed acceptance specification before they become release pass/fail thresholds. Start from existing fixtures and documented native tolerances, explain any browser-specific adjustment, and retain exact checker/default/encoding sentinels where their fixture requires equality. Report measured differences as well as threshold outcomes; do not widen tolerances or reset goldens merely to pass. Semantic plan equivalence is mandatory; a hash mismatch for the same build/request must be explained and corrected or receive an explicit versioned-contract decision, not be hidden by pixel tolerance.

The retained M5 receipt must bind the engine revision, package version/digest, consumer revision, pinned toolchains, browser matrix, deployment URL/base, test results and remaining limitations under the [evidence policy](./evidence-policy.md). Existing native acceptance does not certify the browser matrix. npm publication is a later explicit release action; local tarball consumption does not require it. If approved, an alpha release uses an explicit prerelease tag and the product locks an exact runtime version through a reviewable dependency update. The current Alpha is published; [registry-consumption evidence](./evidence/npm-alpha/README.md) binds the exact installed version.

## Non-goals and follow-up boundary

M5 does not implement a node editor, intermediate-node preview protocol, custom shaders, new material nodes, image resources, `.mixpack`, a second TypeScript/WebGL/CPU renderer, GPUTexture sharing, zero-copy results, 3D preview, all-bundler adapters, Node.js GPU or SSR rendering. npm is the distribution channel, not a claim that every JavaScript host can execute the runtime.

Player/Studio product work remains consumer-owned. Engine capability work may also originate from maintainer-defined use cases and measurements under the [Post-Alpha roadmap](../ROADMAP.md). [M6](../ROADMAP.md#m6--resources-and-portable-packaging) now separates external image input from portable packaging; each needs its own entry decision, and neither starts in ENG-01/02.

ENG-03 adds an [independent browser SDK example and qualification entry](../examples/browser-consumer/README.md) using this public contract. Candidate and exact registry builds are verified separately; the pinned Studio material and wider contract coverage remain required.

## ENG-04 compatibility and unpublished versions

The source packages advance to Rust 0.2.0 because adding ScalarBlend to the exhaustive public KernelId/KernelInvocation enums may break downstream exhaustive matches. No non_exhaustive retrofit or other API redesign is made. The browser candidate advances to 0.2.0-alpha.0; API schema 1, .mix version 1 and plan version/hash domain remain unchanged. Serialized existing variants and old plan hash snapshots remain unchanged. Public npm 0.1.0-alpha.0 stays pinned in the registry consumer and must reject scalar-blend with MIX_NODE_UNKNOWN_TYPE. Candidate installation changes only the staged runtime archive/version/integrity; frozen tool dependencies and the pinned disposable Studio source remain intact. Rust packages and the new browser candidate are unpublished; this work does not authorize publication.

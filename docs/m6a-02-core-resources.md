# M6A-02 — Core resource semantics and validation

English | [简体中文](./m6a-02-core-resources.zh-CN.md)

M6A-02 implements the Core portion of the [accepted resource contract](./m6a-resource-contract.md). It adds `image-input@1`, `resourceRef`, `ImageBinding`, separate `ResourceLimits`, immutable `ResourceSnapshot` and opaque `PreparedRender`. It does not complete the image node's GPU/browser vertical slice: native upload/execution remains M6A-03 and browser resource requests remain M6A-04. No new image pixels or package publication are accepted here.

## Public preparation

Decode and validate a `.mix` with ordinary `MaterialDocument` APIs, then call:

```rust,ignore
let prepared = mixture_core::prepare(
    &validated_document, &compile_request, &image_bindings, &resource_limits,
)?;
let plan = prepared.plan();
let captured = prepared.resources();
```

The variables above are caller-owned values, not a complete executable example. The [independent Rust consumer test](../examples/native-consumer/tests/resources.rs) is a runnable public-API example. `ImageBinding` borrows packed bytes for the synchronous preparation call; snapshots subsequently own their bytes. Neither plans nor prepared inputs expose mutation/deserialization constructors. Snapshots intentionally omit pixel bytes from Debug output and plan serialization.

All source nodes and exposed resource-ID overrides validate even when unused. Missing bindings are required only for selected nodes; unknown supplied IDs are checked against the normalized full graph. Duplicate entries cannot overwrite one another. Supplied unused resources still consume input budgets and undergo format/size/length checks, but are not captured, hashed or uploaded. Each distinct selected ID creates one snapshot and upload estimate regardless of reference count; equal bytes under different IDs are not deduplicated.

The resource policy defaults to 8 entries, 16,777,216 pixels and 64 MiB of packed bytes. Checked arithmetic, host-size conversion, exact strides, exact lengths and all input budgets precede pixel copying/hashing. Native snapshot allocation uses fallible reservation. Existing GPU transient limits include external textures and conservative aligned upload staging. Limit equality is accepted and zero stays zero. Policy ceilings do not enter the plan hash.

Ordinary `compile` uses an empty resource set: selected image nodes return `MIX_RESOURCE_MISSING`; a sliced-out image does not block a resource-free plan. CLI `validate` remains source-only. CLI `inspect --plan` and browser inspect/validate currently have no resource delivery argument and explicitly fail if the selected slice requires images.

## Plan and consumer compatibility

All plans now have version 2, hash prefix `mixture-render-plan-v2\0`, a lexical `imageResources` identity table and four explicit resource estimate fields. Content digests bind the captured domain/dimensions/all RGBA bytes per the contract. Executable v1 hash expectations are replaced by separately named v2 snapshots; original v1 JSON files remain unchanged historical records. Existing procedural kernels and material golden pixels are unchanged.

Rust workspace packages are 0.3.0; the unpublished browser candidate is 0.3.0-alpha.0, API schema 2. CLI inspect and graph-render report schemas are 2, while validate, doctor and fixed-checker envelopes retain their existing versions. Type declarations expose `resourceRef`, plan image identities and bigint resource estimates; browser upload arguments are deliberately not exposed yet. The standalone registry consumer continues to install exact published 0.2.0-alpha.0. Candidate qualification explicitly updates the pinned disposable host's catalog expectation to thirteen and its package-version expectation to 0.3.0-alpha.0, retaining original/adapted test hashes and all other checks. This does not edit Studio or accept candidate resource execution.

The exhaustive kernel mapping includes one image WGSL source with integer R sampling and the existing half-float store convention. Shader parsing/ABI checks cover it. Until M6A-03 supplies prepared-resource execution, the executor rejects image invocations before GPU allocation, pipeline lookup or submission, retaining structured resource and adapter evidence. This guard is not a fallback and must not be removed without implementing the upload path and its acceptance gates.

## Verification and retained inputs

- `cargo xtask test-format`, `test-core`, `test-plan`: source/override errors, slicing, duplicate/unknown/missing bindings, immutable capture, metadata/budget overflow and exact boundaries, deterministic identities and old document behavior.
- [Core resource tests](../crates/mixture-core/tests/resources.rs): asymmetric byte inputs, all-channel digest sensitivity, same-ID content replacement, reordered bindings/source, shared-ID references, 65×3 upload padding, dimension-sensitive identity and limit-policy independence.
- `cargo xtask shader-check`: new shader validation/ABI and existing shader checks; not new-node GPU qualification.
- `cargo xtask test-consumer`: independently authored document/pixels consumed through the public preparation API, plus existing Rust/CLI contracts.
- `cargo xtask check` and all six required CI checks remain integration gates. GPU/native/browser material CI exercises existing procedural content with the new plan and package identity; it does not claim uploaded-image execution.

New `plan-v2-*.json` snapshots were independently constructed and SHA-256 hashed with Node crypto. For old resource-free plans, preserve the original numeric tokens (including `.0`), change the plan version, prepend four zero-valued resource estimates and append an empty image table, then hash the compact body under the v2 prefix. The [image-plan snapshot](../crates/mixture-core/tests/snapshots/plan-v2-images.json) independently enumerates two 2×2 images, their content digests, three passes and the 1,712-byte peak/cumulative estimate. Tests compare production output to these separate expectations. No pixel goldens were regenerated.

The next step is M6A-03: consume the prepared pairing, upload each selected ID once, bind the image kernel, account for actual resource allocations, and execute known-value/lifecycle/Native pixel gates. Browser buffer capture, GPU qualification, CLI decoders, packaging, caches and publication remain outside M6A-02.

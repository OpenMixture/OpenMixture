# M6B-01 — Start portable asset packaging

English | [简体中文](./m6b-portable-assets.zh-CN.md)

Status: M6-B explicitly started by the maintainer on 2026-09-22. This first slice establishes a reproducible delivery problem, bounded requirements and the implementation sequence. It does not implement a loader, select a binary format, change an architecture boundary, or publish a package. The next slice is M6B-02 format/ownership selection. All API and command names for future work remain unimplemented until their owning PR lands.

## First consumer outcome

A caller can move **one offline asset** containing one unchanged `.mix v1` document and its existing `rgba8-linear` height resources to a different directory or machine. A Native public consumer and the browser public package can inspect it without a GPU, then prepare the same content-bound plan and render through the existing wgpu path. The recipient supplies bytes and rendering options; no author-machine paths, companion-file search, URLs, service account or network resolution is required. CLI file handling is an adapter over the same loader, not another implementation.

Use the existing `image-input` / `scalar-blend` height-normal fixture, one 1024×1024 image, weights 0/0.25/0.5/1 and a small asymmetric image control. No new node, image semantics, sampler, resolution conversion, graph syntax or pixel kernel is needed. This scope is independent of NUM-01: the initial baseline is main `c2a08e14be9991d75df24e0f045d842d8db4b570` / 0.3.0, not the unmerged 0.4 noise branch. Later qualification must identify the exact integrated node versions; packaging must never silently migrate them.

## Measured entry problem

[Retained measurement](./evidence/m6b-01/measurement.json), reproduced by [the CPU-only probe](../scripts/probe-portable-assets.mjs):

- The 1,157-byte `.mix` validates; its resource-free `baseColor` slice compiles successfully.
- Requesting height/normal returns `MIX_RESOURCE_MISSING` at compile with resource ID `heightSource`, exit 2. Putting the correctly sized raw image next to the copied document produces the same structured failure. Current CLI options provide no resource-binding loader. The SDK supports caller-supplied bindings; their transport is currently application-specific.
- A 1K packed RGBA8 image is 4,194,304 bytes; source plus pixels is 4,195,461 bytes across two files, still requiring external ID/format/size/stride knowledge.
- A deliberately non-public JSON/base64 representation is 5,593,949 bytes; payload base64 alone is 5,592,408 bytes, approximately 33.3% expansion. These are byte counts, not measured loading time, peak memory or network performance.

This is evidence for a portable binding and content-integrity contract. It is **not** evidence that a custom binary format, compression or a cache is necessary. A directory plus manifest remains a viable multi-file baseline; a standard archive of such a manifest is a comparison candidate. An application could already implement transport itself; M6-B supplies a shared producer contract so recipients do not reinvent incompatible loaders.

Reproduce from the recorded baseline (or pass its separately built CLI to the probe):

```sh
cargo build --locked -p mixture-cli
node scripts/probe-portable-assets.mjs tmp/m6b-entry-run
```

The destination must be fresh. The probe does not acquire a GPU, render pixels, overwrite fixtures or claim a production format. Its synthetic RGBA bytes are input data. It deliberately asserts the baseline's current missing-binding behavior; after a loader is implemented, reproduce historical evidence with the baseline CLI rather than rewriting the expected failure.

## Required contract for M6B-02

1. **Source and identity:** retain exact source bytes and explicit node versions; `.mix` remains the semantic source of truth. Record logical resource IDs, format, dimensions, exact packed length and content integrity. Recompute identities from captured bytes; reuse the M6-A resource digest and prepared-request authority. Package byte identity is distinct from plan identity. Filename, archive ordering, timestamps and transport metadata must not change a plan for identical source/resources/request.
2. **Single bounded load:** reject unsupported package versions, duplicate identities, malformed/truncated entries, inconsistent lengths, overflow and content mismatch before GPU work. Apply explicit outer byte/count/metadata limits before large allocation, plus all current Core resource/graph limits. Define source, temporary transfer and final owned-snapshot memory accounting on Native and browser; no unlimited whole-file JSON parse or decompression before limits.
3. **Path safety:** runtime loaders consume bytes; they never search a filesystem, extract entries to disk or download URLs. If an archive candidate has entry names, specify one normalized namespace and reject absolute paths, drive/UNC names, backslashes, `.`/`..`, aliases, duplicate names, links and unexpected entries. CLI authoring paths are caller-controlled inputs, never trusted runtime paths embedded in the asset. No symlink-based escape or implicit adjacent-file discovery.
4. **One implementation owner:** Rust owns package decoding/validation shared by Native and WASM; typed adapters remain thin. Existing Core owns graph semantics, preparation and resource identity; existing wgpu owns all pixel execution. M6B-02 must select the codec's owning module and document the boundary in an ADR before implementation. A new crate needs an actual dependency/compilation boundary; neither a new crate nor putting archive handling in Core is pre-authorized here.
5. **Capture and lifecycle:** preserve synchronous snapshot and mutation/destroy/busy guarantees. A loader cannot create a late mutable resource-ID binding or retain a hidden global asset cache. Document who owns the source/package bytes and each decoded snapshot, allocation failure behavior and release on every error path.
6. **Compatibility:** keep loose `.mix` plus caller-supplied bindings working. Treat the outer package version independently from document/node/plan/API versions. Identical unpacked inputs must produce the same plan identity and pixels as the loose public path. Decide public API/package version changes explicitly in M6B-02; do not reserve 0.4 while a separate maintenance PR owns that candidate.
7. **Request behavior:** define complete resource closure, missing/unused/unknown IDs and resource-ID override behavior against the existing M6-A slicing rules. Do not duplicate Core traversal in JS or hide newly ambiguous override semantics. Scalar overrides and requested channels remain caller options, not serialized editor state.

## Work items and exit gates

| Item | Deliverable | Required exit evidence |
|---|---|---|
| **M6B-01 — entry and scope (this PR)** | Reproducible missing-delivery evidence, single-asset outcome, constraints and sequence | Probe assertions, paired documentation, links, repository check; no runtime acceptance claim |
| **M6B-02 — format and ownership** | Compare inspectable JSON/base64, a manifest plus stored standard archive, and a directory baseline; select the smallest sufficient format and write ADR/byte-level contract | Measured byte size, bounded decode/copy/allocation strategy at normal and maximum resource budgets; exact version/digest/error/override rules; dependency and crate-boundary review; malicious-input fixtures. A custom binary format requires additional measured justification |
| **M6B-03 — shared CPU loader/writer** | Deterministic authoring and bounded typed loading through a public Rust API | Repeatable package bytes, round trip, exact source/resource preservation, mutation/lifetime checks; negative corpus for every limit, duplicate, version, hash, truncation and path rule; independent Native public consumer |
| **M6B-04 — CLI and browser adapters** | Explicit local-file CLI workflow and public browser byte-loading workflow using the same codec | CPU-only inspection without GPU acquisition; untrusted object/byte-view checks, offset/length, ownership and structured errors; normal bundle remains inert; no private imports or network resolution |
| **M6B-05 — qualification and closeout** | Exact candidate package and offline moved-asset consumption on Native/browser | Loose-vs-packaged plan/pixel identity, four weights, rectangular control, cleanup/repeated loads, rejection then recovery, isolated archive consumer, existing three-material gates and all six required CI checks; retained receipts and exact platform limits |

The implementation PRs stay independently reviewable; work-item IDs are not GitHub PR numbers. M6B-01 completion starts the phase but does not close M6-B. Source/contract acceptance, implementation, qualification and publication are distinct. Release publication and Studio upgrades remain separate tasks.

## Stop and scope rules

Stop format selection if the measured problem can be met more simply, ownership would violate the architecture, or bounded memory cannot be demonstrated. Do not raise existing budgets to admit an inefficient container without evidence. No compression/encryption/signatures, remote downloader, resource cache/deduplication, streaming subsystem, multi-material library, `.mix` graph migration, PNG/JPEG/HDR codec expansion, engine-specific export, Studio changes, new renderer or publication is included. Compression and additional formats require their own measured case. Preserve all current checks, goldens, numerical gates and historical hardware limitations.

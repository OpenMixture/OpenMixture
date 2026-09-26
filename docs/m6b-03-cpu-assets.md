# M6B-03 — shared CPU asset codec

English | [简体中文](./m6b-03-cpu-assets.zh-CN.md)

**Current status:** this implementation is merged into main with M6-B; M6B-05 has completed [comprehensive qualification](./m6b-05-qualification.md) within its recorded scope. Follow-up tasks mentioned in this implementation-slice account were the plan at that checkpoint; see [release status](./release.md) for versions and hardware scope.

The optional [mixture-asset crate](../crates/mixture-asset/README.md) implements the [M6B-02 byte contract](./m6b-package-format.md). Source and isolated Cargo consumers can write, validate, inspect and prepare `.mixpack v1` without a GPU. Rust source is 0.5.0; the browser build identity advances to unpublished 0.5.0-alpha.0, while the registry consumer stays pinned to published 0.3.0-alpha.0. CLI/browser package APIs and package pixel qualification remain M6B-04/05. No release or new hardware numerical guarantee is claimed.

## Public operations and ownership

| API | Ownership and validation |
|---|---|
| `write(source, bindings, &limits)` | Borrows exact source/pixels; validates full closure and metadata, recomputes identities, reserves final archive once and returns deterministic bytes |
| `AssetView::load(bytes, &limits)` | Borrows archive, validates strict headers/manifest, all digests and source/closure; keeps ranges into the original allocation |
| `OwnedAsset::from_vec(bytes, &limits)` | Moves archive allocation; checks and charges its actual capacity, including spare capacity |
| `OwnedAsset::copy_from(bytes, &limits)` | Validates and checks budget before one fallible archive reservation/copy |
| `view.source/resources/image/bindings` | Read-only source, metadata and borrowed payload access |
| `view.inspect()` | Schema 1 transport/source identities and sorted resource metadata, separate from plan hash |
| `view.preparation_buffer_bytes(&request)` | Validates request and reports retained archive capacity + D + M + selected snapshot bytes |
| `view.prepare(&request)` / `owned.prepare(&request)` | Applies intersected policy, rejects resourceRef overrides, delegates slicing/capture/plan to Core; snapshots outlive the asset |

Core exports `resources::{valid_image_id, image_identity, document_image_ids, image_references}`. Shared traversal selects the same nodes for inventory and compilation. The codec contains no second graph traversal or resource digest algorithm. Core metadata validation and capture share one implementation; no behavior moves into generated bindings. The new crate depends only on Core and existing serde/serde_json/sha2. Neither Core nor wgpu depends on it. Native consumer and archive verification include it; CLI/WASM do not yet depend on it.

## Error and memory contract

`AssetError::Package` provides a serialized diagnostic with a typed `PackageCode` (stable `MIX_PACKAGE_*` spelling), stage, `error` severity, message, evidence and suggestion; CLI and WASM serialize it directly rather than rebuilding the fields. `Document` and `Compile` variants retain the original Core error/report and source chain. Resource-reference overrides are rejected even when equal to the default and outside the output slice, after Core validates override IDs/types. Other overrides keep Core behavior. No catch-all fallback changes semantics.

All source/resource entries are checked, including disconnected images. Resource tables are bounded to eight before indexing; only exact ordinal names are accepted. Strict typed JSON rejects duplicate/unknown fields, floats/exponents for integers, signed zero, overflow and BOM. Canonical header equality also rejects extension fields, links, traversal paths, nonzero padding and alternate octal encodings. Exactly two terminal blocks and EOF are required. Every digest is recomputed, and source bytes are never normalized for writing.

Buffer accounting uses D+M scratch conservatively even when borrowed slices avoid copies. Borrowed preparation charges D+M+S; owning preparation adds actual retained Vec capacity. Borrowed writing charges P+D+M. Large buffers reserve their checked size once using `try_reserve_exact`; failed reservations are structured. Core owns selected pixel capture; typed manifest/graph allocations remain bounded separately. No global allocator instrumentation or unsafe code is introduced. Pointer identity, actual archive capacity, snapshot sizes and exact-budget rejection instrument the real Rust paths in [memory tests](../crates/mixture-asset/tests/memory.rs). Multiple concurrent assets, caller-owned input and outputs must be counted by the host. Browser double-copy accounting is still M6B-04.

## Reproduce

[Local verification receipt](./evidence/m6b-03/README.md) records the exact tested commit, commands, actual buffer measurements and isolated archive provenance.

```sh
cargo test --locked -p mixture-asset -- --nocapture
cargo xtask test-core
cargo xtask test-consumer
cargo xtask package-check
cargo check --locked -p mixture-asset --target wasm32-unknown-unknown
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/consumer.test.mjs
tar -tf fixtures/packages/mixpack-v1/valid.mixpack
cargo xtask check
```

[Codec tests](../crates/mixture-asset/tests/codec.rs) consume every committed corpus case, reproduce the independently generated archive exactly, reject every truncated prefix and single-byte mutation of all header positions, compare loose/package plan hashes at all four weights, and cover moved/copied/borrowed lifetimes. Package-local unit tests cover strict JSON, zero/exact/over limits and original Core errors. The [external Native test](../examples/native-consumer/tests/assets.rs) uses only public APIs and its own source/pixels, also against isolated extracted archives. Memory tests exercise 4 MiB and 64 MiB resource payloads and fail a preparation budget lowered by one byte. These are CPU/ownership claims, not package GPU/browser acceptance.

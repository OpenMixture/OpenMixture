# M6B-02 — `.mixpack v1` format, ownership and budgets

English | [简体中文](./m6b-package-format.zh-CN.md)

**Implementation status:** M6B-03–05 implement this contract and complete bounded qualification; see [asset qualification](./m6b-05-qualification.md). The version-selection procedure below is M6B-02 design history; current source and publication status are in the [release guide](./release.md).

**M6B-03 implementation update:** the [shared Rust CPU codec](./m6b-03-cpu-assets.md) is implemented. The following is the selected M6B-02 byte/ownership contract; CLI/browser adapters and combined qualification remain M6B-04/05. The design PR changed no code; the implementation PR introduces the crate and 0.5 version.

## Selection and evidence

Use an **uncompressed POSIX USTAR archive**, extension `.mixpack`, with a readable UTF-8 JSON manifest, exact `.mix` bytes and packed linear RGBA8 bytes. This is a restricted standard archive, not a custom binary graph format. Existing tar tools can list/read the entries; arbitrary tar files are not necessarily valid assets. The profile deliberately excludes extensions and extraction. Reference: [standard tar layout](https://www.gnu.org/s/tar/manual/html_node/Standard.html).

The [reproducible experiment](../scripts/measure-package-formats.py) and [measurements](./evidence/m6b-02/measurement.json) compare identical content, including the same manifest. The maximum-resource case uses eight 2048×1024 images (64 MiB), within current count/pixel/axis limits. These source sizes differ from M6B-01 because the experiment serializes explicit multi-resource documents.

| Representation | One 1K image | Eight images / 64 MiB | Assessment |
|---|---:|---:|---|
| Directory plus manifest | 4,195,771 bytes / 3 files | 67,112,543 bytes / 10 files | Minimal loose baseline; not one offline artifact |
| JSON with base64 entry bytes | 5,594,488 | 89,483,790 | Readable outer structure, but substantial expansion and decoded-string/copy costs |
| ZIP stored | 4,196,103 | 67,113,631 | Compact standard option; local/central metadata agreement and optional ZIP features add validation surface |
| **USTAR profile** | **4,198,912** | **67,119,104** | Selected; only 2,809 / 5,473 bytes above ZIP stored, sequential fixed headers and raw entry slices |

ZIP comparison follows the [PKWARE specification](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT); no compression advantage is claimed. Python stdlib decode measurements retain wall time and `tracemalloc` peaks. On the measured maximum case JSON requires roughly 179 MB of traced decoder allocations versus roughly 67 MB for both archives. These are single-run Python observations, not Rust/WASM latency or heap guarantees. No custom binary format, compression dependency or performance promise is justified.

## Exact archive profile

All numbers below describe **v1 acceptance**, not general tar support. Read checked byte offsets directly over a bounded immutable byte slice. Never extract to disk. Every 512-byte header must equal the canonical USTAR header for its accepted name and length:

- Names are ASCII and NUL-padded in the 100-byte name field. Exact order: `manifest.json`, `material.mix`, then `images/0000.rgba` through `images/0007.rgba` for manifest resources in ascending ASCII ID order. No directory entries. Ordinal paths avoid filesystem case aliases while resource IDs remain case-sensitive.
- Mode is `0000644\0`; uid/gid are `0000000\0`; size and mtime are 11 octal digits plus NUL, mtime all zero. Typeflag is ASCII `0`. Linkname, uname, gname, device numbers, prefix and unused bytes are all zero. Magic is `ustar\0`, version `00`.
- Header checksum is the unsigned sum of all 512 bytes with the checksum field treated as eight ASCII spaces; encode as six octal digits, NUL, space. Reject base-256, negative, overflowing or noncanonical numeric fields. All arithmetic and host-size conversion are checked.
- Entry bytes follow the header, followed by zero bytes to the next 512-byte boundary. After the final entry require **exactly two zero blocks (1024 bytes)** and EOF. Reject extra record padding, concatenated archives, missing end blocks, trailing data and nonzero padding. Writers do not add tar's optional larger blocking padding.
- Reject every other name/order/type/header: absolute/drive/UNC paths, backslashes, `.`/`..`, prefix aliases, duplicate entries, links, sparse/PAX/GNU records, devices, directories, timestamps, extended names, compression and ZIP signatures. Rebuilding the expected canonical header and comparing bytes is sufficient after bounded size/name decoding; do not implement a general tar parser.

The profile can be implemented with checked slices and standard Rust arithmetic. No tar/ZIP dependency is selected. The reviewed [tar-rs manifest](https://raw.githubusercontent.com/alexcrichton/tar-rs/main/Cargo.toml) includes file metadata dependencies and general archive facilities unnecessary for this strict byte-only subset. This is not a claim that tar-rs is unsafe. M6B-03 must cross-read valid fixtures with an independent standard tool and reject the negative corpus; any need for broader archive support reopens this decision instead of silently expanding the parser.

## Manifest and identity

Exact required object shape (example values abbreviated only for digests):

```json
{"format":"openmixture-asset","version":1,"document":{"path":"material.mix","byteLength":1060,"sha256":"<64 lowercase hex>"},"resources":[{"id":"heightSource","path":"images/0000.rgba","width":1024,"height":1024,"format":"rgba8-linear","bytesPerRow":4096,"byteLength":4194304,"contentDigest":"<64 lowercase hex>"}]}
```

Reject duplicate keys at every nesting level, unknown/missing fields, non-UTF-8/BOM, non-integer numeric tokens (including exponent/float forms), negative values, overflow, invalid IDs and non-lowercase/non-64-character digests. Width/height are positive u32; other numeric metadata is u64 except version (u32). JSON whitespace and object-key order may vary within the manifest byte cap; array order must be strictly ascending unique resource ID. Writer emits compact JSON, keys in the order above, ASCII strings, shortest decimal integers and no final newline. Exact source bytes are never reserialized or migrated by the writer.

`byteLength` must equal the entry length. Resource format is only `rgba8-linear`; `bytesPerRow=4*width`, length `4*width*height`. No PNG decoding, alpha/gamma conversion or resampling. Document digest is SHA-256 of exact source bytes. Resource `contentDigest` uses the existing M6-A domain `mixture-image-rgba8-linear-v1\0`, little-endian u32 dimensions and packed RGBA bytes; Core must expose/reuse its own checked digest/metadata helper, not a second semantic implementation in the package crate. Validate every resource, including unused branches. Manifest hashes are recomputed, never trusted. These hashes detect corruption, not author authenticity.

Package SHA-256, when reported, hashes the entire archive and stays outside the plan. Changing manifest whitespace can change package identity while preserving source/resource/plan identity. No cached executable plan, requests, shader binaries, editor state, defaults overrides, runtime version pins or engine-export settings are stored.

## Closure and overrides

The package contains exactly the set of IDs referenced by all image nodes in the validated source at document defaults, including disconnected branches; no missing or extra entries. Core provides the validated reference inventory so the codec does not duplicate traversal/catalog semantics. Zero-image documents have an empty resource array and exactly two entries. All image dimensions must agree; the caller's output size must match them when preparing, as in the loose API. Resource-free packages retain normal size freedom.

Package v1 rejects **every supplied override targeting a `resourceRef` parameter**, even one equal to its default, with `MIX_PACKAGE_RESOURCE_OVERRIDE`. Numeric/enum/color overrides retain Core behavior. This explicit restriction avoids an ambiguous bank of inactive alternative resources; callers needing resource-ID replacement use the existing loose APIs. It does not restrict those APIs. Validate override targets/types through Core; unknown/invalid overrides keep their original Core diagnostics. No override repairs, source rewrite or automatic node-version upgrade.

For allowed requests, provide the complete validated binding table to existing `prepare`; Core applies current slicing, validates supplied-but-unused bindings, hashes/captures only selected resources and retains its existing missing/unknown/size rules. The package and loose source plus the same full binding table must have identical plan hashes, allocations and per-backend pixels. Package closure is stricter than loose optional bindings and is checked before channel slicing.

## Rust ownership and public seam

**M6B-03 introduces `mixture-asset`**, as documented in the implementation guide above. This is an optional public CPU codec/API dependency boundary: callers of `mixture-core` need not compile or trust an archive decoder. Both Native consumers and `mixture-wasm` need the same codec; it cannot live in CLI, JS or generated bindings. Dependencies: `mixture-asset -> mixture-core`, existing workspace serde/serde_json/sha2; M6B-04 plans `mixture-cli` and `mixture-wasm -> mixture-asset`. No edge from Core or wgpu to asset. No filesystem, browser, GPU, async worker or process-global state in this crate. Core exposes only reusable resource metadata/digest/reference helpers; package codes and archive policy stay out of Core.

API roles (now callable in Rust): immutable borrowed `AssetView<'a>` validates archive bytes and exposes read-only entry slices; an owned asset retains one moved or checked-copy buffer. Inspection validates all integrity/closure without GPU. Preparation calls Core synchronously and returns the existing `PreparedRender`, which owns selected snapshots independently of the view. Rust borrowing prevents mutation during inspection/preparation. A borrowed view cannot outlive its source. Native writer accepts validated source/bindings, hashes inputs, reserves the checked final size once and writes deterministic bytes; no growth-doubling buffers or filesystem traversal. Writer and loader must share the manifest model, not graph execution.

Browser adapters snapshot an accepted Uint8Array and options synchronously before any await, reject shared/resizable/detached/accessor-backed inputs and invalid offset/length without coercion, and transfer to owned Rust bytes. Busy/closing rejection precedes copying. Conservatively budget both JS and Rust package copies simultaneously with Core's selected snapshot; do not depend on prompt JS GC. Release package transfer buffers after preparation and before GPU execution; returned `PreparedRender`/output lifetime and destruction continue under existing contracts. Browser authoring UI and arbitrary file extraction are out of scope.

## Explicit limits and allocation sequence

`PackageLimits` is a separate policy; browser u64 fields use bigint. Omission chooses defaults below; zero is a valid lower ceiling, equality passes, no implicit raising. These v1 maxima are hard format-profile ceilings; policies may lower them, not exceed them. Existing `SafetyLimits` and `ResourceLimits` apply additionally using the stricter limit. Raising a v1 profile ceiling requires a deliberate compatibility decision.

| Limit | Default / v1 ceiling | Checked before |
|---|---:|---|
| `packageBytes` | 67 MiB = 70,254,592 | Any owned package copy; byte length known from input |
| `manifestBytes` | 64 KiB = 65,536 | Manifest parsing/copy |
| Source bytes | 2 MiB = 2,097,152, plus Core `decodedBytes` | Source parsing/copy |
| Resource count / archive entries | 8 / 10 | Building entry/index tables |
| Image axis / aggregate pixels | 2048 / 16,777,216 | Multiplication, hashing and resource capture |
| Packed resource bytes | 64 MiB = 67,108,864 | Hashing/copying any payload |
| `packageBufferBytes` | 202 MiB = 211,812,352 | Any operation's captured/transfer/output-buffer reservations |

Order: validate policy and input length; read first header and bounded manifest; validate version/fields/counts/metadata/budgets; walk exact headers/ranges with checked offsets and zero padding; verify hashes and source with Core; check closure; validate request/override restrictions and operation-specific buffer budget; then capture/prepare or write. Browser must charge its preliminary package snapshots from known input length before copying; later metadata may still reject. No decompression, archive expansion or unbounded entry indexing occurs.

Let `P` = actual package bytes, `D` = source bytes, `M` = manifest bytes, `R` = all packed resource bytes, `S<=R` = selected Core snapshots. Reserve `D+M` conservatively for byte scratch even when slices avoid copies. Native borrowed preparation ≤ `D+M+S`; owned preparation/CLI writer ≤ `P+D+M+R`; browser accepted preparation ≤ **`2P+D+M+R`**. Native byte writer from caller-borrowed inputs reserves `P+D+M`. Sum every simultaneously owned asset/view, and charge writer output separately if retaining an input asset. Refuse over-budget allocations before reserving. Only one prepare per browser runtime follows existing busy semantics; callers may explicitly own multiple independent assets, with no global cap promised.

Measured normal payloads give browser ledger 12,593,595 bytes; maximum-resource fixture 201,350,751 bytes. At all independent ceilings the conservative expression is 209,780,736 bytes (200.0625 MiB), below 202 MiB. Archive framing fits under 67 MiB even with maximum source/manifest/resources. These are **byte-buffer bounds**, not process RSS: caller-owned input, parsed typed graph/manifest objects, allocator overhead, JS garbage and retained output pixels are separate. Typed objects remain bounded by manifest/source/graph limits; allocation failure must be structured where allocation is fallible. M6B-03/04 must instrument the actual Rust/WASM copy path and reject an extra full package copy; Python peaks do not substitute for that acceptance. Existing GPU 512 MiB transient accounting is unchanged.

## Errors, versions and acceptance

`AssetError` owns package errors with stable `code`, `stage="package"`, message, optional entry/offset/resource ID, measured/configured evidence and suggestion. Reserved v1 codes: `MIX_PACKAGE_INVALID` (framing/manifest/path/closure), `MIX_PACKAGE_UNSUPPORTED_VERSION`, `MIX_PACKAGE_LIMIT_EXCEEDED`, `MIX_PACKAGE_CONTENT_MISMATCH`, `MIX_PACKAGE_RESOURCE_OVERRIDE`, `MIX_PACKAGE_ALLOCATION_FAILED`. Preserve original Core errors/stages and GPU errors when delegation fails; never replace them with a generic package wrapper. Reject in the sequence above, entry order then lexical ID; retain the first decisive error. CLI I/O failures keep adapter ownership and the existing I/O exit category.

Outer package version 1; `.mix v1`, explicit node versions, plan v2 and API schema 2 stay unchanged. Package inspection uses its own report schema 1. Existing browser APIs remain available; new package APIs are additive. The implementation target is the next unpublished minor after both 0.3 and the separately owned 0.4 maintenance candidate: **Rust 0.5.0 / browser 0.5.0-alpha.0**. M6B-03 must verify remote/source version availability before applying the bump; collision requires a documented next-minor adjustment, never reused bytes/version. No publication is authorized here.

[Committed binary fixtures](../fixtures/packages/mixpack-v1/cases.json) include one valid standard-readable asset and malformed/version/hash/path/size/duplicate cases. These expected codes now execute as M6B-03 Rust regressions, covering all corpus entries, source/hash mismatch, zero/exact/over limits, full closure, every header field, overflow/truncation at every boundary, borrowed/owned lifetimes, determinism and independent tar reading. M6B-04 covers browser copy limits, offset views, mutation and busy/destroy/rejection. M6B-05 compares loose/package plan and pixels, all four weights, 65×3, packaged Native/browser consumers, existing materials and all six CI checks. Preserve v1/noise history; package acceptance is not broader numerical support.

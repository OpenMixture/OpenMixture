# ADR 0008: Optional byte-only portable assets

English | [简体中文](./0008-portable-assets.zh-CN.md)

Status: selected for M6B-02 review, 2026-09-22; accepted on integration. Runtime implementation and qualification remain M6B-03–05.

## Context and decision

M6B-01 demonstrates that a graph and adjacent pixels do not carry the binding needed for offline delivery. Adopt the [exact `.mixpack v1` contract](../m6b-package-format.md): uncompressed canonical USTAR, manifest first, exact source bytes and linear RGBA8 resources, no disk extraction, network or optional archive features. Measured normal/maximum content makes ZIP's small size saving insufficient to justify its larger validation surface; JSON/base64 adds substantial size and temporary allocation. This is a standard archive subset, not a new binary graph representation.

Authorize a real optional public CPU codec boundary, `mixture-asset`, in M6B-03. It depends on Core and existing serialization/hash dependencies; Native, CLI and WASM consume it. Core and wgpu never depend on it. Native source-only consumers retain their current trust/dependency boundary. Checked canonical header parsing needs no archive/compression dependency. Core continues to own resource semantics and exposes shared metadata/digest/reference helpers; no graph logic moves to the codec or JS.

## Alternatives

A directory manifest cannot satisfy the selected single-artifact handoff. JSON/base64 is simpler structurally but expands maximum content by about 22 MB and creates extra decoded representations. ZIP stored is interoperable but requires additional central/local metadata agreement. General tar libraries support file/extraction metadata and extensions outside this byte-only profile. Putting the codec in CLI prevents public Native/WASM reuse; putting it in Core expands every compiler consumer's archive trust surface. A custom binary format and compression are unnecessary for this measured problem.

## Consequences, compatibility and migration

The new crate is justified by the optional compilation/public-API boundary, not file count. Narrowly extend the architecture to permit producer-owned portable input assets outside Core; Core still owns no ZIP/export/editor packaging. `.mix` stays the exact source of truth, loose resource APIs remain valid, and no pixels or plan identities change for equivalent inputs. Restrict package v1 resource-ID overrides explicitly while preserving existing loose APIs. The separate package/profile limits and conservative browser buffer ledger are defined in the contract; they are not total RSS or a relaxed GPU budget.

Outer package version 1 and inspection schema 1 are independent of unchanged `.mix`/plan/API versions. Target the next unpublished minor after the 0.4 maintenance candidate (currently 0.5), checking version availability before implementation. No source migration, new node, crate stub, lockfile update or publication occurs in this design PR.

## Verification

Retain reproducible directory/JSON/ZIP/USTAR size and Python allocation experiments, a standard-readable canonical fixture and malformed-input corpus, paired docs and `cargo xtask check`. These verify the design artifacts, not a nonexistent Rust loader. M6B-03 must prove bounded allocation, malicious-input rejection, deterministic writing, exact source/resource round trips and isolated public consumption. M6B-04/05 add browser lifecycle/copy instrumentation, loose-versus-packaged plan/pixel equivalence and unchanged required CI/material gates. A requirement for compression, generic extraction or broader archive compatibility requires a new scoped decision.

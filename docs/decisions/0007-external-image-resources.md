# ADR 0007: Caller-owned external image input

English | [简体中文](./0007-external-image-resources.zh-CN.md)

Status: selected design for M6A-01 review, 2026-09-21; accepted when this change integrates. Implementation and GPU acceptance remain pending.

## Context

The published 0.2.0-alpha.0 runtime combines procedural Scalar fields but cannot consume external pixels. Resource input needs an explicit request, ownership, identity and budget contract. ADR 0003's initial separation of resources excluded resource delivery from the document; M6-A now needs a logical reference without embedding delivery details.

## Decision

Adopt the [minimal resource contract](../m6a-resource-contract.md). Extend the node catalog with only `image-input@1` and a real `resourceRef` parameter kind inside the existing `.mix v1` parameters object. Caller-supplied, same-size, tightly packed linear RGBA8 pixels provide normalized R height through one wgpu WGSL kernel. Core owns validation, immutable snapshots, digest-bound plans and explicit budgets; wgpu owns upload/execution/lifetime; bindings only adapt representations. Files, decoding, networking and permissions remain caller-owned.

This narrowly supersedes ADR 0003's exclusion of logical resource references from material semantics. It does not allow embedded bytes, resource locations, top-level manifests or editor state. No invariant, dependency direction, crate boundary or sole-executor rule changes. Explicitly advance the implementation's plan/hash contract to v2, browser API schema to 2 and pre-1.0 package minor version; retain old document/pixel semantics and separate distribution decisions.

## Alternatives

Top-level resource fields or `.mix v2` are unnecessary for one node parameter. Fake enum values cannot describe caller-defined resource IDs. Hashing only IDs permits stale content reuse. Late mutable bindings create capture/identity ambiguity. Implicit scaling, color conversion, downloads or a packaging container add independent semantics and security boundaries. Keeping every old plan hash while introducing a second conditional resource plan model adds complexity; an explicit global v2 cache invalidation is chosen instead.

## Consequences and migration

Owned input copies make asynchronous behavior deterministic at bounded memory cost. Same-size inputs avoid sampling policy and multi-resolution propagation. Exact byte identity hashes ignored channels conservatively. Existing documents need no migration, but downstream code must handle the new exhaustive variants/catalog kind, schema 2 and changed plan hashes. Keep v1 receipts historical and reproduce new v2 snapshots independently. Old runtimes reject unknown nodes. The current published package remains unchanged, and this decision makes no new platform or pixel-support claim.

## Verification

M6A-01 runs paired-document/link checks and `cargo xtask check`. Later tasks execute the contract's known-value, resource budget/identity, capture/lifecycle, Native/browser and existing material gates, using public consumers. Keep all six required CI checks. Contract acceptance is distinct from implementation, pixel qualification and publication; no shader, fixture baseline or package changes belong to this design PR.

## M6A-05 scope addendum — 2026-09-21

The [contract scope decision](../m6a-resource-contract.md#m6a-05-acceptance-scope-decision--2026-09-21) bounds pixel acceptance to the measured Linux software matrix. Retained Windows failure evidence prevents a hardware parity claim; ≤1, existing shader semantics and all regression gates remain unchanged. This selects bounded qualification over an unreviewed numerical semantic change or fitting a higher tolerance. Broader hardware acceptance and publication remain separate decisions.

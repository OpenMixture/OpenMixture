# ADR 0010: Caller-supplied recipes expanded by Core

English | [简体中文](./0010-reusable-recipes.zh-CN.md)

Status: proposed for MAT-04a review, 2026-10-12. Accepted on integration of the [MAT-04a contract](../mat-04a-recipe-contract.md). Implementation and qualification remain MAT-04b–d. Refines [ADR 0003](./0003-readable-mix-dag.md), whose "one readable DAG" decision stays true for `.mix v1`.

## Context

The [MAT-04 entry](../mat-04-reusable-recipes.md) measures duplication in accepted materials: brick and woven each repeat seven controls across three pattern nodes (21 duplicated bindings each) and keep them equal in caller code, and a masked two-layer surface is rebuilt by hand in three families. `.mix v1` binds each public ID to one node parameter and has no reusable fragment. ADR 0003 deferred subgraphs because they add compilation and migration complexity before consumer evidence existed; that evidence now exists. The maintainer selected separate caller-supplied recipe documents, loose input only and no nesting.

## Decision

- A **recipe** is a separate strict JSON document (`id@recipeVersion`) with a built-in-only graph fragment and a typed interface of inputs, outputs and direct-assignment parameters that may fan out to several targets.
- A **`.mix` v2** parent adds an `instances` array. v1 is unchanged and still the only version accepted by existing entry points and by `.mixpack v1`.
- **Core expands** instances into ordinary nodes, with IDs `<instance>--<node>`, before normalization. RenderPlan v3, lowering, wgpu kernels and the node catalog are unchanged. Parents' plans record `documentVersion: 2`.
- The caller supplies the **exact closure** as bytes; Core rejects missing, duplicate and unused recipes and never resolves paths or URLs. Recipe digests are reported for provenance, not pinned.
- Every no-default parameter (seeds) inside a recipe must be interface-bound and set by each instance; nothing is derived.

## Alternatives

- **Inline definitions in the parent.** Simplest closure, but no reuse across files, which is the stated MAT-04 outcome. Deferred.
- **Expansion in wgpu, WASM or JS.** Would duplicate graph semantics outside Core and violate the ownership boundary. Rejected.
- **Recipes as compiled plan fragments.** Would need a new plan version and executor semantics for no measured benefit. Rejected.
- **Make parents hash-identical to hand-written v1 graphs.** Requires renaming expanded IDs to match arbitrary author IDs or excluding IDs from the hash, changing plan v3 semantics. Rejected; byte-identical pixels are the equivalence gate.
- **Fan-out directly on v1 exposed parameters.** Reinterprets a v1 field and breaks v1 uniqueness. Rejected.
- **Nesting and package transport now.** No accepted material needs them; each widens closure and limit rules. Deferred by maintainer direction.

## Consequences, compatibility and migration

Existing documents, packages, plans, hashes, goldens and receipts keep their meaning. Old runtimes reject v2 parents with `MIX_FORMAT_UNSUPPORTED_VERSION` and do not know `.mixrecipe` documents. A parent and the v1 graph it replaces may differ in pass order, plan hash and allocation estimates while producing identical pixels; frozen budgets apply to the parent. Core gains public recipe types, `RecipeLimits` and eight diagnostic codes, so the first implementation selects Rust 0.11.0 / browser 0.11.0-alpha.0. No automatic v1→v2 conversion exists; authors opt in by writing a v2 parent.

## Verification

The contract's frozen gates: CPU structural equivalence against the accepted brick and woven graphs, byte-identical pixels on the recorded adapters for both frozen matrices, frozen budgets, control mapping, instance isolation, a negative corpus for every rule, and unchanged v1 fixtures, hash snapshots and required checks.

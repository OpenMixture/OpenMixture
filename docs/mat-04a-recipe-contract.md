# MAT-04a — Reusable recipe contract

English | [简体中文](./mat-04a-recipe-contract.zh-CN.md)

**Status:** MAT-04 is accepted within its [recorded scope](./evidence/mat-04/README.md) (2026-10-13). This contract was accepted on 2026-10-12 and integrated through PR #99 together with [ADR 0010](./decisions/0010-reusable-recipes.md). MAT-04b implements it; implementation, qualification and publication remain separate states. The contract PR changed no code, fixture, golden, schema or version. It answers the questions in the [MAT-04 entry](./mat-04-reusable-recipes.md) within the maintainer's 2026-10-12 direction: separate caller-supplied recipe documents, loose input only, no nesting.

## Summary

A **recipe** is a separate, versioned JSON document holding a graph fragment with a typed interface. A new **`.mix` v2** parent document places recipe **instances** beside ordinary nodes. Core validates the parent against an explicit, caller-supplied recipe closure and **expands** every instance into ordinary built-in nodes before normalization. Everything after expansion — normalization, slicing, lowering, RenderPlan v3, wgpu kernels — is unchanged. `.mix v1`, `.mixpack v1`, existing entry points and every accepted pixel keep their exact meaning.

## Recipe document

UTF-8 JSON, conventional extension `.mixrecipe`, decoded with the same strictness as `.mix` (unknown or duplicate fields, comments, non-integer versions, NaN and trailing data are rejected).

```json
{
  "version": 1,
  "id": "masked-layer-surface",
  "recipeVersion": 1,
  "nodes": [
    { "id": "underColor", "type": "constant-color", "version": 1, "parameters": { "value": [0.16, 0.15, 0.13, 1] } },
    { "id": "underRoughness", "type": "constant-scalar", "version": 1, "parameters": { "value": 0.9 } },
    { "id": "color", "type": "blend", "version": 1 },
    { "id": "roughness", "type": "scalar-mask-blend", "version": 1 }
  ],
  "edges": [
    { "from": { "nodeId": "underColor", "portId": "color" }, "to": { "nodeId": "color", "portId": "a" } },
    { "from": { "nodeId": "underRoughness", "portId": "value" }, "to": { "nodeId": "roughness", "portId": "a" } }
  ],
  "inputs": [
    { "id": "mask", "targets": [ { "nodeId": "color", "portId": "mask" }, { "nodeId": "roughness", "portId": "mask" } ] },
    { "id": "overColor", "targets": [ { "nodeId": "color", "portId": "b" } ] },
    { "id": "overRoughness", "targets": [ { "nodeId": "roughness", "portId": "b" } ] }
  ],
  "outputs": [
    { "id": "baseColor", "from": { "nodeId": "color", "portId": "color" } },
    { "id": "roughness", "from": { "nodeId": "roughness", "portId": "value" } }
  ],
  "parameters": [
    { "id": "underColor", "targets": [ { "nodeId": "underColor", "parameterId": "value" } ] },
    { "id": "underRoughness", "targets": [ { "nodeId": "underRoughness", "parameterId": "value" } ] }
  ]
}
```

| Field | Rule |
|---|---|
| `version` | Recipe document format version; only `1`. Others fail with `MIX_FORMAT_UNSUPPORTED_VERSION`. |
| `id` | Recipe identity, `[a-z][a-z0-9-]{0,63}`. Independent from built-in node type IDs. |
| `recipeVersion` | Unsigned 32-bit integer ≥ 1. Identity is the pair `id@recipeVersion`. |
| `nodes`, `edges` | Same shape and validation as `.mix v1` nodes and edges, built-in catalog only. `material-output` and `image-input` are forbidden (`MIX_RECIPE_FORBIDDEN_NODE`). There is no `instances` field, so a recipe cannot nest another recipe; an `instances` key fails as an unknown field. |
| `inputs` | Interface input ports. Each names one or more internal **unconnected** input ports. All targets must have the same port kind, which becomes the interface kind. The input is **required** if any target port has no contract default; otherwise it is optional and, when the parent leaves it unconnected, every target keeps its own contract default. |
| `outputs` | Interface output ports. Each names exactly one internal output port; two outputs may not alias the same port. |
| `parameters` | Interface parameters. Each names one or more mutable internal parameters with **identical** parameter contracts (type, range, enum values). See mapping rules below. |

Interface IDs use the node-ID pattern; inputs, outputs and parameters are three separate namespaces, each unique. An internal input port is either connected by an internal edge or targeted by one interface input, never both and never twice. An internal parameter is targeted by at most one interface parameter, once. Interface errors fail with `MIX_RECIPE_INVALID_INTERFACE` and name the recipe, interface ID and offending target.

A recipe is validated on its own: nodes, versions, parameters, edges, acyclicity and port kinds follow v1 rules, except that targeted required inputs need no internal connection and targeted required parameters need no source value.

## Parameter mapping and defaults

- An interface parameter assigns its value **directly** to every target. No arithmetic, scaling or expressions.
- **Default:** if every target has the same effective value in the recipe source (explicit value or contract default), that value is the interface default. If **no** target has a value (a contract parameter without default, such as a seed), the interface parameter is **required**. Mixed or unequal values are an interface error.
- Recipe parameters not targeted by the interface are fixed by the recipe bytes; a parent cannot read or set them.
- Cross-parameter relations of each node contract are checked on the expanded graph, after instance values and overrides are applied. Diagnostics carry the instance ID, interface parameter and expanded node ID.

## Seeds

Every built-in parameter without a contract default — today the explicit seeds — **must** be targeted by an interface parameter, and that interface parameter is required: each instance sets it explicitly in the parent. Two instances of one recipe therefore never share a seed by accident; sharing is an explicit equal value. No seed is derived from instance IDs, positions or hashes, so no existing pixel can change through derivation.

## `.mix` v2 parent document

```json
{
  "version": 2,
  "nodes": [ { "id": "out", "type": "material-output", "version": 1 } ],
  "instances": [
    { "id": "surface", "recipe": "masked-layer-surface", "recipeVersion": 1,
      "parameters": { "underColor": [0.015, 0.012, 0.01, 1], "underRoughness": 0.95 } }
  ],
  "edges": [
    { "from": { "nodeId": "surface", "portId": "baseColor" }, "to": { "nodeId": "out", "portId": "baseColor" } }
  ],
  "exposedParameters": [
    { "id": "backingColor", "nodeId": "surface", "parameterId": "underColor" }
  ]
}
```

| Object | Rule |
|---|---|
| Document | Required `version: 2`, `nodes`, `edges`; optional `instances` and `exposedParameters`, default `[]`. Exactly one `material-output` among ordinary nodes, as in v1. |
| Instance | Required `id`, `recipe`, `recipeVersion`; optional `parameters` (interface values). Instance and node IDs share one namespace and must be unique together. |
| Edge | Endpoints name a node port or an instance interface port. Kinds match exactly; each input, including an interface input, accepts at most one edge. |
| Exposed parameter | Same shape as v1. `nodeId` names a node or an instance; `parameterId` names a node parameter or an interface parameter. One public ID binds one target; two public IDs may not bind the same target. |

In v2, node and instance IDs must not contain `--`, which is reserved for expansion. Instance parameters are validated against the interface (unknown → `MIX_PARAMETER_UNKNOWN`, invalid → `MIX_PARAMETER_INVALID_VALUE`, missing required → `MIX_PARAMETER_INVALID_VALUE` with instance evidence). Missing required interface inputs fail with `MIX_PORT_REQUIRED_CONNECTION`, kind mismatches with `MIX_PORT_TYPE_MISMATCH`, unknown interface ports with `MIX_PORT_UNKNOWN`. A v2 document without instances is valid and behaves like its v1 equivalent apart from `documentVersion`.

## Closure and identity

- The caller supplies recipe documents as bytes together with the parent. Core never reads paths, URLs or registries.
- The closure must be **exact**: every instance resolves to exactly one supplied `id@recipeVersion` (`MIX_RECIPE_MISSING`, with the supplied versions of that ID as evidence); two supplied documents with the same identity fail (`MIX_RECIPE_DUPLICATE`); a supplied recipe that no instance uses fails (`MIX_RECIPE_UNUSED`).
- Core records each supplied recipe's SHA-256 over its exact bytes and byte length. Inspection reports the closure as `[{ id, recipeVersion, sha256, byteLength }]` sorted by identity. Parents do not pin digests in this slice; semantic content still reaches the plan hash through expansion.

## Expansion and plan identity

1. Validate the recipe set and the parent, then apply instance parameter values and request overrides at the parent level.
2. Replace each instance with copies of its recipe nodes, IDs `<instanceId>--<recipeNodeId>`. Every expanded ID must satisfy the v1 node-ID pattern, including the 64-character limit (`MIX_NODE_INVALID_ID` with instance evidence). Because v2 IDs cannot contain `--`, the first `--` uniquely recovers the instance.
3. Rewrite interface edges: an edge into an interface input becomes one edge per target; an edge out of an interface output starts at its internal port. Internal edges are copied with renamed endpoints.
4. Validate the expanded graph with the v1 graph rules (cycles, multiplicity, required connections, cross-parameter relations) and the existing node, edge and exposed-parameter ceilings.
5. Normalize, slice, lower and hash exactly as today.

Consequences: RenderPlan stays v3 and needs no new field. `documentVersion` records `2`. Pass origins carry expanded node IDs. Because passes are ordered lexically by node ID, a parent and the hand-written v1 graph it replaces may order passes and texture slots differently and therefore have **different plan hashes and allocation estimates**; their kernels, parameters and dependency structure are identical, so their pixels must be byte-identical. Frozen time and memory budgets still apply to the parent.

## Limits

A new caller-owned `RecipeLimits` sits beside `SafetyLimits`, `ResourceLimits` and `PackageLimits`; ceilings may be lowered by callers.

| Limit | Default | Code |
|---|---|---|
| Supplied recipe documents | 16 | `MIX_LIMIT_RECIPE_COUNT_EXCEEDED` |
| Instances in one parent | 32 | `MIX_LIMIT_INSTANCE_COUNT_EXCEEDED` |
| Total recipe bytes | 2 MiB | `MIX_LIMIT_RECIPE_BYTES_EXCEEDED` |

Each recipe document is also bounded by `SafetyLimits` (decoded bytes, nodes, edges; interface parameters by `exposedParameters`). The expanded graph is bounded by the existing node (128) and edge (512) ceilings, checked from the instance and recipe counts **before** materializing the expanded graph. Counts are checked before allocation; decoding stops at the first item beyond a ceiling, as in v1.

## Diagnostics

New codes, all stage `validation` except limit and decode failures at `parse`: `MIX_RECIPE_MISSING`, `MIX_RECIPE_DUPLICATE`, `MIX_RECIPE_UNUSED`, `MIX_RECIPE_INVALID_INTERFACE`, `MIX_RECIPE_FORBIDDEN_NODE`, `MIX_LIMIT_RECIPE_COUNT_EXCEEDED`, `MIX_LIMIT_INSTANCE_COUNT_EXCEEDED`, `MIX_LIMIT_RECIPE_BYTES_EXCEEDED`. Everything else reuses existing codes with new evidence keys `recipeId`, `recipeVersion`, `instanceId` and `interfaceId`. Diagnostics inside a recipe name the recipe and its local node ID; diagnostics after expansion name the instance and the expanded node ID. Ordering follows the shared diagnostic order. Cycles are detected on the expanded graph, so a parent path through an instance is a cycle only if it is one internally; the reported path uses expanded IDs.

## Public surfaces and ownership

| Surface | Contract |
|---|---|
| Core (Rust) | Adds recipe decoding, a validated recipe set, `RecipeLimits`, and a v2 decode/validate entry that takes the recipe set and returns the existing `ValidatedDocument`, consumed unchanged by `compile`, `normalize` and `prepare`. Existing `MaterialDocument::decode` and `into_validated` stay v1-only and reject v2 with `MIX_FORMAT_UNSUPPORTED_VERSION`. Overrides on interface-bound public IDs apply to every target. Exact type names are settled in the MAT-04b review and recorded in the format guide. |
| wgpu | No change. |
| `mixture-asset` | No change. Package v1 keeps requiring a v1 document, so a v2 parent fails with the existing unsupported-version error. |
| CLI | `validate`, `inspect` and `render` accept repeated explicit `--recipe <file>` options; no directory scan. Reports for v1 inputs stay byte-identical; v2 reports add the closure list. Report schemas remain 3 and the validate envelope is unchanged. `asset pack` rejects v2. |
| Browser | Validate, inspect and render requests gain optional `recipes: Source[]` and `recipeLimits`, captured synchronously with the source before yielding. Package APIs reject v2. API schema stays 3; v1 calls are unchanged. WASM and JS remain thin over Core. |

The first implementation selects the unpublished **Rust 0.11.0 / browser 0.11.0-alpha.0** candidate because Core gains public types and diagnostic codes. Node catalog: unchanged (no new node type or version).

## Frozen acceptance cases

The MAT-04b PR's first commit converts this section into a machine-readable plan without changing any value.

**Recipes** (under `fixtures/recipes/`):

| Recipe | Internal nodes | Interface |
|---|---|---|
| `masked-layer-surface@1` | constant-color, constant-scalar, blend, scalar-mask-blend | inputs mask (Scalar, 2 targets), overColor, overRoughness; outputs baseColor, roughness; parameters underColor, underRoughness |
| `brick-structure@1` | three `brick-pattern@1` (height, color, mask; mask variation fixed at 0) | outputs height, color, mask; parameters seed, columns, rows, rowOffset, mortarX, mortarY, bevel (3 targets each), heightVariation, colorVariation (1 target each) |
| `weave-structure@1` | three `weave-pattern@2` (modes warp-share, coverage, height fixed) | outputs warpShare, coverage, height; parameters warpCount, weftCount, warpWidth, weftWidth, bevel, crown, underRatio (3 targets each) |
| `directional-grain@1` | fractal-noise@2, transform-2d, levels, gradient-map | outputs grain (Scalar), color (Color); parameters seed (required), quarterTurns, darkColor, color |

**Parents:**

| Parent | Replaces accepted graph | Instances | Exposed IDs (v1 → v2) |
|---|---|---|---|
| `fixtures/materials/brick-paving/material-recipes.mix` | `brick-paving/material.mix` (MAT-01) | brick-structure, masked-layer-surface | 24 → 10: seed, columns, rows, rowOffset, mortarX, mortarY, bevel, heightVariation, colorVariation, normalStrength |
| `fixtures/materials/woven-fabric/material-recipes.mix` | `woven-fabric/material.mix` (MAT-03 r3) | weave-structure, two directional-grain (warp, weft), masked-layer-surface | 33 → 19: warpSeed, weftSeed, the seven weave controls, warpDark, warpColor, weftDark, weftColor, backingColor, roughnessMin, yarnRoughness, backingRoughness, relief, normalStrength |

`masked-layer-surface` is the cross-family recipe used by both parents; the structure recipes demonstrate fan-out; two `directional-grain` instances in one parent demonstrate instance isolation. Painted metal is not a parent in this slice: its rust layer uses the opposite blend polarity, and adapting it would need a second recipe or a parameterized polarity that no case requires yet.

**Gates:**

1. **Structural equivalence (CPU).** For the defaults and every case of the frozen brick and woven qualification plans, the expanded parent graph is isomorphic to the accepted v1 graph under a node-ID renaming: same node types, versions, normalized parameters and edges. Compiling both yields the same pass multiset (kernel invocation and parameters) and output wiring.
2. **Pixel identity.** For every case and size of both frozen matrices, every requested channel of the parent is **byte-identical** to the accepted v1 graph on the recorded adapters: pinned SwiftShader in CI, and GT 1030 Vulkan and DX12; Chrome for the default and one varied case per family. Byte identity carries the existing human visual acceptance over; no new visual review is required. Any difference stops the increment.
3. **Budgets.** Parent plans stay within each material's frozen `maxDescriptorPeakBytes` (brick 469,762,048; woven 536,870,912) and frozen timing budgets; changes against the v1 estimate are recorded.
4. **Controls.** Conceptual control requests (brick [controls.json](../fixtures/materials/brick-paving/controls.json), woven request builder) map one-to-one onto the v2 public IDs and reproduce gate 2.
5. **Isolation.** Overriding one `directional-grain` instance leaves the other instance's expanded nodes unchanged; two instances with different seeds produce distinct expanded seeds and no shared node.
6. **Negative corpus.** One focused fixture per rule in this document: every new code, each reused code with instance evidence, every limit at and beyond its ceiling, `--` IDs, expanded-ID overflow, nesting attempt, forbidden nodes, unequal defaults, unset required seed, missing/duplicate/unused recipes, cycles through and not through an instance.
7. **Compatibility.** All v1 format fixtures, plan-hash snapshots, goldens and accepted material receipts unchanged; v1 entry points reject v2; package v1 rejects v2; CLI v1 reports byte-identical; all six required checks pass.

**Presets.** A small corpus under `fixtures/recipes/presets/`: three named override maps per parent taken from existing frozen cases (brick regular, staggered, varied; woven plain, varied, combined-low). A preset is a fixture of request overrides validated against the parent's exposed IDs; it is **not** a runtime or document format.

## Out of scope

Recipe nesting, inline definitions, package transport of recipes, digest pinning in parents, parameter expressions or range intersection, painted-metal extraction, new node types, implicit lookup, registries, automatic v1→v2 conversion, Studio changes and publication. Each needs its own demonstrated case.

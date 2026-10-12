# MAT-04 — Start reusable recipes

English | [简体中文](./mat-04-reusable-recipes.zh-CN.md)

**Current status:** see [release status](./release.md) for integrated features, published versions and hardware qualification scope. Earlier dated records describe their original checkpoints.

On 2026-10-12 the maintainer started MAT-04 as the single active increment. This entry document records the measured duplication, the first consumer outcome, the questions the MAT-04a contract must answer and the work-item sequence. It selects no format, syntax, node, crate or version and changes no code, fixture, golden or schema. `.mix v1`, `.mixpack v1` and plan/API v3 remain unchanged until an accepted contract and ADR say otherwise.

## First consumer outcome

An author writes one shared recipe once and uses it in at least two independent accepted material graphs. The recipe has typed inputs and outputs, a public parameter interface that can drive several internal parameters, explicit seed handling and a version. A parent graph that instantiates it renders exactly the pixels of the equivalent hand-flattened graph through the existing Core compiler and the sole wgpu executor. Native and browser callers supply every dependency explicitly; no path, URL or registry lookup occurs. A small preset corpus sets interface values only.

## Measured entry problem

All counts come from the accepted fixtures on main and can be read directly from the cited files.

| Material family | Nodes / edges / exposed | Duplication that `.mix v1` cannot express |
|---|---|---|
| [brick-paving](../fixtures/materials/brick-paving/material.mix) (MAT-01) | 11 / 12 / 24 | Three `brick-pattern@1` instances (height, color, mask) repeat seven layout controls; 21 of 24 exposed bindings are those copies. [controls.json](../fixtures/materials/brick-paving/controls.json) fans each conceptual control out to three bindings in caller code. |
| [woven-fabric](../fixtures/materials/woven-fabric/material.mix) (MAT-03 r3) | 22 / 26 / 33 | Three `weave-pattern@2` instances (warp-share, coverage, height) repeat seven controls; 21 of 33 exposed bindings are copies. [woven-fabric-requests.mjs](../scripts/woven-fabric-requests.mjs) keeps them equal and also derives `warpDark`/`weftDark` from colors and `detailAmount`. Warp and weft each repeat the same noise → transform → levels → gradient-map grain chain. |
| [painted-metal](../fixtures/materials/painted-metal/material.mix) (MAT-02 r2) | 24 / 34 / 21 | The rust layer applies one mask to baseColor, roughness, height and metallic through four parallel blend nodes. |

Two concrete gaps follow:

1. **Parameter fan-out.** A v1 exposed parameter targets exactly one node parameter and targets must be unique, so one conceptual control cannot drive several nodes. Brick and woven work around this in caller code; each new consumer must re-implement that mapping. Woven already uses 33 of the default 64 exposed-parameter ceiling.
2. **Shared layer composition across families.** brick (mortar/brick), woven (backing/yarn) and painted metal (paint/rust) each build a masked two-layer surface: one mask selects between two colors through `blend` and between two roughness values through `scalar-mask-blend`, and painted metal extends it to height and metallic. Each graph rebuilds that structure by hand with different node IDs and bindings.

This is evidence for a reusable-recipe contract with typed interfaces and parameter mapping. It is **not** evidence for general expressions: woven's derived dark colors are caller arithmetic, and user scripting remains excluded by the [roadmap](../ROADMAP.md#material-capability-roadmap--staged-delivery). It does not show that nested recipes, a recipe marketplace or automatic migration are needed.

## Required contract for MAT-04a

The MAT-04a contract and its ADR must answer each item before any implementation PR:

1. **Carriage and versioning.** Where recipe definitions live (inline in a parent document, separate caller-supplied documents, or both) and which document version carries them. `.mix v1` rejects unknown fields and must keep its exact meaning; old runtimes must reject new documents with a structured error, never reinterpret them. Compare the candidates with measured byte size and validation cost, as M6B-02 did.
2. **Identity and dependency closure.** A recipe reference names an explicit identity and version. The caller supplies the complete dependency set as bytes; Core validates the closure, rejects missing, extra-version and duplicate definitions and never searches a filesystem or network. Content integrity and how definition identity enters the plan hash must be specified.
3. **Typed interface.** Inputs and outputs use the existing port kinds with exact matching; no implicit conversion. Define which interface ports are optional and their defaults.
4. **Parameter mapping.** One interface parameter may drive several internal parameters of compatible type and range. Specify range intersection, defaults, which interface parameters a parent may expose, and how existing exposed-parameter uniqueness and the 64-binding ceiling apply. Mapping is direct assignment; arithmetic expressions need a separately demonstrated case.
5. **Seeds.** Every randomized node still needs an explicit seed. Decide how instance seeds are supplied and whether two instances of one recipe can share or must separate seeds, without hidden derivation that changes existing pixels.
6. **Expansion and plan identity.** Core expands instances into the existing node graph before compilation, so RenderPlan, wgpu kernels and pixels need no new executor semantics. Expanded node IDs must be deterministic and diagnosable. A parent graph and its hand-flattened equivalent must compile to the same passes and pixels; decide whether their plan hashes are equal or intentionally differ.
7. **Limits.** Define nesting depth (or forbid nesting in the first slice), and apply node/edge/parameter ceilings to the expanded graph as well as each source document. Expansion must stay bounded before any large allocation.
8. **Diagnostics.** Actionable structured errors for recipe cycles, missing or unsupported versions, interface type mismatches, unknown or duplicate interface parameters and invalid mappings, using the shared diagnostic ordering.
9. **Package compatibility.** `.mixpack v1` holds exactly one `material.mix`. Decide whether recipe closures travel in packages now (which requires a new package version under ADR 0008's rules) or stay a loose-input feature first. Package v1 bytes keep their meaning either way.
10. **Ownership.** Core owns decoding, validation, expansion and hashing; `mixture-asset` owns any package change; wgpu is unchanged unless the contract proves otherwise; CLI, WASM and JS stay thin. No new crate without an actual boundary.

## Maintainer direction for MAT-04a (2026-10-12)

The maintainer selected three directions for the contract to specify; they narrow questions 1, 7 and 9 and are not yet a contract:

- **Separate recipe documents.** A recipe is its own document with an explicit identity and version, supplied by the caller as bytes. A parent graph references it from a new document version; `.mix v1` keeps its exact meaning. Inline definitions are not part of the first slice.
- **Loose input only.** `.mixpack v1` is unchanged and recipe closures do not travel in packages in this increment. A later package version needs its own demonstrated delivery case.
- **No nesting.** A recipe cannot instantiate another recipe; expansion depth is exactly one. Nesting needs a later measured case.

## Work items and exit gates

| Item | Deliverable | Required exit evidence |
|---|---|---|
| **MAT-04 entry (this PR)** | Measured duplication, first outcome, contract questions and sequence | Paired documentation, links and repository check; no format or runtime claim |
| **MAT-04a — contract and ADR** ([proposed](./mat-04a-recipe-contract.md)) | Selected carriage/version, identity, interface, mapping, seed, expansion, limit, diagnostic and package rules; ADR 0010; frozen acceptance cases naming the two parent graphs and the extracted recipes | Measured comparison of carriage candidates; malicious-input list; catalog/version review; maintainer acceptance before implementation |
| **MAT-04b — Core decode, validation and expansion** | Public Rust API for supplying dependencies, validating closures and expanding instances | Format/core/plan tests; negative corpus for every rule; deterministic expansion and hashing; old v1 fixtures and plan hashes unchanged |
| **MAT-04c — recipes, parents, presets and adapters** | Extracted recipes used by two accepted families, preset corpus, CLI/browser dependency supply, package support only if selected | Instance isolation; flattened-equivalence pixel comparisons for the existing material matrices; browser lifecycle tests; no implicit lookup |
| **MAT-04d — qualification and closeout** | Exact candidate across Native and browser | Existing material gates unchanged, cross-runtime comparisons on recorded adapters, all six required checks; retained receipts and explicit limits |

Work-item IDs are not GitHub PR numbers. Contract acceptance, implementation, qualification, integration and publication remain separate states.

## Stop and scope rules

Stop if the contract needs a second executor, changes the meaning of any existing `.mix v1` document or node, or requires relaxing an accepted material gate. Recipe extraction must not alter pixels of the accepted brick, painted-metal or woven matrices; equivalence is measured, not assumed. Excluded: implicit filesystem or network lookup, a recipe registry or marketplace, user scripting or parameter expressions, automatic document migration, editor layout or thumbnails in documents, Studio changes and publication. Any of these needs its own demonstrated case and decision.

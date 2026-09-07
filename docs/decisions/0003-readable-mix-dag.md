# ADR 0003: .mix v1 is one readable DAG

Status: Accepted for M0 foundation. Source: [architecture sections 5–8](../../ARCHITECTURE.md).

## Context

The first material format must be inspectable, deterministic, bounded, and useful without an editor. A binary container or nested graph system would add complexity before consumer evidence exists.

## Decision

Use versioned UTF-8 JSON containing one directed acyclic graph: stable node IDs, node type/version, typed parameters, edges, and exposed bindings. Material semantics remain separate from editor layout, thumbnails, presets, engine targets, resources, and custom shaders. M2 defines decoding and executable schema behavior.

## Alternatives

A binary `.mixar` format impedes manual inspection. Embedded editor state couples runtime assets to a product UI. Subgraphs introduce unnecessary initial compilation and migration complexity.

## Consequences

Validation must reject unknown or unsupported semantics, cycles, non-finite parameters, and budget violations. Traversal and serialization are stable, randomized nodes require seeds, and no silent graph repair occurs.

## Migration

No old format is silently reinterpreted. Future changes declare whether they are additive, migratable, or breaking; semantic changes require versioning. M0 intentionally creates no placeholder `.mix` fixture.

## Verification

M0 documentation checks keep the contract discoverable. M2 adds decode, reject, round-trip, graph, budget, and deterministic-plan tests before claiming format support.

# .mix v1 decoding and validation

English | [简体中文](./file-format.zh-CN.md)

**M6A-02 update:** [M6A-02 Core implementation](./m6a-02-core-resources.md) now provides resource references, immutable prepared requests and content-bound plan v2. Rust source is 0.3.0; the unpublished browser candidate is 0.3.0-alpha.0/API schema 2, with schema 2 inspect/graph-render reports. The [M6A-03 Native path](./m6a-03-native-resources.md) now executes prepared images; [M6A-04 browser resources](./m6a-04-browser-resources.md) now add synchronous capture and public rendering. Final cross-platform qualification remains M6A-05. Read historical version descriptions below in that context.

PR-005 introduces the first executable `.mix` source schema. It is UTF-8 JSON with document version `1` and independently required node version `1`. No earlier executable format or migration exists. This adds decoding, six node contracts, graph validation, and a CLI validator; PR-006 [graph compilation](./render-plan.md) is now implemented; PR-007 [graph execution](./graph-rendering.md) is implemented.

## Use the validator

```bash
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
cargo run --locked -p mixture-cli -- validate fixtures/format/valid/all-m2.mix
cargo xtask test-format
cargo xtask test-core
cargo xtask check
```

`validate <file.mix> [--json]` reads one file and never changes it or acquires a GPU. `--json` writes exactly the shared `{ "ok": ..., "diagnostics": [...] }` report to stdout. A valid document returns `0`; invalid invocation, parse/schema/version/graph/parameter/budget failures return `2`; file or report I/O failures return `1`. Usage errors go to stderr even with `--json`. Operational and input diagnostics include the supplied document path. Use `--` before a filename beginning with `-`; `--json` must then precede `--`.

The CLI reads at most 2 MiB plus one detection byte. It reports the observed prefix size on oversized files, not an invented total file size. It does not read embedded resources or resolve paths from the document.

## Source schema

The [checker example](../examples/checker.mix) is a minimal complete material:

```json
{
  "version": 1,
  "nodes": [
    { "id": "checker", "type": "checker", "version": 1 },
    { "id": "out", "type": "material-output", "version": 1 }
  ],
  "edges": [
    {
      "from": { "nodeId": "checker", "portId": "color" },
      "to": { "nodeId": "out", "portId": "baseColor" }
    }
  ],
  "exposedParameters": [
    { "id": "frequency", "nodeId": "checker", "parameterId": "cellsX" }
  ]
}
```

| Object | Required fields | Optional fields |
| --- | --- | --- |
| Document | `version`: unsigned 32-bit integer; `nodes`: array; `edges`: array | `exposedParameters`: array, defaults to `[]` |
| Node | `id`: string; `type`: string; `version`: unsigned 32-bit integer | `parameters`: object, defaults to `{}` |
| Edge | `from`: output endpoint; `to`: input endpoint | None |
| Endpoint | `nodeId`: string; `portId`: string | None |
| Exposed parameter | `id`: public string ID; `nodeId`: target; `parameterId`: mutable parameter | None |

Node and public parameter IDs match `[A-Za-z][A-Za-z0-9_-]{0,63}`. Node IDs and public IDs have separate namespaces and must each be unique. Type, port, and parameter IDs are exact, case-sensitive strings from the [eleven node contracts](./node-contracts.md). All fields above use their exact spellings.

Unknown fields are rejected at every object boundary, including layout, metadata, thumbnails, presets, resources, subgraphs, export targets, and arbitrary WGSL. Duplicate JSON keys are rejected, even if values agree or keys use equivalent JSON escapes. This includes nested parameter objects; invalid parameter shapes are retained only to report semantic type errors. No last-key-wins interpretation is allowed.

Required fields cannot be omitted or null. Arrays must be arrays. Version numbers must be integer tokens, so `1.0`, negative numbers, strings, and values above `u32::MAX` are rejected. Comments, trailing commas, trailing documents, invalid UTF-8, NaN, infinity, and numeric overflow are rejected. The normal JSON recursion limit stays enabled; deeply nested input cannot grow an unbounded recursive parser stack.

## Limits and validation order

The caller supplies [SafetyLimits](./diagnostics.md). Defaults remain 2 MiB input bytes, 128 nodes, 512 edges, and 64 exposed parameters. Zero ceilings remain zero. The byte check precedes UTF-8/JSON parsing. Collection decoding does not reserve an untrusted size hint and stops at the first item beyond the ceiling, without constructing that typed item. `observed` is that first exceeding count, not the count of an unread suffix.

After structural decoding, an unsupported document version fails with `MIX_FORMAT_UNSUPPORTED_VERSION`. Graph validation checks every node and edge, including unused components:

1. Collection ceilings and document version are rechecked for constructed Rust documents. Failures stop further graph analysis.
2. Node IDs, known types, node versions, parameter names, types, ranges, enums, and cross-parameter relations are checked. An unsupported node version is never interpreted with version 1 defaults.
3. Edge endpoints must identify existing, unambiguous nodes and declared output-to-input ports of exactly the same kind. All inputs accept at most one edge. Repeated edge identities and multiple sources are rejected separately.
4. Iterative, lexically ordered depth-first traversal detects directed cycles, including self-loops and disconnected cycles. Evidence names actual closed paths; downstream nodes are not mislabeled as cycle members. The validator reports discovered back-edge cycles, not an exhaustive enumeration of all possible cycles.
5. Exactly one `material-output` is required. Its `baseColor` input needs a valid `Color` connection. Other inputs use their documented defaults. Every required input in the node contracts needs a valid connection, including both Scalar inputs of `warp`.
6. Exposed public names must be valid and unique. Each targets an existing mutable parameter on a supported node version, including a parameter using a default. Targets are unique too: aliases and duplicate bindings are rejected. Node IDs, node versions, ports, and structural fields are not mutable parameters.

Independent diagnostics are accumulated and sorted using the [shared ordering](./diagnostics.md). Invalid or ambiguous nodes suppress dependent port interpretation; valid independent checks still run. Validation never adds nodes/edges, repairs cycles, coerces or clamps values, inserts conversions, or edits explicit parameters.

Output dimensions, requested channels, transient GPU estimates, exposed override application, dependency slicing, and RenderPlan generation belong to the [PR-006 compiler request](./render-plan.md). They are not document fields or claims made by `validate`.

## Public Rust API

```rust
use mixture_core::{DocumentError, MaterialDocument, SafetyLimits, ValidatedDocument};

fn validate_source(source: &[u8]) -> Result<ValidatedDocument, DocumentError> {
    let limits = SafetyLimits::default();
    MaterialDocument::decode(source, &limits)?.into_validated(&limits)
}
```

The executable [consumer test](../crates/mixture-core/tests/format.rs) and crate doctest exercise the same API. [document.rs](../crates/mixture-core/src/document.rs) owns source types and errors; [validation.rs](../crates/mixture-core/src/validation.rs) owns validation and read-only access to validated semantics; [registry.rs](../crates/mixture-core/src/registry.rs) and [node modules](../crates/mixture-core/src/nodes/) own contracts.

`decode` returns editable `MaterialDocument` data after structural/version/decoding-limit checks. It does not certify graph validity. `validate(&limits)` returns an immutable `DiagnosticReport`. `into_validated(&limits)` consumes the source only on success and returns `ValidatedDocument`; this wrapper has no public constructor, deserializer, or mutable accessor. `DocumentError::report()` preserves all returned diagnostics and native parser/UTF-8/limit error sources. Library calls never infer a filesystem path.

`parameter` resolves an explicit value or versioned default without inserting it into the source. `input_source` distinguishes a validated connection from an unconnected input default. `material_channels` reports all eight channels in contract order with their kinds and sources. These are semantic descriptions, not computed textures.

`to_json` produces deterministic pretty JSON for valid source documents: nodes sorted lexically by ID, edges by the complete source/destination endpoint tuple, public bindings by ID, and object keys in lexical order within parameters. It emits explicit empty parameter objects and exposed arrays when omitted on input. It retains explicit values, meaningful array order, and source semantics; it does not insert parameter defaults. It is structural serialization, not the normalized plan hash. JSON numeric parameters use the existing `serde_json` dependency with `float_roundtrip`, so finite f64 values survive serialization/decoding without bit drift. Original decimal spelling and whitespace are not retained.

## Diagnostic additions and evidence

Existing code spellings and stage order remain unchanged. PR-005 adds these codes to the [vocabulary snapshot](../crates/mixture-core/tests/snapshots/diagnostics-vocabulary.json):

| Code | Stage / meaning |
| --- | --- |
| `MIX_FORMAT_INVALID_DOCUMENT` | `parse`: field schema, duplicates, or structural JSON types |
| `MIX_NODE_INVALID_ID`, `MIX_NODE_DUPLICATE_ID` | `validation`: invalid or ambiguous node identity |
| `MIX_GRAPH_UNKNOWN_NODE` | `validation`: edge endpoint node does not exist |
| `MIX_GRAPH_DUPLICATE_EDGE`, `MIX_GRAPH_MULTIPLE_INPUTS` | `validation`: connection identity/cardinality |
| `MIX_GRAPH_MATERIAL_OUTPUT_COUNT` | `validation`: not exactly one material sink |
| `MIX_PORT_REQUIRED_CONNECTION` | `validation`: missing valid required input |
| `MIX_PARAMETER_UNKNOWN` | `validation`: name absent from the node contract |
| `MIX_EXPOSED_PARAMETER_INVALID` | `validation`: public ID or target binding problem |
| `MIX_IO_READ_FAILED` | `parse`: caller-side source-file I/O failure; CLI exit 1 |

Existing UTF-8/JSON/version/limit/node/port/parameter/cycle codes handle their corresponding failures. Parse errors retain native source, line/column where supplied by the parser, and selected source text. Semantic diagnostics identify node, port, parameter, public ID, counts, or cycle path as applicable.

[Format fixtures](../fixtures/format/README.md) include a two-node checker, all six M2 contracts, and focused invalid documents. Tests cover strict syntax/schema, duplicate escapes, deep nesting, byte/collection boundaries, explicit limits, float round trips, defaults, graph order permutations, exact cycle evidence, CLI exit codes, bounded reads, and unchanged source files. `cargo xtask test-format` runs core format/validation/registry tests and CLI validation tests; `cargo xtask test-core` runs the entire GPU-free core suite. Remote cross-platform CI was still pending at PR-005; the documented [three-platform CPU gate](./evidence/remote-ci/README.md) has since passed.

## Additive PR-009 catalog extension

The document JSON shape remains `.mix v1`; existing documents retain their meaning and need no migration. Three new version-1 node types are described in [node contracts](./node-contracts.md). Only `fractal-noise.seed` has no parameter default: it must be an explicit unsigned integer token in the source, from 0 through 4294967295. Missing seed returns `MIX_PARAMETER_INVALID_VALUE` with parameter evidence, including for unused nodes. Decoding, deterministic serialization and old source/plan fixtures remain unchanged.

## Additive PR-010 resampling nodes

`transform-2d` and `warp` add two version-1 node types without adding fields or changing existing `.mix v1` meanings. Their required ports accept exact `Scalar` connections. Transform scales and quarter-turn counts require bounded unsigned integer tokens; offsets and warp strengths accept bounded finite JSON numbers. Defaults, parameter bounds, input requirements and explicit public bindings follow the same validator rules as the existing catalog, including on unused branches. See [node contracts](./node-contracts.md) and the [public resampling tests](../crates/mixture-core/tests/resampling.rs). No migration, source repair or implicit conversion is introduced.

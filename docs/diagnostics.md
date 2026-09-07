# Diagnostics and safety limits

English | [简体中文](./diagnostics.zh-CN.md)

PR-002 establishes the public diagnostic and budget vocabulary in `mixture-core`. It does not parse `.mix`, validate a graph, acquire an adapter, or implement runtime CLI commands.

## Public API

The crate re-exports `DiagnosticCode`, `Stage`, `Severity`, `EvidenceValue`, `Diagnostic`, `DiagnosticReport`, `LimitKind`, `SafetyLimits`, and `LimitExceeded` from its root. Implementation lives in [error.rs](../crates/mixture-core/src/error.rs) and [limits.rs](../crates/mixture-core/src/limits.rs).

```rust
use mixture_core::{DiagnosticReport, LimitKind, SafetyLimits, Stage};

let limits = SafetyLimits::default();
let diagnostics = limits
    .check(LimitKind::Nodes, 129)
    .err()
    .map(|violation| violation.diagnostic(Stage::Validation));
let report = DiagnosticReport::new(diagnostics);
assert!(!report.is_ok());
```

The caller explicitly supplies the stage when converting a limit failure, and can attach document, node, port, or parameter context to the diagnostic. The library does not infer a path, initialize a device, repair an input, or change a configured ceiling.

Run the [public-API example](../crates/mixture-core/examples/diagnostics.rs):

```bash
cargo run --locked -p mixture-core --example diagnostics
```

It intentionally checks two oversized counts and prints a report with `ok: false`. The example process exits successfully when it finishes demonstrating the API; it is not a CLI validation operation or a rendering health probe.

## JSON contract

`Diagnostic` implements Serde serialization and deserialization. `DiagnosticReport` is a serializable output envelope with exactly `ok` and `diagnostics`. Its list is private and immutable after construction. `ok` is derived from the absence of error-severity diagnostics, including an empty report; it never claims a render or GPU probe occurred. Warnings and informational observations alone do not invalidate the report.

Each diagnostic contains:

| Field | Contract |
| --- | --- |
| `code` | Stable `MIX_*` string from `DiagnosticCode` |
| `stage` | Explicit lifecycle stage |
| `severity` | `error`, `warning`, or `info` |
| `message` | Human-readable text; callers branch on the code instead |
| `documentPath` | Optional caller-supplied string |
| `nodeId`, `portId`, `parameterId` | Optional document-local identifiers |
| `evidence` | Optional map with lexically ordered string keys |
| `suggestion` | Optional actionable guidance |

Absent optional values and empty evidence maps are omitted. Unknown fields, codes, stages, and severities are rejected when decoding a diagnostic. `DiagnosticReport` itself is output-only; it has no deserializer that could accept a forged `ok` flag or an unsorted list.

Evidence values initially support booleans, exact unsigned 64-bit integers, and strings. They serialize as ordinary JSON scalar values. Negative numbers, floating-point values, nulls, arrays, and nested objects are not supported. This keeps counts exact and prevents non-finite measurements from silently becoming JSON null. Future numeric or structured evidence needs a deliberate contract extension. Consumers must preserve integer precision when reading values above their language's safe integer range.

See the [report snapshot](../crates/mixture-core/tests/snapshots/diagnostics-report.json) and [limit-failure snapshot](../crates/mixture-core/tests/snapshots/limits-exceeded.json) for complete examples. Snapshot changes must accompany an explained contract change; there is no automatic baseline update command.

## Stable ordering

`DiagnosticReport::new` orders diagnostics by these keys, in order:

1. Stage: `parse`, `validation`, `compile`, `gpuAdapter`, `gpuDevice`, `gpuShader`, `gpuPipeline`, `gpuExecution`, `readback`, `encoding`.
2. Severity: `error`, `warning`, `info`.
3. Document path, node ID, port ID, then parameter ID; absent values sort before present ones, strings use Rust's lexical order.
4. Stable code string, message, evidence, then suggestion.

Evidence keys are stored in a `BTreeMap`. Map comparisons use ordered key/value pairs; different evidence value kinds sort boolean before unsigned integer before text, then by value. Every serialized diagnostic field participates in ordering. Duplicates are retained. Native source errors do not affect the order, so ties still have identical serialized JSON. The guarantee is for equal diagnostic data, not messages, paths, or evidence that differ across environments.

## Source errors

`Diagnostic` implements `std::error::Error` and `Display`. `with_source` retains the actual `Send + Sync + 'static` error, including nested sources and downcasting, through an owned `Arc`. A cloned diagnostic retains the same source. `Display` emits the stable code and concise message; native callers may traverse `Error::source()` separately for a human report.

Source objects and their messages are not serialized automatically, and deserialized diagnostics have no native source. A `source` field supplied in JSON is rejected. Callers can explicitly select safe driver details for evidence while preserving the original typed source for native debugging. This separates machine-stable data from platform-specific errors without swallowing the underlying failure.

## Reserved code families

Only limit checking produces failures in this PR. Other codes reserve a shared vocabulary for subsequent work; their presence does not imply implemented parsing, graph semantics, rendering, or recovery.

| Family | Initial codes |
| --- | --- |
| `MIX_PARSE_*` | `MIX_PARSE_INVALID_UTF8`, `MIX_PARSE_INVALID_JSON` |
| `MIX_FORMAT_*` | `MIX_FORMAT_UNSUPPORTED_VERSION` |
| `MIX_LIMIT_*` | The seven `MIX_LIMIT_*_EXCEEDED` codes listed below |
| `MIX_NODE_*` | `MIX_NODE_UNKNOWN_TYPE`, `MIX_NODE_UNSUPPORTED_VERSION` |
| `MIX_PORT_*` | `MIX_PORT_UNKNOWN`, `MIX_PORT_TYPE_MISMATCH` |
| `MIX_GRAPH_*` | `MIX_GRAPH_CYCLE` |
| `MIX_PARAMETER_*` | `MIX_PARAMETER_INVALID_VALUE` |
| `MIX_COMPILE_*` | `MIX_COMPILE_INVALID_REQUEST` |
| `MIX_GPU_ADAPTER_*` | `MIX_GPU_ADAPTER_UNAVAILABLE` |
| `MIX_GPU_DEVICE_*` | `MIX_GPU_DEVICE_REQUEST_FAILED` |
| `MIX_GPU_SHADER_*` | `MIX_GPU_SHADER_VALIDATION_FAILED` |
| `MIX_GPU_EXECUTION_*` | `MIX_GPU_EXECUTION_FAILED` |
| `MIX_READBACK_*` | `MIX_READBACK_FAILED` |
| `MIX_ENCODING_*` | `MIX_ENCODING_FAILED` |

`DiagnosticCode` and `Stage` are non-exhaustive Rust enums. Consumers match known cases with a fallback; existing wire spellings and relative ordering must remain stable. The [vocabulary snapshot](../crates/mixture-core/tests/snapshots/diagnostics-vocabulary.json) pins the initial strings. Exit codes are a CLI concern, not methods on core diagnostics.

## Safety limits

All fields use `u64`, with the same units on every target. `SafetyLimits::default()` is the explicit source of default policy:

| Field / limit kind | Default upper bound | Failure code |
| --- | ---: | --- |
| `decodedBytes` / `DecodedBytes` | 2 MiB = 2097152 bytes | `MIX_LIMIT_DECODED_BYTES_EXCEEDED` |
| `nodes` / `Nodes` | 128 | `MIX_LIMIT_NODES_EXCEEDED` |
| `edges` / `Edges` | 512 | `MIX_LIMIT_EDGES_EXCEEDED` |
| `exposedParameters` / `ExposedParameters` | 64 | `MIX_LIMIT_EXPOSED_PARAMETERS_EXCEEDED` |
| `outputDimension` / `OutputDimension` | 2048 per axis | `MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED` |
| `requestedOutputs` / `RequestedOutputs` | 8 | `MIX_LIMIT_REQUESTED_OUTPUTS_EXCEEDED` |
| `transientBytes` / `TransientBytes` | 512 MiB = 536870912 bytes | `MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED` |

`check(kind, observed)` accepts equality and returns a `LimitExceeded` containing `kind`, `configured`, and `observed` when the ceiling is exceeded. It only compares values: no addition, narrowing cast, allocation estimate, or policy mutation occurs. Conversion to a diagnostic includes `configured`, `limit`, and `observed` evidence, a suggestion, and the typed limit failure as its native source.

Callers may explicitly construct a stricter or larger policy. Zero remains zero; it is never treated as missing or replaced by a default. JSON decoding requires all seven fields and rejects unknown, missing, negative, fractional, and overflowing values. See the [defaults snapshot](../crates/mixture-core/tests/snapshots/limits-defaults.json).

These are upper-bound primitives, not document or request validation. A zero-sized image may satisfy a ceiling but must later fail request validation. Check width and height separately. Callers must safely compute actual counts and estimated bytes before checking them; the compiler's resource estimator is not implemented. Embedded resources remain unsupported in v1 with a zero budget; PR-002 adds no resource field or loading API.

## CLI exit-code policy

This policy applies to PR-003 doctor and future runtime commands; it does not imply that all commands exist:

| Exit code | Meaning |
| --- | --- |
| `0` | Command completed successfully; warning/info-only reports may accompany a real result |
| `1` | Operational failure: file I/O, adapter/device/shader/execution, readback, or encoding failure |
| `2` | Invalid invocation or input: usage, parsing, format, node/port/graph/parameter validation, budget violation, or invalid compile request |

The CLI owns this mapping and `ok` consistency. Aggregate failures choose exit `1` if any operational error exists, otherwise `2` when any input error exists, otherwise `0`; this is independent of discovery order. Runtime command implementations must add integration tests for this policy. A missing required adapter is a failure, never a successful fallback. An explicitly skipped future doctor execution probe remains `unverified`, not `healthy`.

PR-003 implements [doctor](./gpu-context.md): acquired contexts return `0`, `ok: true`, and `unverified`; acquisition failures return `1`, `ok: false`, and `unhealthy`. Both probes remain `notRun`. Usage errors return `2` on stderr even in JSON mode; output I/O failures return `1`. Help/version return `0`, and missing/unknown/future commands return `2`. Remaining runtime mappings will be implemented with their owning commands.

## Verification and scope

```bash
cargo test --locked -p mixture-core diagnostics
cargo test --locked -p mixture-core limits
cargo test --locked -p xtask
cargo run --locked -p mixture-core --example diagnostics
cargo xtask check
```

Tests exercise public imports, JSON snapshots and rejection, ordering under input permutations, optional context, exact integer evidence, native source chains, every default boundary, explicit overrides, zero ceilings, and extreme counts. The public example and crate doctest provide consumer-level API evidence.

Core's only runtime dependency is `serde`; `serde_json` is a dev dependency for snapshots and the example. The [dependency policy](./development.md) enforces that distinction. PR-002 introduced no `.mix` format field, graph implementation, GPU dependency, shader, CLI runtime command, or automatic remediation. PR-003 acquisition and doctor are documented separately. The M0 remote CI gate remains pending; PR-002 does not claim to close it.

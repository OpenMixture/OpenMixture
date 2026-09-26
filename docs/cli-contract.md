# Native CLI reports and exit codes

English | [简体中文](./cli-contract.zh-CN.md)

**Current status:** see [release status](./release.md) for integrated features, published versions and hardware qualification scope. Earlier dated records describe their original checkpoints.

The [asset CLI contract](./m6b-04-adapters.md) adds the schema-1 envelopes, file rules and memory budgets for `asset pack` / `inspect` / `render`; nested graph-render reports use schema 3 as documented here.

PR-012 verifies consumption of the built `mixture` executable from an [independent Rust test program](../examples/native-consumer/tests/cli_contract.rs), with its own input and output directory. It repairs human diagnostic context and records the existing JSON and exit behavior. It does not introduce a schema version, change pixel semantics, or complete the remaining [M4 work](../M4_PRS.md).

## Process boundary

Spawn the built executable with an argument array, select `--json`, capture stdout and stderr separately, wait for process completion, then check the exit code and parse stdout as one JSON object. For successful renders, consume the reported files after the process exits. Paths in reports preserve the supplied input/output paths; resolve relative paths against the child process's working directory.

| Exit | Meaning and report |
|---|---|
| `0` | The requested command completed. `validate` checked source; `inspect --plan` compiled a plan; `doctor` either verified its checker probe or explicitly skipped it; `render` completed every requested PNG write. JSON has `ok: true`. |
| `2` | Invocation, source or compile-request error. Parsed commands produce `ok: false` with structured diagnostics. Invalid invocation is the exception below. |
| `1` | Operational error: source-file I/O, GPU acquisition/execution/readback, encoding, file writing or report output. When the report can be written, JSON has `ok: false` and available evidence. |

**Usage errors remain text on stderr, including with `--json`; stdout is empty.** Examples include missing `--plan`, missing `--out`, unknown options and malformed option syntax. A syntactically valid but invalid exposed value such as `--set 'repeat=0'` reaches compilation and returns a JSON diagnostic instead. A missing source file is operational exit `1`, while malformed file contents are source exit `2`. Help/version are text with exit `0`, not JSON report commands.

Parsed command diagnostics go to the chosen stdout report. The tested CPU paths leave stderr empty. GPU backends may emit their own stderr messages; those are not a stable Mixture error interface and must not be merged into JSON stdout. A report-write failure returns `1` and attempts an explanation on stderr; stdout may then be incomplete. Callers must handle that transport failure without assuming all exit-`1` output is parseable JSON.

## Report envelopes

The following fields describe the current wire contract. Use field names rather than JSON object-member order. Diagnostic arrays, plan arrays and completed output order have the deterministic rules described below. Messages, paths, adapter capabilities and timings are not portable byte snapshots.

| Command | Required top-level fields | Presence on failure |
|---|---|---|
| `validate` | `ok: boolean`, `diagnostics: array` | Same two fields; this original envelope has **no `schemaVersion` field**. |
| `inspect --plan` | `schemaVersion: 3`, `plan: object or null`, `ok`, `diagnostics` | `plan` is present and null. |
| `doctor` | `schemaVersion: 1`, `verdict: string`, `requested: object`, `adapter: object or null`, `device: object or null`, `computeProbe: string`, `readbackProbe: string`, `ok`, `diagnostics` | Adapter/device fields remain present; selected adapter evidence can survive device-request failure. `execution` is an **optional, omitted** checker report, present only for a completed probe. |
| `render` | `schemaVersion: 3`, `input: string`, `outputDirectory: string`, `planHash: string or null`, `context: object or null`, `execution: object or null`, `outputs: array`, `ok`, `diagnostics` | `planHash`, `context` and `execution` remain **present**, using null when unavailable. `outputs` contains only completed file writes. |

`schemaVersion`, plan `version`, source `documentVersion` and node versions describe different boundaries. The current inspection body is [RenderPlan v3](./render-plan.md): `version`, `documentVersion`, `size: [width,height]`, `materialOutput`, `passes`, `outputs`, `allocation`, `estimates`, `imageResources` and `hash`. Pass/resource IDs and byte estimates are nonnegative integers; `hash` is `sha256:` plus 64 lowercase hexadecimal digits. Core owns plan serialization and hashing. CLI requests with reordered channels compile to the same canonical plan; changing a meaningful override changes its hash, while slicing out an unused branch can remove its work.

## Diagnostic data and human context

Every diagnostic has string `code`, `stage`, `severity` and `message`. Optional string fields are `documentPath`, `nodeId`, `portId`, `parameterId` and `suggestion`. Absent optional values are **omitted**, not null. Nonempty `evidence` is a map of string keys to booleans, unsigned 64-bit integers or strings; empty evidence is omitted. Native source-error objects are not serialized. Explicit `sourceMessage` evidence retains relevant OS/parser/driver text without making that text a stable branching contract.

For example, missing warp displacement produces this context through `validate`, `inspect --plan` and `render`:

```json
{
  "code": "MIX_PORT_REQUIRED_CONNECTION",
  "stage": "validation",
  "severity": "error",
  "message": "Required input has no valid connection.",
  "documentPath": "missing-warp.mix",
  "nodeId": "sample",
  "portId": "displacement",
  "suggestion": "Connect one compatible output to this required input."
}
```

Human diagnostics now share a [small formatting function](../crates/mixture-cli/src/commands/human_diagnostics.rs) across validation, inspection, graph rendering, doctor and the built-in checker. It prints the original code/message, stage/severity, supplied document path, node, port, parameter, evidence and suggestion. Stage/severity in human text use Rust enum spelling (`Validation`, `Compile`, `Error`); JSON retains camel-case machine spellings. Human `inspect`/`render` now retain `port: displacement`, and invalid render overrides retain `parameter: cellsX` and public-ID evidence. Human text is for readers; consumers branch on JSON codes and context.

Diagnostics sort by lifecycle stage, severity, document/node/port/parameter IDs, code, message, evidence and suggestion; absent IDs sort first and evidence keys sort lexically. Duplicates remain observations. The [diagnostic contract](./diagnostics.md) defines the complete ordering, vocabulary and exact integer types. A budget rejection has numeric `configured`/`observed`; an invalid parameter's `observed` may instead be a **string containing the original JSON value**, such as `"0"`. Do not coerce evidence values into one numeric type.

## GPU and completed-file evidence

Doctor's `requested` object records `backend`, `powerPreference`, `softwareAdapter`, `effectiveBackends`, `requiredFeatures` and `requiredLimits`. Actual adapter identity includes `name`, `backend`, `deviceType`, numeric `vendor`/`device`, driver strings, supported limits and sorted features. Device evidence has acquired limits and enabled features. Capability-limit keys expose the pinned wgpu version; see [GPU context](./gpu-context.md) and the [dependency-exposure decision](./native-sdk.md). Adapter names, driver details and optional capabilities vary by host.

| Doctor state | `verdict` / `ok` | Probes and execution |
|---|---|---|
| Completed checker probe | `healthy` / true | `computeProbe` and `readbackProbe` are `passed`; `execution` is present. |
| Explicit `--skip-probe` | `unverified` / true | Both probes are `notRun`; `execution` is omitted. |
| Acquisition failure, including `--backend none` | `unhealthy` / false | Both probes are `notRun`; `execution` is omitted. No adapter/device exists for the explicit no-backend case. |
| Probe failure after acquisition | `unhealthy` / false | Selected context remains; failed/not-run probe states and the first structured diagnostic identify the failed stage. No successful execution report is invented. |

Doctor's checker `execution` describes width/height, format, dispatch, pass count, transferred/mapped/RGBA bytes, estimated GPU bytes and wall timings. Graph render's top-level `execution` instead has `planHash`, `adapter`, `size`, `passCount`, `pipelineCache`, `estimates`, `allocations`, `readbackBytes`, `mappedBytes`, `rgbaBytes` and `timings`. Cache/allocation/byte counters are nonnegative integers; `pipelineMs`, `executionMs`, `readbackMs`, `totalMs` are finite nonnegative CPU wall-clock milliseconds, not GPU timestamp measurements. See [allocation accounting](./development.md#2k-resource-evidence) for exclusions and lifetime limits.

A render's `context` is the acquisition snapshot, so it remains `unverified` with `notRun` probes even after a successful material render. It is not a second doctor verdict. The graph execution report carries the actual adapter and completed work. `context.ok` may be true while the outer render's `ok` is false after an encoding/write failure.

Each completed `outputs` entry has:

| Field | Type and meaning |
|---|---|
| `channel`, `kind` | Supported channel name and `color`, `scalar` or `normal` interpretation. |
| `source` | The plan's connected endpoint or versioned default input provenance. |
| `size` | `[width,height]` in integer pixels. |
| `encoding` | `rgba8-srgb` for color RGB with linear straight alpha; `rgba8-linear` for scalar/encoded-normal values with opaque alpha. |
| `path` | Caller-relative or absolute PNG path, following `--out`. |
| `writtenBytes` | Completed compressed PNG file length, not raw pixel-buffer length. |

PNGs use 8-bit RGBA. Color files carry sRGB metadata; linear scalar/normal files carry gamma 1.0. Scalar values replicate to RGB; encoded normals remain linear XYZ bytes. Requested files are ordered by the material contract: `baseColor`, `normal`, `roughness`, `metallic`, `height`, `ambientOcclusion`, `opacity`, `emissive`. Only selected channels are emitted. Source metadata and encoding come from the public plan/renderer; the CLI adds paths, PNG encoding and write completion.

## Failure boundaries and partial writes

| Failure point | Plan hash | Context | Graph execution | Completed outputs |
|---|---|---|---|---|
| Source read/decode/validation or request compilation | null | null | null | empty |
| GPU acquisition | present | failed context | null | empty |
| GPU execution/readback | present | acquired context | null | empty |
| Output-directory creation or first PNG encoding/write | present | acquired context | present | empty |
| Later PNG encoding/write | present | acquired context | present | completed canonical prefix |

The CLI compiles before acquiring a GPU and completes GPU work before creating/writing output files. Output-directory creation happens after rendering. It creates missing directories and replaces same-named files; unrelated files remain. There is no atomic directory replacement or rollback. A failed target can be partially written or truncated by file I/O even though it is absent from `outputs`; only entries in that list are completed writes.

The independent test creates a directory at `normal.png`, then requests four channels. `baseColor.png` completes, writing `normal.png` fails with `MIX_ENCODING_FAILED` at `encoding`, and the report retains the completed baseColor entry, full execution evidence and the failed `outputPath`. Roughness/height files are not written. The same case in human mode names the completed file and failure context. No pixel baseline is updated. Consumers needing result freshness or controlled publication will use the generation-specific directories introduced in PR-014.

## Reproduce and acceptance limits

```bash
cargo xtask test-consumer
cargo test --locked --all-features -p mixture-cli
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

`test-consumer` builds the real CLI and explicitly invokes the independent CPU contract test. Its 32 child invocations cover successful validation/inspection, deterministic ordering, overrides/slicing, human/JSON missing warp input, unknown port, malformed input, missing source files, invalid overrides, numeric budgets, disabled-backend acquisition and usage errors. Ordinary checks do not acquire a GPU. `gpu-smoke` additionally invokes the independent GPU contract test on the selected adapter, checking doctor states, three successful render requests, decoded PNGs and partial writes in both report modes.

The test fixture uses `serde_json`, the standard process/filesystem APIs and a dev-only PNG decoder. It imports no Mixture Rust API or CLI-private modules and uses only its own embedded source fixtures. The application remains a separate Cargo workspace. Test completion receipts and raw child stdout/stderr are retained in fresh directories; a skipped or incomplete test cannot satisfy `test-consumer`/`gpu-smoke`. The [example guide](../examples/native-consumer/README.md) gives standalone invocation details, and [PR-012 evidence](./evidence/pr-012/README.md) records local results.

No JSON field, null/omission rule, enum spelling, source/plan version or exit-code policy is changed in PR-012. Tests pin relevant semantic fields rather than volatile adapter/timing snapshots. Current data is verified locally; packaged consumption, stale-result scheduling, full compatibility/release policy and deferred remote platform CI remain their later M4 gates.

PR-013 extends the diagnostic vocabulary with `MIX_GPU_DEVICE_LOST` and `MIX_GPU_OUT_OF_MEMORY`; report envelopes and exit codes are unchanged. Failure diagnostics can carry adapter, delivered loss and released-allocation evidence while retaining the first error and its phase. See the [GPU failure contract](./gpu-failures.md) for strict-decoder compatibility and classification limits.

PR-014 adds a separate consumer test using fresh generation directories. The product CLI still writes files sequentially; the consumer selects only the newest completed directory, removes obsolete/partial owned directories and propagates cleanup failures. See [the scheduling contract](./stale-results.md).

PR-015 runs the same independent 32-case CPU and ten-case GPU process contracts against a CLI built solely from its local Cargo archive and archived library peers, with a working directory outside the producer repository. This adds [package evidence](./package-consumption.md), without changing the product report envelopes or exits.

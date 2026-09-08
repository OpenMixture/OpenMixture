# M3 public diagnostic evidence

English | [简体中文](./README.zh-CN.md)

The 16 CPU-only probes confirm useful structured diagnostics and the documented exit-code split on source revision `e9dd03bb272a29de6243aec79a3450b47b90530e` (`e9dd03b`). They also reproduce a human-report defect: **`inspect` and `render` omit the missing warp input's `portId`, while JSON and human `validate` identify `displacement`.** No runtime change or GPU execution is part of this evidence.

## Captures and provenance

- [Original debug capture](./original-debug/summary.json): all 49 files from `tmp/m3-review/diagnostics/` were copied without changing any bytes. The original harness recorded the binary hash, arguments and exit codes, but did not record a source revision or build flags; these are not retroactively attributed to that run.
- [Release capture](./release/summary.json): 16 probes rerun with the already-built release CLI; 50 files copied from a new temporary directory without normalization. The checkout revision, relevant source status, fixture hashes, helper hash, exact arguments, declared build command/profile, executable hash before/after, raw stream hashes and exit codes are recorded. Relevant source files were clean, and the executable and fixtures remained unchanged.
- [Copy integrity manifest](./capture-files.json): SHA-256 and byte length for every copied file in both captures. Empty stderr files and trailing newlines are retained.
- [Release build log](../performance/release-build.log): the root task built the executable with `cargo build --release --locked --all-features -p mixture-cli` on the recorded revision. The helper never builds. Its profile/build-command fields record caller declarations; the executable hash identifies the binary, not an embedded source-revision attestation.

| Capture | Binary profile | Executable SHA-256 |
| --- | --- | --- |
| `original-debug` | debug | `43e8e26cd69fba66ac492fe14122e73ff1d3f27a2c5a9ba73b075627e5603d16` |
| `release` | release, all features | `56ea817b5716e8db38dfe519e07f5b1f283708bcbcd3fa2eb0a8b21952c7136d` |

Each probe has a `<name>.json` record, complete `<name>.stdout`, and complete `<name>.stderr`. Capture paths, absolute missing-file paths and native OS error strings are intentionally preserved. A replay on another machine need not have byte-identical paths or driver/OS text.

## What the probes establish

| Probe | Expected exit | Observation |
| --- | ---: | --- |
| `help` | 0 | Public CLI entry points are listed. |
| `unknown-port-json` | 2 | Node, missing output port and downstream required input are identified. |
| `unknown-port-human` | 2 | Human `inspect` omits both port identifiers. |
| `missing-warp-json` | 2 | `MIX_PORT_REQUIRED_CONNECTION`, node `sample`, port `displacement`. |
| `missing-warp-human` | 2 | Human `inspect` identifies the node but omits the port. |
| `invalid-parameters` | 2 | Parameter IDs, expected type/range and observed values are retained. |
| `cycle` | 2 | The actual closed path `a -> b -> a` is reported. |
| `duplicate-key` | 2 | Parse-stage code, line, column and duplicate-field source message are retained. |
| `invalid-exposed` | 2 | Public binding and invalid target parameter are identified. |
| `unknown-override` | 2 | Compile-stage failure identifies the unexposed public ID. |
| `budget` | 2 | Both axes report configured `2048` and observed `4096`. |
| `doctor-none` | 1 | Adapter-policy failure occurs with no adapter/device, no effective backend and both probes `notRun`. |
| `missing-file` | 1 | File-I/O code, supplied path and OS failure text are retained. |
| `usage-json` | 2 | Missing `--plan` is documented usage text on stderr, with empty stdout. |
| `missing-warp-render-human` | 2 | Human `render` omits the port; no output directory is created. |
| `missing-warp-validate-human` | 2 | Human `validate` retains `port: displacement`. |

All 16 release exit codes match their expectations. This does not mean every human diagnostic is sufficient: the missing-port observations remain `false` in the release summary. JSON behavior agrees with the [diagnostic contract](../../../diagnostics.md), [format guide](../../../file-format.md), and [doctor policy](../../../gpu-context.md).

## Reproduce without acquiring a GPU

Use Python 3.9 or newer, Git, this checkout and an already-built CLI. From the repository root, choose an output path that does not exist:

```bash
python3 docs/reviews/m3/diagnostics/reproduce.py \
  --binary target/release/mixture \
  --profile release \
  --build-command 'cargo build --release --locked --all-features -p mixture-cli' \
  --out tmp/m3-review/diagnostics-release-rerun
```

The optional `--build-command` string is provenance only and is never executed. The helper records the current checkout revision; use source and a separately built binary from `e9dd03b` when comparing with this historical capture. It cannot infer which sources produced an arbitrary supplied executable.

The [helper](./reproduce.py) runs probes sequentially and copies subprocess streams as raw bytes. It invokes only the supplied CLI and read-only Git metadata commands. `doctor` explicitly uses `--backend none`; the invalid `render` probe also has `--backend none`. This render argument is an intentional additional guard compared with the original debug capture. The other probes use CPU-only validation, inspection, help or usage paths. The missing-file path is generated inside the fresh output directory and is not created.

Existing output paths, including symlinks, are rejected. All output paths inside this durable evidence directory are rejected even when new. The helper does not overwrite, stage or commit evidence. After a failure it retains available streams and a failure record; choose another new directory for a retry. Exit `0` means the capture completed with expected command exits, unchanged inputs/binary and the explicit no-backend checks; human-context observations are findings and do not require the existing bug to persist.

## Limits and smallest follow-up

This is a public CLI failure-path sample, not the full parser/graph test suite, an independent native SDK consumer test or a GPU health result. No real adapter acquisition, device/shader/pipeline failure, execution, readback, device loss or remote CI was exercised. The no-GPU statement follows the explicit disabled-backend policy and reviewed command paths; it is not hardware telemetry. Historical debug `gpuInitialized: false` has that same evidentiary limit.

The smallest M4 fix is to preserve document, stage, node, port and parameter context in human `inspect`/`render` output, using the same public diagnostics already available to JSON. Add focused CLI regression assertions with the missing-warp and unknown-port fixtures; preserve the existing JSON contract and exit codes. There is no need to add graph repair, a new diagnostic taxonomy or a new renderer for this issue.

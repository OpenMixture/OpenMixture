# PR-012 CLI contract evidence

English | [简体中文](./README.zh-CN.md)

**Local acceptance, 2026-09-08:** human diagnostics retain document, stage/severity, node, port and parameter context. Independent process tests verify existing CLI reports, exit codes, actual PNGs and partial writes. The implementing revision is the PR-012 commit containing this record, parent `244b384aba257ab62ab225385ad6af1960775517`, on `codex/pr-012-cli-contract`.

The [CLI contract](../../cli-contract.md) documents the reviewed behavior. Product changes are a shared human formatter and its five callers; parsing, JSON serialization, core/GPU semantics, shaders and product dependencies are unchanged. The separate consumer adds a dev-only PNG decoder using versions already resolved in the product lockfile. It adds no pixel executor or production binding.

## Checks

| Verification | Evidence |
|---|---|
| Reproduce missing context before the fix | [Before log](./checks/context-before.log): four expected failing regressions, including omitted warp port and render override parameter; JSON context already passes. |
| Fix the same regressions | [After log](./checks/context-after.log): all five pass. |
| Existing CLI tests | [CLI tests](./checks/cli-tests.log) pass; GPU tests remain explicitly ignored here. |
| Tooling guards | [Consumer tooling tests](./checks/xtask-consumer-tests.log) reject skipped/incomplete receipts and wrong exits. |
| Independent CPU consumption | [Command](./checks/test-consumer.log), [test output](./checks/consumer/cli-tests.stdout.log), [completion status](./checks/consumer/status.json): 32 child CLI calls pass alongside five public-Rust unit tests and three process tests. |
| Final repository check | [check.log](./checks/check.log) passes, including CPU consumer checks and local documentation links. |
| Paired documentation | [Pair check](./checks/doc-pairs.json): 54 English/Chinese pairs, no missing translation or executable-block mismatch. |
| Metal GPU smoke | [Command](./gpu-smoke/metal/command.log), [CLI test output](./gpu-smoke/metal/consumer/cli-tests.stdout.log), [completion](./gpu-smoke/metal/consumer/status.json): Apple M5, Metal; 10 independent CLI calls pass. |
| Software GPU smoke | [Command](./gpu-smoke/software/command.log), [CLI test output](./gpu-smoke/software/consumer/cli-tests.stdout.log), [completion](./gpu-smoke/software/consumer/status.json): SwiftShader Device (LLVM 10.0.0), Vulkan; the same 10 calls pass. |

Both GPU runs also pass the existing checker/graph regressions and PR-011 public-Rust output-ownership probe. Selected raw reports and test logs are archived here; full smoke snapshots remain in `tmp/pr-012/gpu-smoke-{metal,software}/`. Software source is clean at pin `694585a05946e1ed49b6bd577ca6537cbb57f025`, using the configured loader under `tmp/pr-004/swiftshader-build/bin/`. This is local macOS evidence, not remote Linux/Windows CI.

## What the independent consumer checks

The [test program](../../../examples/native-consumer/tests/cli_contract.rs) has no Mixture Rust/private imports. It embeds its own sources, writes them to a fresh case directory, then spawns the built CLI with that working directory and explicit paths/options. CPU cases cover successful validation/inspection, canonical plan ordering, overrides/slicing, human and JSON missing-warp context, unknown ports, malformed input, missing files, numeric budgets, invalid overrides, no-backend acquisition and usage errors with `--json`.

Each GPU suite checks healthy/skipped doctor, then three 65×3 render requests: four channels at `repeat=16`, four at `repeat=4`, and roughness only. It compares plan/adapter/source/encoding metadata, decodes completed PNGs, checks literal color/straight-alpha/scalar/normal values and changed override pixels, and validates execution/counter/timing types. No CPU pixel algorithm is implemented.

For partial writes, a directory at `normal.png` blocks the second canonical output. Both JSON and human modes return `1`, retain the completed baseColor file and identify the encoding failure. JSON lists exactly that completed file with its byte length and preserves full graph execution evidence. Roughness/height are absent and an unrelated marker remains. This proves the tested failure behavior; file export is not atomic and a failed file I/O operation may leave a partial target.

[runs.json](./runs.json) locates each selected CLI capture. Its status contains exact arguments/exits and raw-stream filenames; owned inputs and actual PNGs are retained beside it. Original absolute working directories and stderr/OS strings are preserved. [capture-manifest.json](./capture-manifest.json) records final source and archive hashes; timestamps and binary hashes are capture evidence, not embedded build attestations.

## JSON compatibility

The unchanged [M3 diagnostic helper](../../reviews/m3/diagnostics/reproduce.py) was rerun against the new debug/all-features CLI. [The replay](./diagnostic-replay/summary.json) completes all 16 probes with expected exits, unchanged binary/fixtures and all missing-port observations now true. [Comparison results](./json-compatibility.json) show nine JSON stdout reports byte-identical to the historical M3 release capture. The tenth differs only in the intentionally generated missing-file `documentPath`; all other data agrees. Raw captures were not normalized.

`validate` retains its original unversioned `{ok, diagnostics}` envelope. Other reviewed commands retain `schemaVersion: 1`; null/omission rules and exit codes are unchanged. The independent GPU tests assert semantic fields and finite timings, rather than comparing volatile reports byte for byte.

## Reproduce

```bash
cargo xtask test-consumer
cargo test --locked --all-features -p mixture-cli
cargo test --locked -p xtask consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

Prepare/verify the software loader using the [GPU guide](../../gpu-context.md). Standalone independent test commands are in the [consumer guide](../../../examples/native-consumer/README.md). Ordinary checks require no GPU; completion guards prevent skipped tests from appearing successful.

**Out of scope:** JSON/version/exit-policy redesign, transactional export, device-loss/OOM classification (PR-013), stale-result scheduling (PR-014), packaged consumption and release policy (PR-015), publication, remote push/CI, bindings and services. Deferred remote gates remain open.

# M3 review evidence

English | [简体中文](./README.zh-CN.md)

This directory supports the [six-question M3 review](../../m3-review.md) and [M4 train](../../../M4_PRS.md). It captures local evidence from PR-010 source `e9dd03bb272a29de6243aec79a3450b47b90530e`. It does not implement M4 or close deferred remote CI gates.

| Evidence | Scope |
|---|---|
| [acceptance.json](./acceptance.json) | Read-only verification of current human acceptance bindings, retained 1K gates and 2K traces. |
| [Diagnostic probes](./diagnostics/README.md) | Sixteen CPU-only public CLI probes, original/debug and release captures; includes the human port-context defect. |
| [Independent Cargo probe](./native-consumer/README.md) | CPU path executed, public GPU call shape compiled; GPU execution and packaged consumption remain unverified. |
| [Metal performance](./performance/metal/summary.json) | Five workloads × three sequential release renders, including output slicing and an exposed override. |
| [Software performance](./performance/software/summary.json) | Three sequential release wood renders with pinned SwiftShader. |
| [Release build](./performance/release-build.log), [toolchain](./performance/toolchain.txt) | Actual build output, Rust toolchain and checked SwiftShader source revision. |

## Performance method and retained files

[measure_cli.py](./measure_cli.py) invokes an already-built CLI from a newly created working directory, with copied `.mix` sources. It times each process through exit, captures complete stdout/stderr, checks the selected adapter, and then decodes PNGs with Pillow/NumPy to compare them to accepted baselines. These Python libraries inspect rendered output; they do not execute graph pixels. No baseline or acceptance file is modified.

Each policy's summary records the binary/helper/runtime-source/input/contract/manifest hashes, host and library versions, exact command/cwd, per-trial elapsed time, renderer timings, pass/cache/allocation metrics, and per-channel decoded comparisons. Referenced `.stdout`/`.stderr` are retained unchanged beside it. Generated PNGs remain in ignored `tmp/`; their hashes and comparisons are retained, and the accepted baseline PNGs stay in `fixtures/materials/`. Regenerate candidates with the helper to inspect them. This review introduces no visual change requiring a new appearance decision.

The recorded executable was built from unchanged PR-010 runtime sources with the command below. The helper records the caller's release-profile claim; an arbitrary binary's source/profile cannot be inferred from its hash. For historical reproduction, build the recorded source revision and compare the saved runtime hashes. No slow trials or warmups were discarded. Mixture caches start empty per process; OS/driver caches are uncontrolled. Three trials on one host are descriptive evidence, not a statistical performance gate. `--version` is a minimal-call proxy, not isolated render startup.

In these timing summaries, `comparisons[].changedPixelRatio` means the fraction of pixels whose maximum RGBA byte difference is **greater than the material's `pixelThreshold`** (an above-threshold ratio). It is not the ratio of all changed pixels used by the separate golden report field of the same name. A positive `meanAbsolute` with a zero ratio is valid when every difference stays at or below that threshold. These original captures and the helper schema are preserved together.

## Reproduce

Use Python 3 with Pillow and NumPy available, the repository Rust toolchain, and an explicit adapter. From the repository root, use fresh output paths:

```bash
cargo build --release --locked --all-features -p mixture-cli
python3 docs/reviews/m3/measure_cli.py \
  --repo . --binary target/release/mixture \
  --out tmp/m3-review/performance-metal-rerun \
  --backend metal --expect-adapter 'Apple M5'
```

For another hardware host, change the expected adapter string and explicit backend as appropriate; the helper currently accepts Metal or Vulkan. The expectation is checked against both doctor and render reports. These are GPU commands, outside ordinary `cargo xtask check`.

Prepare the pinned macOS loader using the [GPU setup guide](../../gpu-context.md), then run:

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/swiftshader/source" \
python3 docs/reviews/m3/measure_cli.py \
  --repo . --binary target/release/mixture \
  --out tmp/m3-review/performance-software-rerun \
  --backend vulkan --software --expect-adapter SwiftShader --wood-only
```

The retained run used the earlier local loader at `tmp/pr-004/swiftshader-build/bin`; its environment is preserved in the software summary. The setup guide creates the fresh-checkout paths used above. Pin and verify the documented SwiftShader revision before interpreting software results as the same baseline policy; this helper records the supplied source path but does not itself attest the loaded driver's source build. Software pixels must match exactly, hardware pixels must satisfy each material's unchanged declared tolerance. Full structural material gates are audited separately in `acceptance.json`, not recomputed by this timing helper.

The helper refuses an existing output directory, fails on CLI/adapter/metadata/pixel mismatch, and leaves available receipts for inspection. A failed run is not successful evidence; choose a new directory for a retry. The observed medians and their interpretation are in the [review](../../m3-review.md#4-does-1k-performance-support-interactive-consumers).

## Review verification

The [initial repository check](./checks/initial-repository-check.log) exposed a test-only temporary-directory collision: concurrent golden tests could receive the same process ID plus wall-clock timestamp. The review adds an atomic sequence suffix in the existing test helper. It changes no product code, shader, baseline or allocation behavior. The [focused golden tests](./checks/golden-tests.log) and [final repository check](./checks/repository-check.log) pass after that correction. Raw logs and subprocess streams retain their original whitespace.

The repository check covers formatting, dependency boundaries, Clippy, CPU tests, Rustdoc and 105 Markdown-file links; explicit GPU evidence is the release capture above and the separately audited PR-010 material reports. The independent consumer's locked/offline CPU run and 49 active documentation language pairs were checked separately. Historical source bindings in the acceptance audit refer to `e9dd03b`, before the test-helper correction.

# Local package consumption

English | [简体中文](./package-consumption.zh-CN.md)

M6B-03: the [shared CPU asset codec](./m6b-03-cpu-assets.md) provides an independent `mixture-asset` public API, covered by source and isolated archive consumers; Rust 0.5.0 is unpublished.

PR-015 implements `cargo xtask package-check`. It creates actual local Cargo archives, verifies them outside the producing repository, builds the independent Rust consumer and the CLI from those archives, and exercises their CPU contracts. Explicit `gpu-smoke` repeats package verification and adds real packaged GPU/CLI consumption. This establishes the local M4 package gate without publishing a crate or distributing a binary. [Acceptance evidence](./evidence/pr-015/README.md) records the exact runs.

## Archive and dependency boundary

M6A-03 also runs the public prepared-image GPU consumer against extracted archives. Verification cleans the five local packages from the shared build target before building, preserving only external dependency caches: archive mtimes must not allow stale same-version runtime code to pass. See the [Native resource implementation](./m6a-03-native-resources.md) for the original resource contract; the current asset contract is linked above.

The four product packages (core, asset, wgpu, CLI) are version `0.5.0` and `publish = false`. Workspace path dependencies require exactly `=0.5.0`; Cargo's normalized archive manifests retain that version and remove producer paths. The consumer's repository manifest uses three source-path library dependencies.

| Package contents | Verification |
|---|---|
| `Cargo.toml` and Cargo's original-manifest copy | Version, disabled publication, target paths and dependency requirements are checked through resolved metadata. |
| `src/**` | Extracted file hashes must match the producer package inputs. Core/asset/wgpu unit tests and Rustdoc examples build inside the isolated workspace. |
| wgpu `shaders/**` | All eleven embedded kernels ship. Removing packaged `constant.wgsl` must produce a real compilation failure; the bytes are restored afterward and all extracted files rechecked. |
| Both README languages and both license texts | Required archive entries; license bytes must match the repository's MIT/Apache-2.0 texts. |
| wgpu `src/testdata/*.mix` | Three source-embedded unit-test inputs remain package-local and byte-identical to canonical constant-scalar/transform-2d/warp fixtures. |

Repository integration tests, full node/material fixture collections and historical evidence are not package runtime dependencies. Source-embedded unit tests are included and executed. M6B-03 adds only the optional CPU asset crate; no external dependency, shader implementation or pixel algorithm is added.

The checker uses `cargo package --locked --offline --no-verify --exclude-lockfile --allow-dirty` for all four packages. Cargo's usual verification and package-lock generation would try to resolve unpublished peers through a registry; these steps are deliberately replaced by the explicit local verifier below. `--allow-dirty` captures reviewable uncommitted implementation work; source/archive hashes, rather than a clean-VCS claim, identify it. Cargo documents manifest normalization and these switches in [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html).

These local source archives intentionally omit `Cargo.lock`. They are not advertised as registry-installable archives or inputs to `cargo install --locked`. The verifier ships its resolved lock as separate evidence and retains both committed repository/consumer locks unchanged. Actual publication remains disabled; any future distribution must review its final lock/registry/install policy separately.

## Isolated verification sequence

Manifest identity and target containment use canonical filesystem paths, including Windows verbatim-path normalization. Different spellings of the same extracted file are accepted; missing files or targets outside their extracted package are rejected.

1. Snapshot source/build inputs, check duplicated licenses and the three unit fixtures, and fetch only the committed lock's public dependencies.
2. Generate archives, validate member paths/required files, extract into a fresh OS temporary directory **outside** the producer repository, and compare extracted source bytes with the snapshot. Archives themselves are never rewritten.
3. Copy the independent application's own source, tests and `.mix` input into that directory. Only its staging manifest changes: replace the three producer paths with exact version requirements and join a disposable verification workspace. No source or runtime behavior is rewritten.
4. Resolve core/asset/wgpu with `[patch.crates-io]` pointing solely at their extracted directories. All four normalized packages plus the application are members of this isolated workspace. No Mixture registry substitute or producer path is accepted; the resolved package and target paths must match the extraction tree.
5. Seed a disposable lock from the committed workspace lock. One offline metadata pass may add the local application and prune unused packages; external **name/version/source/checksum** identities must remain a subset of the committed pins. All subsequent builds, tests, docs and process-contract tests run `--offline --locked`.
6. Build and test the isolated workspace, including source-embedded unit tests, independent CPU tests and Rustdoc examples; build Rustdoc with warnings denied. A retained compiler cache under `target/package-consumer` holds build artifacts, not source assets. This is filesystem/source-resolution isolation, not an OS security sandbox or a cache-free build claim.
7. Delete the packaged constant shader, run a real failing check, restore it even after a command failure, and ensure every extracted package file still matches its original hash. Missing assets must not pass through a cached successful build.
8. Run the packaged Rust executable in an empty external working directory and the packaged CLI through 32 CPU child-process cases in a fresh external directory. GPU mode also checks owned real Rust pixels after renderer drop and ten CLI GPU cases with decoded PNGs, adapter evidence and partial writes.
9. Reject source drift during the run. Preserve archives, normalized manifests, lock, hashes, raw streams and receipts; remove only the successful run's owned temporary extraction tree. Failure leaves incomplete status and the staging path for diagnosis.

The local patch is a dependency-resolution mechanism, not copied implementation code or a fallback executor. Package metadata proves which extracted crates were used. The inner application's `packagedCratesValidated: false` remains unchanged: a running application cannot attest its own build origin. The outer verifier sets `packagedCratesValidated: true` only after checking provenance and all required consumption tests; GPU completion is reported separately.

## Commands and evidence

```bash
cargo xtask package-check
cargo xtask test-consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

Prepare the loader using the [GPU guide](./gpu-context.md). The verifier requires Cargo/Rust from the repository toolchain and a `tar` command supporting `-tzf`/`-xzf`/`-C`; absent tools fail explicitly. `package-check` takes no GPU flag and ordinary `check` does not acquire a GPU. `check` now includes both source-consumer and package checks, so the existing CPU CI matrix runs the same gate. GPU CI retains package evidence with smoke/material/trace artifacts; these workflow edits do not constitute remote passes.

`tmp/package-check/latest.json` references a fresh `cpu-*` or `gpu-*` attempt. Its `status.json` begins incomplete and only becomes successful after verification. Successful status contains `packagedCratesValidated`, `gpuExecuted`, the removed external staging path, and a `verification` object linking reports and recording archive/binary hashes and lock policy. `phase.json` and per-command receipts preserve exact arguments, working directory and exit code. Missing-shader logs intentionally record a nonzero exit.

`gpu-smoke` invalidates `tmp/gpu-smoke/package-status.json` before execution and only links a successful package attempt after its packaged GPU checks pass. CI retains `tmp/package-check/`; build caches stay under ignored `target/`. Raw paths describe the original temporary working directories, which no longer exist after success; CLI evidence and PNGs are copied before removal.

Out of scope: publication, registry upload/download of Mixture packages, installation or binary distribution, WebAssembly, a new binding, material changes and remote execution. The [compatibility record](./compatibility.md) and [release checklist](./release.md) separate local acceptance from remaining release gates.

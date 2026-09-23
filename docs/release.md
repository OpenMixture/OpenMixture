# M4 release status and checklist

English | [简体中文](./release.zh-CN.md)

**Working MAT-02b candidate (2026-09-23):** Source manifests select unpublished Rust 0.7.0 / browser 0.7.0-alpha.0 for scalar-morphology@1, admitted by the [integrated contract](./mat-02-layered-weathering.md). The catalog has sixteen types/fourteen kernels. Downstream exhaustive Rust matches must handle ScalarMorphology. Node/public-consumer qualification and painted-metal material acceptance remain pending; the last qualified MAT-01 baseline is 0.6. Formats, existing semantics, historical failures, migration policy and publication remain unchanged.

**MAT-01 implementation integration (2026-09-23):** Rust 0.6.0 / browser 0.6.0-alpha.0 nodes, brick fixture and qualification tooling are integrated through PRs #50–52 at `0611d7273e368b12628bc827627779ea338dbb92`. All six PR #52 pre-merge checks pass; [the integration record](./evidence/mat-01/integration/README.md) separates their tested source from post-merge main verification. The [human decision](./evidence/mat-01/human-decision.json) and six passing post-merge checks complete MAT-01 qualification within its recorded software/GT 1030 scope; the 0.5 baseline below is historical. No package is published and no format or migration rule changes.

**Qualified baseline (2026-09-22):** [M6-B #45](https://github.com/OpenMixture/OpenMixture/pull/45) and [NUM-01 #40](https://github.com/OpenMixture/OpenMixture/pull/40), including their prerequisite PRs #39 and #41–44, are merged into `main` at `ac219c901e52e3f079c16a931ed4463465756ba5`. All six required checks passed on that commit: [three-platform CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35719781509), [software GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35719781245), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35719781222), and [Chromium materials](https://github.com/OpenMixture/OpenMixture/actions/runs/35719781440). Source versions at that checkpoint were **Rust 0.5.0 / browser 0.5.0-alpha.0**, both unpublished. The latest recorded published browser package is **0.3.0-alpha.0**; Rust crates remain unpublished.

**Windows qualification scope:** the [old v1 resource/asset inputs](./evidence/m6b-05/README.md) retain the failed normal maximum of 8/255 against the ≤1/255 gate. Explicitly migrated `fractal-noise@2` **value** inputs pass the recorded GT 1030 Vulkan/DX12-versus-Chrome resource and Scalar comparisons, with height/normal maximum difference **0**; see [NUM-01 evidence](./evidence/stable-noise/README.md). These are different node versions, not conflicting verdicts. Portable asset regression still uses frozen v1 inputs; existing documents are not automatically migrated. Passing software CI does not qualify arbitrary Windows graphs, GPUs, cellular or warp. Earlier 0.4 candidate evidence retains its original build identity.

**Current release (2026-09-22):** [Browser 0.3.0-alpha.0](./evidence/npm-030-alpha/README.md) is published from the main archive that passed all six checks. API schema 2 / plan v2; Rust crates remain unpublished. That published version retains the v1 Windows resource-normal limitation; the v2 repair above is not yet published.

**M6A-05 qualification, 2026-09-21:** [Retained acceptance](./evidence/m6a-05/README.md) closes comprehensive qualification for the recorded Linux software matrix: eight resource channel comparisons are byte-exact, Scalar and three-material regressions and all six required checks pass. Windows hardware parity remains failed and outside accepted coverage; the ≤1 gate is unchanged. The 0.3.0 Rust source / 0.3.0-alpha.0 API-schema-2 browser candidate are unpublished. Implementation PR integration and release remain separate.

**Historical checkpoint (2026-09-20):** Native M4/M4.1, bounded M5 and the recorded Studio MVP are accepted. Ordinary Windows Chrome/Edge/Firefox each pass the [v2 material matrix](./evidence/browser-quality-v2/README.md) for the recorded existing archive. [npm Alpha publication and exact registry consumption](./evidence/npm-alpha/README.md) are complete within the recorded scope. Rust crates remain unpublished. [Alpha closeout](./browser-alpha.md) records the delivery and enforced browser checks; hosted trial redeployment remains separate and M6 does not start.

The dated checkpoints below retain their status at the time; they are not the current backlog.

**M5 acceptance, 2026-09-15:** [Browser acceptance](./evidence/m5-05/README.md) closes M5-05 for the recorded macOS/Linux Chromium matrix: each environment passes 11 post-freeze 1K cases/44 channel comparisons, semantics/quality gates and 12 additional lifecycle renders, alongside independent product isolation, 28 browser contracts and normal production static deployment. Complete pixels and provenance are retained. Alpha readiness is bounded to tested coverage; npm remains unpublished and Studio/M6 require separate decisions. Earlier dated entries below retain their historical status.

**M4 accepted locally and remotely, 2026-09-12; packages remain unpublished.** PR-011–015 and the CI tooling fixes pass the documented acceptance matrix. The previously deferred clean-checkout CPU and Linux SwiftShader gates are now closed; see [remote CI evidence](./evidence/remote-ci/README.md). All product packages remain pre-alpha `0.1.0` with publication disabled. The recorded CI repair pushed the implementation and fixes without a merge or release tag. Subsequent default-branch, PR, and checkpoint work follows [M4.1 repository governance](./governance.md); it does not publish packages, installers, or binary distributions. [PR-015 evidence](./evidence/pr-015/README.md) retains the earlier local assessment.

## M4 exit assessment

| Exit criterion | Local evidence and result |
|---|---|
| Independent native consumer | PR-011's application uses public parse/validate/compile/render APIs, explicit overrides/channels/context and CPU-owned output after renderer drop. |
| CLI boundary | PR-012 verifies report envelopes, exits, diagnostics and actual PNG/partial-write behavior from separate working directories. |
| Failure contract | PR-013 adds typed device-loss/OOM classification, context-owned loss state, first-error preservation and guarded cleanup. OOM classification uses synthetic typed errors; destruction tests use real GPU devices. |
| Stale work and retained state | PR-014 bounds work to one active/one pending request, publishes only the newest generation, cleans owned directories and verifies the nine-kernel cache/lifetime bound. |
| Package consumption | PR-015 builds and tests real normalized Cargo archives in a temporary workspace outside the repository, checks locked external dependencies, rejects a missing embedded shader and runs packaged Rust/CLI CPU and GPU consumers. |
| Contracts and materials | Paired public docs agree with the tests. Ceramic, leather and wood 1K machine/golden checks and the ranked 2K trace pass under both local adapter policies without changing accepted baselines. |

The separate consumer can load `.mix`, validate, override exposed parameters, request channels, render, and consume outputs/metrics using public APIs resolved from the actual local package contents. No producer-private import or outer repository runtime asset is required. [Package resolution and limitations](./package-consumption.md) describe the local patch and separate verification lock precisely.

The local implementation and documented remote matrix complete M4 acceptance and close the deferred M0/M1 clean-checkout/platform gates. This does not authorize M5. Choose the next train explicitly; distribution still requires the release checklist below.

## Verified host/backend matrix and open gates

| Environment | Recorded status |
|---|---|
| Current local macOS/aarch64, CPU | `package-check`, source consumer, repository checks, isolated package unit tests/Rustdoc and 32 packaged CLI CPU cases pass. This is an existing local working tree, not remote clean-checkout evidence. |
| Same host, Apple M5 / Metal | Full GPU smoke, source and packaged public-Rust/CLI consumption, all three 1K materials and largest 2K trace pass. |
| Same host, pinned SwiftShader Device (LLVM 10.0.0) / Vulkan / CPU adapter | The same GPU/package/material/trace gates pass; source pin `694585a05946e1ed49b6bd577ca6537cbb57f025` is recorded. This is not the Linux CI result. |
| Remote Linux/macOS/Windows CPU matrix | **Passed on `8b43c84`.** [Three-platform run](https://github.com/OpenMixture/OpenMixture/actions/runs/34622547271) executes `cargo xtask check`, including isolated packages, and retains evidence. |
| Remote Linux pinned SwiftShader GPU/material job | **Passed on `8b43c84`.** [Linux run](https://github.com/OpenMixture/OpenMixture/actions/runs/34622547778) includes serial GPU tests, packaged consumption, all three 1K materials and the 2K trace. [Evidence and concurrency limits](./evidence/remote-ci/README.md) retain the earlier failed parallel run. |
| Other native hardware/drivers, including Windows/DX12 | **Not certified by this matrix.** Exposed adapter options are not proof that every supported backend/device passed the acceptance matrix. |

The current largest-case selection is wood/default at 2048×2048 with eight passes. The budget is the existing 512 MiB descriptor limit; reports distinguish estimated/recorded peak, cumulative bytes, release and zero reuse. This is a bounded correctness/resource trace, not a new latency benchmark or a physical VRAM measurement. [Trace semantics](./development.md#2k-resource-evidence) and the raw final evidence define its scope.

## Reproducible verification

```bash
cargo xtask package-check
cargo xtask test-consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask golden check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask trace-2k
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask golden check
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask trace-2k
```

The macOS loader paths are local preparation paths, not portable installation instructions; use the [GPU guide](./gpu-context.md) or the pinned Linux CI setup for the intended host. `golden check` never accepts new baselines. Do not substitute `golden update` to repair a failing release check.

## Checklist before an actual release

- [x] Preserve independently reviewable PR-011–015 local commits, matching bilingual docs and local evidence.
- [x] Verify actual local package source/assets/licenses, exact peer versions, independent consumers and missing-asset rejection.
- [x] Record source/lock/archive identity and the tested host/backend policies; retain accepted material pixels unchanged.
- [x] Obtain remote clean-checkout CPU and pinned-software GPU/material/trace results on the same revision; see [accepted CI evidence](./evidence/remote-ci/README.md).
- [ ] Select the intended published version and support scope; review API/dependency/wire compatibility and any required migration using [the compatibility record](./compatibility.md).
- [ ] Review final registry/package-lock/install metadata for the intended distribution. Current local archives omit locks and use a separate pinned verifier; `publish = false` stays until a separately authorized release change.
- [ ] Review final release notes, actual package contents and source identity at the release revision. A local archive or successful CI run does not itself authorize publication, push/merge or a release tag.

## MAT-01 qualified unpublished candidate

The accepted MAT-01 source manifests selected unpublished Rust 0.6.0 / browser 0.6.0-alpha.0 for the [MAT-01 contracts](./mat-01-structured-materials.md). The working candidate adds BrickPattern and ScalarMaskBlend (fifteen node types/thirteen kernels), requiring downstream exhaustive Rust matches to handle both. The four-channel brick fixture is accepted within the retained MAT-01 scope. Existing format/plan/API schemas, old node semantics and published archives remain unchanged. BrickPattern is integrated through [PR #50](https://github.com/OpenMixture/OpenMixture/pull/50); composition and tools are integrated through PRs #51–52, with recorded machine gates and human acceptance closing MAT-01. This does not publish a package. The 0.5 checkpoint and its source-bound evidence above remain historical facts.

## Local unreleased notes

PR-011 proved public Rust consumption and owned output, including explicit context and repeated rendering. PR-012 fixed omitted human port/parameter context and verified existing CLI reports/exits/files. PR-013 made typed GPU loss/OOM actionable and fixed an uncaptured destroyed-buffer unmap failure. PR-014 added consumer-only freshness and bounded retention without GPU cancellation. PR-015 now verifies package-local assets and isolated archive consumers, adds exact peer dependency metadata and ships README/license files, while retaining disabled publication and existing lockfiles.

Document/node/plan versions, the eleven-node vocabulary, nine WGSL implementations and accepted material appearance remain unchanged by PR-011–015. Diagnostic vocabulary gained the two PR-013 codes, which strict older decoders must account for. No fallback executor, hidden GPU state, new product crate, runtime dependency, pooling optimizer, WebAssembly or editor was introduced. The next decision follows this M4 assessment, the completed remote matrix and the remaining compatibility/distribution checklist.

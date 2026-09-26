# PR-010 evidence

English | [简体中文](./README.zh-CN.md)

**Storage update (2026-09-25):** Ordinary smoke logs and per-run CLI output now live in the [complete historical archive](../../../../docs/evidence/archives/README.md), restored with the `early-runs` group. Environment/measurement records, human acceptance images and failure/precision experiments remain inspectable. The account below describes the original runs, not the current location of every attachment.

All three materials pass 1024×1024 checks locally on pinned SwiftShader Vulkan and Apple M5 Metal. Software decoded RGBA is exact; hardware uses each material's declared tolerance. [Human wood review](./human-review.json) is accepted. Remote CI remains deferred; the workflow now includes the same three-material check and 2K trace, but no remote result is claimed.

## Material comparisons

| Material | Software | Metal | Cases / connected channels |
| --- | --- | --- | --- |
| Glazed ceramic | [Passed](./glazed-ceramic-software-regression.json) | [Passed](./glazed-ceramic-metal-regression.json) | 3 cases; optional normal/height are intentional defaults. |
| Leather | [Passed](./leather-software-regression.json) | [Passed](./leather-metal-regression.json) | 4 cases; all four connected. |
| Wood | [Passed](./software.json) | [Passed](./metal.json) | 4 cases; all four connected. |

The ceramic/leather expected trees are unchanged from the accepted parent commit. Wood has sixteen first-baseline PNGs. [Bootstrap acceptance](./bootstrap-acceptance.json) records a separate `--accept` operation with no rendering or Git operation. [Initial software measurements](./software-initial.json) correctly report missing previous goldens; the `initial-contact-*.png` sheets show BEFORE MISSING. Final `software-candidate.json` and `metal-candidate.json` bind the current runtime, shaders, tooling, material, contract, variants and artifact hashes.

[Overview](./overview.png), [2×2 repeat](./tiling.png), and [controlled PBR comparison](./pbr/comparison.png) were inspected by the agent. The [appearance report](./pbr/review.json) binds the actual baseline, both scene scripts, four 1000×1000 frames and explicit Blender 4.5.13 / Apple M5 Metal settings. All frames use 512 samples, seed 8 and identical camera, lights, geometry and BRDF. The exported normal is applied once; height is diagnostic. These images support a human decision; they do not make that decision.

| Wood case | Height cross/along energy ratio (SwiftShader) | Declared grain axis |
| --- | ---: | --- |
| Default | 209.03 | Vertical |
| Coarse | 75.10 | Vertical |
| Straight | 274.47 | Vertical |
| Quarter turn | 273.93 | Horizontal |

The minimum ratio is 4, with independent span, standard deviation, neighbor correlation and seam gates. Height/normal sign agreement is 100% on both tested adapters. Coarser grain has lower normalized gradient energy; every variant changes all four channels. Full measurements and per-channel comparison errors remain in the linked reports.

The [initial coarse-normal measurement](./metal-initial-measurement.json) retains the pre-baseline threshold failure. The later [initial hardware comparison failure](./metal-initial-tolerance-failure.json) retains 222 default-height and two coarse-height pixels differing by two bytes. [Intermediate exports and measurements](./precision/measurements.json) show at most one byte of error before levels, and two after remapping. The wood-only final hardware envelope is max 2 bytes, mean RGBA 0.20, and at most 0.025% of pixels above one byte (262 at 1K). Exact software PNGs and all spatial/causal gates were preserved. This is an empirical bound for the tested adapters, not a universal driver guarantee.

The precision directory contains the actual diagnostic `.mix` files, PNGs and CLI reports. To reproduce an intermediate, use the existing render interface with `--size 1024 --output height` and an explicit backend, for example:

```bash
cargo run --locked --all-features -p mixture-cli -- render fixtures/materials/wood/reports/precision/warp.mix --size 1024 --output height --out tmp/wood-warp-probe --backend metal --json
```

Use `--backend vulkan --software` with the pinned loader for the software counterpart. The `height-linear.mix` probe also changes gamma to 1. CPU measurements compare exported bytes only; they do not execute node formulas.

## Largest-case 2K trace

The [Metal trace](./2k/metal/trace.json) and [software trace](./2k/software/trace.json) independently compile all eleven configured cases at 2048×2048 and select `wood/default`. Wood and ceramic both have eight passes, but wood's uniform descriptors make its peak 48 bytes greater. Selection is by estimated peak memory, not by assumed visual complexity or wall time. Each directory retains every inspect result, exact selection, doctor, render report, four full-size PNGs and hashes.

| Measurement | Both adapters |
| --- | ---: |
| Pass / texture / uniform count | 8 / 8 / 8 |
| Texture bytes | 268,435,456 |
| Uniform bytes | 224 |
| Staging allocations / cumulative bytes | 4 / 134,217,728 |
| Peak staging bytes | 33,554,432 |
| Cumulative estimated / measured bytes | 402,653,408 / 402,653,408 |
| Peak estimated / measured bytes | 301,990,112 / 301,990,112 (about 288 MiB) |
| Budget | 536,870,912 (512 MiB) |
| Reused / final live bytes | 0 / 0 |
| Explicitly released bytes | 402,653,408 |

| Adapter | Pipeline ms | Execution ms | Readback ms | Renderer total ms |
| --- | ---: | ---: | ---: | ---: |
| Apple M5 / Metal | 12.46 | 26.13 | 4091.53 | 4131.89 |
| SwiftShader Device (LLVM 10.0.0) / Vulkan Cpu | 111.45 | 381.42 | 4048.04 | 4541.48 |

These are single validation runs in the development build, measured by CPU wall clocks. They are not GPU timestamp queries or an interactive-latency benchmark. Readback includes map wait, half-float validation and conversion; renderer total excludes CLI PNG file encoding. Descriptor bytes exclude driver/pipeline/bind-group overhead and CPU buffers. Release means explicit `destroy`, not immediate physical memory return by the OS. The traces verify unchanged 1K baselines and actual allocation counters independent of core estimates. The naive lifetime fits the budget, so no last-consumer release, pooling or compatible texture reuse was introduced.

## Verification and reproduction

The `metal-gpu-smoke/` and `software-gpu-smoke/` directories are separate sequential captures of complete smoke runs. Node reports include nineteen graph cases and thirty literal resampling probes (17 transform, 13 warp). They also exercise allocation counts at odd row widths, aliased outputs, per-call reset and failure release. Individual transform/warp suites passed on the pinned adapter. `trace-tests.log`, `direction-tests.log`, `shader-check.log`, `clippy.log`, and `repository-check.log` retain focused and final checks. CLI/diagnostic JSON is retained as emitted; text logs may omit empty terminal lines only.

```bash
cargo xtask test-node transform-2d
cargo xtask test-node warp
cargo xtask test-material wood
cargo xtask golden check
cargo xtask trace-2k
cargo xtask gpu-smoke
cargo xtask check
```

Use the explicit policy environment in [material reproduction](../README.md) for GPU commands. Run smoke policies sequentially because the existing smoke command writes `tmp/gpu-smoke/`; copy its evidence before switching policies. Material and 2K tasks create unique run directories. [Appearance reproduction](../review/README.md) is optional external tooling. No baseline update belongs in CI, and no human approval or M4 implementation is implied by these machine results.

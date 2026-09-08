# Directional wood candidate

English | [简体中文](./README.zh-CN.md)

PR-010 adds a warm brown wood-like surface with elongated, gently warped grain. [material.mix](./material.mix) uses eight compute passes: explicitly seeded value fractal noise → integer repeat/rotation → a second seeded field displaces the grain → levels height → gradient-map color, height-to-normal, and inverted levels roughness. All four 1024×1024 channels are connected. The total built-in vocabulary is eleven nodes and nine kernels.

The [reports](./reports/README.md) separate full-image comparisons, machine measurements, agent inspection, and [human acceptance](./reports/human-review.json). Wood human acceptance is recorded. Ceramic and leather already have user acceptance; their baseline PNGs remain unchanged. The [full M3 review](../../../docs/m3-review.md) is complete; remote CI remains deferred.

## Controls and intended effects

| Public control | Node contract range / material default | Purpose |
| --- | --- | --- |
| `grainSeed` | u32 `[0,4294967295]` / `161803` | Explicit deterministic source arrangement. The independent distortion seed is fixed at `314159`. |
| `grainRepeat` | Integer `[1,64]` / `32` | X repeat count of the source field; `16` broadens the grain. |
| `warpStrength` | Float `[-1,1]` / `0.018` | Signed X sampling displacement in UV units; `0` removes the warp. |
| `orientation` | Integer `[0,3]` / `0` | Visible clockwise quarter turns of the grain before warp; `1` changes vertical grain to horizontal. |

[Variants](./variants/) change one exposed control at a time: `coarse-grain`, `straight-grain`, and `horizontal-grain`. All channels must change, and the coarse height field must have lower contrast-normalized gradient energy than default. Rotation changes the stretched grain; the separate displacement field and X warp direction remain fixed, so this is not an exact rotation of the entire final material. Direction labels refer to image-space grain, not camera-space appearance.

The source grain uses scale 4, four octaves and persistence 0.45. Distortion uses scale 3, three octaves and persistence 0.45. Height remapping is `[0.12,0.88]`, gamma 0.7; normal strength is 0.0015; roughness maps height from 0.65 to 0.45. Palette endpoints are scene-linear `[0.05,0.017,0.006,1]` and `[0.34,0.17,0.063,1]`. These remain fixed across cases. This is directional texture structure consumed with a normal isotropic BRDF, not a physical anisotropic-reflection model.

## Tiling and directionality

Both noise fields are periodic. Integer transform scales and quarter turns preserve unit-square periodicity; transform/warp use explicit wrapped four-load bilinear sampling at pixel centers. Height-to-normal uses wrapped derivatives. See [node conventions](../../../docs/node-contracts.md). Filtering is bilinear, not a general anisotropic or mipmapped filter; high frequencies and reduced previews can alias.

Every case requires height span ≥100 bytes, standard deviation ≥20, adjacent correlation ≥0.6 on both axes and cross-grain / along-grain gradient energy ≥4. Energy is the mean squared adjacent red difference, including wrap; the denominator has a one-byte-squared floor. Span, variance and correlation guards reject flat or scrambled fields. Transposed literal tests prove that matching histograms cannot pass the wrong axis. The default/coarse/straight cases declare vertical grain; the rotated case declares horizontal grain.

All channels have repeat-edge / interior adjacent-jump ratios ≤2.5. Normals must be opaque, positive Z, near unit length, and agree in direction with measurable height slopes. At least 20% of axis samples must be measurable and ≥98% must agree. The coarser case has a lower minimum mean tilt (0.0003 versus 0.001): wider features at fixed height and normal strength have shallower slopes. Its initial 0.001 gate failed at about 0.00062; the [initial measurement](./reports/metal-initial-measurement.json) is retained. This threshold was set before establishing the first baseline and still rejects a neutral normal map. No failing golden was overwritten.

## Reproduce

Run from the repository root with the explicit [GPU setup](../../../docs/gpu-context.md):

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask test-material wood

MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask test-material wood
cargo xtask golden check
cargo xtask trace-2k
cargo xtask check
```

`golden check` and `trace-2k` use the same GPU-policy variables. The former checks all three materials at 1K. The latter inspects all eleven cases at 2K, selects the highest estimated peak, and measures that workload; its separate trace never updates 1K goldens. Software pixel comparison is exact; wood hardware tolerance is max absolute ≤2 bytes, mean RGBA ≤0.20 byte, and at most 0.025% of pixels above one byte. The [initial comparison failure](./reports/metal-initial-tolerance-failure.json) and [intermediate precision measurements](./reports/precision/measurements.json) are retained: noise/transform/warp exports differ by at most one byte, while the levels remap can amplify that to two. This measured precision propagation led to a wood-only tolerance change; all software baseline PNGs and structural gates remain unchanged. See [protected updates](../../../docs/material-goldens.md); `golden update wood --accept` consumes a previously reviewed software candidate without rendering and refuses CI.

## Appearance and scope

The optional [controlled review](./review/README.md) consumes actual verified PNGs with identical lights, camera, geometry and BRDF. Exported normal is applied once; height remains diagnostic, without a second bump or displacement. No knots, extra fibers, procedural noise or surface relief are added by the consumer. The target is a useful stylized wood-like surface, not a scanned species or physically calibrated timber.

The [2K evidence](./reports/README.md) compares descriptor allocation measurements with the existing 512 MiB peak budget before any lifetime optimization. Retaining all pass textures is allowed when the measured workload fits; reuse is not added without that need. Descriptor bytes exclude driver overhead and CPU image buffers. Out of scope: new texture formats, generalized optimization, last-consumer reuse without a measured failure, M4 API stabilization, bindings, daemon, editor, embedded resources, remote CI closure, and a thirteenth node.

# Controlled appearance review

English | [简体中文](./README.zh-CN.md)

This optional, offline Blender scene consumes the twelve verified PNGs in [expected/](../expected/). It addresses the limitation of flat channel swatches: those show a checker but cannot show roughness or reflected light. The scene is fixture-owned review tooling. It neither reads `.mix` nor implements a Mixture node, and it is not required by the Rust runtime or `cargo xtask check`.

## Reproduce

Use **Blender 4.5.13** from the [official release directory](https://download.blender.org/release/Blender4.5/), and verify the download against its published `blender-4.5.13.sha256`. The captured macOS arm64 DMG has SHA-256 `663ce944257c61ff1d6aa09e15c8f57bbd8d59023adb2fa7edde33a9ed960b53`. An extracted local application is sufficient; no system installation is required.

Run from the repository root, setting `MIXTURE_REVIEW_BLENDER` to the executable:

```bash
"$MIXTURE_REVIEW_BLENDER" --background --factory-startup --python-exit-code 1 \
  --python fixtures/materials/glazed-ceramic/review/render.py -- \
  --expected fixtures/materials/glazed-ceramic/expected \
  --out tmp/ceramic-pbr-review --device METAL --samples 512
```

The output directory must be new. `METAL` explicitly requires a Cycles Metal device and records its identity; there is no automatic CPU retry. On another platform, explicitly use `--device CPU` for this external scene review. That option does not change Mixture's sole `wgpu` texture execution path. `--size` and `--samples` are optional; defaults are 1000×1000 and 128 samples with fixed seed 8, no denoising, and no adaptive sampling. The retained evidence uses 512 samples to reduce visible sampling noise. The script requires the pinned Blender version, all twelve 1K inputs, and matching manifest hashes; it rechecks identity after rendering.

The three PNGs and `review.json` are written only to the new output directory. They do not update a golden, stage files, or grant human approval. Create the labeled comparison using Python and **Pillow 12.3.0**:

```bash
python3 fixtures/materials/glazed-ceramic/review/compare.py \
  --review tmp/ceramic-pbr-review
```

The comparison script verifies render hashes, downsamples with Lanczos, adds captions, and writes `comparison.png` plus `comparison.json`; it performs no relighting or retouching. The scripts and captured inputs/outputs are bound by SHA-256 in the reports. Supplementary PBR frames are review evidence rather than exact texture goldens; Cycles/platform differences can change them. Retained evidence lives under [reports/pbr/](../reports/pbr/).

## What the scene demonstrates

Every case uses the same camera, three neutral rectangular area lights, world, sphere, planar sample, and AgX display transform. All four exported channels are consumed:

| Channel | Consumer connection |
| --- | --- |
| baseColor | sRGB texture → Principled Base Color. |
| roughness | Non-Color texture → Roughness, with no remapping. |
| normal | Non-Color texture → tangent-space Normal Map → Bump Normal. |
| height | Non-Color texture → Bump Height, distance 0.02; the constant zero input has no gradient. |

The shared dielectric BRDF uses IOR 1.5, metallic 0, and no additional coat, transmission, subsurface scattering, noise, or procedural surface detail. These are consumer display assumptions, not fields added to `.mix`. See Blender's [Principled BSDF documentation](https://docs.blender.org/manual/en/4.5/render/shader_nodes/shader/principled.html) and [normal-map documentation](https://docs.blender.org/manual/en/4.5/render/shader_nodes/vector/normal_map.html).

Compare default with matte: baseColor, normal, and height are byte-identical, while roughness rises from 43/255 to 112/255. Broader, softer reflections therefore test the exported roughness response under this BRDF. Fine-tiles preserves default roughness and doubles pattern density. The material's public `glaze` control is a color blend, not a consumer coat layer.

The sphere curvature, slab thickness, and rounded backing edge come from display geometry. They are not generated normals, grout, or height detail. The planar top shows one UV repeat; the sphere uses a latitude/longitude UV projection and therefore distorts the checker near its poles. Neutral normals and zero height intentionally keep the current material smooth. A glossy patterned dielectric alone does not establish a unique physical substance or a photoreal ceramic finish; that judgment remains with the human reviewer.

## Verification

After rendering, inspect the three frames and contact sheet, then retain the complete report with the images. Input/overwrite guard probes are available separately:

```bash
python3 fixtures/materials/glazed-ceramic/review/test_inputs.py \
  --blender "$MIXTURE_REVIEW_BLENDER"
cargo xtask check
```

The normal machine gates remain `cargo xtask golden check` and `cargo xtask test-material glazed-ceramic`; see the [golden workflow](../../../../docs/material-goldens.md). Review tooling does not change material pixels and cannot close the [human review record](../reports/human-review.json), remote CI, M3, or the M4 external-consumer milestone.

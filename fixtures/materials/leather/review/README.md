# Controlled leather review

English | [简体中文](./README.zh-CN.md)

This optional fixture consumer renders the actual sixteen 1K PNGs with a fixed external Blender scene. [render.py](./render.py) reuses the unchanged [ceramic scene helper](../../glazed-ceramic/review/render.py), verifies every manifest hash and records both script hashes. It never reads `.mix` or executes a node formula. Use Blender 4.5.13 and the verified portable setup described in [ceramic reproduction](../../glazed-ceramic/review/README.md).

```bash
"$MIXTURE_REVIEW_BLENDER" --background --factory-startup --python-exit-code 1 \
  --python fixtures/materials/leather/review/render.py -- \
  --expected fixtures/materials/leather/expected \
  --out tmp/leather-pbr-review --device METAL --samples 512
python3 fixtures/materials/leather/review/compare.py --review tmp/leather-pbr-review
python3 fixtures/materials/leather/review/test_inputs.py --blender "$MIXTURE_REVIEW_BLENDER"
```

Run from the repository root. The output directory must be new and outside `expected/`. An explicit Metal device is required for `--device METAL`; no automatic retry occurs. Explicit `--device CPU` is available solely for this external scene consumer on other platforms. Defaults are 1000×1000, 512 samples, fixed seed 8, no denoising or adaptive sampling. Python with Pillow 12.3.0 assembles the labeled contact sheet with Lanczos downsampling and no retouching. Input/overwrite guard tests run separately from Rust checks.

BaseColor is read as sRGB; roughness and normal as Non-Color. The generated normal enters tangent Normal Map (strength 1) directly into Principled Normal. Height is loaded for diagnostic inspection but has no extra bump or displacement connection: its slope is already represented by the normal. The fixed BRDF has IOR 1.5, metallic 0, coat 0, transmission 0 and subsurface 0. No additional noise, sheen or relief is added.

All cases share the camera, three area lights, UV sphere, one-repeat planar swatch, backing geometry and AgX display transform. The comparison orders detail 0, 0.35, 1, then scale 32. More detail increases fine structure but reduces large-grain contrast; coarser scale broadens grains. Sphere UV distortion near the poles and slight Cycles sampling grain are display effects. This is supplementary appearance evidence, not exact pixel goldens or M4 acceptance. The [human decision](../reports/human-review.json) remains separate.

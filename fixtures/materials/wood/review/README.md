# Controlled wood-like review

English | [简体中文](./README.zh-CN.md)

This optional fixture consumer renders the sixteen verified 1K PNGs in a fixed external Blender scene. [render.py](./render.py) reuses the unchanged [ceramic scene helper](../../glazed-ceramic/review/render.py), requires the version-one wood manifest with exactly four channels for each of four cases, verifies every PNG hash, and records both script hashes. It never reads `.mix` or executes a node formula. Use Blender 4.5.13 and the verified portable setup described in [ceramic reproduction](../../glazed-ceramic/review/README.md).

Create and accept the wood software baseline through the [guarded golden workflow](../../../../docs/material-goldens.md) before running the appearance render. No PBR result or human acceptance is claimed by adding these scripts.

```bash
"$MIXTURE_REVIEW_BLENDER" --background --factory-startup --python-exit-code 1 \
  --python fixtures/materials/wood/review/render.py -- \
  --expected fixtures/materials/wood/expected \
  --out tmp/wood-pbr-review --device METAL --samples 512
python3 fixtures/materials/wood/review/compare.py --review tmp/wood-pbr-review
python3 fixtures/materials/wood/review/test_inputs.py --blender "$MIXTURE_REVIEW_BLENDER"
```

Run from the repository root. The output directory must be new and outside `expected/`; inputs and both scene scripts are rechecked before a completed report is written. `--device METAL` explicitly requires a Cycles Metal device and records its identity, without automatic retry. Other platforms may explicitly use `--device CPU` solely for this external scene consumer. The Blender version remains pinned for either device. Defaults are 1000×1000, 512 samples, fixed seed 8, no denoising or adaptive sampling. Python with Pillow 12.3.0 assembles the labeled contact sheet with Lanczos downsampling and no retouching, after checking the wood review identity, all four render hashes, and dimensions. Existing comparison files are never replaced.

BaseColor is read as sRGB; roughness and normal as Non-Color. The generated normal enters tangent Normal Map at strength 1 directly into Principled Normal. Height is loaded for diagnostic inspection but has no extra bump or displacement connection: its slope is already represented by the normal. The fixed BRDF has IOR 1.5, metallic 0, coat 0, transmission 0, and subsurface 0. No additional noise, sheen, or relief is added.

Every case shares the camera, three area lights, UV sphere, one-repeat planar swatch, backing geometry, and AgX display transform. [compare.py](./compare.py) orders the cases as follows; only the verified PNG inputs change:

| Case | grainRepeat | warpStrength | orientation | Intended comparison |
| --- | ---: | ---: | ---: | --- |
| `default` | 32 | 0.018 | 0 | Wood-like candidate with warped vertical grain. |
| `coarse-grain` | 16 | 0.018 | 0 | Fewer repeats broaden the grain. |
| `straight-grain` | 32 | 0 | 0 | Zero displacement removes the warp. |
| `horizontal-grain` | 32 | 0.018 | 1 | A quarter-turn changes the principal grain direction. |

These are texture-space directions; scene perspective and sphere UV distortion change their displayed orientation. Cycles sampling grain is a display effect. The scene demonstrates directional texture structure under the same isotropic consumer BRDF; it does not add an anisotropic shader, wood fibers, knots, or a physical wood model. This is supplementary appearance evidence, not exact pixel goldens or M4 acceptance. Human acceptance of the wood-like result is recorded separately in [human-review.json](../reports/human-review.json). The original image caption and automated report retain their pre-acceptance wording as historical capture evidence.

[Input/overwrite guard tests](./test_inputs.py) use temporary hash-only preflight fixtures and deliberately stop before scene setup or image loading. They need neither a wood baseline nor PBR rendering and run separately from Rust checks. They verify the exact sixteen-file manifest, source hashes, output isolation, and preservation of existing review files. The material machine gates remain `cargo xtask test-material wood` and `cargo xtask golden check`.

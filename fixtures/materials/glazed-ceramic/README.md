# Glazed ceramic / checker

English | [简体中文](./README.zh-CN.md)

The PR-008 candidate is a smooth, two-tone ceramic/checker target with independently controlled roughness. Following feedback that the flat previews only showed a checker, a [controlled PBR review](./review/README.md) now consumes the actual exported channels on a sphere and planar sample. It shows the glossy versus matte response; the user has accepted the controlled appearance comparison. The generated textures contain no grout, bevels, cracks, micro-height, or illumination.

## Source and outputs

[material.mix](./material.mix) uses all six existing M2 node types. `checker` and `constant-color` feed a screen `blend` for base color. A `constant-scalar` feeds `levels` for roughness. `material-output` supplies the documented neutral normal and flat zero height. The plan has eight compute passes, including defaults, and four requested channels. There is no color-to-scalar conversion or generated normal producer in M2; no node was added to imitate either.

| Output | Meaning and expected default |
| --- | --- |
| `baseColor` | Opaque 8×8 alternating tiles, half each color; sRGB RGBA8 `[224,232,226,255]` and `[98,128,142,255]` on the tested software adapter. |
| `normal` | Default encoded tangent-space +Z: linear RGBA8 `[128,128,255,255]`. |
| `roughness` | Connected, spatially uniform glaze response: linear RGBA8 `[43,43,43,255]` (approximately 0.17). |
| `height` | Documented default plane: linear RGBA8 `[0,0,0,255]`. Zero is intentional; there is no relief claim. |

Even tile counts, with dimensions divisible by those counts, preserve alternating continuity. The current acceptance uses 1024×1024 and 8 or 16 cells per axis. Other parameter values remain subject to the existing node contracts; this material's acceptance does not promise seamless tiling for odd tile counts.

## Exposed controls and variants

| Public control | Source parameter | Default / role |
| --- | --- | --- |
| `tilesX` / `tilesY` | checker `cellsX` / `cellsY` | 8 / 8; set both to change square-tile density. |
| `glaze` | blend `opacity` | 0.12; mixes toward the screen result in linear color. |
| `roughness` | scalar `value` | 0.2, remapped by levels to a gloss-oriented range `[0.05,0.65]`. |

[fine-tiles](./variants/fine-tiles.json) sets both counts to 16. The palette and all other channels remain unchanged, while 50% of base-color pixels move to the other tile color. The accepted minimum is 45%; structural checks independently require the denser period and transition counts.

[matte](./variants/matte.json) sets roughness to 0.65. All roughness pixels rise from 43 to 112, a normalized increase of about 0.2706; the acceptance minimum is 0.25. Color, normal, and height remain byte-identical. These variants establish causality, not presets stored in `.mix`.

## Reproduce and inspect

```bash
cargo run --locked -p mixture-cli -- validate fixtures/materials/glazed-ceramic/material.mix --json
cargo run --locked -p mixture-cli -- inspect fixtures/materials/glazed-ceramic/material.mix \
  --plan --size 1024 --output baseColor,normal,roughness,height --json
cargo run --locked -p mixture-cli -- render fixtures/materials/glazed-ceramic/material.mix \
  --size 1024 --output baseColor,normal,roughness,height --out tmp/ceramic --json
cargo xtask test-material glazed-ceramic
```

The [golden workflow](../../../docs/material-goldens.md) documents software setup, exact versus tolerant comparison, and the separate guarded update. [acceptance.json](./acceptance.json) is the executable gate contract; [the schema](../acceptance.schema.json) and Rust cross-field checks reject incomplete acceptance configurations.

Software expectations live in [expected/](./expected/). Review images and machine evidence live in [reports/](./reports/). The 1K logical peak is 75,497,648 bytes (about 72 MiB), with eight passes, as reported by the existing naive lifetime model. This excludes driver and pipeline overhead and is not a 2K acceptance or optimization result.

## Review state

[human-review.json](./reports/human-review.json) explicitly separates agent inspection from human acceptance. A passing golden comparison and `--accept` do not sign that record. The [controlled appearance comparison](./reports/pbr/comparison.png) now complements the flat channel swatches. Its [input and scene record](./reports/pbr/review.json) binds the twelve unchanged PNGs, Blender version/device, and script. Default/fine-tiles show sharper softbox and sample reflections than matte. Sphere curvature and backing edges are consumer geometry; normal remains neutral and height remains zero. This supports review of a smooth glazed checker candidate, not a claim of generated surface detail or unique physical material identity. PR-008 human acceptance is recorded against the unchanged manifest and PBR evidence. Full M3 and remote CI were still open at PR-008; the [full M3 review](../../../docs/m3-review.md) is now complete and the documented [remote CI gates](../../../docs/evidence/remote-ci/README.md) are closed. Historical reports remain unchanged.

Out of scope for PR-008: new nodes, noise, generated normals, relief, resource pooling, 2K optimization, a runtime viewer, browser work, and remote CI acceptance.

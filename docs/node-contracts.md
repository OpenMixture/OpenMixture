# M2 node contracts, version 1

English | [简体中文](./node-contracts.zh-CN.md)

PR-005 registers six static contracts in [mixture-core](../crates/mixture-core/src/registry.rs). These define `.mix` source validation and defaults before PR-007 graph execution. Only the separate PR-004 fixed checker probe currently executes pixels. No new shader, CPU pixel implementation, `KernelId`, or renderer is introduced by these contracts.

## Common rules

All six node types require `version: 1`. Connections match `Scalar`, `Color`, or `Normal` exactly; every input has at most one incoming edge. An input without a default is required. Omitted parameters use the defaults below; unknown names, invalid types, and out-of-range values are errors. All parameters below are mutable and can be exposed through a unique public binding. No node requires a seed because none is randomized.

Float parameters accept finite JSON numbers; integer parameters require unsigned integer tokens (`8` is valid, `8.0` and `8e0` are not). Colors are arrays of exactly four finite numbers in `[0, 1]`, representing linear RGBA, with straight alpha. Float/color bounds are inclusive. The source model retains f64 JSON values; future execution must explicitly lower them to the GPU representation. Parameter validation does not execute pixels or convert color spaces.

Coordinates use a top-left origin. Pointwise constant, levels, and blend operations introduce no coordinate transform. Their tiling depends on their inputs; constant outputs are seamless. Detailed GPU precision and golden evidence for graph execution belong to PR-007.

## constant-scalar

[Contract module](../crates/mixture-core/src/nodes/constant_scalar.rs). No inputs; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `value` | Float `[0, 1]` | `0.0` |

Defines the same scalar at every pixel. Its meaning as roughness, height, mask, or another scalar channel is assigned by its consumer.

## constant-color

[Contract module](../crates/mixture-core/src/nodes/constant_color.rs). No inputs; output `color: Color`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `value` | RGBA `[0, 1]` per component | `[1, 1, 1, 1]` |

Defines the same linear RGBA color at every pixel.

## checker

[Contract module](../crates/mixture-core/src/nodes/checker.rs). No inputs; output `color: Color`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `cellsX` | Integer `[1, 1024]` | `8` |
| `cellsY` | Integer `[1, 1024]` | `8` |
| `colorA` | RGBA `[0, 1]` per component | `[0, 0, 0, 1]` |
| `colorB` | RGBA `[0, 1]` per component | `[1, 1, 1, 1]` |

For output pixel `(x, y)`, cell indices are `floor(x * cellsX / width)` and `floor(y * cellsY / height)`. Even summed parity selects `colorA`; odd parity selects `colorB`. Uneven dimensions produce uneven cell widths, and small images can undersample cells. Even cell counts preserve alternating continuity when repeated across that axis; odd counts are permitted but repeat adjacent same-color boundary cells. No filtering or randomness is implied.

Defaults match the [PR-004 fixed checker](./builtin-checker.md). The graph contract permits colors and frequencies that the fixed probe command does not expose. PR-007 must extend the single existing WGSL path when graph execution is implemented.

## levels

[Contract module](../crates/mixture-core/src/nodes/levels.rs). Required input `in: Scalar`; output `value: Scalar`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `inputMin` | Float `[0, 1]` | `0.0` |
| `inputMax` | Float `[0, 1]` | `1.0` |
| `gamma` | Float `[0.01, 100]` | `1.0` |
| `outputMin` | Float `[0, 1]` | `0.0` |
| `outputMax` | Float `[0, 1]` | `1.0` |

`inputMin < inputMax` is required after resolving defaults. The declared operation is `t = clamp((in - inputMin) / (inputMax - inputMin), 0, 1)`, followed by `outputMin + pow(t, 1 / gamma) * (outputMax - outputMin)`. Reversed output bounds are allowed for inversion. Positive gamma and distinct input bounds prevent undefined divisions. This formula documents future shader semantics; Rust validates the parameters only.

## blend

[Contract module](../crates/mixture-core/src/nodes/blend.rs). Required inputs `a: Color` and `b: Color`; optional `mask: Scalar`, default `1.0`; output `color: Color`.

| Parameter | Type / range | Default |
| --- | --- | --- |
| `mode` | Enum `normal`, `multiply`, `screen` | `normal` |
| `opacity` | Float `[0, 1]` | `1.0` |

Define `t = opacity * clamp(mask, 0, 1)`. RGB is a linear interpolation from `a.rgb` to the mode result by `t`: `b.rgb` for normal, `a.rgb * b.rgb` for multiply, and `1 - (1 - a.rgb) * (1 - b.rgb)` for screen. Alpha interpolates from `a.a` to `b.a` by `t` for every mode. This is a component blend contract, not source-over compositing. No hidden premultiplication or extra blend modes are implied.

## material-output

[Contract module](../crates/mixture-core/src/nodes/material_output.rs). Exactly one sink is required per document. It has no outputs and no parameters. Inputs appear in this stable contract order:

| Input | Kind | Policy / default |
| --- | --- | --- |
| `baseColor` | Color | Required valid connection |
| `normal` | Normal | Encoded neutral XYZ `[0.5, 0.5, 1.0]` |
| `roughness` | Scalar | `1.0` |
| `metallic` | Scalar | `0.0` |
| `height` | Scalar | `0.0` |
| `ambientOcclusion` | Scalar | `1.0` |
| `opacity` | Scalar | `1.0` |
| `emissive` | Color | Linear opaque black `[0, 0, 0, 1]` |

The encoded normal corresponds to tangent-space +Z; its future RGBA storage filler is separate from this logical XYZ value. M2 contains no `Normal` producer, so valid M2 documents use this default. A color output cannot masquerade as a normal. `ValidatedDocument::material_channels()` exposes the connected/default status without generating textures.

## Fixtures and checks

The [two-node checker](../fixtures/format/valid/checker.mix) exercises default parameters and an exposed defaulted frequency. [all-m2.mix](../fixtures/format/valid/all-m2.mix) connects all six contracts, applies levels to a scalar blend mask, and supplies explicit roughness. [Invalid documents](../fixtures/format/README.md) and public API tests cover parameter boundaries, enums, port direction and kinds, required/defaulted inputs, graph errors, and public bindings.

```bash
cargo test --locked -p mixture-core --test registry
cargo test --locked -p mixture-core --test validation
cargo xtask test-format
```

These tests validate contracts without a GPU. The existing fixed checker golden is unchanged. A contract's presence is not evidence that its graph pixel executor or node-specific golden exists yet.
